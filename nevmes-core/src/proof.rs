use crate::{monero, reqres, utils};
use log::{error, info};
use std::error::Error;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::{request, Request};

use hmac::{Hmac, Mac};
use jwt::*;
use serde::{Deserialize, Serialize};
use sha2::Sha512;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct TxProof {
    pub address: String,
    pub confirmations: u64,
    pub hash: String,
    pub message: String,
    pub signature: String,
}

impl Default for TxProof {
    fn default() -> Self {
        TxProof {
            address: utils::empty_string(),
            confirmations: 0,
            hash: utils::empty_string(),
            message: utils::empty_string(),
            signature: utils::empty_string(),
        }
    }
}

pub async fn create_invoice() -> reqres::Invoice {
    info!("creating invoice");
    let m_address = monero::get_address().await;
    let address = m_address.result.address;
    let pay_threshold = utils::get_payment_threshold();
    let conf_threshold = utils::get_conf_threshold();
    reqres::Invoice { address, conf_threshold, pay_threshold }
}

pub async fn create_jwp(proof: &TxProof) -> String {
    info!("creating jwp");
    // validate the proof
    let c_txp: TxProof = validate_proof(proof).await;
    if c_txp.confirmations == 0 {
        return utils::empty_string();
    }
    let jwp_secret_key = utils::get_jwp_secret_key();
    let key: Hmac<Sha512> = Hmac::new_from_slice(&jwp_secret_key.as_bytes()).expect("hash");
    let header = Header {
        algorithm: AlgorithmType::Hs512,
        ..Default::default()
    };
    let mut claims = BTreeMap::new();
    let address = &proof.address;
    let hash = &proof.hash;
    let expire = &format!("{}", utils::get_payment_threshold());
    let message = &proof.message;
    let signature = &proof.signature;
    claims.insert("address", address);
    claims.insert("hash", hash);
    claims.insert("expire", expire);
    claims.insert("message", message);
    claims.insert("signature", signature);
    let token = Token::new(header, claims).sign_with_key(&key);
    String::from(token.expect("expected token").as_str())
}

/// Send transaction proof to contact for JWP generation
pub async fn prove_payment(contact: String, txp: &TxProof) -> Result<reqres::Jwp, Box<dyn Error>> {
    // TODO(c2m): Error handling for http 402 status
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?.post(format!("http://{}/prove", contact)).json(txp).send().await {
        Ok(response) => {
            let res = response.json::<reqres::Jwp>().await;
            log::debug!("prove payment response: {:?}", res);
            match res {
                Ok(r) => {
                    Ok(r)
                },
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
/// address: this server's xmr address
/// 
/// hash: hash of the payment
/// 
/// message: (optional) default: empty string
/// 
/// signature: validates proof of payment
#[derive(Debug)]
pub struct PaymentProof(String);

impl PaymentProof { pub fn get_jwp(self) -> String { self.0 } }

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
        let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
        let nevmes_address = m_address.result.address;
        match proof {
            Some(proof) => {
                // check validity of address, payment amount and tx confirmations
                let jwp_secret_key = utils::get_jwp_secret_key();
                let key: Hmac<Sha512> = Hmac::new_from_slice(&jwp_secret_key.as_bytes()).expect("");
                let jwp: Result<
                    Token<jwt::Header, BTreeMap<std::string::String, std::string::String>, _>,
                    jwt::Error,
                > = proof.verify_with_key(&key);
                return match jwp {
                    Ok(j) => {
                        let claims = j.claims();
                        let address = &claims["address"];
                        if address != &nevmes_address {
                            return Outcome::Failure((
                                Status::PaymentRequired,
                                PaymentProofError::Invalid,
                            ));
                        }
                        let hash = &claims["hash"];
                        let message = &claims["message"];
                        let signature = &claims["signature"];
                        // verify proof
                        let txp: TxProof = TxProof {
                            address: String::from(address),
                            hash: String::from(hash),
                            confirmations: 0,
                            message: String::from(message),
                            signature: String::from(signature),
                        };
                        let c_txp = validate_proof(&txp).await;
                        if c_txp.confirmations == 0 {
                            return Outcome::Failure((
                                Status::PaymentRequired,
                                PaymentProofError::Invalid,
                            ));
                        }
                        // verify expiration
                        let expire = utils::get_conf_threshold();
                        if c_txp.confirmations > expire {
                            return Outcome::Failure((
                                Status::Unauthorized,
                                PaymentProofError::Expired,
                            ));
                        }
                        Outcome::Success(PaymentProof(String::from(proof)))
                    }
                    Err(e) => {
                        error!("jwp error: {:?}", e);
                        return Outcome::Failure((Status::PaymentRequired, PaymentProofError::Invalid));
                    }
                };
            }
            None => Outcome::Failure((Status::PaymentRequired, PaymentProofError::Missing)),
        }
    }
}

async fn validate_proof(txp: &TxProof) -> TxProof {
    // verify unlock time isn't something funky (e.g. > 20)
    let tx: reqres::XmrRpcGetTxByIdResponse = monero::get_transfer_by_txid(&txp.hash).await;
    let unlock_time = tx.result.transfer.unlock_time;
    let p = monero::check_tx_proof(txp).await;
    let cth = utils::get_conf_threshold();
    let pth = utils::get_payment_threshold();
    let lgtm = p.result.good && !p.result.in_pool 
        && unlock_time < monero::LockTimeLimit::Blocks.value()
        && p.result.confirmations < cth && p.result.received >= pth;
    if lgtm {
        return TxProof {
            address: String::from(&txp.address),
            hash: String::from(&txp.hash),
            confirmations: p.result.confirmations,
            message: String::from(&txp.message),
            signature: String::from(&txp.signature)
        }
    }
    Default::default()
}
