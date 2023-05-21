// User repo/service layer
use crate::{
    db,
    models::*,
    utils,
};
use log::{
    debug,
    error,
    info,
};
use rocket::serde::json::Json;

// This module is only used for remote access

/// Create a new user
pub fn create(address: &String) -> User {
    let f_uid: String = format!("u{}", utils::generate_rnd());
    let new_user = User {
        uid: String::from(&f_uid),
        xmr_address: String::from(address),
        name: utils::empty_string(),
    };
    debug!("insert user: {:?}", &new_user);
    let s = db::Interface::open();
    let k = &new_user.uid;
    db::Interface::write(&s.env, &s.handle, k, &User::to_db(&new_user));
    new_user
}

/// User lookup
pub fn find(uid: &String) -> User {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(uid));
    if r == utils::empty_string() {
        error!("user not found");
        return Default::default();
    }
    User::from_db(String::from(uid), r)
}

/// Modify user - not implemented
fn _modify(u: Json<User>) -> User {
    info!("modify user: {}", u.uid);
    let f_cust: User = find(&u.uid);
    if f_cust.uid == utils::empty_string() {
        error!("user not found");
        return Default::default();
    }
    let u_user = User::update(f_cust, String::from(&u.name));
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, &u_user.uid);
    db::Interface::write(&s.env, &s.handle, &u_user.uid, &User::to_db(&u_user));
    todo!()
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
        let test_user = create(&address);
        tokio::spawn(async move {
            let s = db::Interface::async_open().await;
            let r = db::Interface::async_read(&s.env, &s.handle, &test_user.uid).await;
            let id = String::from(&test_user.uid);
            let cleanup_id = String::from(&test_user.uid);
            let expected_user = User::from_db(id, r);
            assert_eq!(test_user.xmr_address, expected_user.xmr_address);
            cleanup(&cleanup_id).await;
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
        let expected_user = User {
            xmr_address: address,
            ..Default::default()
        };
        tokio::spawn(async move {
            let s = db::Interface::async_open().await;
            db::Interface::async_write(&s.env, &s.handle, k, &User::to_db(&expected_user)).await;
            let actual_user: User = find(&String::from(k));
            assert_eq!(expected_user.xmr_address, actual_user.xmr_address);
            cleanup(&String::from(k)).await;
        });
        Runtime::shutdown_background(rt);
    }
}
