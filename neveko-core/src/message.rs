// Message repo/service layer
use crate::{
    contact,
    db,
    gpg,
    i2p,
    models::*,
    monero,
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

pub const EXCHANGE_MSIG: &str = "exchange";
pub const EXPORT_MSIG: &str = "export";
pub const MAKE_MSIG: &str = "make";
pub const PREPARE_MSIG: &str = "prepare";

#[derive(PartialEq)]
pub enum MessageType {
    Normal,
    Multisig,
}

struct MultisigMessageData {
    info: String,
    sub_type: String,
    orid: String,
}

/*
   TODOs(c2m): 
     - add wallet open before multisig methods
     - API to valid payment and import multisig info
     - API to sign and submit the signed tx set
 */ 

/// Create a new message
pub async fn create(m: Json<Message>, jwp: String, m_type: MessageType) -> Message {
    let rnd = utils::generate_rnd();
    let mut f_mid: String = format!("m{}", &rnd);
    if m_type == MessageType::Multisig {
        f_mid = format!("msig{}", &rnd);
    }
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
    let send = send_message(&new_message, &jwp, m_type).await;
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

/// Parse the multisig message type and info
fn parse_multisig_message(mid: String) -> MultisigMessageData {
    let d: reqres::DecryptedMessageBody = decrypt_body(mid);
    let mut bytes = hex::decode(d.body.into_bytes()).unwrap_or(Vec::new());
    let decoded = String::from_utf8(bytes).unwrap_or(utils::empty_string());
    let values = decoded.split(":");
    let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
    let sub_type: String = v.remove(0);
    let orid: String = v.remove(0);
    let info: String = v.remove(0);
    bytes = Vec::new();
    debug!("zero decryption bytes: {:?}", bytes);
    MultisigMessageData {
        info,
        sub_type,
        orid,
    }
}

/// Rx multisig message
///
/// Upon multisig message receipt the message is automatically
///
/// decrypted for convenience sake. The client must determine which
///
/// .b32.i2p address belongs to the vendor / mediator.
///
/// ### Example
///
/// ```rust
/// // lookup prepare info for vendor
/// use neveko_core::db;
/// let s = db::Interface::open();
/// let key = "prepare-o123-test.b32.i2p";
/// db::Interface::read(&s.env, &s.handle, &key);
/// ```
pub async fn rx_multisig(m: Json<Message>) {
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
    debug!("insert multisig message: {:?}", &new_message);
    let s = db::Interface::open();
    let k = &new_message.mid;
    db::Interface::async_write(&s.env, &s.handle, k, &Message::to_db(&new_message)).await;
    let data: MultisigMessageData = parse_multisig_message(new_message.mid);
    debug!(
        "writing multisig message type {} for order {}",
        &data.sub_type, &data.orid
    );
    // lookup msig message data by {type}-{order id}-{contact .b32.i2p address}
    let msig_key = format!("{}-{}-{}", &data.sub_type, &data.orid, &m.from);
    db::Interface::async_write(&s.env, &s.handle, &msig_key, &data.info).await;
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
async fn send_message(out: &Message, jwp: &str, m_type: MessageType) -> Result<(), Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    let mut url = format!("http://{}/message/rx", out.to);
    if m_type == MessageType::Multisig {
        url = format!("http://{}/message/rx/multisig", out.to)
    }
    // check if the contact is online
    let is_online: bool = is_contact_online(&out.to, String::from(jwp))
        .await
        .unwrap_or(false);
    if is_online {
        return match client?
            .post(url)
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
                    let m_type = if message.mid.contains("misg") {
                        MessageType::Multisig
                    } else {
                        MessageType::Normal
                    };
                    send_message(&message, &jwp, m_type).await.unwrap();
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

/// Encrypts and sends the output from the monero-rpc
///
/// `prepare_multisig_info` method.
pub async fn send_prepare_info(orid: &String, contact: &String) {
    let s = db::Interface::open();
    let prepare_info = monero::prepare_wallet().await;
    let k = format!("{}-{}", "fts-jwp", contact);
    let jwp = db::Interface::read(&s.env, &s.handle, &k);
    let body_str = format!(
        "{}:{}:{}",
        PREPARE_MSIG, orid, &prepare_info.result.multisig_info
    );
    let message: Message = Message {
        body: body_str.into_bytes(),
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    create(j_message, jwp, MessageType::Multisig).await;
}

/// Encrypts and sends the output from the monero-rpc
///
/// `make_multisig_info` method.
pub async fn send_make_info(orid: &String, contact: &String, info: Vec<String>) {
    let s = db::Interface::open();
    let make_info = monero::make_wallet(info).await;
    let k = format!("{}-{}", "fts-jwp", contact);
    let jwp = db::Interface::read(&s.env, &s.handle, &k);
    let body_str = format!("{}:{}:{}", MAKE_MSIG, orid, &make_info.result.multisig_info);
    let message: Message = Message {
        body: body_str.into_bytes(),
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    create(j_message, jwp, MessageType::Multisig).await;
}

/// Encrypts and sends the output from the monero-rpc
///
/// `exchange_multisig_keys` method.
pub async fn send_exchange_info(orid: &String, contact: &String, info: Vec<String>) {
    let s = db::Interface::open();
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    let exchange_info = monero::exchange_multisig_keys(false, info, wallet_password).await;
    let k = format!("{}-{}", "fts-jwp", contact);
    let jwp = db::Interface::read(&s.env, &s.handle, &k);
    let body_str = format!(
        "{}:{}:{}",
        EXCHANGE_MSIG, orid, &exchange_info.result.multisig_info
    );
    let message: Message = Message {
        body: body_str.into_bytes(),
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    create(j_message, jwp, MessageType::Multisig).await;
}

/// Encrypts and sends the output from the monero-rpc
///
/// `export_multisig_info` method.
pub async fn send_export_info(orid: &String, contact: &String) {
    let s = db::Interface::open();
    let exchange_info = monero::export_multisig_info().await;
    let k = format!("{}-{}", "fts-jwp", contact);
    let jwp = db::Interface::read(&s.env, &s.handle, &k);
    let body_str = format!("{}:{}:{}", EXPORT_MSIG, orid, &exchange_info.result.info);
    let message: Message = Message {
        body: body_str.into_bytes(),
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    create(j_message, jwp, MessageType::Multisig).await;
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    async fn cleanup(k: &String) {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let s = db::Interface::async_open().await;
        db::Interface::async_delete(&s.env, &s.handle, k).await;
    }

    #[test]
    fn create_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let body: String = String::from("test body");
        let message = Message {
            body: body.into_bytes(),
            ..Default::default()
        };
        let j_message = utils::message_to_json(&message);
        let jwp = String::from("test-jwp");
        tokio::spawn(async move {
            let test_message = create(j_message, jwp, MessageType::Normal).await;
            let expected: Message = Default::default();
            assert_eq!(test_message.body, expected.body);
            cleanup(&test_message.mid).await;
        });
        Runtime::shutdown_background(rt);
    }

    #[test]
    fn find_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let body: String = String::from("test body");
        let expected_message = Message {
            body: body.into_bytes(),
            ..Default::default()
        };
        let k = "test-key";
        tokio::spawn(async move {
            let s = db::Interface::async_open().await;
            db::Interface::async_write(&s.env, &s.handle, k, &Message::to_db(&expected_message))
                .await;
            let actual_message: Message = find(&String::from(k));
            assert_eq!(expected_message.body, actual_message.body);
            cleanup(&String::from(k)).await;
        });
        Runtime::shutdown_background(rt);
    }

    #[test]
    fn validate_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let body: String = String::from("test body");
        let message = Message {
            body: body.into_bytes(),
            ..Default::default()
        };
        let j_message = utils::message_to_json(&message);
        tokio::spawn(async move {
            // validation should fail
            let is_valid = validate_message(&j_message);
            assert_eq!(is_valid, false);
        });
        Runtime::shutdown_background(rt);
    }
}
