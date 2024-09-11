//! Internal authorization module that uses JWTs

use crate::{
    args,
    db::{self, DATABASE_LOCK},
    models::*,
    monero,
    reqres,
    user,
    utils,
};
use clap::Parser;
use kn0sys_lmdb_rs::MdbError;
use log::{
    debug,
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

use hmac::{
    Hmac,
    Mac,
};
use jwt::*;
use sha2::Sha384;
use std::collections::BTreeMap;

/// Create authorization data to sign and expiration
pub fn create(address: &String) -> Result<Authorization, MdbError> {
    info!("creating auth");
    let aid: String = format!("{}{}", crate::AUTH_DB_KEY, utils::generate_rnd());
    let rnd: String = utils::generate_rnd();
    let created: i64 = chrono::offset::Utc::now().timestamp();
    let token: String = create_token(String::from(address), created);
    let new_auth = Authorization {
        aid,
        created,
        uid: String::new(),
        rnd,
        token,
        xmr_address: String::from(address),
    };
    debug!("insert auth: {:?}", &new_auth);
    let k = &new_auth.aid.as_bytes();
    let v = bincode::serialize(&new_auth).unwrap_or_default();
    let db = &DATABASE_LOCK;
    db::write_chunks(&db.env, &db.handle, k, &v)?;
    Ok(new_auth)
}

/// Authorization lookup for recurring requests
pub fn find(aid: &String) -> Result<Authorization, MdbError> {
    info!("searching for auth: {}", aid);
    let db = &DATABASE_LOCK;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &aid.as_bytes().to_vec())?;
    if r.is_empty() {
        return Err(MdbError::NotFound);
    }
    let result: Authorization = bincode::deserialize(&r[..]).unwrap_or_default();
    Ok(result)
}

/// Update new authorization creation time
fn update_expiration(f_auth: &Authorization, address: &String) -> Result<Authorization, MdbError> {
    info!("modify auth expiration");
    let data = utils::generate_rnd();
    let time: i64 = chrono::offset::Utc::now().timestamp();
    // update time, token and data to sign
    let u_auth = Authorization::update_expiration(
        f_auth,
        time,
        data,
        create_token(String::from(address), time),
    );
    let db = &DATABASE_LOCK;
    let _ = db::DatabaseEnvironment::delete(&db.env, &db.handle, &u_auth.aid.as_bytes().to_vec())?;
    let k = u_auth.aid.as_bytes();
    let v = bincode::serialize(&u_auth).unwrap_or_default();
    db::write_chunks(&db.env, &db.handle, k, &v)?;
    Ok(u_auth)
}

/// Performs the signature verfication against stored auth
pub async fn verify_login(aid: String, uid: String, signature: String) -> Result<Authorization, MdbError> {
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
    let address = m_address.result.address;
    let f_auth: Authorization = find(&aid)?;
    if f_auth.xmr_address.is_empty() {
        error!("auth not found");
        monero::close_wallet(&wallet_name, &wallet_password).await;
        return create(&address);
    }
    let data: String = String::from(&f_auth.rnd);
    let is_valid_sig: bool =
        monero::verify(String::from(&address), data, String::from(&signature)).await;
    let sig_address: String = if is_valid_sig {
        String::from(&address)
    } else {
        utils::ApplicationErrors::LoginError.value()
    };
    if sig_address == utils::ApplicationErrors::LoginError.value() {
        error!("signature validation failed");
        monero::close_wallet(&wallet_name, &wallet_password).await;
        return Ok(f_auth);
    }
    let f_user: User = user::find(&uid)?;
    if f_user.xmr_address.is_empty() {
        info!("creating new user");
        let u: User = user::create(&address)?;
        // update auth with uid
        let u_auth = Authorization::update_uid(f_auth, String::from(&u.uid));
        let db = &DATABASE_LOCK;
        let _ = db::DatabaseEnvironment::delete(&db.env, &db.handle, &u_auth.aid.as_bytes())?;
        let v = bincode::serialize(&u_auth).unwrap_or_default();
        db::write_chunks(&db.env, &db.handle, u_auth.aid.as_bytes(), &v)?;
        monero::close_wallet(&wallet_name, &wallet_password).await;
        Ok(u_auth)
    } else if !f_user.xmr_address.is_empty() {
        info!("returning user");
        let m_access = verify_access(&address, &signature).await?;
        if !m_access {
            monero::close_wallet(&wallet_name, &wallet_password).await;
            return Ok(Default::default());
        }
        monero::close_wallet(&wallet_name, &wallet_password).await;
        return Ok(f_auth);
    } else {
        error!("error creating user");
        monero::close_wallet(&wallet_name, &wallet_password).await;
        return Ok(Default::default());
    }
}

/// Called during auth flow to update data to sign and expiration
async fn verify_access(address: &String, signature: &String) -> Result<bool, MdbError> {
    // look up auth for address
    let f_auth: Authorization = find(address)?;
    if !f_auth.xmr_address.is_empty() {
        // check expiration, generate new data to sign if necessary
        let now: i64 = chrono::offset::Utc::now().timestamp();
        let expiration = get_auth_expiration();
        if now > f_auth.created + expiration {
            update_expiration(&f_auth, address)?;
            return Ok(false);
        }
    }
    // verify signature on the data if not expired
    let data = f_auth.rnd;
    let is_valid_sig: bool =
        monero::verify(String::from(address), data, String::from(signature)).await;
    let sig_address: String = if is_valid_sig {
        String::from(address)
    } else {
        utils::ApplicationErrors::LoginError.value()
    };
    if sig_address == utils::ApplicationErrors::LoginError.value() {
        debug!("signing failed");
        return Ok(false);
    }
    info!("auth verified");
    Ok(true)
}

/// get the auth expiration command line configuration
fn get_auth_expiration() -> i64 {
    let args = args::Args::parse();
    args.token_timeout * 60
}

fn create_token(address: String, created: i64) -> String {
    let jwt_secret_key = utils::get_jwt_secret_key().unwrap_or_default();
    let key: Hmac<Sha384> = Hmac::new_from_slice(jwt_secret_key.as_bytes()).expect("hash");
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };
    let mut claims = BTreeMap::new();
    let expiration = get_auth_expiration() * created;
    claims.insert("address", address);
    claims.insert("expiration", expiration.to_string());
    let token = Token::new(header, claims).sign_with_key(&key);
    String::from(token.expect("expected token").as_str())
}

/// This token is used for internal micro server authentication
#[derive(Debug)]
pub struct BearerToken(String);

impl BearerToken {
    pub fn get_token(self) -> String {
        self.0
    }
}


#[derive(Debug)]
pub enum BearerTokenError {
    Expired,
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken {
    type Error = BearerTokenError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let env = utils::get_release_env();
        let dev = utils::ReleaseEnvironment::Development;
        if env == dev {
            return Outcome::Success(BearerToken(String::new()));
        }
        let token = request.headers().get_one("token");
        let wallet_name = String::from(crate::APP_NAME);
        let wallet_password =
            std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
        monero::open_wallet(&wallet_name, &wallet_password).await;
        let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
        monero::close_wallet(&wallet_name, &wallet_password).await;
        let address = m_address.result.address;
        debug!("{}", address);
        match token {
            Some(token) => {
                // check validity
                let jwt_secret_key = utils::get_jwt_secret_key().unwrap_or_default();
                let key: Hmac<Sha384> = Hmac::new_from_slice(jwt_secret_key.as_bytes()).expect("");
                let jwt: Result<
                    Token<jwt::Header, BTreeMap<std::string::String, std::string::String>, _>,
                    jwt::Error,
                > = token.verify_with_key(&key);
                return match jwt {
                    Ok(j) => {
                        let claims = j.claims();
                        debug!("claim address: {}", claims["address"]);
                        // verify address
                        if claims["address"] != address {
                            return Outcome::Failure((
                                Status::Unauthorized,
                                BearerTokenError::Invalid,
                            ));
                        }
                        // verify expiration
                        let now: i64 = chrono::offset::Utc::now().timestamp();
                        let expire = claims["expiration"].parse::<i64>().unwrap_or(0);
                        if now > expire {
                            return Outcome::Failure((
                                Status::Unauthorized,
                                BearerTokenError::Expired,
                            ));
                        }
                        Outcome::Success(BearerToken(String::from(token)))
                    }
                    Err(_) => Outcome::Failure((Status::Unauthorized, BearerTokenError::Invalid)),
                };
            }
            None => Outcome::Failure((Status::Unauthorized, BearerTokenError::Missing)),
        }
    }
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn find_test_auth(k: &String) -> Result<Authorization, MdbError> {
        let s: db::Interface = db::DatabaseEnvironment::open()?;
        let v = db::DatabaseEnvironment::read(&db.env, &s.handle, &k.as_bytes().to_vec())?;
        let result: Authorization = bincode::deserialize(&v[..]).unwrap_or_default();
        Ok(result)
    }

    fn cleanup(k: &String) -> Result<(), MdbError>{
        let db = &DATABASE_LOCK;
        let _ = db::DatabaseEnvironment::delete(&db.env, &db.handle, k.as_bytes())?;
        Ok(())
    }

    #[test]
    fn create_test() {
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let test_auth = create(&address);
        assert_eq!(test_auth.xmr_address, address);
        cleanup(&test_auth.aid);
    }

    #[test]
    fn find_test() -> Result<(), MdbError> {
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let test_auth = create(&address);
        let aid = String::from(&test_auth.aid);
        let f_auth: Authorization = find_test_auth(&aid);
        assert_ne!(f_auth.xmr_address, address);
        cleanup(&test_auth.aid);
        Ok(())
    }

    #[test]
    fn update_expiration_test() {
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let test_auth = create(&address);
        let aid = String::from(&test_auth.aid);
        let f_auth = find_test_auth(&aid);
        let u_auth = update_expiration(&f_auth, &address);
        assert!(f_auth.created < u_auth.created);
        cleanup(&test_auth.aid);
    }

    #[test]
    fn create_token_test() {
        let test_value = "test";
        let test_jwt = create_token(String::from(test_value), 0);
        let jwt_secret_key = utils::get_jwt_secret_key();
        let key: Hmac<Sha384> = Hmac::new_from_slice(&jwt_secret_key.as_bytes()).expect("");
        let jwt: Result<
            Token<jwt::Header, BTreeMap<std::string::String, std::string::String>, _>,
            jwt::Error,
        > = test_jwt.verify_with_key(&key);
        match jwt {
            Ok(j) => {
                let claims = j.claims();
                let expected = String::from(test_value);
                let actual = String::from(&claims["address"]);
                assert_eq!(expected, actual);
            }
            Err(_) => error!("create_token_test error"),
        }
    }
}
