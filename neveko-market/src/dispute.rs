use log::{
    debug,
    error,
    info,
};
use neveko_core::{
    db,
    models::*,
    utils,
};
use rocket::serde::json::Json;

/// Create a new dispute
pub fn create(d: Json<Dispute>) -> Dispute {
    let f_did: String = format!("dispute{}", utils::generate_rnd());
    info!("create dispute: {}", &f_did);
    let new_dispute = Dispute {
        did: String::from(&f_did),
        created: chrono::offset::Utc::now().timestamp(),
        orid: String::from(&d.orid),
        tx_set: String::from(&d.tx_set),
    };
    debug!("insert dispute: {:?}", &d);
    let s = db::Interface::open();
    let k = &d.did;
    db::Interface::write(&s.env, &s.handle, k, &Dispute::to_db(&new_dispute));
    new_dispute
}

/// Dispute lookup
pub fn find(did: &String) -> Dispute {
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(did));
    if r == utils::empty_string() {
        error!("dispute not found");
        return Default::default();
    }
    Dispute::from_db(String::from(did), r)
}
