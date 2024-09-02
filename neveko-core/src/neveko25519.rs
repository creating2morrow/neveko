//! NEVEKO modified ed25519 library extending curve25519-dalek

use curve25519_dalek::{
    edwards::{
        CompressedEdwardsY,
        EdwardsPoint,
    },
    scalar::Scalar,
};
use num::{
    bigint::Sign,
    BigInt,
};
use sha2::{
    Digest,
    Sha512,
};

use crate::{
    monero,
    utils,
};

#[derive(Debug)]
/// Container for the Neveko Message Keys
pub struct NevekoMessageKeys {
    /// Neveko Message Secret Key
    pub nmsk: [u8; 32],
    /// Neveko Message Public Key
    pub nmpk: [u8; 32],
    /// Hex encoding of NMSK
    pub hex_nmsk: String,
    /// Hex encoding of NMPK
    pub hex_nmpk: String,
}

impl Default for NevekoMessageKeys {
    fn default() -> Self {
        NevekoMessageKeys {
            nmpk: [0u8; 32],
            nmsk: [0u8; 32],
            hex_nmpk: utils::empty_string(),
            hex_nmsk: utils::empty_string(),
        }
    }
}

/// L value as defined at https://eprint.iacr.org/2008/013.pdf
const CURVE_L: &str = "edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";
pub const ENCIPHER: &str = "ENCIPHER";

fn curve_l_as_big_int() -> BigInt {
    BigInt::from_bytes_le(Sign::Plus, CURVE_L.as_bytes())
}

fn big_int_to_string(b: &BigInt) -> String {
    String::from_utf8(b.to_signed_bytes_le()).unwrap_or_default()
}

/// Hash string input to scalar
fn hash_to_scalar(s: Vec<&str>) -> Scalar {
    let mut result = String::new();
    for v in &s {
        let mut hasher = Sha512::new();
        hasher.update(v);
        let hash = hasher.finalize().to_owned();
        result += &hex::encode(&hash[..]);
    }
    loop {
        let mut hasher = Sha512::new();
        hasher.update(&result);
        let hash = hasher.finalize().to_owned();
        let mut hash_container: [u8; 32] = [0u8; 32];
        let mut index = 0;
        for byte in result.as_bytes() {
            if index == hash_container.len() - 1 {
                break;
            }
            hash_container[index] = *byte;
            index += 1;
        }
        let hash_value = BigInt::from_bytes_le(Sign::Plus, &hash_container);
        if hash_value < curve_l_as_big_int() {
            return Scalar::from_bytes_mod_order(hash_container);
        }
        result = hex::encode(&hash[..]);
    }
}

/// Hash the secret view key and the application name
///
/// to a valid scalar creating the Neveko Secret Message Key.
///
/// Multiply the NMSK by the ed25519 basepoint to create the
///
/// Neveko Message Public Key (NMPK).
pub async fn generate_neveko_message_keys() -> NevekoMessageKeys {
    log::info!("generating neveko message keys");
    let password = std::env::var(crate::MONERO_WALLET_PASSWORD).unwrap_or(utils::empty_string());
    let filename = String::from(crate::APP_NAME);
    let m_wallet = monero::open_wallet(&filename, &password).await;
    if !m_wallet {
        log::error!("failed to open wallet");
        return Default::default();
    }
    let svk_res = monero::query_view_key().await;
    monero::close_wallet(&filename, &password).await;
    let svk = svk_res.result.key;
    let scalar_nmsk = hash_to_scalar(vec![&svk[..], crate::APP_NAME]);
    let point_nmpk = EdwardsPoint::mul_base(&scalar_nmsk);
    let nmsk = *scalar_nmsk.as_bytes();
    let nmpk: [u8; 32] = *point_nmpk.compress().as_bytes();
    let hex_nmpk = hex::encode(nmpk);
    let hex_nmsk = hex::encode(nmsk);
    NevekoMessageKeys {
        nmpk,
        nmsk,
        hex_nmpk,
        hex_nmsk,
    }
}

/// Encipher a string by using the contact's Neveko Message Public Key.
///
/// E.g. shared_secret_alice = nmpk_bob * nmsk_alice = h`
///
/// `m = "some message to encipher"`
///
/// Return `x = m + h` as a string of the enciphered message.
///
/// Pass `None` to encipher parameter to perform deciphering.
pub async fn cipher(hex_nmpk: &String, message: String, encipher: Option<String>) -> String {
    let unwrap_encipher: String = encipher.unwrap_or(utils::empty_string());
    let keys: NevekoMessageKeys = generate_neveko_message_keys().await;
    // shared secret = nmpk * nmsk
    let scalar_nmsk = Scalar::from_bytes_mod_order(keys.nmsk);
    let mut nmpk: [u8; 32] = [0u8; 32];
    hex::decode_to_slice(hex_nmpk, &mut nmpk as &mut [u8]).unwrap_or_default();
    let compress_y = CompressedEdwardsY::from_slice(&nmpk).unwrap_or_default();
    let compress_nmpk = compress_y.decompress().unwrap_or_default();
    let shared_secret = compress_nmpk * scalar_nmsk;
    let ss_hex = hex::encode(shared_secret.compress().as_bytes());
    // x = m + h or x = m - h'
    let h = hash_to_scalar(vec![&ss_hex[..]]);
    let h_bi = BigInt::from_bytes_le(Sign::Plus, h.as_bytes());
    if unwrap_encipher == *ENCIPHER {
        let msg_bi = BigInt::from_bytes_le(Sign::Plus, message.as_bytes());
        let x = msg_bi + h_bi;
        hex::encode(x.to_bytes_le().1)
    } else {
        let msg_bi = BigInt::from_bytes_le(Sign::Plus, &hex::decode(&message).unwrap_or_default());
        let x = msg_bi - h_bi;
        big_int_to_string(&x)
    }
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cipher(message: &String, encipher: Option<String>) -> String {
        let unwrap_encipher: String = encipher.unwrap_or(utils::empty_string());
        let test_nmpk: [u8; 32] = [
            203, 2, 188, 13, 167, 96, 59, 189, 38, 238, 2, 71, 84, 155, 153, 73, 241, 137, 9, 30,
            28, 134, 91, 137, 134, 73, 231, 45, 174, 98, 103, 158,
        ];
        let nmsk: [u8; 32] = [
            54, 55, 48, 48, 99, 48, 101, 52, 102, 99, 99, 56, 54, 56, 50, 50, 52, 101, 101, 55, 51,
            48, 102, 54, 54, 57, 101, 97, 54, 100, 101, 0,
        ];
        let hex_nmpk = hex::encode(test_nmpk);
        let hex_nmsk = hex::encode(nmsk);
        let mut nmpk: [u8; 32] = [0u8; 32];
        hex::decode_to_slice(hex_nmpk.clone(), &mut nmpk as &mut [u8]).unwrap_or_default();
        assert_eq!(test_nmpk, nmpk);
        let keys: NevekoMessageKeys = NevekoMessageKeys {
            nmsk,
            nmpk,
            hex_nmpk,
            hex_nmsk,
        };
        // shared secret = nmpk * nmks
        let scalar_nmsk = Scalar::from_bytes_mod_order(keys.nmsk);
        let compress_y = CompressedEdwardsY::from_slice(&nmpk).unwrap_or_default();
        let nmpk_compress = compress_y.decompress().unwrap_or_default();
        let shared_secret = nmpk_compress * scalar_nmsk;
        let ss_hex = hex::encode(shared_secret.compress().as_bytes());
        // x = m + h or x = m - h'
        let h = hash_to_scalar(vec![&ss_hex[..]]);
        let h_bi = BigInt::from_bytes_le(Sign::Plus, h.as_bytes());
        if unwrap_encipher == String::from(ENCIPHER) {
            let msg_bi = BigInt::from_bytes_le(Sign::Plus, &message.as_bytes());
            let x = msg_bi + h_bi;
            return hex::encode(x.to_bytes_le().1);
        } else {
            let msg_bi =
                BigInt::from_bytes_le(Sign::Plus, &hex::decode(&message).unwrap_or_default());
            let x = msg_bi - h_bi;
            return big_int_to_string(&x);
        };
    }

    #[test]
    pub fn encipher_decipher() {
        let message = String::from(
            "This is message that will be enciphered by the network. 
        it is really long for testing and breaking stuff",
        );
        let do_encipher = Some(String::from(ENCIPHER));
        let encipher = test_cipher(&message, do_encipher);
        assert_ne!(encipher, message);
        let decipher = test_cipher(&encipher, None);
        assert_eq!(decipher, message);
    }
}
