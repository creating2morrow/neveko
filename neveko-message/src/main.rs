#[macro_use]
extern crate rocket;

use neveko_core::*;
use neveko_message::*;

// The only changes in here should be mounting new controller methods

#[launch]
async fn rocket() -> _ {
    let config = rocket::Config {
        port: utils::get_app_message_port(),
        ..rocket::Config::debug_default()
    };
    env_logger::init();
    log::info!("neveko-message is online");
    rocket::custom(&config)
        .mount("/message/remove", routes![controller::remove_message])
        .mount("/message/decipher", routes![controller::decipher])
        .mount("/messages", routes![controller::get_messages])
        .mount("/tx", routes![controller::send_message])
}
