// Contact repo/service layer
use crate::{
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
use rocket::serde::json::Json;
use std::error::Error;

/// Environment variable for activating vendor functionality
pub const NEVEKO_VENDOR_ENABLED: &str = "NEVEKO_VENDOR_ENABLED";

/// Create a new contact
pub async fn create(c: &Json<Contact>) -> Contact {
    let f_cid: String = format!("c{}", utils::generate_rnd());
    info!("creating contact: {}", f_cid);
    let new_contact = Contact {
        cid: String::from(&f_cid),
        gpg_key: c.gpg_key.iter().cloned().collect(),
        i2p_address: String::from(&c.i2p_address),
        is_vendor: false,
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
    let contact_list = [r, String::from(&f_cid)].join(",");
    debug!(
        "writing contact index {} for key {}",
        contact_list, list_key
    );
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &contact_list);
    new_contact
}

/// Contact lookup
pub fn find(cid: &String) -> Contact {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(cid));
    if r == utils::empty_string() {
        error!("contact not found");
        return Default::default();
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
        return Default::default();
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
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let validate_address = monero::validate_address(&j.xmr_address).await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    j.cid.len() < utils::string_limit()
        && j.i2p_address.len() < utils::string_limit()
        && j.i2p_address.contains(".b32.i2p")
        && j.gpg_key.len() < utils::gpg_key_limit()
        && validate_address.result.valid
}

/// Send our information
pub async fn share() -> Contact {
    let vendor_env = std::env::var(NEVEKO_VENDOR_ENABLED).unwrap_or(String::from("0"));
    let is_vendor = vendor_env == String::from("1");
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    let gpg_key = gpg::export_key().unwrap_or(Vec::new());
    let i2p_address = i2p::get_destination(None);
    let xmr_address = m_address.result.address;
    Contact {
        cid: utils::empty_string(),
        gpg_key,
        i2p_address,
        is_vendor,
        xmr_address,
    }
}

pub fn exists(from: &String) -> bool {
    let all = find_all();
    let mut addresses: Vec<String> = Vec::new();
    for c in all {
        addresses.push(c.i2p_address);
    }
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
pub fn trust_gpg(key: String) {
    gpg::sign_key(&key).unwrap();
}

/// Get invoice for jwp creation
pub async fn request_invoice(contact: String) -> Result<reqres::Invoice, Box<dyn Error>> {
    // TODO(c2m): Error handling for http 402 status
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

/// Send the request to contact to add them
pub async fn add_contact_request(contact: String) -> Result<Contact, Box<dyn Error>> {
    // TODO(c2m): Error handling for http 402 status
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
            let s = db::Interface::async_open().await;
            db::Interface::async_write(&s.env, &s.handle, k, &Contact::to_db(&expected_contact))
                .await;
            let actual_contact: Contact = find(&String::from(k));
            assert_eq!(expected_contact.xmr_address, actual_contact.xmr_address);
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
