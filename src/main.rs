#[macro_use]
extern crate rocket;

use neveko::*;
use neveko_core::*;
use rocket::data::{
    Limits,
    ToByteUnit,
};

// The only changes below here should be mounting new controller methods
#[launch]
async fn rocket() -> _ {
    let config = rocket::Config {
        ident: rocket::config::Ident::none(),
        ip_header: None,
        limits: Limits::default().limit("json", 10_i32.mebibytes()),
        port: utils::get_app_port(),
        ..rocket::Config::debug_default()
    };
    env_logger::init();
    let _ = utils::start_up().await;
    rocket::custom(&config)
        .register(
            "/",
            catchers![
                controller::internal_error,
                controller::not_found,
                controller::payment_required
            ],
        )
        .mount("/multisig/info", routes![controller::get_multisig_info])
        .mount("/invoice", routes![controller::gen_invoice])
        .mount("/message/rx", routes![controller::rx_message])
        .mount(
            "/message/rx/multisig",
            routes![controller::rx_multisig_message],
        )
        .mount("/prove", routes![controller::gen_jwp])
        .mount("/share", routes![controller::share_contact_info])
        .mount("/i2p", routes![controller::get_i2p_status])
        .mount("/xmr/rpc", routes![controller::get_version])
        .mount(
            "/market",
            routes![
                controller::create_order,
                controller::create_dispute,
                controller::get_product,
                controller::get_products,
                controller::request_shipment,
                controller::retrieve_order,
                controller::trigger_nasr,
                controller::finalize_order,
            ],
        )
}
