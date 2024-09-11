//! TODO(c2m): remove this module since there is only support for a single
//! authenticated user

use crate::{
    db::{self, DATABASE_LOCK},
    models::*,
    utils,
};
use kn0sys_lmdb_rs::MdbError;
use log::{
    debug,
    error,
};

/// Create a new user
pub fn create(address: &String) -> Result<User, MdbError> {
    let f_uid: String = format!("{}{}", crate::USER_DB_KEY, utils::generate_rnd());
    let new_user = User {
        uid: String::from(&f_uid),
        xmr_address: String::from(address),
        name: String::new(),
    };
    debug!("insert user: {:?}", &new_user);
    let db = &DATABASE_LOCK;
    let k = &new_user.uid;
    let v = bincode::serialize(&new_user).unwrap_or_default();
    db::write_chunks(&db.env, &db.handle, k.as_bytes(), &v)?;
    Ok(new_user)
}

/// User lookup
pub fn find(uid: &String) -> Result<User, MdbError> {
    let db = &DATABASE_LOCK;
    let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &uid.as_bytes().to_vec())?;
    if r.is_empty() {
        error!("user not found");
        return Err(MdbError::NotFound);
    }
    let user: User = bincode::deserialize(&r[..]).unwrap_or_default();
    Ok(user)
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use kn0sys_lmdb_rs::MdbError;

    use super::*;

    fn cleanup(k: &String) -> Result<(), MdbError> {
        let db = &DATABASE_LOCK;
        db::DatabaseEnvironment::delete(&db.env, &db.handle, k.as_bytes());
        Ok(())
    }

    #[test]
    fn create_test() {
        // run and async cleanup so the test doesn't fail when deleting test data
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let test_user = create(&address);
        let db = &DATABASE_LOCK;
        let r = db::DatabaseEnvironment::read(&db.env, &db.handle, &test_user.uid);
        let id = String::from(&test_user.uid);
        let cleanup_id = String::from(&test_user.uid);
        let expected_user = User::from_db(id, r);
        assert_eq!(test_user.xmr_address, expected_user.xmr_address);
        cleanup(&cleanup_id);
    }

    #[test]
    fn find_test() {
        let address: String = String::from(
            "73a4nWuvkYoYoksGurDjKZQcZkmaxLaKbbeiKzHnMmqKivrCzq5Q2JtJG1UZNZFqLPbQ3MiXCk2Q5bdwdUNSr7X9QrPubkn"
        );
        let k = "c123";
        let expected_user = User {
            xmr_address: address,
            ..Default::default()
        };
        let db = &DATABASE_LOCK;
        db::DatabaseEnvironment::write_chunks(&db.env, &db.handle, k, &User::to_db(&expected_user));
        let actual_user: User = find(&String::from(k));
        assert_eq!(expected_user.xmr_address, actual_user.xmr_address);
        cleanup(&String::from(k));
    }
}
