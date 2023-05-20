use crate::{
    args,
    db,
    models::*,
    monero,
    reqres,
    user,
    utils,
};
use clap::Parser;
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
pub fn create(address: &String) -> Authorization {
    info!("creating auth");
    let aid: String = format!("auth{}", utils::generate_rnd());
    let rnd: String = utils::generate_rnd();
    let created: i64 = chrono::offset::Utc::now().timestamp();
    let token: String = create_token(String::from(address), created);
    let new_auth = Authorization {
        aid,
        created,
        uid: utils::empty_string(),
        rnd,
        token,
        xmr_address: String::from(address),
    };
    let s = db::Interface::open();
    debug!("insert auth: {:?}", &new_auth);
    let k = &new_auth.aid;
    db::Interface::write(&s.env, &s.handle, k, &Authorization::to_db(&new_auth));
    new_auth
}

/// Authorization lookup for recurring requests
pub fn find(aid: &String) -> Authorization {
    info!("searching for auth: {}", aid);
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(aid));
    debug!("auth read: {}", r);
    if r == utils::empty_string() {
        return Default::default();
    }
    Authorization::from_db(String::from(aid), r)
}

/// Update new authorization creation time
fn update_expiration(f_auth: &Authorization, address: &String) -> Authorization {
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
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, &u_auth.aid);
    db::Interface::write(
        &s.env,
        &s.handle,
        &u_auth.aid,
        &Authorization::to_db(&u_auth),
    );
    return u_auth;
}

/// Performs the signature verfication against stored auth
pub async fn verify_login(aid: String, uid: String, signature: String) -> Authorization {
    let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
    let address = m_address.result.address;
    let f_auth: Authorization = find(&aid);
    if f_auth.xmr_address == utils::empty_string() {
        error!("auth not found");
        return create(&address);
    }
    let data: String = String::from(&f_auth.rnd);
    let sig_address: String =
        monero::verify_signature(String::from(&address), data, String::from(&signature)).await;
    if sig_address == utils::ApplicationErrors::LoginError.value() {
        error!("signature validation failed");
        return f_auth;
    }
    let f_user: User = user::find(&uid);
    if f_user.xmr_address == utils::empty_string() {
        info!("creating new user");
        let u: User = user::create(&address);
        // update auth with uid
        let u_auth = Authorization::update_uid(f_auth, String::from(&u.uid));
        let s = db::Interface::open();
        db::Interface::delete(&s.env, &s.handle, &u_auth.aid);
        db::Interface::write(
            &s.env,
            &s.handle,
            &u_auth.aid,
            &Authorization::to_db(&u_auth),
        );
        return u_auth;
    } else if f_user.xmr_address != utils::empty_string() {
        info!("returning user");
        let m_access = verify_access(&address, &signature).await;
        if !m_access {
            return Default::default();
        }
        return f_auth;
    } else {
        error!("error creating user");
        return Default::default();
    }
}

/// Called during auth flow to update data to sign and expiration
async fn verify_access(address: &String, signature: &String) -> bool {
    // look up auth for address
    let f_auth: Authorization = find(address);
    if f_auth.xmr_address != utils::empty_string() {
        // check expiration, generate new data to sign if necessary
        let now: i64 = chrono::offset::Utc::now().timestamp();
        let expiration = get_auth_expiration();
        if now > f_auth.created + expiration {
            update_expiration(&f_auth, address);
            return false;
        }
    }
    // verify signature on the data if not expired
    let data = f_auth.rnd;
    let sig_address: String =
        monero::verify_signature(String::from(address), data, String::from(signature)).await;
    if sig_address == utils::ApplicationErrors::LoginError.value() {
        debug!("signing failed");
        return false;
    }
    info!("auth verified");
    return true;
}

/// get the auth expiration command line configuration
fn get_auth_expiration() -> i64 {
    let args = args::Args::parse();
    args.token_timeout * 60
}

fn create_token(address: String, created: i64) -> String {
    let jwt_secret_key = utils::get_jwt_secret_key();
    let key: Hmac<Sha384> = Hmac::new_from_slice(&jwt_secret_key.as_bytes()).expect("hash");
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
            return Outcome::Success(BearerToken(utils::empty_string()));
        }
        let token = request.headers().get_one("token");
        let m_address: reqres::XmrRpcAddressResponse = monero::get_address().await;
        let address = m_address.result.address;
        debug!("{}", address);
        match token {
            Some(token) => {
                // check validity
                let jwt_secret_key = utils::get_jwt_secret_key();
                let key: Hmac<Sha384> = Hmac::new_from_slice(&jwt_secret_key.as_bytes()).expect("");
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
                        let expire = match claims["expiration"].parse::<i64>() {
                            Ok(n) => n,
                            Err(_) => 0,
                        };
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

    async fn find_test_auth(k: &String) -> Authorization {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let s: db::Interface = db::Interface::async_open().await;
        let v = db::Interface::async_read(&s.env, &s.handle, k).await;
        Authorization::from_db(String::from(k), v)
    }

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
        let test_auth = create(&address);
        assert_eq!(test_auth.xmr_address, address);
        tokio::spawn(async move {
            cleanup(&test_auth.aid).await;
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
        let test_auth = create(&address);
        let aid = String::from(&test_auth.aid);
        tokio::spawn(async move {
            let f_auth: Authorization = find_test_auth(&aid).await;
            assert_ne!(f_auth.xmr_address, address);
            cleanup(&test_auth.aid).await;
        });
        Runtime::shutdown_background(rt);
    }

    #[test]
    fn update_expiration_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let test_auth = create(&address);
        let aid = String::from(&test_auth.aid);
        tokio::spawn(async move {
            let f_auth = find_test_auth(&aid).await;
            let u_auth = update_expiration(&f_auth, &address);
            assert!(f_auth.created < u_auth.created);
            cleanup(&test_auth.aid).await;
        });
        Runtime::shutdown_background(rt);
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
