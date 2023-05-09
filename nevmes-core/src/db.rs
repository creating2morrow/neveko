// db created and exported from here
extern crate lmdb_rs as lmdb;

use lmdb::{
    DbFlags,
    DbHandle,
    EnvBuilder,
    Environment,
};
use log::{
    debug,
    error,
};

use crate::utils;

pub struct Interface {
    pub env: Environment,
    pub handle: DbHandle,
}

impl Interface {
    pub fn open() -> Self {
        let release_env = utils::get_release_env();
        let file_path = format!(
            "/home/{}/.nevmes/",
            std::env::var("USER").unwrap_or(String::from("user"))
        );
        let mut env_str: &str = "test-lmdb";
        if release_env != utils::ReleaseEnvironment::Development {
            env_str = "lmdb";
        };
        let env = EnvBuilder::new()
            .open(format!("{}/{}", file_path, env_str), 0o777)
            .unwrap();
        let handle = env.get_default_db(DbFlags::empty()).unwrap();
        Interface { env, handle }
    }
    pub fn write(e: &Environment, h: &DbHandle, k: &str, v: &str) {
        let txn = e.new_transaction().unwrap();
        {
            // get a database bound to this transaction
            let db = txn.bind(&h);
            let pair = vec![(k, v)];
            for &(key, value) in pair.iter() {
                db.set(&key, &value).unwrap();
            }
        }
        match txn.commit() {
            Err(_) => error!("failed to commit!"),
            Ok(_) => (),
        }
    }
    pub fn read(e: &Environment, h: &DbHandle, k: &str) -> String {
        let reader = e.get_reader().unwrap();
        let db = reader.bind(&h);
        let value = db.get::<&str>(&k).unwrap_or_else(|_| "");
        let r = String::from(value);
        {
            if r == utils::empty_string() {
                debug!("Failed to read from db.")
            }
        }
        r
    }
    pub fn delete(e: &Environment, h: &DbHandle, k: &str) {
        let txn = e.new_transaction().unwrap();
        {
            // get a database bound to this transaction
            let db = txn.bind(&h);
            db.del(&k).unwrap_or_else(|_| error!("failed to delete"));
        }
        match txn.commit() {
            Err(_) => error!("failed to commit!"),
            Ok(_) => (),
        }
    }
}
