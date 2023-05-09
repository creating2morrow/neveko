#[macro_use]
extern crate rocket;
use rocket::{
    http::Status,
    response::status::Custom,
    serde::json::Json,
};

use nevmes::*;
use nevmes_core::*;

#[catch(402)]
fn payment_required() -> Custom<Json<reqres::ErrorResponse>> {
    Custom(
        Status::PaymentRequired,
        Json(reqres::ErrorResponse {
            error: String::from("Payment required"),
        }),
    )
}

#[catch(404)]
fn not_found() -> Custom<Json<reqres::ErrorResponse>> {
    Custom(
        Status::NotFound,
        Json(reqres::ErrorResponse {
            error: String::from("Resource does not exist"),
        }),
    )
}

#[catch(500)]
fn internal_error() -> Custom<Json<reqres::ErrorResponse>> {
    Custom(
        Status::InternalServerError,
        Json(reqres::ErrorResponse {
            error: String::from("Internal server error"),
        }),
    )
}

// The only changes below here should be mounting new controller methods
#[launch]
async fn rocket() -> _ {
    let config = rocket::Config {
        ident: rocket::config::Ident::none(),
        ip_header: None,
        port: utils::get_app_port(),
        ..rocket::Config::debug_default()
    };
    env_logger::init();
    utils::start_up().await;
    rocket::custom(&config)
        .register("/", catchers![internal_error, not_found, payment_required])
        .mount("/invoice", routes![controller::gen_invoice])
        .mount("/message/rx", routes![controller::rx_message])
        .mount("/prove", routes![controller::gen_jwp])
        .mount("/share", routes![controller::share_contact_info])
        .mount("/i2p", routes![controller::get_i2p_status])
        .mount("/xmr/rpc", routes![controller::get_version])
}
