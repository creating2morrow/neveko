// use nevmes_core::*;
// use log::{debug, error, info};
// use rocket::serde::json::Json;
// use crate::product;

// enum StatusType {
//     Delivered,
//     MultisigMissing,
//     MulitsigComplete,
//     Shipped,
// }

// impl StatusType {
//     pub fn value(&self) -> String {
//         match *self {
//             StatusType::Delivered => String::from("Delivered"),
//             StatusType::MultisigMissing => String::from("MultisigMissing"),
//             StatusType::MulitsigComplete => String::from("MulitsigComplete"),
//             StatusType::Shipped => String::from("Shipped"),
//         }
//     }
// }

// /// Create a skeleton for order
// pub fn create(cid: String, pid: String) -> models::Order {
//     let ts = chrono::offset::Utc::now().timestamp();
//     let orid: String = format!("O{}", utils::generate_rnd());
//     let m_product: models::Product = product::find(&pid);
//     let new_order = models::Order {
//         orid,
//         cid: String::from(&cid),
//         pid: String::from(&pid),
//         cust_kex_1: utils::empty_string(),
//         cust_kex_2: utils::empty_string(),
//         cust_kex_3: utils::empty_string(),
//         cust_msig_make: utils::empty_string(),
//         cust_msig_prepare: utils::empty_string(),
//         cust_msig_txset: utils::empty_string(),
//         date: 0,
//         deliver_date: 0,
//         hash: utils::empty_string(),
//         mediator_kex_1: utils::empty_string(),
//         mediator_kex_2: utils::empty_string(),
//         mediator_kex_3: utils::empty_string(),
//         mediator_msig_make: utils::empty_string(),
//         mediator_msig_prepare: utils::empty_string(),
//         ship_date: 0,
//         subaddress: utils::empty_string(),
//         status: utils::empty_string(),
//         quantity: 0,
//         vend_kex_1: utils::empty_string(),
//         vend_kex_2: utils::empty_string(),
//         vend_kex_3: utils::empty_string(),
//         vend_msig_make: utils::empty_string(),
//         vend_msig_prepare: utils::empty_string(),
//         vend_msig_txset: utils::empty_string(),
//         xmr_address: utils::empty_string(),
//     };
//     debug!("insert order: {:?}", new_order);
//     let m_wallet = monero::create_wallet(String::from(&orid), &utils::empty_string()).await;
//     if !m_wallet {
//         error!("error creating msig wallet for order {}", &orid);
//     }
//     debug!("insert order: {:?}", &new_order);
//     let s = db::Interface::open();
//     let k = &new_order.orid;
//     db::Interface::write(&s.env, &s.handle, k, &models::Order::to_db(&new_order));
//     // in order to retrieve all orders, write keys to with ol
//     let list_key = format!("ol");
//     let r = db::Interface::read(&s.env, &s.handle, &String::from(&list_key));
//     if r == utils::empty_string() {
//         debug!("creating order index");
//     }
//     let order_list = [r, String::from(&orid)].join(",");
//     debug!("writing order index {} for id: {}", order_list, list_key);
//     db::Interface::write(&s.env, &s.handle, &String::from(list_key), &order_list);
//     new_order
// }

// /// Lookup order
// pub fn find(oid: String) -> models::Order {
//     let s = db::Interface::open();
//     let r = db::Interface::read(&s.env, &s.handle, &String::from(&oid));
//     if r == utils::empty_string() {
//         error!("order not found");
//         return Default::default();
//     }
//     models::Order::from_db(String::from(&oid), r)
// }

// /// Lookup all orders for customer
// pub async fn find_all_customer_orders(cid: String) -> Vec<models::Order> {
//     let i_s = db::Interface::open();
//     let i_list_key = format!("ol");
//     let i_r = db::Interface::read(&i_s.env, &i_s.handle, &String::from(i_list_key));
//     if i_r == utils::empty_string() {
//         error!("order index not found");
//     }
//     let i_v_oid = i_r.split(",");
//     let i_v: Vec<String> = i_v_oid.map(|s| String::from(s)).collect();
//     let mut orders: Vec<models::Order> = Vec::new();
//     for o in i_v {
//         let order: models::Order = find(o);
//         if order.orid != utils::empty_string() && order.cid == cid {
//             orders.push(order);
//         }
//     }
//     orders
// }
