// Message repo/service layer
use crate::{
    contact,
    db,
    gpg,
    i2p,
    models::*,
    reqres,
    utils,
};
use log::{
    debug,
    error,
    info,
};
use reqwest::StatusCode;
use rocket::serde::json::Json;
use std::error::Error;

/// Create a new message
pub async fn create(m: Json<Message>, jwp: String) -> Message {
    let f_mid: String = format!("m{}", utils::generate_rnd());
    info!("creating message: {}", &f_mid);
    let created = chrono::offset::Utc::now().timestamp();
    // get contact public gpg key and encrypt the message
    debug!("sending message: {:?}", &m);
    let e_body = gpg::encrypt(String::from(&m.to), &m.body).unwrap_or(Vec::new());
    let new_message = Message {
        mid: String::from(&f_mid),
        uid: String::from(&m.uid),
        from: i2p::get_destination(),
        body: e_body,
        created,
        to: String::from(&m.to),
    };
    debug!("insert message: {:?}", &new_message);
    let s = db::Interface::open();
    let k = &new_message.mid;
    db::Interface::write(&s.env, &s.handle, k, &Message::to_db(&new_message));
    // in order to retrieve all message, write keys to with ml
    let list_key = format!("ml");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
    if r == utils::empty_string() {
        debug!("creating message index");
    }
    let msg_list = [r, String::from(&f_mid)].join(",");
    debug!("writing message index {} for id: {}", msg_list, list_key);
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &msg_list);
    info!("attempting to send message");
    let send = send_message(&new_message, &jwp).await;
    send.unwrap();
    new_message
}

/// Rx message
pub async fn rx(m: Json<Message>) {
    // make sure the message isn't something strange
    let is_valid = validate_message(&m);
    if !is_valid {
        return;
    }
    // don't allow messages from outside the contact list
    let is_in_contact_list = contact::exists(&m.from);
    if !is_in_contact_list {
        return;
    }
    let f_mid: String = format!("m{}", utils::generate_rnd());
    let new_message = Message {
        mid: String::from(&f_mid),
        uid: String::from("rx"),
        from: String::from(&m.from),
        body: m.body.iter().cloned().collect(),
        created: chrono::offset::Utc::now().timestamp(),
        to: String::from(&m.to),
    };
    debug!("insert message: {:?}", &new_message);
    let s = db::Interface::open();
    let k = &new_message.mid;
    db::Interface::write(&s.env, &s.handle, k, &Message::to_db(&new_message));
    // in order to retrieve all message, write keys to with rx
    let list_key = format!("rx");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
    if r == utils::empty_string() {
        debug!("creating message index");
    }
    let msg_list = [r, String::from(&f_mid)].join(",");
    debug!("writing message index {} for {}", msg_list, list_key);
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &msg_list);
}

/// Message lookup
pub fn find(mid: &String) -> Message {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(mid));
    if r == utils::empty_string() {
        error!("message not found");
        return Default::default();
    }
    Message::from_db(String::from(mid), r)
}

/// Message lookup
pub fn find_all() -> Vec<Message> {
    let i_s = db::Interface::open();
    let i_list_key = format!("ml");
    let i_r = db::Interface::read(&i_s.env, &i_s.handle, &String::from(i_list_key));
    if i_r == utils::empty_string() {
        error!("message index not found");
    }
    let i_v_mid = i_r.split(",");
    let i_v: Vec<String> = i_v_mid.map(|s| String::from(s)).collect();
    let mut messages: Vec<Message> = Vec::new();
    for m in i_v {
        let message: Message = find(&m);
        if message.mid != utils::empty_string() {
            messages.push(message);
        }
    }
    let o_list_key = format!("rx");
    let o_s = db::Interface::open();
    let o_r = db::Interface::read(&o_s.env, &o_s.handle, &String::from(o_list_key));
    if o_r == utils::empty_string() {
        error!("message index not found");
    }
    let o_v_mid = o_r.split(",");
    let o_v: Vec<String> = o_v_mid.map(|s| String::from(s)).collect();
    for m in o_v {
        let message: Message = find(&m);
        if message.mid != utils::empty_string() {
            messages.push(message);
        }
    }
    messages
}

/// Tx message
async fn send_message(out: &Message, jwp: &str) -> Result<(), Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();

    // check if the contact is online
    let is_online: bool = is_contact_online(&out.to, String::from(jwp))
        .await
        .unwrap_or(false);
    if is_online {
        return match client?
            .post(format!("http://{}/message/rx", out.to))
            .header("proof", jwp)
            .json(&out)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                debug!("send response: {:?}", status.as_str());
                if status == StatusCode::OK || status == StatusCode::PAYMENT_REQUIRED {
                    remove_from_fts(String::from(&out.mid));
                    return Ok(());
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                error!("failed to send message due to: {:?}", e);
                Ok(())
            }
        };
    } else {
        send_to_retry(String::from(&out.mid)).await;
        Ok(())
    }
}

/// Returns decrypted hex string of the encrypted message
pub fn decrypt_body(mid: String) -> reqres::DecryptedMessageBody {
    let m = find(&mid);
    let d = gpg::decrypt(&mid, &m.body).unwrap();
    let body = hex::encode(d);
    reqres::DecryptedMessageBody { mid, body }
}

/// Message deletion
pub fn delete(mid: &String) {
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, &String::from(mid));
}

/// ping the contact health check over i2p
async fn is_contact_online(contact: &String, jwp: String) -> Result<bool, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .get(format!("http://{}/xmr/rpc/version", contact))
        .header("proof", jwp)
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::XmrRpcVersionResponse>().await;
            debug!("check is contact online by version response: {:?}", res);
            match res {
                Ok(r) => {
                    if r.result.version != 0 {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                _ => Ok(false),
            }
        }
        Err(e) => {
            error!("failed to send message due to: {:?}", e);
            Ok(false)
        }
    }
}

/// stage message for async retry
async fn send_to_retry(mid: String) {
    info!("sending {} to fts", &mid);
    let s = db::Interface::open();
    // in order to retrieve FTS (failed-to-send), write keys to db with fts
    let list_key = format!("fts");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
    if r == utils::empty_string() {
        debug!("creating fts message index");
    }
    let mut msg_list = [String::from(&r), String::from(&mid)].join(",");
    // don't duplicate message ids in fts
    if String::from(&r).contains(&String::from(&mid)) {
        msg_list = r;
    }
    debug!(
        "writing fts message index {} for id: {}",
        msg_list, list_key
    );
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &msg_list);
    // restart fts if not empty
    let list_key = format!("fts");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
    let v_mid = r.split(",");
    let v: Vec<String> = v_mid.map(|s| String::from(s)).collect();
    debug!("fts contents: {:#?}", v);
    let cleared = is_fts_clear(r);
    if !cleared {
        debug!("restarting fts");
        utils::restart_retry_fts();
    }
}

/// clear fts message from index
fn remove_from_fts(mid: String) {
    info!("removing id {} from fts", &mid);
    let s = db::Interface::open();
    // in order to retrieve FTS (failed-to-send), write keys to with fts
    let list_key = format!("fts");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
    if r == utils::empty_string() {
        debug!("fts is empty");
    }
    let pre_v_fts = r.split(",");
    let v: Vec<String> = pre_v_fts
        .map(|s| {
            if s != &mid {
                String::from(s)
            } else {
                utils::empty_string()
            }
        })
        .collect();
    let msg_list = v.join(",");
    debug!(
        "writing fts message index {} for id: {}",
        msg_list, list_key
    );
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &msg_list);
}

/// Triggered on app startup, retries to send fts every minute
///
/// FTS thread terminates when empty and gets restarted on the next
///
/// failed-to-send message.
pub async fn retry_fts() {
    let tick: std::sync::mpsc::Receiver<()> = schedule_recv::periodic_ms(60000);
    loop {
        debug!("running retry failed-to-send thread");
        tick.recv().unwrap();
        let s = db::Interface::open();
        let list_key = format!("fts");
        let r = db::Interface::read(&s.env, &s.handle, &String::from(list_key));
        if r == utils::empty_string() {
            info!("fts message index not found");
            break; // terminate fts if no message to send
        }
        let v_mid = r.split(",");
        let v: Vec<String> = v_mid.map(|s| String::from(s)).collect();
        debug!("fts contents: {:#?}", v);
        let cleared = is_fts_clear(r);
        if cleared {
            // index was created but cleared
            info!("terminating retry fts thread");
            db::Interface::delete(&s.env, &s.handle, "fts");
            break;
        }
        for m in v {
            let message: Message = find(&m);
            if message.mid != utils::empty_string() {
                let s = db::Interface::open();
                // get jwp from db
                let k = format!("{}-{}", "fts-jwp", &message.to);
                let jwp = db::Interface::read(&s.env, &s.handle, &k);
                if jwp != utils::empty_string() {
                    send_message(&message, &jwp).await.unwrap();
                } else {
                    error!("not jwp found for fts id: {}", &message.mid);
                }
            }
        }
    }
}

/// check message field lengths to prevent db spam
fn validate_message(j: &Json<Message>) -> bool {
    info!("validating message: {}", &j.mid);
    j.mid.len() < utils::string_limit()
        && j.body.len() < utils::message_limit()
        && j.to == i2p::get_destination()
        && j.uid.len() < utils::string_limit()
}

fn is_fts_clear(r: String) -> bool {
    let v_mid = r.split(",");
    let v: Vec<String> = v_mid.map(|s| String::from(s)).collect();
    debug!("fts contents: {:#?}", v);
    v.len() >= 2 && v[v.len() - 1] == utils::empty_string() && v[0] == utils::empty_string()
}
