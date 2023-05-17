use rocket::{
    get,
    http::Status,
    post,
    response::status::Custom,
    serde::json::Json,
};

use nevmes_core::{
    contact,
    i2p,
    message,
    models,
    monero,
    proof,
    reqres,
};

// JSON APIs exposed over i2p

/// Get payment API version
///
/// Protected: false
///
/// This also functions as a health check
#[get("/version")]
pub async fn get_version(_jwp: proof::PaymentProof) -> Custom<Json<reqres::XmrRpcVersionResponse>> {
    Custom(Status::Ok, Json(monero::get_version().await))
}

/// If i2p not in the state of rejecting tunnels this will return `open: true`
///
/// Protected: false
///
/// This also functions as a health check
#[get("/status")]
pub async fn get_i2p_status() -> Custom<Json<i2p::ProxyStatus>> {
    Custom(Status::Ok, Json(i2p::check_connection().await))
}

/// Share your contact information
/// TODO(c2m): configurable option to only allow adding after JWP creation
/// Protected: false
#[get("/")]
pub async fn share_contact_info() -> Custom<Json<models::Contact>> {
    let info: models::Contact = contact::share().await;
    Custom(Status::Ok, Json(info))
}

/// Recieve messages here
///
/// Protected: true
#[post("/", data = "<message>")]
pub async fn rx_message(
    _jwp: proof::PaymentProof,
    message: Json<models::Message>,
) -> Custom<Json<models::Message>> {
    message::rx(message).await;
    Custom(Status::Ok, Json(Default::default()))
}

/// invoice generation
///
/// Protected: false
#[get("/")]
pub async fn gen_invoice() -> Custom<Json<reqres::Invoice>> {
    let invoice = proof::create_invoice().await;
    Custom(Status::Ok, Json(invoice))
}

/// jwp generation
///
/// Protected: false
#[post("/", data = "<proof>")]
pub async fn gen_jwp(proof: Json<proof::TxProof>) -> Custom<Json<reqres::Jwp>> {
    let jwp = proof::create_jwp(&proof).await;
    Custom(Status::Ok, Json(reqres::Jwp { jwp }))
}
