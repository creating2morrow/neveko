use std::error::Error;

use crate::{
    contact,
    db,
    gpg,
    i2p,
    message,
    models::*,
    monero,
    product,
    reqres,
    utils,
};
use log::{
    debug,
    error,
    info,
};
use rocket::serde::json::Json;

enum StatusType {
    _Cancelled,
    Delivered,
    MultisigMissing,
    MulitsigComplete,
    Shipped,
}

impl StatusType {
    pub fn value(&self) -> String {
        match *self {
            StatusType::_Cancelled => String::from("Cancelled"),
            StatusType::Delivered => String::from("Delivered"),
            StatusType::MultisigMissing => String::from("MultisigMissing"),
            StatusType::MulitsigComplete => String::from("MulitsigComplete"),
            StatusType::Shipped => String::from("Shipped"),
        }
    }
}

/// Create a intial order
pub async fn create(j_order: Json<reqres::OrderRequest>) -> Order {
    info!("creating order");
    let wallet_name = String::from(crate::APP_NAME);
    let wallet_password =
        std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(String::from("password"));
    monero::open_wallet(&wallet_name, &wallet_password).await;
    let ts = chrono::offset::Utc::now().timestamp();
    let orid: String = format!("{}{}", crate::ORDER_DB_KEY, utils::generate_rnd());
    let r_subaddress = monero::create_address().await;
    monero::close_wallet(&wallet_name, &wallet_password).await;
    let subaddress = r_subaddress.result.address;
    let new_order = Order {
        orid: String::from(&orid),
        cid: String::from(&j_order.cid),
        pid: String::from(&j_order.pid),
        date: ts,
        ship_address: j_order.ship_address.iter().cloned().collect(),
        subaddress,
        status: StatusType::MultisigMissing.value(),
        quantity: j_order.quantity,
        ..Default::default()
    };
    debug!("insert order: {:?}", new_order);
    let order_wallet_password = utils::empty_string();
    let m_wallet = monero::create_wallet(&orid, &order_wallet_password).await;
    if !m_wallet {
        error!("error creating msig wallet for order {}", &orid);
        monero::close_wallet(&orid, &wallet_password).await;
        return Default::default();
    }
    // enable multisig
    monero::close_wallet(&orid, &order_wallet_password).await;
    monero::enable_experimental_multisig(&orid);
    debug!("insert order: {:?}", &new_order);
    let s = db::Interface::open();
    let k = &new_order.orid;
    db::Interface::write(&s.env, &s.handle, k, &Order::to_db(&new_order));
    // in order to retrieve all orders, write keys to with ol
    let list_key = crate::ORDER_LIST_DB_KEY;
    let r = db::Interface::read(&s.env, &s.handle, &String::from(list_key));
    if r == utils::empty_string() {
        debug!("creating order index");
    }
    let order_list = [r, String::from(&orid)].join(",");
    debug!("writing order index {} for id: {}", order_list, list_key);
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &order_list);
    new_order
}

/// Backup order for customer
pub fn backup(order: &Order) {
    info!("creating backup of order: {}", order.orid);
    let s = db::Interface::open();
    let k = &order.orid;
    db::Interface::write(&s.env, &s.handle, k, &Order::to_db(&order));
    // in order to retrieve all orders, write keys to with col
    let list_key = crate::CUSTOMER_ORDER_LIST_DB_KEY;
    let r = db::Interface::read(&s.env, &s.handle, &String::from(list_key));
    if r == utils::empty_string() {
        debug!("creating customer order index");
    }
    let order_list = [r, String::from(&order.orid)].join(",");
    debug!("writing order index {} for id: {}", order_list, list_key);
    db::Interface::write(&s.env, &s.handle, &String::from(list_key), &order_list);
}

/// Lookup order
pub fn find(oid: &String) -> Order {
    info!("find order: {}", &oid);
    let s = db::Interface::open();
    let r = db::Interface::read(&s.env, &s.handle, &String::from(oid));
    if r == utils::empty_string() {
        error!("order not found");
        return Default::default();
    }
    Order::from_db(String::from(oid), r)
}

/// Lookup all orders from admin server
pub fn find_all() -> Vec<Order> {
    let i_s = db::Interface::open();
    let i_list_key = crate::ORDER_LIST_DB_KEY;
    let i_r = db::Interface::read(&i_s.env, &i_s.handle, &String::from(i_list_key));
    if i_r == utils::empty_string() {
        error!("order index not found");
    }
    let i_v_oid = i_r.split(",");
    let i_v: Vec<String> = i_v_oid.map(|s| String::from(s)).collect();
    let mut orders: Vec<Order> = Vec::new();
    for o in i_v {
        let order: Order = find(&o);
        if order.orid != utils::empty_string() {
            orders.push(order);
        }
    }
    orders
}

/// Lookup all orders that customer has saved from gui
pub fn find_all_backup() -> Vec<Order> {
    let i_s = db::Interface::open();
    let i_list_key = crate::CUSTOMER_ORDER_LIST_DB_KEY;
    let i_r = db::Interface::read(&i_s.env, &i_s.handle, &String::from(i_list_key));
    if i_r == utils::empty_string() {
        error!("customer order index not found");
    }
    let i_v_oid = i_r.split(",");
    let i_v: Vec<String> = i_v_oid.map(|s| String::from(s)).collect();
    let mut orders: Vec<Order> = Vec::new();
    for o in i_v {
        let order: Order = find(&o);
        if order.orid != utils::empty_string() {
            orders.push(order);
        }
    }
    orders
}

/// Lookup all orders for customer
pub async fn find_all_customer_orders(cid: String) -> Vec<Order> {
    info!("lookup orders for customer: {}", &cid);
    let i_s = db::Interface::open();
    let i_list_key = crate::ORDER_LIST_DB_KEY;
    let i_r = db::Interface::read(&i_s.env, &i_s.handle, &String::from(i_list_key));
    if i_r == utils::empty_string() {
        error!("order index not found");
    }
    let i_v_oid = i_r.split(",");
    let i_v: Vec<String> = i_v_oid.map(|s| String::from(s)).collect();
    let mut orders: Vec<Order> = Vec::new();
    for o in i_v {
        let order: Order = find(&o);
        if order.orid != utils::empty_string() && order.cid == cid {
            orders.push(order);
        }
    }
    orders
}

/// Modify order from admin server
pub fn modify(o: Json<Order>) -> Order {
    info!("modify order: {}", &o.orid);
    let f_order: Order = find(&o.orid);
    if f_order.orid == utils::empty_string() {
        error!("order not found");
        return Default::default();
    }
    let u_order = Order::update(String::from(&f_order.orid), &o);
    let s = db::Interface::open();
    db::Interface::delete(&s.env, &s.handle, &u_order.pid);
    db::Interface::write(&s.env, &s.handle, &u_order.pid, &Order::to_db(&u_order));
    return u_order;
}

/// Sign and submit multisig
pub async fn sign_and_submit_multisig(
    orid: &String,
    tx_data_hex: &String,
) -> reqres::XmrRpcSubmitMultisigResponse {
    info!("signing and submitting multisig");
    let r_sign: reqres::XmrRpcSignMultisigResponse =
        monero::sign_multisig(String::from(tx_data_hex)).await;
    let r_submit: reqres::XmrRpcSubmitMultisigResponse =
        monero::submit_multisig(r_sign.result.tx_data_hex).await;
    if r_submit.result.tx_hash_list.is_empty() {
        error!("unable to submit payment for order: {}", orid);
    }
    r_submit
}

/// In order for the order (...ha) to only be accessed by the customer
///
/// they must sign the order id with their NEVEKO wallet instance. This means
///
/// that the mediator can see order id for disputes without being able to access
///
/// the details of said order.
pub async fn secure_retrieval(orid: &String, signature: &String) -> Order {
    // get customer address for NEVEKO NOT order wallet
    let m_order: Order = find(&orid);
    let mut xmr_address: String = String::new();
    let a_customers: Vec<Contact> = contact::find_all();
    for customer in a_customers {
        if customer.i2p_address == m_order.cid {
            xmr_address = customer.xmr_address;
            break;
        }
    }
    // send address, orid and signature to verify()
    let id: String = String::from(&m_order.orid);
    let sig: String = String::from(signature);
    let wallet_password = utils::empty_string();
    monero::open_wallet(&orid, &wallet_password).await;
    let is_valid_signature = monero::verify(xmr_address, id, sig).await;
    monero::close_wallet(&orid, &wallet_password).await;
    if !is_valid_signature {
        return Default::default();
    }
    m_order
}

/// Check for import multisig info, validate block time and that the
///
/// order wallet has been funded properly. Update the order to multisig complete
pub async fn validate_order_for_ship(orid: &String) -> bool {
    info!("validating order for shipment");
    let mut m_order: Order = find(orid);
    let m_product: Product = product::find(&m_order.pid);
    let price = m_product.price;
    let total = price * m_order.quantity;
    // import multisig info
    let s = db::Interface::open();
    let key = format!("export-{}-{}", orid, &m_order.cid);
    let info_str = db::Interface::async_read(&s.env, &s.handle, &key).await;
    let info_split = info_str.split(":");
    let v_info: Vec<String> = info_split.map(|s| String::from(s)).collect();
    let wallet_password = utils::empty_string();
    monero::open_wallet(&orid, &wallet_password).await;
    let r_import = monero::import_multisig_info(v_info).await;
    // check balance and unlock_time
    let r_balance = monero::get_balance().await;
    monero::close_wallet(&orid, &wallet_password).await;
    // update the order status to multisig complete
    let ready_to_ship: bool = r_import.result.n_outputs > 0
        && r_balance.result.balance >= total as u128
        && r_balance.result.blocks_to_unlock < monero::LockTimeLimit::Blocks.value();
    if ready_to_ship {
        m_order.status = StatusType::MulitsigComplete.value();
        db::Interface::async_delete(&s.env, &s.handle, &m_order.orid).await;
        db::Interface::async_write(&s.env, &s.handle, &m_order.orid, &Order::to_db(&m_order)).await;
    }
    ready_to_ship
}

/// Write encrypted delivery info to lmdb. Once the customer releases the signed
/// txset
///
/// they will have access to this information (tracking number, locker code,
/// etc.)
pub async fn upload_delivery_info(orid: &String, delivery_info: &Vec<u8>) {
    info!("uploading delivery info");
    let name = i2p::get_destination(None);
    let e_delivery_info: Vec<u8> = gpg::encrypt(name, &delivery_info).unwrap_or(Vec::new());
    if e_delivery_info.is_empty() {
        error!("unable to encrypt delivery info");
    }
    // write delivery info {delivery}-{order id}
    let s = db::Interface::async_open().await;
    let k = format!("delivery-{}", orid);
    let data = hex::encode(e_delivery_info);
    db::Interface::async_write(&s.env, &s.handle, &k, &data).await;
    // update the order
    let mut m_order: Order = find(orid);
    m_order.status = StatusType::Shipped.value();
    db::Interface::async_delete(&s.env, &s.handle, &m_order.orid).await;
    db::Interface::async_write(&s.env, &s.handle, &m_order.orid, &Order::to_db(&m_order)).await;
}

/// The vendor will first search for an encrypted multisig message in the form
///  
/// txset-{order id}-{.b32.i2p}
pub async fn finalize_order(orid: &String) -> reqres::FinalizeOrderResponse {
    info!("finalizing order");
    let mut m_order: Order = find(orid);
    let s = db::Interface::async_open().await;
    let key = format!("{}-{}-{}", message::TXSET_MSIG, orid, &m_order.cid);
    let txset = db::Interface::async_read(&s.env, &s.handle, &key).await;
    // describe transer to check amount, address and unlock_time
    let wallet_password = utils::empty_string();
    monero::open_wallet(&orid, &wallet_password).await;
    let r_describe: reqres::XmrRpcDescribeTransferResponse =
        monero::describe_transfer(&txset).await;
    let m_product: Product = product::find(&m_order.pid);
    let total: u128 = m_product.price * m_order.quantity;
    let description: &reqres::TransferDescription = &r_describe.result.desc[0];
    let is_valid_payment: bool = description.amount_out + description.fee >= total
        && description.unlock_time < monero::LockTimeLimit::Blocks.value();
    if !is_valid_payment {
        return Default::default();
    }
    let r_submit: reqres::XmrRpcSubmitMultisigResponse =
        sign_and_submit_multisig(orid, &txset).await;
    monero::close_wallet(&orid, &wallet_password).await;
    if r_submit.result.tx_hash_list.is_empty() {
        return Default::default();
    }
    // lookup delivery info
    let delivery_key = format!("delivery-{}", orid);
    let r_delivery_info: String = db::Interface::async_read(&s.env, &s.handle, &delivery_key).await;
    let delivery_info: Vec<u8> = hex::decode(r_delivery_info).unwrap_or(Vec::new());
    // update the order
    m_order.status = StatusType::Delivered.value();
    db::Interface::async_delete(&s.env, &s.handle, &m_order.orid).await;
    db::Interface::async_write(&s.env, &s.handle, &m_order.orid, &Order::to_db(&m_order)).await;
    reqres::FinalizeOrderResponse {
        orid: String::from(orid),
        delivery_info,
    }
}

/// Send order request to vendor and start multisig flow
pub async fn transmit_order_request(
    contact: String,
    jwp: String,
    request: reqres::OrderRequest,
) -> Result<Order, Box<dyn Error>> {
    let host = utils::get_i2p_http_proxy();
    let proxy = reqwest::Proxy::http(&host)?;
    let client = reqwest::Client::builder().proxy(proxy).build();
    match client?
        .post(format!("http://{}/market/order/create", contact))
        .header("proof", jwp)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            let res = response.json::<Order>().await;
            debug!("create order response: {:?}", res);
            match res {
                Ok(r) => Ok(r),
                _ => Ok(Default::default()),
            }
        }
        Err(e) => {
            error!("failed to generate order due to: {:?}", e);
            Ok(Default::default())
        }
    }
}

pub async fn init_mediator_wallet(orid: &String) {
    let password = utils::empty_string();
    let m_wallet = monero::create_wallet(orid, &password).await;
    if !m_wallet {
        log::error!("failed to create mediator wallet");
    }
    // enable multisig
    monero::close_wallet(&orid, &password).await;
    monero::enable_experimental_multisig(&orid);
}
