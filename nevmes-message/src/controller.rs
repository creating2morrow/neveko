use rocket::{
    get,
    http::Status,
    post,
    response::status::Custom,
    serde::json::Json,
};

use nevmes_core::{
    auth,
    message,
    models::*,
    proof,
    reqres,
};

/// Send message
#[post("/", data = "<m_req>")]
pub async fn send_message(
    m_req: Json<Message>,
    token: proof::PaymentProof,
) -> Custom<Json<Message>> {
    let res: Message = message::create(m_req, token.get_jwp()).await;
    Custom(Status::Ok, Json(res))
}

/// Return all messages
#[get("/")]
pub async fn get_messages(_token: auth::BearerToken) -> Custom<Json<Vec<Message>>> {
    let messages = message::find_all();
    Custom(Status::Ok, Json(messages))
}

/// decrypt a message body
#[get("/<mid>")]
pub async fn decrypt(
    mid: String,
    _token: auth::BearerToken,
) -> Custom<Json<reqres::DecryptedMessageBody>> {
    let d_message = message::decrypt_body(mid);
    Custom(Status::Ok, Json(d_message))
}
