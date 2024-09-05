//! Custom object relational mapping (ORM) for structs into LMBD

use rocket::serde::{
    json::Json,
    Deserialize,
    Serialize,
};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Authorization {
    pub aid: String,
    pub created: i64,
    pub rnd: String,
    pub token: String,
    pub uid: String,
    pub xmr_address: String,
}


impl Authorization {
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Contact {
    pub cid: String,
    pub i2p_address: String,
    pub is_vendor: bool,
    pub xmr_address: String,
    pub nmpk: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    pub mid: String,
    pub uid: String,
    pub body: String,
    pub created: i64,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub uid: String,
    pub xmr_address: String,
    pub name: String,
}

impl User {
    pub fn update(u: User, name: String) -> User {
        User {
            uid: u.uid,
            name,
            xmr_address: u.xmr_address,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
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

impl Product {
    pub fn update(p: Product, jp: &Json<Product>) -> Product {
        Product {
            pid: p.pid,
            description: String::from(&jp.description),
            image: jp.image.to_vec(),
            in_stock: jp.in_stock,
            name: String::from(&jp.name),
            price: jp.price,
            qty: jp.qty,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
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
    pub adjudicator_kex_1: String,
    pub adjudicator_kex_2: String,
    pub adjudicator_kex_3: String,
    pub adjudicator_msig_make: String,
    pub adjudicator_msig_prepare: String,
    /// Address enciphered by nmpk
    pub ship_address: String,
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

impl Order {
    pub fn update(orid: String, o: &Json<Order>) -> Order {
        Order {
            orid,
            cid: String::from(&o.cid),
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
            adjudicator_kex_1: String::from(&o.adjudicator_kex_1),
            adjudicator_kex_2: String::from(&o.adjudicator_kex_2),
            adjudicator_kex_3: String::from(&o.adjudicator_kex_3),
            adjudicator_msig_make: String::from(&o.adjudicator_msig_make),
            adjudicator_msig_prepare: String::from(&o.adjudicator_msig_prepare),
            ship_address: String::from(&o.ship_address),
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Dispute {
    pub did: String,
    pub created: i64,
    pub orid: String,
    pub tx_set: String,
}
