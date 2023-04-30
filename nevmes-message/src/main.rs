#[macro_use]
extern crate rocket;

use nevmes_message::*;
use nevmes_core::*;

// The only changes in here should be mounting new controller methods

#[launch]
async fn rocket() -> _ {
    let config = rocket::Config {
        port: utils::get_app_message_port(),
        ..rocket::Config::debug_default()
    };
    env_logger::init();
    log::info!("nevmes-message is online");
    rocket::custom(&config)
        .mount("/message/decrypt", routes![controller::decrypt])
        .mount("/messages", routes![controller::get_messages])
        .mount("/tx", routes![controller::send_message])
}
