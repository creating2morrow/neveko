use rocket::{
    get,
    http::Status,
    post,
    response::status::Custom,
    serde::json::Json,
};

use neveko_core::*;

// JSON APIs exposed over i2p

/// Get payment API version
///
/// Protected: true
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
pub async fn get_i2p_status() -> Custom<Json<i2p::HttpProxyStatus>> {
    let status: i2p::ProxyStatus = i2p::check_connection().await;
    if status == i2p::ProxyStatus::Open {
        Custom(Status::Ok, Json(i2p::HttpProxyStatus { open: true }))
    } else {
        Custom(Status::Ok, Json(i2p::HttpProxyStatus { open: false }))
    }
}

/// Share your contact information
///
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

// NEVEKO Market APIs
//-----------------------------------------------

/// Get all products
///
/// Protected: false
#[get("/products")]
pub async fn get_products(_jwp: proof::PaymentProof) -> Custom<Json<Vec<models::Product>>> {
    let m_products: Vec<models::Product> = product::find_all();
    Custom(Status::Ok, Json(m_products))
}

/// Create order
///
/// Protected: true
#[post("/order/create", data = "<r_order>")]
pub async fn create_order(
    r_order: Json<reqres::OrderRequest>,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Order>> {
    let m_order: models::Order = order::create(r_order).await;
    Custom(Status::Created, Json(m_order))
}

/// TODO: Customer order retreival. Must send `signature`
///
/// which is the order id signed by the wallet.
///
/// Protected: true
#[get("/order/retrieve/<orid>/<_signature>")]
pub async fn retrieve_order(
    orid: String,
    _signature: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Order>> {
    // get customer address

    // send address, orid and signature to verify()

    let m_order: models::Order = order::find(&orid);
    Custom(Status::Created, Json(m_order))
}

/// Send multisig info for contact's order
///
/// Protected: true
#[post("/", data = "<r_info>")]
pub async fn get_multisig_info(
    r_info: Json<reqres::MultisigInfoRequest>,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Order>> {
    let info: Vec<String> = r_info.info.iter().cloned().collect();
    if r_info.msig_type == String::from(message::PREPARE_MSIG) {
        message::send_prepare_info(&r_info.orid, &r_info.contact).await;
    } else if r_info.msig_type == String::from(message::MAKE_MSIG) {
        message::send_make_info(&r_info.orid, &r_info.contact, info).await;
    } else if r_info.msig_type == String::from(message::EXPORT_MSIG) {
        message::send_export_info(&r_info.orid, &r_info.contact).await;
    } else {
        message::send_exchange_info(&r_info.orid, &r_info.contact, info).await;
    }
    Custom(Status::Ok, Json(Default::default()))
}

/// Recieve multisig messages here
///
/// Protected: true
#[post("/", data = "<message>")]
pub async fn rx_multisig_message(
    _jwp: proof::PaymentProof,
    message: Json<models::Message>,
) -> Custom<Json<models::Message>> {
    message::rx_multisig(message).await;
    Custom(Status::Ok, Json(Default::default()))
}
