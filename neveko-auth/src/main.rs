#[macro_use]
extern crate rocket;

use neveko_auth::*;
use neveko_core::*;

// The only changes in here should be mounting new controller methods

#[launch]
async fn rocket() -> _ {
    let config = rocket::Config {
        port: utils::get_app_auth_port(),
        ..rocket::Config::debug_default()
    };
    env_logger::init();
    log::info!("neveko-auth is online");
    rocket::custom(&config).mount("/", routes![controller::login])
}
