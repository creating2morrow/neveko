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

/// LMDB Interface allows access to the env
///
/// and handle for the write, read and delete
///
/// functionality.
pub struct Interface {
    pub env: Environment,
    pub handle: DbHandle,
}

impl Interface {
    /// Instantiation of ```Environment``` and ```DbHandle```
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
            .expect(&format!("could not open LMDB at {}", file_path));
        let handle = env.get_default_db(DbFlags::empty()).unwrap();
        Interface { env, handle }
    }
    /// Write a key-value to LMDB. NEVMES does not currently support
    ///
    /// writing multiple key value pairs.
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
    /// Read a value from LMDB by passing the key as a static
    ///
    /// string. If the value does not exist an empty string is
    ///
    /// returned. NEVMES does not currently support duplicate keys.
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
    /// Delete a value from LMDB by passing the key as a
    ///
    /// static string. If the value does not exist then an
    ///
    /// error will be logged.
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

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read_test() {
        let s = Interface::open();
        let k = "test-key";
        let v = "test-value";
        Interface::write(&s.env, &s.handle, k, v);
        let expected = String::from(v);
        let actual = Interface::read(&s.env, &s.handle, k);
        assert_eq!(expected, actual);
        Interface::delete(&s.env, &s.handle, &k);
    }

    #[test]
    fn write_and_delete_test() {
        let s = Interface::open();
        let k = "test-key";
        let v = "test-value";
        Interface::write(&s.env, &s.handle, k, v);
        let expected = utils::empty_string();
        Interface::delete(&s.env, &s.handle, &k);
        let actual = Interface::read(&s.env, &s.handle, k);
        assert_eq!(expected, actual);
    }
}
