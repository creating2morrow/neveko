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
