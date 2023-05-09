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

/// Modify user
pub fn modify(u: Json<User>) -> User {
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
    return u_user;
}
