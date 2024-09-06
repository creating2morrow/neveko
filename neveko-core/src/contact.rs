//! contact operations module

use crate::{
    db,
    i2p,
    models::*,
    monero,
    reqres,
    utils,
};
use kn0sys_lmdb_rs::MdbError;
use log::{
    debug,
    error,
    info,
};
use rocket::serde::json::Json;
use std::error::Error;

/// Environment variable for activating vendor functionality
pub const NEVEKO_VENDOR_ENABLED: &str = "NEVEKO_VENDOR_ENABLED";
pub const NEVEKO_VENDOR_MODE_OFF: &str = "0";
pub const NEVEKO_VENDOR_MODE_ON: &str = "1";

/// Create a new contact
pub async fn create(c: &Json<Contact>) -> Result<Contact, MdbError> {
    let f_cid: String = format!("{}{}", crate::CONTACT_DB_KEY, utils::generate_rnd());
    info!("creating contact: {}", f_cid);
    let new_contact = Contact {
        cid: String::from(&f_cid),
        nmpk: String::from(&c.nmpk),
        i2p_address: String::from(&c.i2p_address),
        is_vendor: false,
        xmr_address: String::from(&c.xmr_address),
    };
    let is_valid = validate_contact(c).await;
    if !is_valid {
        log::error!("invalid contact");
        return Ok(Default::default());
    }
    debug!("insert contact: {:?}", &new_contact);
    
    let s = db::DatabaseEnvironment::open()?;
    let k = &new_contact.cid;
    let v = bincode::serialize(&new_contact).unwrap_or_default();
    db::write_chunks(&s.env, &s.handle?, k.as_bytes(), &v);
    // in order to retrieve all contact, write keys to with cl
    let list_key = crate::CONTACT_LIST_DB_KEY;
    let str_lk = String::from(list_key);
    let s = db::DatabaseEnvironment::open()?;
    let lk_bytes = str_lk.as_bytes();
    let r = db::DatabaseEnvironment::read(&s.env, &s.handle?, &lk_bytes.to_vec())?;
    if r.is_empty() {
        debug!("creating contact index");
    }
    let old: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let contact_list = [old, String::from(&f_cid)].join(",");
    debug!(
        "writing contact index {} for key {}",
        contact_list, list_key
    );
    let s = db::DatabaseEnvironment::open()?;
    db::write_chunks(&s.env, &s.handle?, list_key.as_bytes(), &contact_list.as_bytes());
    Ok(new_contact)
}

/// Contact lookup
pub fn find(cid: &String) -> Result<Contact, MdbError> {
    
    let s = db::DatabaseEnvironment::open()?;
    let r = db::DatabaseEnvironment::read(&s.env, &s.handle?, &cid.as_bytes().to_vec())?;
    if r.is_empty() {
        error!("contact not found");
        return Err(MdbError::NotFound);
    }
    let result: Contact = bincode::deserialize(&r[..]).unwrap_or_default();
    Ok(result)
}

/// Contact lookup
pub fn find_by_i2p_address(i2p_address: &String) -> Result<Contact , MdbError> {
    let contacts = find_all()?;
    for c in contacts {
        if c.i2p_address == *i2p_address {
            return Ok(c);
        }
    }
    Err(MdbError::NotFound)
}

/// Contact deletion
pub fn delete(cid: &String) -> Result<(), MdbError> {
    
    let s = db::DatabaseEnvironment::open()?;
    let r = db::DatabaseEnvironment::read(&s.env, &s.handle?, &cid.as_bytes().to_vec())?;
    if r.is_empty() {
        error!("contact not found");
        return Err(MdbError::NotFound);
    }
    let s = db::DatabaseEnvironment::open()?;
    db::DatabaseEnvironment::delete(&s.env, &s.handle?, cid.as_bytes());
    Ok(())
}

/// All contact lookup
pub fn find_all() -> Result<Vec<Contact>, MdbError> {
    info!("looking up all contacts");
    
    let s = db::DatabaseEnvironment::open()?;
    let list_key = crate::CONTACT_LIST_DB_KEY;
    let r = db::DatabaseEnvironment::read(&s.env, &s.handle?, &list_key.as_bytes().to_vec())?;
    if r.is_empty() {
        error!("contact index not found");
        return Err(MdbError::NotFound);
    }
    let str_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let v_cid = str_r.split(",");
    let v: Vec<String> = v_cid.map(String::from).collect();
    let mut contacts: Vec<Contact> = Vec::new();
    for id in v {
        if !id.is_empty() {
            let contact: Contact = find(&id)?;
            contacts.push(contact);
        }
    }
    Ok(contacts)
}

async fn validate_contact(j: &Json<Contact>) -> bool {
    info!("validating contact: {}", &j.cid);
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let validate_address = monero::validate_address(&j.xmr_address).await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    j.cid.len() < utils::string_limit()
        && j.i2p_address.len() < utils::string_limit()
        && j.i2p_address.contains(".b32.i2p")
        && j.nmpk.len() < utils::npmk_limit()
        && validate_address.result.valid
}

/// Send our information
pub async fn share() -> Result<Contact, MdbError> {
    
    let s = db::DatabaseEnvironment::open()?;
    let r = db::DatabaseEnvironment::read(&s.env, &s.handle?, &NEVEKO_VENDOR_ENABLED.as_bytes().to_vec())?;
    let str_r: String = bincode::deserialize(&r[..]).unwrap_or_default();
    let is_vendor = str_r == NEVEKO_VENDOR_MODE_ON;
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    let nmpk = utils::get_nmpk()?;
    let i2p_address = i2p::get_destination(None);
    let xmr_address = m_address.result.address;
    Ok(Contact {
        cid: String::new(),
        nmpk,
        i2p_address,
        is_vendor,
        xmr_address,
    })
}

pub fn exists(from: &String) -> Result<bool, MdbError> {
    let all = find_all()?;
    let mut addresses: Vec<String> = Vec::new();
    for c in all {
        addresses.push(c.i2p_address);
    }
    Ok(addresses.contains(from))
}

/// Get invoice for jwp creation
pub async fn request_invoice(contact: String) -> Result<reqres::Invoice, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .get(format!("http://{}/invoice", contact))
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::Invoice>().await;
            debug!("invoice request response: {:?}", res);
            match res {
                Ok(r) => Ok(r),
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to generate invoice due to: {:?}", e);
            Ok(Default::default())
        }
    }
}

/// Send the request to contact to add them.
pub async fn add_contact_request(contact: String) -> Result<Contact, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .get(format!("http://{}/share", contact))
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<Contact>().await;
            debug!("share response: {:?}", res);
            match res {
                Ok(r) => Ok(r),
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to fetch contact info due to: {:?}", e);
            Ok(Default::default())
        }
    }
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cleanup(k: &String) {
        
        let s = db::DatabaseEnvironment::open();
        db::DatabaseEnvironment::delete(&s.env, &s.handle, k);
    }

    #[test]
    fn create_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let contact = Contact {
            xmr_address: address,
            ..Default::default()
        };
        let j_contact = utils::contact_to_json(&contact);

        tokio::spawn(async move {
            let test_contact = create(&j_contact).await;
            let expected: Contact = Default::default();
            assert_eq!(test_contact.xmr_address, expected.xmr_address);
        });
        Runtime::shutdown_background(rt);
    }

    #[test]
    fn find_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let k = "c123";
        let expected_contact = Contact {
            xmr_address: address,
            ..Default::default()
        };
        tokio::spawn(async move {
            let s = db::DatabaseEnvironment::open().unwrap();
            let v = bincode::serialize(&expected_contact).unwrap_or_default();
            db::write_chunks(&s.env, &s.handle, k.as_bytes(), &v);
            let actual_contact: Contact = find(&String::from(k));
            assert_eq!(expected_contact.xmr_address, actual_contact.xmr_address);
            cleanup(&String::from(k));
        });
        Runtime::shutdown_background(rt);
    }

    #[test]
    fn validate_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let contact = Contact {
            xmr_address: address,
            ..Default::default()
        };
        let j_contact = utils::contact_to_json(&contact);
        tokio::spawn(async move {
            // validation should fail
            let is_valid = validate_contact(&j_contact).await;
            assert_eq!(is_valid, false);
        });
        Runtime::shutdown_background(rt);
    }
}
