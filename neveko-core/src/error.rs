use kn0sys_lmdb_rs::MdbError;
use thiserror::Error;

/// Use for mapping errors in functions that can throw multiple errors.
#[derive(Debug, Error)]
#[error("Neveko error. See logs for more info.")]
pub enum NevekoError {
    Auth,
    Contact,
    Database(MdbError),
    Dispute,
    I2P,
    J4I2PRS,
    Message,
    MoneroRpc,
    MoneroDaemon,
    Nasr,
    Order,
    Product,
    Unknown,
}
