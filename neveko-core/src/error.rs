use kn0sys_lmdb_rs::MdbError;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("neveko error. See logs for more info.")]
pub enum NevekoError {
    ///J4I2PRS(J4RsError),
    Database(MdbError),
    MoneroRpc,
    MoneroDaemon,
    Unknown,
}