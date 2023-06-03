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
    order,
    product,
};

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
#[post("/<pid>")]
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
#[patch("/")]
pub async fn get_products(_token: auth::BearerToken) -> Custom<Json<Vec<models::Product>>> {
    let m_products: Vec<models::Product> = product::find_all();
    Custom(Status::Ok, Json(m_products))
}

/// Get a order by passing id
#[post("/<orid>")]
pub async fn get_order(orid: String, _token: auth::BearerToken) -> Custom<Json<models::Order>> {
    let m_order: models::Order = order::find(&orid);
    Custom(Status::Ok, Json(m_order))
}

/// Get a order by passing id
#[post("/")]
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

/// Create a dispute
#[get("/<did>")]
pub async fn get_dispute(_token: auth::BearerToken, did: String) -> Custom<Json<models::Dispute>> {
    let m_dispute: models::Dispute = dispute::find(&did);
    Custom(Status::Ok, Json(m_dispute))
}
// END JSON APIs
