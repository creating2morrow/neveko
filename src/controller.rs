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

/// Share your contact information.
///
/// 0 - returns full info with gpg key
///
/// 1 - return pruned info without gpg key
///
/// Protected: false
#[get("/<pruned>")]
pub async fn share_contact_info(pruned: u32) -> Custom<Json<models::Contact>> {
    let info: models::Contact = contact::share().await;
    if pruned == 1 {
        let p_info: models::Contact = models::Contact {
            gpg_key: Vec::new(),
            ..info
        };
        return Custom(Status::Ok, Json(p_info));
    }
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

/// Get a product by passing id
#[get("/<pid>")]
pub async fn get_product(pid: String, _jwp: proof::PaymentProof) -> Custom<Json<models::Product>> {
    let m_product: models::Product = product::find(&pid);
    Custom(Status::Ok, Json(m_product))
}

/// Get all products
///
/// Protected: true
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
    let m_order = order::secure_retrieval(&orid, &signature).await;
    if m_order.cid == utils::empty_string() {
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

/// Customer can request shipment after the wallet is funded
///
/// with the amount of the order. The vendor will then request export
///
/// multisig info, check balance and sanity check `unlock_time`.
///
/// Protected: true
#[post("/ship/<orid>")]
pub async fn request_shipment(
    orid: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<models::Message>> {
    let is_ready: bool = order::validate_order_for_ship(&orid).await;
    if !is_ready {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(Default::default()))
}

/// Send tx_data_hex, which is the output from signing
///
/// a multisig transaction from the order wallet to the
///
/// vendor's subaddress. After that the vendor will submit the
///
/// transaction and return the encrypted delivery information.
///
/// Protected: true
#[post("/finalize/<orid>")]
pub async fn finalize_order(
    orid: String,
    _jwp: proof::PaymentProof,
) -> Custom<Json<reqres::FinalizeOrderResponse>> {
    let is_ready: bool = order::validate_order_for_ship(&orid).await;
    if !is_ready {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(Default::default()))
}

/// Create a dispute (customer)
#[post("/create", data = "<dispute>")]
pub async fn create_dispute(
    dispute: Json<models::Dispute>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Dispute>> {
    let m_dispute: models::Dispute = dispute::create(dispute);
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
