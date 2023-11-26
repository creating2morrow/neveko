use rocket::{
    get,
    http::Status,
    patch,
    post,
    response::status::Custom,
    serde::json::Json,
};

use neveko_core::*;

// JSON APIs

/// Create a product by passings json product
#[post("/create", data = "<req_product>")]
pub async fn create_product(
    req_product: Json<models::Product>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Product>> {
    let m_product: models::Product = product::create(req_product);
    Custom(Status::Created, Json(m_product))
}

/// Get a product by passing id
#[get("/<pid>")]
pub async fn get_product(pid: String, _token: auth::BearerToken) -> Custom<Json<models::Product>> {
    let m_product: models::Product = product::find(&pid);
    Custom(Status::Ok, Json(m_product))
}

/// Update product information
#[patch("/update", data = "<product>")]
pub async fn update_product(
    product: Json<models::Product>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Product>> {
    let m_product: models::Product = product::modify(product);
    Custom(Status::Ok, Json(m_product))
}

/// Return all products
#[get("/")]
pub async fn get_products(_token: auth::BearerToken) -> Custom<Json<Vec<models::Product>>> {
    let m_products: Vec<models::Product> = product::find_all();
    Custom(Status::Ok, Json(m_products))
}

/// Get a order by passing id
#[get("/<orid>")]
pub async fn get_order(orid: String, _token: auth::BearerToken) -> Custom<Json<models::Order>> {
    let m_order: models::Order = order::find(&orid);
    Custom(Status::Ok, Json(m_order))
}

/// Get all orders
#[get("/")]
pub async fn get_orders(_token: auth::BearerToken) -> Custom<Json<Vec<models::Order>>> {
    let m_orders: Vec<models::Order> = order::find_all();
    Custom(Status::Ok, Json(m_orders))
}

/// Update order information
#[patch("/update", data = "<order>")]
pub async fn update_order(
    order: Json<models::Order>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Order>> {
    let m_order: models::Order = order::modify(order);
    Custom(Status::Ok, Json(m_order))
}

/// Create a dispute
#[post("/create", data = "<dispute>")]
pub async fn create_dispute(
    dispute: Json<models::Dispute>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Dispute>> {
    let m_dispute: models::Dispute = dispute::create(dispute);
    Custom(Status::Ok, Json(m_dispute))
}

/// Fetch a dispute
#[get("/<did>")]
pub async fn get_dispute(_token: auth::BearerToken, did: String) -> Custom<Json<models::Dispute>> {
    let m_dispute: models::Dispute = dispute::find(&did);
    Custom(Status::Ok, Json(m_dispute))
}

/// Sign and submit multisig
#[post("/sign/submit", data = "<r_data>")]
pub async fn sign_and_submit_multisig(
    r_data: Json<reqres::SignAndSubmitRequest>,
    _token: auth::BearerToken,
) -> Custom<Json<reqres::SignAndSubmitRequest>> {
    let result: reqres::XmrRpcSubmitMultisigResponse =
        order::sign_and_submit_multisig(&r_data.orid, &r_data.txset).await;
    if result.result.tx_hash_list.is_empty() {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(Default::default()))
}

/// API for uploading delivery info in vendor mode
///
/// Protected: true
#[post("/<orid>", data = "<r_data>")]
pub async fn upload_delivery_info(
    orid: String,
    r_data: Json<reqres::FinalizeOrderResponse>,
    _token: auth::BearerToken,
) -> Custom<Json<reqres::FinalizeOrderResponse>> {
    let upload: reqres::FinalizeOrderResponse =
        order::upload_delivery_info(&orid, &r_data.delivery_info).await;
    if upload.delivery_info.is_empty() {
        return Custom(Status::BadRequest, Json(Default::default()));
    }
    Custom(Status::Ok, Json(upload))
}

/// toggle vendor mode
#[get("/")]
pub async fn toggle_vendor_mode(
    _token: auth::BearerToken,
) -> Custom<Json<reqres::VendorModeResponse>> {
    let mode = utils::toggle_vendor_enabled();
    Custom(Status::Ok, Json(reqres::VendorModeResponse { mode }))
}
// END JSON APIs
