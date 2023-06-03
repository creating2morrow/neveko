#[macro_use]
extern crate rocket;

use neveko_contact::*;
use neveko_core::*;

// The only changes in here should be mounting new controller methods

#[launch]
async fn rocket() -> _ {
    let config = rocket::Config {
        port: utils::get_app_contact_port(),
        ..rocket::Config::debug_default()
    };
    env_logger::init();
    log::info!("neveko-contact is online");
    rocket::custom(&config)
        .mount("/trust", routes![controller::trust_contact])
        .mount("/prove", routes![controller::prove_payment])
        .mount("/contact", routes![controller::add_contact])
        .mount("/contacts", routes![controller::get_contacts])
}
