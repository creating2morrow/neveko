//! Message processing module

use crate::{
    contact,
    db::{
        self,
        DATABASE_LOCK,
    },
    error::NevekoError,
    i2p,
    models::*,
    monero,
    neveko25519,
    order,
    reqres,
    utils,
};
use kn0sys_lmdb_rs::MdbError;
use log::{
    debug,
    error,
    info,
};
use reqwest::StatusCode;
use rocket::serde::json::Json;
use std::error::Error;

pub const KEX_ONE_MSIG: &str = "kexone";
pub const KEX_TWO_MSIG: &str = "kextwo";
pub const EXPORT_MSIG: &str = "export";
pub const IMPORT_MSIG: &str = "import";
pub const MAKE_MSIG: &str = "make";
pub const PREPARE_MSIG: &str = "prepare";
pub const SIGN_MSIG: &str = "sign";
pub const TXSET_MSIG: &str = "txset";
pub const VALID_MSIG_MSG_LENGTH: usize = 3;

#[derive(PartialEq)]
pub enum MessageType {
    Normal,
    Multisig,
}

#[derive(Default)]
struct MultisigMessageData {
    info: String,
    sub_type: String,
    orid: String,
}

/// Create a new message
pub async fn create(
    m: Json<Message>,
    jwp: String,
    m_type: MessageType,
) -> Result<Message, NevekoError> {
    let rnd = utils::generate_rnd();
    let mut f_mid: String = format!("{}{}", crate::MESSAGE_DB_KEY, &rnd);
    if m_type == MessageType::Multisig {
        f_mid = format!("{}{}", crate::MSIG_MESSAGE_DB_KEY, &rnd);
    }
    info!("creating message: {}", &f_mid);
    let created = chrono::offset::Utc::now().timestamp();
    // get contact public message key and encipher the message
    debug!("sending message: {:?}", &m);
    let contact: Contact = contact::find(&m.to).map_err(|_| NevekoError::Message)?;
    let hex_nmpk: String = contact.nmpk;
    let encipher = Some(String::from(neveko25519::ENCIPHER));
    let e_body = neveko25519::cipher(&hex_nmpk, String::from(&m.body), encipher).await;
    let new_message = Message {
        mid: String::from(&f_mid),
        uid: String::from(&m.uid),
        from: i2p::get_destination(i2p::ServerTunnelType::App)?,
        body: e_body,
        created,
        to: String::from(&m.to),
    };
    debug!("insert message: {:?}", &new_message);
    let db = &DATABASE_LOCK;
    let k = &new_message.mid;
    let message = bincode::serialize(&new_message).unwrap_or_default();
    db::write_chunks(&db.env, &db.handle, k.as_bytes(), &message)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    // in order to retrieve all message, write keys to with ml
    let list_key = crate::MESSAGE_LIST_DB_KEY;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        debug!("creating message index");
    }
    let d_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let msg_list = [d_r, String::from(&f_mid)].join(",");
    let s_msg_list = bincode::serialize(&msg_list).unwrap_or_default();
    debug!("writing message index {} for id: {}", msg_list, list_key);
    let db = &DATABASE_LOCK;
    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_msg_list)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    info!("attempting to send message");
    let send = send_message(&new_message, &jwp, m_type).await;
    send.unwrap();
    Ok(new_message)
}

/// Rx message
pub async fn rx(m: Json<Message>) -> Result<(), NevekoError> {
    info!("rx from: {}", &m.from);
    // make sure the message isn't something strange
    let is_valid = validate_message(&m);
    if !is_valid {
        error!("invalid contact");
        return Err(NevekoError::Contact);
    }
    // don't allow messages from outside the contact list
    let is_in_contact_list = contact::exists(&m.from).map_err(|_| NevekoError::Contact)?;
    if !is_in_contact_list {
        error!("not a mutual contact");
        return Err(NevekoError::Contact);
    }
    let f_mid: String = format!("{}{}", crate::MESSAGE_DB_KEY, utils::generate_rnd());
    let new_message = Message {
        mid: String::from(&f_mid),
        uid: String::from(crate::RX_MESSAGE_DB_KEY),
        from: String::from(&m.from),
        body: String::from(&m.body),
        created: chrono::offset::Utc::now().timestamp(),
        to: String::from(&m.to),
    };
    debug!("insert message: {:?}", &new_message);
    let db = &DATABASE_LOCK;
    let k = &new_message.mid;
    let message = bincode::serialize(&new_message).unwrap_or_default();
    db::write_chunks(&db.env, &db.handle, k.as_bytes(), &message)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    // in order to retrieve all message, write keys to with rx
    let list_key = crate::RX_MESSAGE_DB_KEY;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        debug!("creating message index");
    }
    let old: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let msg_list = [old, String::from(&f_mid)].join(",");
    let s_msg_list = bincode::serialize(&msg_list).unwrap_or_default();
    debug!("writing message index {} for {}", msg_list, list_key);
    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_msg_list)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(())
}

/// Parse the multisig message type and info
async fn parse_multisig_message(mid: String) -> Result<MultisigMessageData, NevekoError> {
    let d: reqres::DecipheredMessageBody = decipher_body(mid).await?;
    let mut bytes = hex::decode(d.body.into_bytes()).unwrap_or_default();
    let decoded = String::from_utf8(bytes).unwrap_or(String::new());
    let values = decoded.split(":");
    let mut v: Vec<String> = values.map(String::from).collect();
    if v.len() != VALID_MSIG_MSG_LENGTH {
        error!("invalid msig message length");
        return Err(NevekoError::Message);
    }
    let sub_type: String = v.remove(0);
    let orid: String = v.remove(0);
    let a_info: String = v.remove(0);
    let mut info = String::from(&a_info);
    // on prepare info and txset msig messages customer only receives one set of
    // info
    if !v.is_empty() {
        let b_info: String = v.remove(0);
        info = format!("{}:{}", a_info, b_info);
    }
    bytes = Vec::new();
    debug!("zero decipher bytes: {:?}", bytes);
    Ok(MultisigMessageData {
        info,
        sub_type,
        orid,
    })
}

/// Rx multisig message
///
/// Upon multisig message receipt the message is automatically
///
/// decipher for convenience sake. The client must determine which
///
/// .b32.i2p address belongs to the vendor / adjudicator.
///
/// The result should be a string that needs to be decomposed into a
///
/// vector.
/// ### Example
///
/// ```rust
/// // lookup prepare info for vendor
/// use neveko_core::db::*;
/// let db = &DATABASE_LOCK;
/// let key = "prepare-o123-test.b32.i2p";
/// let info_str = DatabaseEnvironment::read(&db.env, &db.handle, &key.as_bytes().to_vec());
/// ```
pub async fn rx_multisig(m: Json<Message>) -> Result<(), NevekoError> {
    info!("rx multisig from: {}", &m.from);
    // make sure the message isn't something strange
    let is_valid = validate_message(&m);
    if !is_valid {
        error!("invalid contact");
        return Err(NevekoError::Contact);
    }
    // don't allow messages from outside the contact list
    let is_in_contact_list = contact::exists(&m.from).map_err(|_| NevekoError::Contact)?;
    if !is_in_contact_list {
        error!("not a mutual contact");
        return Err(NevekoError::Contact);
    }
    let f_mid: String = format!("msig{}", utils::generate_rnd());
    let new_message = Message {
        mid: String::from(&f_mid),
        uid: String::from(crate::RX_MESSAGE_DB_KEY),
        from: String::from(&m.from),
        body: String::from(&m.body),
        created: chrono::offset::Utc::now().timestamp(),
        to: String::from(&m.to),
    };
    let db = &DATABASE_LOCK;
    let k = &new_message.mid;
    let message = bincode::serialize(&new_message).unwrap_or_default();

    db::write_chunks(&db.env, &db.handle, k.as_bytes(), &message)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    // in order to retrieve all msig messages, write keys to with msigl
    let list_key = crate::MSIG_MESSAGE_LIST_DB_KEY;
    let db = &DATABASE_LOCK;

    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        debug!("creating msig message index");
    }
    let old: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let msg_list = [old, String::from(&f_mid)].join(",");
    let s_msg_list = bincode::serialize(&msg_list).unwrap_or_default();
    debug!(
        "writing msig message index {} for id: {}",
        msg_list, list_key
    );
    let db = &DATABASE_LOCK;

    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_msg_list)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let data: MultisigMessageData = parse_multisig_message(new_message.mid).await?;
    debug!(
        "writing multisig message type {} for order {}",
        &data.sub_type, &data.orid
    );
    // lookup msig message data by {type}-{order id}-{contact .b32.i2p address}
    // store info as {a_info}:{a_info (optional)}
    let s_msig =
        db::DatabaseEnvironment::open().map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let msig_key = format!("{}-{}-{}", &data.sub_type, &data.orid, &m.from);
    let db = &DATABASE_LOCK;

    db::write_chunks(
        &s_msig.env,
        &db.handle,
        msig_key.as_bytes(),
        data.info.as_bytes(),
    )
    .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(())
}

/// Message lookup()
pub fn find(mid: &String) -> Result<Message, NevekoError> {
    let db = &DATABASE_LOCK;

    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &mid.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        error!("message not found");
        return Err(NevekoError::Message);
    }
    let result: Message = bincode::deserialize(&r[..]).unwrap_or_default();
    Ok(result)
}

/// Message lookup
pub fn find_all() -> Result<Vec<Message>, NevekoError> {
    let db = &DATABASE_LOCK;
    let i_list_key = crate::MESSAGE_LIST_DB_KEY;
    let i_r = db::DatabaseEnvironment::read(&db.env, &db.handle, &i_list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if i_r.is_empty() {
        error!("message index not found");
    }
    let i_r: String = bincode::deserialize(&i_r[..]).unwrap_or_default();
    let i_v_mid = i_r.split(",");
    let i_v: Vec<String> = i_v_mid.map(String::from).collect();
    let mut messages: Vec<Message> = Vec::new();
    for m in i_v {
        let message: Message = find(&m)?;
        if !message.mid.is_empty() {
            messages.push(message);
        }
    }
    let o_list_key = crate::RX_MESSAGE_DB_KEY;
    let o_r = db::DatabaseEnvironment::read(&db.env, &db.handle, &o_list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if o_r.is_empty() {
        error!("message index not found");
    }
    let o_r: String = bincode::deserialize(&o_r[..]).unwrap_or_default();
    let o_v_mid = o_r.split(",");
    let o_v: Vec<String> = o_v_mid.map(String::from).collect();
    for m in o_v {
        let message: Message = find(&m)?;
        if !message.mid.is_empty() {
            messages.push(message);
        }
    }
    Ok(messages)
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
        match client?
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
                    remove_from_fts(String::from(&out.mid))?;
                    Ok(())
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                error!("failed to send message due to: {:?}", e);
                Ok(())
            }
        }
    } else {
        send_to_retry(String::from(&out.mid)).await?;
        Ok(())
    }
}

/// Returns deciphered message
pub async fn decipher_body(mid: String) -> Result<reqres::DecipheredMessageBody, NevekoError> {
    let m = find(&mid)?;
    let contact = contact::find_by_i2p_address(&m.from)?;
    let nmpk = contact.nmpk;
    let message = String::from(&m.body);
    let body = neveko25519::cipher(&nmpk, message, None).await;
    Ok(reqres::DecipheredMessageBody { mid, body })
}

/// Message deletion
pub fn delete(mid: &String) -> Result<(), NevekoError> {
    let db = &DATABASE_LOCK;
    let _ = db::DatabaseEnvironment::delete(&db.env, &db.handle, mid.as_bytes())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(())
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
                    if r.result.version != monero::INVALID_VERSION {
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
async fn send_to_retry(mid: String) -> Result<(), NevekoError> {
    info!("sending {} to fts", &mid);
    let db = &DATABASE_LOCK;
    // in order to retrieve FTS (failed-to-send), write keys to db with fts
    let list_key = crate::FTS_DB_KEY;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        debug!("creating fts message index");
    }
    let i_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let mut msg_list = [String::from(&i_r), String::from(&mid)].join(",");
    let s_msg_list = bincode::serialize(&msg_list).unwrap_or_default();
    // don't duplicate message ids in fts
    if String::from(&i_r).contains(&String::from(&mid)) {
        msg_list = i_r;
    }
    debug!(
        "writing fts message index {} for id: {}",
        msg_list, list_key
    );
    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_msg_list)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    // restart fts if not empty
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let str_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let v_mid = str_r.split(",");
    let v: Vec<String> = v_mid.map(String::from).collect();
    debug!("fts contents: {:#?}", v);
    let cleared = is_fts_clear(str_r);
    if !cleared {
        debug!("restarting fts");
        utils::restart_retry_fts();
    }
    Ok(())
}

/// clear fts message from index
fn remove_from_fts(mid: String) -> Result<(), NevekoError> {
    info!("removing id {} from fts", &mid);
    let db = &DATABASE_LOCK;
    // in order to retrieve FTS (failed-to-send), write keys to with fts
    let list_key = crate::FTS_DB_KEY;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    if r.is_empty() {
        debug!("fts is empty");
    }
    let s_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let pre_v_fts = s_r.split(",");
    let v: Vec<String> = pre_v_fts
        .map(|s| {
            if s != &mid {
                String::from(s)
            } else {
                String::new()
            }
        })
        .collect();
    let msg_list = v.join(",");
    let s_msg_list = bincode::serialize(&msg_list).unwrap_or_default();
    debug!(
        "writing fts message index {} for id: {}",
        msg_list, list_key
    );
    db::write_chunks(&db.env, &db.handle, list_key.as_bytes(), &s_msg_list)
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    Ok(())
}

/// Triggered on app startup, retries to send fts every minute
///
/// FTS thread terminates when empty and gets restarted on the next
///
/// failed-to-send message.
pub async fn retry_fts() -> Result<(), NevekoError> {
    let tick: std::sync::mpsc::Receiver<()> = schedule_recv::periodic_ms(crate::FTS_RETRY_INTERVAL);
    loop {
        debug!("running retry failed-to-send thread");
        tick.recv().unwrap();
        let db = &DATABASE_LOCK;
        let list_key = crate::FTS_DB_KEY;
        let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &list_key.as_bytes().to_vec())
            .map_err(|_| NevekoError::Database(MdbError::Panic))?;
        if r.is_empty() {
            info!("fts message index not found");
            return Err(NevekoError::Database(MdbError::NotFound)); // terminate fts if no message to send
        }
        let s_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
        let v_mid = s_r.split(",");
        let v: Vec<String> = v_mid.map(String::from).collect();
        debug!("fts contents: {:#?}", v);
        let cleared = is_fts_clear(s_r);
        if cleared {
            // index was created but cleared
            info!("terminating retry fts thread");
            let _ =
                db::DatabaseEnvironment::delete(&db.env, &db.handle, &list_key.as_bytes().to_vec())
                    .map_err(|_| NevekoError::Database(MdbError::Panic))?;
            break Err(NevekoError::Database(MdbError::NotFound));
        }
        for m in v {
            let message: Message = find(&m)?;
            if !message.mid.is_empty() {
                // get jwp from db
                let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, &message.to);
                let jwp =
                    db::DatabaseEnvironment::read(&db.env, &db.handle, &k.as_bytes().to_vec())
                        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
                if !jwp.is_empty() {
                    let m_type = if message.mid.contains("msig") {
                        MessageType::Multisig
                    } else {
                        MessageType::Normal
                    };
                    let str_jwp: String = bincode::deserialize(&jwp[..]).unwrap_or_default();
                    send_message(&message, &str_jwp, m_type).await.unwrap();
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
        && j.to == i2p::get_destination(i2p::ServerTunnelType::App).unwrap_or_default()
        && j.uid.len() < utils::string_limit()
}

fn is_fts_clear(r: String) -> bool {
    let v_mid = r.split(",");
    let v: Vec<String> = v_mid.map(String::from).collect();
    debug!("fts contents: {:#?}", v);
    let limit = v.len() <= 1;
    if !limit {
        v.len() >= 2 && v[v.len() - 1].is_empty() && v[0].is_empty()
    } else {
        limit
    }
}

/// Enciphers and sends the output from the monero-rpc
///
/// `prepare_multisig_info` method.
pub async fn send_prepare_info(orid: &String, contact: &String) -> Result<(), NevekoError> {
    let db = &DATABASE_LOCK;
    let wallet_name = String::from(orid);
    let wallet_password = String::new();
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let prepare_info = monero::prepare_wallet().await;
    let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, contact);
    let jwp = db::DatabaseEnvironment::read(&db.env, &db.handle, &k.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let body_str = format!(
        "{}:{}:{}",
        PREPARE_MSIG, orid, &prepare_info.result.multisig_info
    );
    let message: Message = Message {
        body: body_str,
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    monero::close_wallet(orid, &wallet_password).await;
    let str_jwp: String = bincode::deserialize(&jwp[..]).unwrap_or_default();
    create(j_message, str_jwp, MessageType::Multisig).await?;
    Ok(())
}

/// Enciphers and sends the output from the monero-rpc
///
/// `make_multisig_info` method.
pub async fn send_make_info(
    orid: &String,
    contact: &String,
    info: Vec<String>,
) -> Result<(), NevekoError> {
    let db = &DATABASE_LOCK;
    let wallet_name = String::from(orid);
    let wallet_password = String::new();
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let make_info = monero::make_wallet(info).await;
    let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, contact);
    let jwp = db::DatabaseEnvironment::read(&db.env, &db.handle, &k.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let body_str = format!("{}:{}:{}", MAKE_MSIG, orid, &make_info.result.multisig_info);
    let message: Message = Message {
        body: body_str,
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    monero::close_wallet(orid, &wallet_password).await;
    let str_jwp: String = bincode::deserialize(&jwp[..]).unwrap_or_default();
    create(j_message, str_jwp, MessageType::Multisig).await?;
    Ok(())
}

/// Enciphers and sends the output from the monero-rpc
///
/// `exchange_multisig_keys` method.
pub async fn send_exchange_info(
    orid: &String,
    contact: &String,
    info: Vec<String>,
    kex_init: bool,
) -> Result<(), NevekoError> {
    let db = &DATABASE_LOCK;
    let wallet_name = String::from(orid);
    let wallet_password = String::new();
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let exchange_info = monero::exchange_multisig_keys(false, info, &wallet_password).await;
    let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, contact);
    let jwp = db::DatabaseEnvironment::read(&db.env, &db.handle, &k.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let mut body_str = format!(
        "{}:{}:{}",
        KEX_ONE_MSIG, orid, &exchange_info.result.multisig_info
    );
    if !kex_init {
        body_str = format!(
            "{}:{}:{}",
            KEX_TWO_MSIG, orid, &exchange_info.result.address
        );
    }
    let message: Message = Message {
        body: body_str,
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    monero::close_wallet(orid, &wallet_password).await;
    let str_jwp: String = bincode::deserialize(&jwp[..]).unwrap_or_default();
    create(j_message, str_jwp, MessageType::Multisig).await?;
    Ok(())
}

/// Enciphers and sends the output from the monero-rpc
///
/// `export_multisig_info` method.
pub async fn send_export_info(orid: &String, contact: &String) -> Result<(), NevekoError> {
    let db = &DATABASE_LOCK;
    let wallet_name = String::from(orid);
    let wallet_password = String::new();
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let exchange_info = monero::export_multisig_info().await;
    let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, contact);
    let jwp = db::DatabaseEnvironment::read(&db.env, &db.handle, &k.as_bytes().to_vec())
        .map_err(|_| NevekoError::Database(MdbError::Panic))?;
    let body_str = format!("{}:{}:{}", EXPORT_MSIG, orid, &exchange_info.result.info);
    let message: Message = Message {
        body: body_str,
        created: chrono::Utc::now().timestamp(),
        to: String::from(contact),
        ..Default::default()
    };
    let j_message: Json<Message> = utils::message_to_json(&message);
    monero::close_wallet(orid, &wallet_password).await;
    let str_jwp: String = bincode::deserialize(&jwp[..]).unwrap_or_default();
    create(j_message, str_jwp, MessageType::Multisig).await?;
    Ok(())
}

/// The customer or vendor (dispute only) needs to export
///
/// multisig info after funding. Once the info is imported
///
/// successfully the order needs to be updated to `MultisigComplete`.
pub async fn send_import_info(orid: &String, info: &Vec<String>) -> Result<(), NevekoError> {
    let wallet_name = String::from(orid);
    let wallet_password = String::new();
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let pre_import = monero::import_multisig_info(info.to_vec()).await;
    monero::close_wallet(orid, &wallet_password).await;
    if pre_import.result.n_outputs == 0 {
        error!("unable to import multisig info for order: {}", orid);
        return Err(NevekoError::Database(MdbError::Panic))?;
    }
    let mut old_order = order::find(orid)?;
    let status = order::StatusType::MulitsigComplete.value();
    old_order.status = String::from(&status);
    let j_old_order = Json(old_order);
    order::modify(j_old_order)?;
    debug!("order: {} updated to: {}", orid, status);
    Ok(())
}

/// Customer begins multisig orchestration by requesting the prepare info
///
/// from the adjudicator and the vendor. In response they create an enciphered
///
/// multisig message with the requested data. Customer manages multisig by
///
/// injecting...
async fn trigger_msig_info_request(
    contact: String,
    jwp: String,
    request: reqres::MultisigInfoRequest,
) -> Result<Order, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .post(format!("http://{}/multisig/info", contact))
        .header("proof", jwp)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<Order>().await;
            debug!("{} info for order response: {:?}", &request.msig_type, res);
            match res {
                Ok(r) => Ok(r),
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!(
                "failed to {} info for order due to: {:?}",
                &request.msig_type, e
            );
            Ok(Default::default())
        }
    }
}

/// Deconstruction pass-through so that we can send the request from an async
///
/// channel in the neveko-gui module.
pub async fn d_trigger_msig_info(
    contact: &String,
    jwp: &String,
    request: &reqres::MultisigInfoRequest,
) -> Order {
    let d_contact: String = String::from(contact);
    let d_jwp: String = String::from(jwp);
    let d_request: reqres::MultisigInfoRequest = reqres::MultisigInfoRequest {
        contact: String::from(&request.contact),
        info: request.info.clone(),
        init_adjudicator: request.init_adjudicator,
        kex_init: request.kex_init,
        msig_type: String::from(&request.msig_type),
        orid: String::from(&request.orid),
    };
    let pre = trigger_msig_info_request(d_contact, d_jwp, d_request).await;
    if pre.is_err() {
        log::error!("failed to trigger {} info request", request.msig_type);
        return Default::default();
    }
    pre.unwrap_or(Default::default())
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup(k: &String) -> Result<(), NevekoError> {
        let db = &DATABASE_LOCK;
        let _ = db::DatabaseEnvironment::delete(&db.env, &db.handle, k.as_bytes())
            .map_err(|_| NevekoError::Database(MdbError::Panic))?;
        Ok(())
    }

    #[test]
    fn create_test() -> Result<(), NevekoError> {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let body: String = String::from("test body");
        let message = Message {
            body,
            ..Default::default()
        };
        let j_message = utils::message_to_json(&message);
        let jwp = String::from("test-jwp");
        tokio::spawn(async move {
            let a_test_message = create(j_message, jwp, MessageType::Normal).await;
            let test_message = a_test_message.unwrap_or_default();
            let expected: Message = Default::default();
            assert_eq!(test_message.body, expected.body);
            cleanup(&test_message.mid).unwrap();
        });
        Runtime::shutdown_background(rt);
        Ok(())
    }

    #[test]
    fn find_test() -> Result<(), NevekoError> {
        // run and async cleanup so the test doesn't fail when deleting test data
        let body: String = String::from("test body");
        let expected_message = Message {
            body,
            ..Default::default()
        };
        let k = "test-key";
        let db = &DATABASE_LOCK;
        let message = bincode::serialize(&expected_message).unwrap_or_default();
        db::write_chunks(&db.env, &db.handle, k.as_bytes(), &message)
            .map_err(|_| NevekoError::Database(MdbError::Panic))?;
        let actual_message: Message = find(&String::from(k))?;
        assert_eq!(expected_message.body, actual_message.body);
        cleanup(&String::from(k))?;
        Ok(())
    }

    #[test]
    fn validate_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let body: String = String::from("test body");
        let message = Message {
            body,
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
