// Contact repo/service layer
use crate::{db, gpg, models::*, utils, reqres, monero, i2p};
use rocket::serde::json::Json;
use log::{debug, error, info};
use std::error::Error;

/// Create a new contact
pub async fn create(c: &Json<Contact>) -> Contact {
    let f_cid: String = format!("c{}", utils::generate_rnd());
    info!("creating contact: {}", f_cid);
    let new_contact = Contact {
        cid: String::from(&f_cid),
        gpg_key: c.gpg_key.iter().cloned().collect(),
        i2p_address: String::from(&c.i2p_address),
        xmr_address: String::from(&c.xmr_address),
    };
    let is_valid = validate_contact(c).await;
    if !is_valid {
        return Default::default();
    }
    let import = c.gpg_key.iter().cloned().collect();
    gpg::import_key(String::from(&f_cid), import).unwrap();
    debug!("insert contact: {:?}", &new_contact);
    let s = db::Interface::open();
    let k = &new_contact.cid;
    db::Interface::write(&s.env, &s.handle, k, &Contact::to_db(&new_contact));
    // in order to retrieve all contact, write keys to with cl
    let list_key = format!("cl");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
    if r == utils::empty_string() {
        debug!("creating contact index");
    }
    let msg_list = [r, String::from(&f_cid)].join(",");
    debug!("writing contact index {} for key {}", msg_list, list_key);
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &msg_list);
    new_contact
}

/// Contact lookup
pub fn find(cid: &String) -> Contact {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(cid));
    if r == utils::empty_string() {
        error!("contact not found");
        return Default::default()
    }
    Contact::from_db(String::from(cid), r)
}

/// All contact lookup
pub fn find_all() -> Vec<Contact> {
    let s = db::Interface::open();
    let list_key = format!("cl");
    let r = db::Interface::read(&s.env, &s.handle, &String::from(list_key));
    if r == utils::empty_string() {
        error!("contact index not found");
        return Default::default()
    }
    let v_cid = r.split(",");
    let v: Vec<String> = v_cid.map(|s| String::from(s)).collect();
    let mut contacts: Vec<Contact> = Vec::new();
    for id in v {
        if id != utils::empty_string() {
            let contact: Contact = find(&id);
            contacts.push(contact);
        }
    }
    contacts
}

async fn validate_contact(j: &Json<Contact>) -> bool {
    info!("validating contact: {}", &j.cid);
    let validate_address = monero::validate_address(&j.xmr_address).await;
    j.cid.len() < utils::string_limit()
    && j.i2p_address.len() < utils::string_limit()
    && j.i2p_address.contains(".b32.i2p")
    && j.gpg_key.len() < utils::gpg_key_limit()
    && validate_address.result.valid
}

/// Send our information
pub async fn share() -> Contact {
    let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
    let gpg_key = gpg::export_key().unwrap_or(Vec::new());
    let i2p_address = i2p::get_destination();
    let xmr_address = m_address.result.address;
    Contact { cid: utils::empty_string(),gpg_key,i2p_address,xmr_address }
}

pub fn exists(from: &String) -> bool {
    let all = find_all();
    let mut addresses: Vec<String> = Vec::new();
    for c in all { addresses.push(c.i2p_address); }
    return addresses.contains(from);
}

/// Sign for trusted nevmes contacts
/// 
/// UI/UX should have some prompt about the implication of trusting keys
/// 
/// however that is beyond the scope of this app. nevmes assumes contacts
/// 
/// using the app already have some level of knowledge about each other.
/// 
/// Without signing the key message encryption and sending is not possible.
pub fn trust_gpg(key: String) { gpg::sign_key(&key).unwrap(); }

/// Get invoice for jwp creation
pub async fn request_invoice(contact: String) -> Result<reqres::Invoice, Box<dyn Error>> {
    // TODO(c2m): Error handling for http 402 status
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?.get(format!("http://{}/invoice", contact)).send().await {
        Ok(response) => {
            let res = response.json::<reqres::Invoice>().await;
            debug!("invoice request response: {:?}", res);
            match res {
                Ok(r) => {
                    Ok(r)
                },
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to generate invoice due to: {:?}", e);
            Ok(Default::default())
        }
    }
}

/// Send the request to contact to add them
pub async fn add_contact_request(contact: String) -> Result<Contact, Box<dyn Error>> {
    // TODO(c2m): Error handling for http 402 status
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?.get(format!("http://{}/share", contact)).send().await {
        Ok(response) => {
            let res = response.json::<Contact>().await;
            debug!("share response: {:?}", res);
            match res {
                Ok(r) => {
                    Ok(r)
                },
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to fetch contact info due to: {:?}", e);
            Ok(Default::default())
        }
    }
}
