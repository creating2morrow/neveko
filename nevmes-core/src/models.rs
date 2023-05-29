use crate::utils;
use rocket::serde::{
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
    pub xmr_address: String,
    pub gpg_key: Vec<u8>,
}

impl Default for Contact {
    fn default() -> Self {
        Contact {
            cid: utils::empty_string(),
            gpg_key: Vec::new(),
            i2p_address: utils::empty_string(),
            xmr_address: utils::empty_string(),
        }
    }
}

impl Contact {
    pub fn to_db(c: &Contact) -> String {
        let gpg = hex::encode(&c.gpg_key);
        format!("{}!{}!{}", gpg, c.i2p_address, c.xmr_address)
    }
    pub fn from_db(k: String, v: String) -> Contact {
        let values = v.split("!");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let gpg_key = hex::decode(v.remove(0)).unwrap_or(Vec::new());
        let i2p_address = v.remove(0);
        let xmr_address = v.remove(0);
        Contact {
            cid: k,
            gpg_key,
            i2p_address,
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Product {
    pub pid: String,
    pub vid: String,
    pub in_stock: bool,
    pub description: String,
    pub name: String,
    pub price: i64,
    pub qty: i64,
}

impl Default for Product {
    fn default() -> Self {
        Product {
            pid: utils::empty_string(),
            vid: utils::empty_string(),
            description: utils::empty_string(),
            in_stock: false,
            name: utils::empty_string(),
            price: 0,
            qty: 0,
        }
    }
}

impl Product {
    pub fn to_db(p: &Product) -> String {
        format!(
            "{}:{}:{}:{}:{}:{}",
            p.vid, p.description, p.in_stock, p.name, p.price, p.qty
        )
    }
    pub fn from_db(k: String, v: String) -> Product {
        let values = v.split(":");
        let mut v: Vec<String> = values.map(|s| String::from(s)).collect();
        let vid = v.remove(0);
        let description = v.remove(0);
        let in_stock = match v.remove(0).parse::<bool>() {
            Ok(b) => b,
            Err(_) => false,
        };
        let name = v.remove(0);
        let price = match v.remove(0).parse::<i64>() {
            Ok(p) => p,
            Err(_) => 0,
        };
        let qty = match v.remove(0).parse::<i64>() {
            Ok(q) => q,
            Err(_) => 0,
        };
        Product {
            pid: k,
            vid,
            description,
            in_stock,
            name,
            price,
            qty,
        }
    }
    pub fn update(
        p: Product,
        description: String,
        in_stock: bool,
        name: String,
        price: i64,
        qty: i64,
    ) -> Product {
        Product {
            pid: p.pid,
            vid: p.vid,
            description,
            in_stock,
            name,
            price,
            qty,
        }
    }
}

// TODO: add mediator fields

#[derive(Debug)]
pub struct Order {
    pub orid: String,
    pub c_id: String,
    pub p_id: String,
    pub v_id: String,
    pub xmr_address: String,
    pub cust_msig_info: String,
    pub cust_msig_txset: String,
    pub cust_kex_1: String,
    pub cust_kex_2: String,
    pub cust_kex_3: String,
    pub date: i64,
    pub deliver_date: i64,
    pub ship_date: i64,
    pub hash: String,
    pub msig_prepare: String,
    pub msig_make: String,
    pub msig_kex_1: String,
    pub msig_kex_2: String,
    pub msig_kex_3: String,
    pub subaddress: String,
    pub status: String,
    pub quantity: i64,
    pub vend_kex_1: String,
    pub vend_kex_2: String,
    pub vend_kex_3: String,
    pub vend_msig_info: String,
    pub vend_msig_txset: String,
}

impl Default for Order {
    fn default() -> Self {
        Order {
            orid: utils::empty_string(),
            c_id: utils::empty_string(),
            p_id: utils::empty_string(),
            v_id: utils::empty_string(),
            xmr_address: utils::empty_string(),
            cust_msig_info: utils::empty_string(),
            cust_msig_txset: utils::empty_string(),
            cust_kex_1: utils::empty_string(),
            cust_kex_2: utils::empty_string(),
            cust_kex_3: utils::empty_string(),
            date: 0,
            deliver_date: 0,
            ship_date: 0,
            hash: utils::empty_string(),
            msig_prepare: utils::empty_string(),
            msig_make: utils::empty_string(),
            msig_kex_1: utils::empty_string(),
            msig_kex_2: utils::empty_string(),
            msig_kex_3: utils::empty_string(),
            subaddress: utils::empty_string(),
            status: utils::empty_string(),
            quantity: 0,
            vend_kex_1: utils::empty_string(),
            vend_kex_2: utils::empty_string(),
            vend_kex_3: utils::empty_string(),
            vend_msig_info: utils::empty_string(),
            vend_msig_txset: utils::empty_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
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
