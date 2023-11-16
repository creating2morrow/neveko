use crate::utils;
use rocket::serde::{
    json::Json,
    Deserialize,
    Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Authorization {
    pub aid: String,
    pub created: i64,
    pub rnd: String,
    pub token: String,
    pub uid: String,
    pub xmr_address: String,
}

impl Default for Authorization {
    fn default() -> Self {
        Authorization {
            aid: utils::empty_string(),
            created: 0,
            uid: utils::empty_string(),
            rnd: utils::empty_string(),
            token: utils::empty_string(),
            xmr_address: utils::empty_string(),
        }
    }
}

impl Authorization {
    pub fn to_db(a: &Authorization) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            a.created, a.uid, a.rnd, a.token, a.xmr_address
        )
    }
    pub fn from_db(k: String, v: String) -> Authorization {
        let values = v.split(":");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let created_str = v.remove(0);
        let created = match created_str.parse::<i64>() {
            Ok(n) => n,
            Err(_e) => 0,
        };
        let uid = v.remove(0);
        let rnd = v.remove(0);
        let token = v.remove(0);
        let xmr_address = v.remove(0);
        Authorization {
            aid: k,
            created,
            uid,
            rnd,
            token,
            xmr_address,
        }
    }
    pub fn update_uid(a: Authorization, uid: String) -> Authorization {
        Authorization {
            aid: a.aid,
            created: a.created,
            uid,
            rnd: a.rnd,
            token: a.token,
            xmr_address: a.xmr_address,
        }
    }
    pub fn update_expiration(
        a: &Authorization,
        created: i64,
        rnd: String,
        token: String,
    ) -> Authorization {
        Authorization {
            aid: String::from(&a.aid),
            created,
            uid: String::from(&a.uid),
            rnd,
            token,
            xmr_address: String::from(&a.xmr_address),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Contact {
    pub cid: String,
    pub i2p_address: String,
    pub is_vendor: bool,
    pub xmr_address: String,
    pub gpg_key: Vec<u8>,
}

impl Default for Contact {
    fn default() -> Self {
        Contact {
            cid: utils::empty_string(),
            gpg_key: Vec::new(),
            i2p_address: utils::empty_string(),
            is_vendor: false,
            xmr_address: utils::empty_string(),
        }
    }
}

impl Contact {
    pub fn to_db(c: &Contact) -> String {
        let gpg = hex::encode(&c.gpg_key);
        format!(
            "{}!{}!{}!{}",
            gpg, c.i2p_address, c.is_vendor, c.xmr_address
        )
    }
    pub fn from_db(k: String, v: String) -> Contact {
        let values = v.split("!");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let gpg_key = hex::decode(v.remove(0)).unwrap_or(Vec::new());
        let i2p_address = v.remove(0);
        let is_vendor = match v.remove(0).parse::<bool>() {
            Ok(n) => n,
            Err(_e) => false,
        };
        let xmr_address = v.remove(0);
        Contact {
            cid: k,
            gpg_key,
            i2p_address,
            is_vendor,
            xmr_address,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    pub mid: String,
    pub uid: String,
    pub body: Vec<u8>,
    pub created: i64,
    pub from: String,
    pub to: String,
}

impl Default for Message {
    fn default() -> Self {
        Message {
            mid: utils::empty_string(),
            uid: utils::empty_string(),
            body: Vec::new(),
            created: 0,
            from: utils::empty_string(),
            to: utils::empty_string(),
        }
    }
}

impl Message {
    pub fn to_db(m: &Message) -> String {
        let body = hex::encode(&m.body);
        format!("{}:{}:{}:{}:{}", m.uid, body, m.created, m.from, m.to)
    }
    pub fn from_db(k: String, v: String) -> Message {
        let values = v.split(":");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let uid = v.remove(0);
        let body = hex::decode(v.remove(0)).unwrap_or(Vec::new());
        let created_str = v.remove(0);
        let created = match created_str.parse::<i64>() {
            Ok(n) => n,
            Err(_e) => 0,
        };
        let from = v.remove(0);
        let to = v.remove(0);
        Message {
            mid: k,
            uid,
            body,
            created,
            from,
            to,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub uid: String,
    pub xmr_address: String,
    pub name: String,
}

impl Default for User {
    fn default() -> Self {
        User {
            uid: utils::empty_string(),
            xmr_address: utils::empty_string(),
            name: utils::empty_string(),
        }
    }
}

impl User {
    pub fn to_db(u: &User) -> String {
        format!("{}:{}", u.name, u.xmr_address)
    }
    pub fn from_db(k: String, v: String) -> User {
        let values = v.split(":");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let name = v.remove(0);
        let xmr_address = v.remove(0);
        User {
            uid: k,
            name,
            xmr_address,
        }
    }
    pub fn update(u: User, name: String) -> User {
        User {
            uid: u.uid,
            name,
            xmr_address: u.xmr_address,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Product {
    pub pid: String,
    pub description: String,
    pub image: Vec<u8>,
    pub in_stock: bool,
    pub name: String,
    pub price: u128,
    pub qty: u128,
}

impl Default for Product {
    fn default() -> Self {
        Product {
            pid: utils::empty_string(),
            description: utils::empty_string(),
            image: Vec::new(),
            in_stock: false,
            name: utils::empty_string(),
            price: 0,
            qty: 0,
        }
    }
}

impl Product {
    pub fn to_db(p: &Product) -> String {
        let image: String = hex::encode(&p.image);
        format!(
            "{}:{}:{}:{}:{}:{}",
            p.description, image, p.in_stock, p.name, p.price, p.qty
        )
    }
    pub fn from_db(k: String, v: String) -> Product {
        let values = v.split(":");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let description = v.remove(0);
        let image = hex::decode(v.remove(0)).unwrap_or(Vec::new());
        let in_stock = match v.remove(0).parse::<bool>() {
            Ok(b) => b,
            Err(_) => false,
        };
        let name = v.remove(0);
        let price = match v.remove(0).parse::<u128>() {
            Ok(p) => p,
            Err(_) => 0,
        };
        let qty = match v.remove(0).parse::<u128>() {
            Ok(q) => q,
            Err(_) => 0,
        };
        Product {
            pid: k,
            description,
            image,
            in_stock,
            name,
            price,
            qty,
        }
    }
    pub fn update(p: Product, jp: &Json<Product>) -> Product {
        Product {
            pid: p.pid,
            description: String::from(&jp.description),
            image: jp.image.iter().cloned().collect(),
            in_stock: jp.in_stock,
            name: String::from(&jp.name),
            price: jp.price,
            qty: jp.qty,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Order {
    pub orid: String,
    /// Order customer id is their .b32.i2p address
    pub cid: String,
    pub pid: String,
    pub cust_kex_1: String,
    pub cust_kex_2: String,
    pub cust_kex_3: String,
    pub cust_msig_make: String,
    pub cust_msig_prepare: String,
    pub cust_msig_txset: String,
    pub date: i64,
    pub deliver_date: i64,
    /// Transaction hash from vendor or customer signed txset
    pub hash: String,
    pub mediator_kex_1: String,
    pub mediator_kex_2: String,
    pub mediator_kex_3: String,
    pub mediator_msig_make: String,
    pub mediator_msig_prepare: String,
    /// Address gpg key encrypted bytes
    pub ship_address: Vec<u8>,
    pub ship_date: i64,
    /// This is the final destination for the payment
    pub subaddress: String,
    pub status: String,
    pub quantity: u128,
    pub vend_kex_1: String,
    pub vend_kex_2: String,
    pub vend_kex_3: String,
    pub vend_msig_make: String,
    pub vend_msig_prepare: String,
    pub vend_msig_txset: String,
    pub xmr_address: String,
}

impl Default for Order {
    fn default() -> Self {
        Order {
            orid: utils::empty_string(),
            cid: utils::empty_string(),
            pid: utils::empty_string(),
            xmr_address: utils::empty_string(),
            cust_kex_1: utils::empty_string(),
            cust_kex_2: utils::empty_string(),
            cust_kex_3: utils::empty_string(),
            cust_msig_make: utils::empty_string(),
            cust_msig_prepare: utils::empty_string(),
            cust_msig_txset: utils::empty_string(),
            date: 0,
            deliver_date: 0,
            hash: utils::empty_string(),
            mediator_kex_1: utils::empty_string(),
            mediator_kex_2: utils::empty_string(),
            mediator_kex_3: utils::empty_string(),
            mediator_msig_make: utils::empty_string(),
            mediator_msig_prepare: utils::empty_string(),
            ship_address: Vec::new(),
            ship_date: 0,
            subaddress: utils::empty_string(),
            status: utils::empty_string(),
            quantity: 0,
            vend_kex_1: utils::empty_string(),
            vend_kex_2: utils::empty_string(),
            vend_kex_3: utils::empty_string(),
            vend_msig_make: utils::empty_string(),
            vend_msig_prepare: utils::empty_string(),
            vend_msig_txset: utils::empty_string(),
        }
    }
}

impl Order {
    pub fn to_db(o: &Order) -> String {
        let ship_address = hex::encode(&o.ship_address);
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            o.cid,
            o.pid,
            o.cust_kex_1,
            o.cust_kex_2,
            o.cust_kex_3,
            o.cust_msig_make,
            o.cust_msig_prepare,
            o.cust_msig_txset,
            o.date,
            o.deliver_date,
            o.hash,
            o.mediator_msig_make,
            o.mediator_msig_prepare,
            o.mediator_kex_1,
            o.mediator_kex_2,
            o.mediator_kex_3,
            ship_address,
            o.ship_date,
            o.subaddress,
            o.status,
            o.quantity,
            o.vend_kex_1,
            o.vend_kex_2,
            o.vend_kex_3,
            o.vend_msig_make,
            o.vend_msig_prepare,
            o.vend_msig_txset,
            o.xmr_address,
        )
    }
    pub fn from_db(k: String, v: String) -> Order {
        let values = v.split(":");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let orid = k;
        let cid = v.remove(0);
        let pid = v.remove(0);
        let cust_kex_1 = v.remove(0);
        let cust_kex_2 = v.remove(0);
        let cust_kex_3 = v.remove(0);
        let cust_msig_make = v.remove(0);
        let cust_msig_prepare = v.remove(0);
        let cust_msig_txset = v.remove(0);
        let date = match v.remove(0).parse::<i64>() {
            Ok(d) => d,
            Err(_) => 0,
        };
        let deliver_date = match v.remove(0).parse::<i64>() {
            Ok(d) => d,
            Err(_) => 0,
        };
        let hash = v.remove(0);
        let mediator_msig_make = v.remove(0);
        let mediator_msig_prepare = v.remove(0);
        let mediator_kex_1 = v.remove(0);
        let mediator_kex_2 = v.remove(0);
        let mediator_kex_3 = v.remove(0);
        let ship_address = hex::decode(v.remove(0)).unwrap_or(Vec::new());
        let ship_date = match v.remove(0).parse::<i64>() {
            Ok(d) => d,
            Err(_) => 0,
        };
        let subaddress = v.remove(0);
        let status = v.remove(0);
        let quantity = match v.remove(0).parse::<u128>() {
            Ok(d) => d,
            Err(_) => 0,
        };
        let vend_kex_1 = v.remove(0);
        let vend_kex_2 = v.remove(0);
        let vend_kex_3 = v.remove(0);
        let vend_msig_make = v.remove(0);
        let vend_msig_prepare = v.remove(0);
        let vend_msig_txset = v.remove(0);
        let xmr_address = v.remove(0);
        Order {
            orid,
            cid,
            pid,
            cust_kex_1,
            cust_kex_2,
            cust_kex_3,
            cust_msig_make,
            cust_msig_prepare,
            cust_msig_txset,
            date,
            deliver_date,
            hash,
            mediator_kex_1,
            mediator_kex_2,
            mediator_kex_3,
            mediator_msig_make,
            mediator_msig_prepare,
            ship_address,
            ship_date,
            subaddress,
            status,
            quantity,
            vend_kex_1,
            vend_kex_2,
            vend_kex_3,
            vend_msig_make,
            vend_msig_prepare,
            vend_msig_txset,
            xmr_address,
        }
    }
    pub fn update(orid: String, o: &Json<Order>) -> Order {
        Order {
            orid,
            cid: String::from(&o.cid),
            // fml, the mediator .b32 isn't getting sent to vendor on order creation TODO(c2m): fix it
            pid: String::from(&o.pid),
            cust_kex_1: String::from(&o.cust_kex_1),
            cust_kex_2: String::from(&o.cust_kex_2),
            cust_kex_3: String::from(&o.cust_kex_3),
            cust_msig_make: String::from(&o.cust_msig_make),
            cust_msig_prepare: String::from(&o.cust_msig_make),
            cust_msig_txset: String::from(&o.cust_msig_txset),
            date: o.date,
            deliver_date: o.deliver_date,
            hash: String::from(&o.hash),
            mediator_kex_1: String::from(&o.mediator_kex_1),
            mediator_kex_2: String::from(&o.mediator_kex_2),
            mediator_kex_3: String::from(&o.mediator_kex_3),
            mediator_msig_make: String::from(&o.mediator_msig_make),
            mediator_msig_prepare: String::from(&o.mediator_msig_prepare),
            ship_address: o.ship_address.iter().cloned().collect(),
            ship_date: o.ship_date,
            subaddress: String::from(&o.subaddress),
            status: String::from(&o.status),
            quantity: o.quantity,
            vend_kex_1: String::from(&o.vend_kex_1),
            vend_kex_2: String::from(&o.vend_kex_2),
            vend_kex_3: String::from(&o.vend_kex_3),
            vend_msig_make: String::from(&o.vend_msig_make),
            vend_msig_prepare: String::from(&o.vend_msig_prepare),
            vend_msig_txset: String::from(&o.vend_msig_txset),
            xmr_address: String::from(&o.xmr_address),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Dispute {
    pub did: String,
    pub created: i64,
    pub orid: String,
    pub tx_set: String,
}

impl Default for Dispute {
    fn default() -> Self {
        Dispute {
            did: utils::empty_string(),
            created: 0,
            orid: utils::empty_string(),
            tx_set: utils::empty_string(),
        }
    }
}

impl Dispute {
    pub fn to_db(d: &Dispute) -> String {
        format!("{}:{}:{}", d.created, d.orid, d.tx_set)
    }
    pub fn from_db(k: String, v: String) -> Dispute {
        let values = v.split(":");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let created = match v.remove(0).parse::<i64>() {
            Ok(t) => t,
            Err(_) => 0,
        };
        let orid = v.remove(0);
        let tx_set = v.remove(0);
        Dispute {
            did: k,
            created,
            orid,
            tx_set,
        }
    }
}
