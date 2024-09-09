#![allow(non_snake_case)]

use rocket::{
    get,
    http::Status,
    response::status::Custom,
    serde::json::Json,
};

use neveko_core::{
    auth,
    models::*,
};

/// Login with wallet signature
///
/// Creates user on initial login
#[get("/login/<signature>/<aid>/<uid>")]
pub async fn login(aid: String, uid: String, signature: String) -> Custom<Json<Authorization>> {
    let m_auth = auth::verify_login(aid, uid, signature).await;
    Custom(Status::Created, Json(m_auth.unwrap_or_default()))
}
