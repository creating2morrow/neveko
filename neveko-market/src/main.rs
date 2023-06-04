#[macro_use]
extern crate rocket;

use neveko_core::*;
use neveko_market::*;

// The only changes in here should be mounting new controller methods

#[launch]
async fn rocket() -> _ {
    let config = rocket::Config {
        port: utils::get_app_market_port(),
        ..rocket::Config::debug_default()
    };
    env_logger::init();
    log::info!("neveko-market is online");
    rocket::custom(&config)
        .mount(
            "/dispute",
            routes![controller::create_dispute, controller::get_dispute],
        )
        .mount(
            "/order",
            routes![
                controller::get_order,
                controller::sign_and_submit_multisig, 
                controller::update_order],
        )
        .mount("/orders", routes![controller::get_orders])
        .mount("/products", routes![controller::get_products])
        .mount(
            "/product",
            routes![controller::create_product, controller::update_product],
        )
}
