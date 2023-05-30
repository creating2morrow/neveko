use rocket::{
    get,
    http::Status,
    patch,
    post,
    response::status::Custom,
    serde::json::Json,
};

use nevmes_core::*;

use crate::{
    dispute,
    product,
};

// JSON APIs

/// Create a product by passing vendor vid
#[post("/create", data = "<req_product>")]
pub async fn create_product(
    req_product: Json<models::Product>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Product>> {
    let m_product: models::Product = product::create(req_product);
    Custom(Status::Ok, Json(m_product))
}

/// Update product information
#[patch("/<_address>/update", data = "<product>")]
pub async fn update_product(
    _address: String,
    product: Json<models::Product>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Product>> {
    let m_product: models::Product = product::modify(product);
    Custom(Status::Ok, Json(m_product))
}

// /// Initialize order
// #[get("/<address>/create/<pid>")]
// pub async fn initialize_order(
//     address: String,
//     _token: auth::BearerToken,
//     pid: String,
// ) -> Custom<Json<reqres::GetOrderResponse>> {
//     // get the cid from the address after verification
//     let m_customer = customer::find(address).await;
//     let temp_pid = String::from(&pid);
//     let m_order: models::Order = order::create(m_customer.cid, temp_pid).await;
//     Custom(
//         Status::Ok,
//         Json(reqres::GetOrderResponse::build(pid, m_order)),
//     )
// }

// /// Update order information from vendor
// #[patch("/update/<pid>/<oid>/<data>/vendor")]
// pub async fn update_order(
//     _address: String,
//     oid: String,
//     pid: String,
//     _token: auth::BearerToken,
//     data: String,
// ) -> Custom<Json<reqres::GetOrderResponse>> {
//     let temp_pid: String = String::from(&pid);
//     let m_order: models::Order = order::modify(oid, pid, data, update_type).await;
//     Custom(
//         Status::Ok,
//         Json(reqres::GetOrderResponse::build(temp_pid, m_order)),
//     )
// }

// /// Get all orders
// ///  by passing auth
// #[get("/<address>/<corv>")]
// pub async fn get_orders(
//     address: String,
//     corv: String,
//     _token: auth::BearerToken,
// ) -> Custom<Json<reqres::GetOrdersResponse>> {
//     let m_orders: Vec<models::Order> = order::find_all(address, corv).await;
//     Custom(Status::Ok, Json(reqres::GetOrdersResponse::build(m_orders)))
// }

/// Create a dispute
#[post("/create", data = "<dispute>")]
pub async fn create_dispute(
    dispute: Json<models::Dispute>,
    _token: auth::BearerToken,
) -> Custom<Json<models::Dispute>> {
    let m_dispute: models::Dispute = dispute::create(dispute);
    Custom(Status::Ok, Json(m_dispute))
}

/// Create a dispute
#[get("/<did>")]
pub async fn get_dispute(_token: auth::BearerToken, did: String) -> Custom<Json<models::Dispute>> {
    let m_dispute: models::Dispute = dispute::find(&did);
    Custom(Status::Ok, Json(m_dispute))
}
// END JSON APIs
