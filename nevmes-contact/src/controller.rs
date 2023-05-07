use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::{get, post};

use nevmes_core::{auth, contact, models::*, proof, utils, reqres};

/// Add contact
#[post("/", data="<req_contact>")]
pub async fn add_contact
(req_contact: Json<Contact>,_token: auth::BearerToken) -> Custom<Json<Contact>> {
    let res_contact = contact::create(&req_contact).await;
    if res_contact.cid == utils::empty_string() {
        return Custom(Status::BadRequest, Json(Default::default()))
    }
    Custom(Status::Ok, Json(res_contact))
}

/// Return all contacts
#[get("/")]
pub async fn get_contacts
(_token: auth::BearerToken) -> Custom<Json<Vec<Contact>>> {
    let contacts = contact::find_all();
    Custom(Status::Ok, Json(contacts))
}

/// trust contact
#[post("/<key>")]
pub async fn trust_contact
(key: String, _token: auth::BearerToken) -> Status {
    contact::trust_gpg(key);
    Status::Ok
}

/// prove payment
#[get("/<contact>", data="<proof_req>")]
pub async fn prove_payment
(contact: String, proof_req: Json<proof::TxProof>, _token: auth::BearerToken
) -> Custom<Json<reqres::Jwp>> {
    let r_jwp = proof::prove_payment(contact, &proof_req).await;
    Custom(Status::Ok, Json(r_jwp.unwrap()))
}
