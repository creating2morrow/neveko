//! External authorization module via JWPs

use crate::{
    db::{
        self,
        DATABASE_LOCK,
    },
    error::NevekoError,
    monero,
    reqres,
    utils,
};
use kn0sys_lmdb_rs::MdbError;
use log::{
    error,
    info,
};
use rocket::{
    http::Status,
    outcome::Outcome,
    request,
    request::FromRequest,
    Request,
};
use std::error::Error;

use hmac::{
    Hmac,
    Mac,
};
use jwt::*;
use serde::{
    Deserialize,
    Serialize,
};
use sha2::Sha512;
use std::collections::BTreeMap;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TxProof {
    pub subaddress: String,
    pub confirmations: u64,
    pub hash: String,
    pub message: String,
    pub signature: String,
}

/// Provide neccessary information for contacts to
///
/// provide proof of payment.
pub async fn create_invoice() -> reqres::Invoice {
    info!("creating invoice");
    // create a new subaddress
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let c_address = monero::create_address().await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    let address = c_address.result.address;
    let pay_threshold = utils::get_payment_threshold();
    let conf_threshold = utils::get_conf_threshold();
    reqres::Invoice {
        address,
        conf_threshold,
        pay_threshold,
    }
}

/// Technically the same process as creating a JWT
///
/// except that the claims must contain the information
///
/// necessary to verify the payment. Confirmations cannot
///
/// be above some specified threshold. Setting higher
///
/// payment values and lower confirmations works as a spam
///
/// disincentivizing mechanism.
pub async fn create_jwp(proof: &TxProof) -> String {
    info!("creating jwp");
    // validate the proof
    let c_txp: TxProof = validate_proof(proof).await;
    if c_txp.hash.is_empty() {
        error!("invalid transaction proof");
        return String::new();
    }
    let jwp_secret_key = utils::get_jwp_secret_key().unwrap_or_default();
    let key: Hmac<Sha512> = Hmac::new_from_slice(jwp_secret_key.as_bytes()).expect("hash");
    let header = Header {
        algorithm: AlgorithmType::Hs512,
        ..Default::default()
    };
    let mut claims = BTreeMap::new();
    let address = &proof.subaddress;
    let created = chrono::Utc::now().timestamp();
    let created_str = format!("{}", created);
    let hash = &proof.hash;
    let expire = &format!("{}", utils::get_payment_threshold());
    let message = &proof.message;
    let signature = &proof.signature;
    claims.insert("subaddress", address);
    claims.insert("created", &created_str);
    claims.insert("hash", hash);
    claims.insert("expire", expire);
    claims.insert("message", message);
    claims.insert("signature", signature);
    let token = Token::new(header, claims).sign_with_key(&key);
    String::from(token.expect("expected token").as_str())
}

/// Send transaction proof to contact for JWP generation
pub async fn prove_payment(contact: String, txp: &TxProof) -> Result<reqres::Jwp, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .post(format!("http://{}/prove", contact))
        .json(txp)
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<reqres::Jwp>().await;
            log::debug!("prove payment response: {:?}", res);
            match res {
                Ok(r) => {
                    // cache the jwp for for fts
                    let db = &DATABASE_LOCK;
                    let k = format!("{}-{}", crate::FTS_JWP_DB_KEY, &contact);
                    let _ = db::DatabaseEnvironment::delete(&db.env, &db.handle, k.as_bytes())?;
                    db::write_chunks(
                        &db.env,
                        &db.handle,
                        &k.as_bytes(),
                        &r.jwp.as_bytes().to_vec(),
                    )
                    .map_err(|_| NevekoError::Database(MdbError::Panic))?;
                    Ok(r)
                }
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to prove payment: {:?}", e);
            Ok(Default::default())
        }
    }
}

/// # PaymentProof
///
/// is a JWP (JSON Web Proof) with the contents:
///
/// `subaddress`: a subaddress belonging to this neveko instance
///
/// `created`: UTC timestamp the proof was created.
///           <i>Future use</i> Potential offline payments.
///
/// `expire`: blocks approved for
///         <i>Future use</i>. Potential offline payments.
///
/// `hash`: hash of the payment
///
/// `message`: (optional) default: empty string
///
/// `signature`: validates proof of payment
#[derive(Debug)]
pub struct PaymentProof(String);

impl PaymentProof {
    pub fn get_jwp(self) -> String {
        self.0
    }
}

#[derive(Debug)]
pub enum PaymentProofError {
    Expired,
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PaymentProof {
    type Error = PaymentProofError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let proof = request.headers().get_one("proof");
        match proof {
            Some(proof) => {
                // check validity of address, payment amount and tx confirmations
                let jwp_secret_key = utils::get_jwp_secret_key().unwrap_or_default();
                let key: Hmac<Sha512> = Hmac::new_from_slice(jwp_secret_key.as_bytes()).expect("");
                let jwp: Result<
                    Token<jwt::Header, BTreeMap<std::string::String, std::string::String>, _>,
                    jwt::Error,
                > = proof.verify_with_key(&key);
                return match jwp {
                    Ok(j) => {
                        let claims = j.claims();
                        let subaddress = &claims["subaddress"];
                        let is_valid_subaddress = validate_subaddress(subaddress).await;
                        if !is_valid_subaddress {
                            return Outcome::Error((
                                Status::PaymentRequired,
                                PaymentProofError::Invalid,
                            ));
                        }
                        let hash = &claims["hash"];
                        let message = &claims["message"];
                        let signature = &claims["signature"];
                        // verify proof
                        let txp: TxProof = TxProof {
                            subaddress: String::from(subaddress),
                            hash: String::from(hash),
                            confirmations: 0,
                            message: String::from(message),
                            signature: String::from(signature),
                        };
                        let c_txp = validate_proof(&txp).await;
                        // verify expiration
                        let expire = utils::get_conf_threshold();
                        // TODO(c2m): offline verification from created and expire fields
                        if c_txp.confirmations > expire {
                            return Outcome::Error((
                                Status::Unauthorized,
                                PaymentProofError::Expired,
                            ));
                        }
                        Outcome::Success(PaymentProof(String::from(proof)))
                    }
                    Err(e) => {
                        error!("jwp error: {:?}", e);
                        return Outcome::Error((
                            Status::PaymentRequired,
                            PaymentProofError::Invalid,
                        ));
                    }
                };
            }
            None => Outcome::Error((Status::PaymentRequired, PaymentProofError::Missing)),
        }
    }
}

// TODO(c2m): alternative logic for offline payment validations
//            jwp creation, however, will always require blockchain validation?
//            future validations not so much
async fn validate_proof(txp: &TxProof) -> TxProof {
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    // verify unlock time isn't something funky (e.g. > 20)
    let tx: reqres::XmrRpcGetTxByIdResponse = monero::get_transfer_by_txid(&txp.hash).await;
    let unlock_time = tx.result.transfer.unlock_time;
    let tx_type = tx.result.transfer.r#type;
    let propgated = monero::TransactionType::propagated(tx_type);
    if !propgated {
        monero::close_wallet(&wallet_name, &wallet_password).await;
        return Default::default();
    }
    let p = monero::check_tx_proof(txp).await;
    let cth = utils::get_conf_threshold();
    let pth = utils::get_payment_threshold();
    let lgtm = p.result.good
        && unlock_time < monero::LockTimeLimit::Blocks.value()
        && p.result.confirmations < cth
        && p.result.received >= pth;
    if lgtm {
        monero::close_wallet(&wallet_name, &wallet_password).await;
        return TxProof {
            subaddress: String::from(&txp.subaddress),
            hash: String::from(&txp.hash),
            confirmations: p.result.confirmations,
            message: String::from(&txp.message),
            signature: String::from(&txp.signature),
        };
    }
    monero::close_wallet(&wallet_name, &wallet_password).await;
    Default::default()
}

/// Validate that the subaddress in the proof was
///
/// created by us. TODO(?): Use xmr rpc call `get_address_index`
///
/// for faster lookups (check minor > 0)
async fn validate_subaddress(subaddress: &String) -> bool {
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let m_address = monero::get_address().await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    let all_address = m_address.result.addresses;
    let mut address_list: Vec<String> = Vec::new();
    for s_address in all_address {
        address_list.push(s_address.address);
    }
    address_list.contains(subaddress)
}
