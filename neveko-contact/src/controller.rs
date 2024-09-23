#![allow(non_snake_case)]

use rocket::{
    delete,
    get,
    http::Status,
    post,
    response::status::Custom,
    serde::json::Json,
};

use neveko_core::{
    auth,
    contact,
    models::*,
    proof,
    reqres,
};

/// Add contact
#[post("/", data = "<req_contact>")]
pub async fn add_contact(
    req_contact: Json<Contact>,
    _token: auth::BearerToken,
) -> Custom<Json<Contact>> {
    let res_contact = contact::create(&req_contact).await;
    let u_contact = res_contact.unwrap_or_default();
    if u_contact.cid.is_empty() {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(u_contact))
}

/// Return all contacts
#[get("/")]
pub async fn get_contacts(_token: auth::BearerToken) -> Custom<Json<Vec<Contact>>> {
    let contacts = contact::find_all();
    Custom(Status::Ok, Json(contacts.unwrap_or_default()))
}

/// Delete a contact by CID
#[delete("/remove/<contact>")]
pub async fn remove_contact(
    contact: String,
    _token: auth::BearerToken,
) -> Custom<Json<Vec<Contact>>> {
    let _ = contact::delete(&contact);
    let contacts = contact::find_all();
    Custom(Status::Ok, Json(contacts.unwrap_or_default()))
}

/// prove payment
#[get("/<contact>", data = "<proof_req>")]
pub async fn prove_payment(
    contact: String,
    proof_req: Json<proof::TxProof>,
    _token: auth::BearerToken,
) -> Custom<Json<reqres::Jwp>> {
    let r_jwp = proof::prove_payment(contact, &proof_req).await;
    Custom(Status::Ok, Json(r_jwp.unwrap()))
}
