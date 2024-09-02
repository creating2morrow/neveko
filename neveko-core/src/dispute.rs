//! Marketplace disputes operations module

use std::error::Error;

use crate::{
    db,
    models::*,
    monero,
    utils,
};
use log::{
    debug,
    error,
    info,
};
use rocket::serde::json::Json;

/// Create a new dispute
pub fn create(d: Json<Dispute>) -> Dispute {
    let f_did: String = format!("{}{}", crate::DISPUTE_DB_KEY, utils::generate_rnd());
    info!("create dispute: {}", &f_did);
    let new_dispute = Dispute {
        did: String::from(&f_did),
        created: chrono::offset::Utc::now().timestamp(),
        orid: String::from(&d.orid),
        tx_set: String::from(&d.tx_set),
    };
    debug!("insert dispute: {:?}", &d);
    let s = db::Interface::open();
    let k = &d.did;
    db::Interface::write(&s.env, &s.handle, k, &Dispute::to_db(&new_dispute));
    // in order to retrieve all orders, write keys to with dl
    let list_key = crate::DISPUTE_LIST_DB_KEY;
    let r = db::Interface::read(&s.env, &s.handle, &String::from(list_key));
    if r == utils::empty_string() {
        debug!("creating dispute index");
    }
    let dispute_list = [String::from(&r), String::from(&f_did)].join(",");
    debug!(
        "writing dispute index {} for id: {}",
        dispute_list, list_key
    );
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &dispute_list);
    // restart the dispute aut-settle thread
    let cleared = is_dispute_clear(r);
    if !cleared {
        debug!("restarting dispute auto-settle");
        utils::restart_dispute_auto_settle();
    }
    new_dispute
}

/// Dispute lookup
pub fn find(did: &String) -> Dispute {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(did));
    if r == utils::empty_string() {
        error!("dispute not found");
        return Default::default();
    }
    Dispute::from_db(String::from(did), r)
}

/// Lookup all disputes
pub fn find_all() -> Vec<Dispute> {
    let d_s = db::Interface::open();
    let d_list_key = crate::DISPUTE_LIST_DB_KEY;
    let d_r = db::Interface::read(&d_s.env, &d_s.handle, &String::from(d_list_key));
    if d_r == utils::empty_string() {
        error!("dispute index not found");
    }
    let d_v_did = d_r.split(",");
    let d_v: Vec<String> = d_v_did.map(String::from).collect();
    let mut disputes: Vec<Dispute> = Vec::new();
    for o in d_v {
        let dispute: Dispute = find(&o);
        if dispute.did != utils::empty_string() {
            disputes.push(dispute);
        }
    }
    disputes
}

/// Dispute deletion
pub fn delete(did: &String) {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(did));
    if r == utils::empty_string() {
        error!("dispute not found");
        return Default::default();
    }
    db::Interface::delete(&s.env, &s.handle, &String::from(did))
}

/// Triggered on DISPUTE_LAST_CHECK_DB_KEY.
///
/// If the current UNIX timestamp is less than the
///
/// creation date of the dispute plus the one week
///
/// grace period then the dispute is auto-settled.
pub async fn settle_dispute() {
    let tick: std::sync::mpsc::Receiver<()> =
        schedule_recv::periodic_ms(crate::DISPUTE_CHECK_INTERVAL);
    loop {
        debug!("running dispute auto-settle thread");
        tick.recv().unwrap();
        let s = db::Interface::open();
        let list_key = crate::DISPUTE_LIST_DB_KEY;
        let r = db::Interface::read(&s.env, &s.handle, &String::from(list_key));
        if r == utils::empty_string() {
            info!("dispute index not found");
        }
        let v_mid = r.split(",");
        let d_vec: Vec<String> = v_mid.map(String::from).collect();
        debug!("dispute contents: {:#?}", d_vec);
        let cleared = is_dispute_clear(r);
        if cleared {
            // index was created but cleared
            info!("terminating dispute auto-settle thread");
            db::Interface::delete(&s.env, &s.handle, list_key);
            return;
        }
        for d in d_vec {
            let dispute: Dispute = find(&d);
            if dispute.did != utils::empty_string() {
                let now = chrono::offset::Utc::now().timestamp();
                let settle_date = dispute.created + crate::DISPUTE_AUTO_SETTLE as i64;
                if settle_date > now {
                    let wallet_name = dispute.orid;
                    let wallet_password = utils::empty_string();
                    monero::open_wallet(&wallet_name, &wallet_password).await;
                    let signed = monero::sign_multisig(dispute.tx_set).await;
                    let submit = monero::submit_multisig(signed.result.tx_data_hex).await;
                    monero::close_wallet(&wallet_name, &wallet_password).await;
                    if submit.result.tx_hash_list.is_empty() {
                        error!("could not broadcast txset for dispute: {}", &dispute.did);
                        return;
                    }
                    // remove the dispute from the db
                    remove_from_auto_settle(dispute.did);
                }
            }
        }
    }
}

fn is_dispute_clear(r: String) -> bool {
    let v_mid = r.split(",");
    let v: Vec<String> = v_mid.map(String::from).collect();
    debug!("dispute index contents: {:#?}", v);
    let limit = v.len() <= 1;
    if !limit {
        v.len() >= 2
            && v[v.len() - 1] == utils::empty_string()
            && v[0] == utils::empty_string()
    } else {
        limit
    }
}

/// clear dispute from index
fn remove_from_auto_settle(did: String) {
    info!("removing id {} from disputes", &did);
    let s = db::Interface::open();
    let list_key = crate::DISPUTE_LIST_DB_KEY;
    let r = db::Interface::read(&s.env, &s.handle, &String::from(list_key));
    if r == utils::empty_string() {
        debug!("dispute list index is empty");
    }
    let pre_v_fts = r.split(",");
    let v: Vec<String> = pre_v_fts
        .map(|s| {
            if s != &did {
                String::from(s)
            } else {
                utils::empty_string()
            }
        })
        .collect();
    let dispute_list = v.join(",");
    debug!(
        "writing dipsute index {} for id: {}",
        dispute_list, list_key
    );
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &dispute_list);
}

/// Executes POST /market/dispute/create
///
/// cancelling the order on the vendor side.
///
/// Customer needs to verify the response and update their lmdb.
///
/// see `cancel_order`
async fn transmit_dispute_request(
    contact: &String,
    jwp: &String,
    request: &Dispute,
) -> Result<Dispute, Box<dyn Error>> {
    info!("executing transmit_dispute_request");
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .post(format!("http://{}/market/dispute/create", contact))
        .header("proof", jwp)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<Dispute>().await;
            debug!("dispute response: {:?}", res);
            match res {
                Ok(r) => Ok(r),
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to create a dispute due to: {:?}", e);
            Ok(Default::default())
        }
    }
}

/// A decomposition trigger for the dispute request so that the logic
///
/// can be executed from the gui.
pub async fn trigger_dispute_request(contact: &String, dispute: &Dispute) -> Dispute {
    info!("executing trigger_dispute_request");
    let s = db::Interface::async_open().await;
    let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, &contact);
    let jwp = db::Interface::async_read(&s.env, &s.handle, &k).await;
    let dispute = transmit_dispute_request(contact, &jwp, dispute).await;
    // handle a failure to create dispute
    if dispute.is_err() {
        error!("failed to create dispute");
        return Default::default();
    }
    dispute.unwrap_or(Default::default())
}
