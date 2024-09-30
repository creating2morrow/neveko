//! Marketplace disputes operations module

use std::error::Error;

use crate::{
    db::{
        self,
        DATABASE_LOCK,
    },
    error::NevekoError,
    models::*,
    monero,
    utils,
};
use kn0sys_lmdb_rs::MdbError;
use log::{
    debug,
    error,
    info,
};
use rocket::serde::json::Json;

/// Create a new dispute
pub fn create(d: Json<Dispute>) -> Result<Dispute, MdbError> {
    let f_did: String = format!("{}{}", crate::DISPUTE_DB_KEY, utils::generate_rnd());
    info!("create dispute: {}", &f_did);
    let new_dispute = Dispute {
        did: String::from(&f_did),
        created: chrono::offset::Utc::now().timestamp(),
        orid: String::from(&d.orid),
        tx_set: String::from(&d.tx_set),
    };
    debug!("insert dispute: {:?}", &d);
    let db = &DATABASE_LOCK;
    let k = &d.did;
    let v = bincode::serialize(&new_dispute).unwrap_or_default();
    db::write_chunks(&db.env, &db.handle, k.as_bytes(), &v)?;
    // in order to retrieve all orders, write keys to with dl
    let list_key = crate::DISPUTE_LIST_DB_KEY;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())?;
    if r.is_empty() {
        debug!("creating dispute index");
    }
    let s_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let dispute_list = [String::from(&s_r), String::from(&f_did)].join(",");
    let s_dispute_list = bincode::serialize(&dispute_list).unwrap_or_default();
    debug!(
        "writing dispute index {} for id: {}",
        dispute_list, list_key
    );
    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_dispute_list)?;
    // restart the dispute aut-settle thread
    let cleared = is_dispute_clear(s_r);
    if !cleared {
        debug!("restarting dispute auto-settle");
        utils::restart_dispute_auto_settle();
    }
    Ok(new_dispute)
}

/// Dispute lookup
pub fn find(did: &String) -> Result<Dispute, NevekoError> {
    let db = &DATABASE_LOCK;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &did.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        error!("dispute not found");
        return Err(NevekoError::Database(MdbError::Panic));
    }
    let result: Dispute = bincode::deserialize(&r[..]).unwrap_or_default();
    Ok(result)
}

/// Lookup all disputes
pub fn find_all() -> Result<Vec<Dispute>, NevekoError> {
    let db = &DATABASE_LOCK;
    let d_list_key = crate::DISPUTE_LIST_DB_KEY;
    let d_r = db::DatabaseEnvironment::read(&db.env, &db.handle, &d_list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if d_r.is_empty() {
        error!("dispute index not found");
    }
    let str_r: String = bincode::deserialize(&d_r[..]).unwrap_or_default();
    let d_v_did = str_r.split(",");
    let d_v: Vec<String> = d_v_did.map(String::from).collect();
    let mut disputes: Vec<Dispute> = Vec::new();
    for o in d_v {
        let dispute: Dispute = find(&o)?;
        if !dispute.did.is_empty() {
            disputes.push(dispute);
        }
    }
    Ok(disputes)
}

/// Dispute deletion
pub fn delete(did: &String) -> Result<(), MdbError> {
    let db = &DATABASE_LOCK;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &did.as_bytes().to_vec())?;
    if r.is_empty() {
        error!("dispute not found");
        return Err(MdbError::NotFound);
    }

    db::DatabaseEnvironment::delete(&db.env, &db.handle, did.as_bytes())
}

/// Triggered on DISPUTE_LAST_CHECK_DB_KEY.
///
/// If the current UNIX timestamp is less than the
///
/// creation date of the dispute plus the one week
///
/// grace period then the dispute is auto-settled.
pub async fn settle_dispute() -> Result<(), NevekoError> {
    let tick: std::sync::mpsc::Receiver<()> =
        schedule_recv::periodic_ms(crate::DISPUTE_CHECK_INTERVAL);
    loop {
        debug!("running dispute auto-settle thread");
        tick.recv().unwrap();
        let db = &DATABASE_LOCK;
        let list_key = crate::DISPUTE_LIST_DB_KEY;
        let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
            .map_err(|_| NevekoError::Database(MdbError::Panic))?;
        if r.is_empty() {
            info!("dispute index not found");
            return Err(NevekoError::Database(MdbError::NotFound));
        }
        let str_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
        let v_mid = str_r.split(",");
        let d_vec: Vec<String> = v_mid.map(String::from).collect();
        debug!("dispute contents: {:#?}", d_vec);
        let cleared = is_dispute_clear(str_r);
        if cleared {
            // index was created but cleared
            info!("terminating dispute auto-settle thread");
            db::DatabaseEnvironment::delete(&db.env, &db.handle, list_key.as_bytes())
                .map_err(|_| NevekoError::Database(MdbError::Panic))?;
            return Ok(());
        }
        for d in d_vec {
            let dispute: Dispute = find(&d)?;
            if !dispute.did.is_empty() {
                let now = chrono::offset::Utc::now().timestamp();
                let settle_date = dispute.created + crate::DISPUTE_AUTO_SETTLE as i64;
                if settle_date > now {
                    let wallet_name = dispute.orid;
                    let wallet_password = String::new();
                    monero::open_wallet(&wallet_name, &wallet_password).await;
                    let signed = monero::sign_multisig(dispute.tx_set).await;
                    let submit = monero::submit_multisig(signed.result.tx_data_hex).await;
                    monero::close_wallet(&wallet_name, &wallet_password).await;
                    if submit.result.tx_hash_list.is_empty() {
                        error!("could not broadcast txset for dispute: {}", &dispute.did);
                        return Ok(());
                    }
                    // remove the dispute from the db
                    remove_from_auto_settle(dispute.did).map(|_| NevekoError::Dispute)?;
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
        v.len() >= 2 && v[v.len() - 1].is_empty() && v[0].is_empty()
    } else {
        limit
    }
}

/// clear dispute from index
fn remove_from_auto_settle(did: String) -> Result<(), NevekoError> {
    info!("removing id {} from disputes", &did);
    let db = &DATABASE_LOCK;
    let list_key = crate::DISPUTE_LIST_DB_KEY;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        debug!("dispute list index is empty");
    }
    let str_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let pre_v_fts = str_r.split(",");
    let v: Vec<String> = pre_v_fts
        .map(|s| {
            if s != did {
                String::from(s)
            } else {
                String::new()
            }
        })
        .collect();
    let dispute_list = v.join(",");
    let s_dispute_list = bincode::serialize(&dispute_list).unwrap_or_default();
    debug!(
        "writing dipsute index {} for id: {}",
        dispute_list, list_key
    );
    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_dispute_list)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(())
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
pub async fn trigger_dispute_request(
    contact: &String,
    dispute: &Dispute,
) -> Result<Dispute, NevekoError> {
    info!("executing trigger_dispute_request");
    let db = &DATABASE_LOCK;
    let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, &contact);
    let jwp = db::DatabaseEnvironment::read(&db.env, &db.handle, &k.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let str_jwp: String = bincode::deserialize(&jwp[..]).unwrap_or_default();
    let dispute = transmit_dispute_request(contact, &str_jwp, dispute).await;
    // handle a failure to create dispute
    if dispute.is_err() {
        error!("failed to create dispute");
        return Err(NevekoError::Dispute);
    }
    dispute.map_err(|_| NevekoError::Dispute)
}
