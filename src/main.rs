#[macro_use]
extern crate rocket;

use nevmes_core::*;
use nevmes::*;

// The only changes in here should be mounting new controller methods
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
        .mount("/invoice", routes![controller::gen_invoice])
        .mount("/message/rx", routes![controller::rx_message])
        .mount("/prove", routes![controller::gen_jwp])
        .mount("/share", routes![controller::share_contact_info])
        .mount("/i2p", routes![controller::get_i2p_status])
        .mount("/xmr/rpc", routes![controller::get_version])
}
