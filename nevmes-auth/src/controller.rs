use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::get;

use nevmes_core::{auth, models::*};

/// Login with wallet signature
/// 
/// Creates user on initial login
/// 
#[get("/login/<signature>/<aid>/<uid>")]
pub async fn login
(aid: String, uid: String,signature: String) -> Custom<Json<Authorization>> {
    let m_auth: Authorization = auth::verify_login(aid, uid, signature).await;
    Custom(Status::Created, Json(m_auth))
}
