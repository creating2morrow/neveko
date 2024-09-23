#![allow(non_snake_case)]

use rocket::{
    catch,
    get,
    http::Status,
    post,
    response::status::Custom,
    serde::json::Json,
};

use neveko_core::*;

// JSON APIs exposed over i2p
// Take care not to put any admin APIs inside of here

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
    let status = i2p::check_connection().await;
    if status.unwrap_or(i2p::ProxyStatus::Opening) == i2p::ProxyStatus::Open {
        Custom(Status::Ok, Json(i2p::HttpProxyStatus { open: true }))
    } else {
        Custom(Status::Ok, Json(i2p::HttpProxyStatus { open: false }))
    }
}

/// Share your contact information.
///
/// Protected: false
#[get("/")]
pub async fn share_contact_info() -> Custom<Json<models::Contact>> {
    let info = contact::share().await;
    Custom(Status::Ok, Json(info.unwrap_or_default()))
}

/// Recieve messages here
///
/// Protected: true
#[post("/", data = "<message>")]
pub async fn rx_message(
    _jwp: proof::PaymentProof,
    message: Json<models::Message>,
) -> Custom<Json<models::Message>> {
    let _ = message::rx(message).await;
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

/// Get a product by passing id
#[get("/<pid>")]
pub async fn get_product(pid: String, _jwp: proof::PaymentProof) -> Custom<Json<models::Product>> {
    let m_product = product::find(&pid);
    Custom(Status::Ok, Json(m_product.unwrap_or_default()))
}

/// Get all products
///
/// Protected: true
#[get("/products")]
pub async fn get_products(_jwp: proof::PaymentProof) -> Custom<Json<Vec<models::Product>>> {
    let m_products: Vec<models::Product> = product::find_all().unwrap_or_default();
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
    let m_order = order::create(r_order).await;
    Custom(Status::Created, Json(m_order.unwrap_or_default()))
}

/// Customer order retreival. Must send `signature`
///
/// which is the order id signed by the NEVEKO wallet.
///
/// Protected: true
#[get("/order/retrieve/<orid>/<signature>")]
pub async fn retrieve_order(
    orid: String,
    signature: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Order>> {
    let r_m_order = order::secure_retrieval(&orid, &signature).await;
    let m_order = r_m_order.unwrap_or_default();
    if m_order.cid.is_empty() {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
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
    let info: Vec<String> = r_info.info.to_vec();
    if r_info.msig_type == *message::PREPARE_MSIG {
        // adjudicator won't have wallet for order yet do that first
        if r_info.init_adjudicator {
            order::init_adjudicator_wallet(&r_info.orid).await;
        }
        let _ = message::send_prepare_info(&r_info.orid, &r_info.contact).await;
    } else if r_info.msig_type == *message::MAKE_MSIG {
        let _ = message::send_make_info(&r_info.orid, &r_info.contact, info).await;
    } else if r_info.msig_type == *message::EXPORT_MSIG {
        let _ = message::send_export_info(&r_info.orid, &r_info.contact).await;
    } else if r_info.msig_type == *message::IMPORT_MSIG {
        let _ = message::send_import_info(&r_info.orid, &r_info.info).await;
    } else {
        let _ =
            message::send_exchange_info(&r_info.orid, &r_info.contact, info, r_info.kex_init).await;
    }
    Custom(Status::Ok, Json(Default::default()))
}

/// Recieve multisig messages here for vendor order processing
///
/// Protected: true
#[post("/", data = "<message>")]
pub async fn rx_multisig_message(
    _jwp: proof::PaymentProof,
    message: Json<models::Message>,
) -> Custom<Json<models::Message>> {
    let _ = message::rx_multisig(message).await;
    Custom(Status::Ok, Json(Default::default()))
}

/// Customer can request shipment after the wallet is funded
///
/// with the amount of the order. The vendor will then  check
///
/// balance and sanity check `unlock_time`. The vendor also releases
///
/// the draft of the Msig TxSet for the customer to sign and submit
///
/// once they receive their order.
///
/// Protected: true
#[post("/ship/<orid>")]
pub async fn request_shipment(
    orid: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<reqres::FinalizeOrderResponse>> {
    let r_finalize = order::validate_order_for_ship(&orid).await;
    let finalize = r_finalize.unwrap_or_default();
    if finalize.delivery_info.is_empty() {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(finalize))
}

/// The vendor should trigger nasr once they have uploaded delivery info.
///
/// This will automate txset release from the customer
///
/// orid - `order id of the nasr`
///
/// vedor - `vendor's .b32.i2p`
///
/// Protected: true
#[post("/nasr/<vendor>/<orid>")]
pub async fn trigger_nasr(
    orid: String,
    vendor: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Order>> {
    let r_order = order::d_trigger_ship_request(&vendor, &orid).await;
    let order = r_order.unwrap_or_default();
    if order.orid.is_empty() {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(order))
}

/// Customer cancel order logic. Must send `signature`
///
/// which is the order id signed by the NEVEKO wallet.
///
/// Protected: true
#[post("/order/cancel/<orid>/<signature>")]
pub async fn cancel_order(
    orid: String,
    signature: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Order>> {
    let r_m_order = order::cancel_order(&orid, &signature).await;
    let m_order = r_m_order.unwrap_or_default();
    if m_order.cid.is_empty() {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(m_order))
}

/// Customer finalize order logic. Vendor updates order
///
/// to `Delivered` status.
///
/// Protected: true
#[post("/order/finalize/<orid>")]
pub async fn finalize_order(
    orid: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<reqres::FinalizeOrderResponse>> {
    let r_finalize = order::finalize_order(&orid).await;
    let finalize = r_finalize.unwrap_or_default();
    if !finalize.vendor_update_success {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(finalize))
}

/// Create a dispute
#[post("/dispute/create", data = "<dispute>")]
pub async fn create_dispute(
    dispute: Json<models::Dispute>,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Dispute>> {
    let r_m_dispute = dispute::create(dispute);
    let m_dispute = r_m_dispute.unwrap_or_default();
    Custom(Status::Ok, Json(m_dispute))
}

// Catchers
//----------------------------------------------------------------

#[catch(402)]
pub fn payment_required() -> Custom<Json<reqres::ErrorResponse>> {
    Custom(
        Status::PaymentRequired,
        Json(reqres::ErrorResponse {
            error: String::from("Payment required"),
        }),
    )
}

#[catch(404)]
pub fn not_found() -> Custom<Json<reqres::ErrorResponse>> {
    Custom(
        Status::NotFound,
        Json(reqres::ErrorResponse {
            error: String::from("Resource does not exist"),
        }),
    )
}

#[catch(500)]
pub fn internal_error() -> Custom<Json<reqres::ErrorResponse>> {
    Custom(
        Status::InternalServerError,
        Json(reqres::ErrorResponse {
            error: String::from("Internal server error"),
        }),
    )
}
