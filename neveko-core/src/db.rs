// db created and exported from here
extern crate lmdb_rs as lmdb;

use lmdb::{
    DbFlags,
    DbHandle,
    EnvBuilder,
    Environment,
};
use log::{
    info,
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
        info!("excecuting lmdb open");
        let release_env = utils::get_release_env();
        let file_path = format!(
            "/home/{}/.{}/",
            std::env::var("USER").unwrap_or(String::from("user")),
            crate::APP_NAME,
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
    pub async fn async_open() -> Self {
        info!("excecuting lmdb async open");
        tokio::time::sleep(std::time::Duration::from_micros(1)).await;
        self::Interface::open()
    }
    /// Write a key-value to LMDB. NEVEKO does not currently support
    ///
    /// writing multiple key value pairs.
    pub fn write(e: &Environment, h: &DbHandle, k: &str, v: &str) {
        info!("excecuting lmdb write");
        // don't try and write empty keys
        if k.is_empty() {
            error!("can't write empty key");
            return;
        }
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
    pub async fn async_write(e: &Environment, h: &DbHandle, k: &str, v: &str) {
        info!("excecuting lmdb async write");
        tokio::time::sleep(std::time::Duration::from_micros(1)).await;
        self::Interface::write(e, h, k, v)
    }
    /// Read a value from LMDB by passing the key as a static
    ///
    /// string. If the value does not exist an empty string is
    ///
    /// returned. NEVEKO does not currently support duplicate keys.
    pub fn read(e: &Environment, h: &DbHandle, k: &str) -> String {
        info!("excecuting lmdb read");
        // don't try and read empty keys
        if k.is_empty() {
            error!("can't read empty key");
            return utils::empty_string();
        }
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
    pub async fn async_read(e: &Environment, h: &DbHandle, k: &str) -> String {
        info!("excecuting lmdb async read");
        tokio::time::sleep(std::time::Duration::from_micros(1)).await;
        self::Interface::read(e, h, k)
    }
    /// Delete a value from LMDB by passing the key as a
    ///
    /// static string. If the value does not exist then an
    ///
    /// error will be logged.
    pub fn delete(e: &Environment, h: &DbHandle, k: &str) {
        info!("excecuting lmdb delete");
        // don't try and delete empty keys
        if k.is_empty() {
            error!("can't delete empty key");
            return;
        }
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
    pub async fn async_delete(e: &Environment, h: &DbHandle, k: &str) {
        info!("excecuting lmdb async delete");
        tokio::time::sleep(std::time::Duration::from_micros(1)).await;
        self::Interface::delete(e, h, k)
    }
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_write_and_read_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        tokio::spawn(async move {
            let s = Interface::async_open().await;
            let k = "async-test-key";
            let v = "async-test-value";
            Interface::async_write(&s.env, &s.handle, k, v).await;
            let expected = String::from(v);
            let actual = Interface::async_read(&s.env, &s.handle, k).await;
            assert_eq!(expected, actual);
            Interface::async_delete(&s.env, &s.handle, &k).await;
        });
    }

    #[test]
    fn async_write_and_delete_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        use tokio::runtime::Runtime;
        let rt = Runtime::new().expect("Unable to create Runtime for test");
        let _enter = rt.enter();
        tokio::spawn(async move {
            let s = Interface::open();
            let k = "write_and_delete_test_test-key";
            let v = "write_and_delete_test_test-value";
            Interface::async_write(&s.env, &s.handle, k, v).await;
            let expected = utils::empty_string();
            Interface::async_delete(&s.env, &s.handle, &k).await;
            let actual = Interface::async_read(&s.env, &s.handle, k).await;
            assert_eq!(expected, actual);
        });
    }
}
