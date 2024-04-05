use curve25519_dalek::{
    constants,
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
use rand_core::{
    OsRng,
    RngCore,
};
use sha2::{
    Digest,
    Sha512,
};

use crate::utils;

/// L value as defined at https://eprint.iacr.org/2008/013.pdf
const CURVE_L: &str = "edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010";

fn curve_l_as_big_int() -> BigInt {
    BigInt::from_bytes_le(Sign::Plus, CURVE_L.as_bytes())
}

fn big_int_to_string(b: &BigInt) -> String {
    String::from_utf8(b.to_signed_bytes_le()).unwrap_or(utils::empty_string())
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

/// Convert Monero secret view Key to a Scalar.
fn xmr_svk_to_scalar(svk: String) -> Scalar {
    todo!()
}

/// Extract public view key from Monero address to be represented
///
/// as a CompressedEdwards point.
fn xmr_address_to_pvk_point(address: String) -> CompressedEdwardsY {
    todo!()
}

/// Encipher a string by using the contact's public view key.
///
/// E.g. ss_alice = pvk_bob(address) * svk_alice = h`
///
/// `m = "some message to encipher"`
///
/// Return `x = m + h`
pub fn encipher(address: String, message: String) -> CompressedEdwardsY {
    todo!()
}

/// Decipher a string by using the secret view key.
///
/// E.g. ss_bob = `pvk_alice(address) * svk_bob = h'`
///
/// `m = "some message to decipher"`
///
/// Return `m = x - h'`
pub fn decipher(address: String, message: String) -> CompressedEdwardsY {
    todo!()
}

// Tests
//-------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn encipher_decipher() {
        let csprng = &mut OsRng;
        let mut a_bytes = [0u8; 32];
        let mut b_bytes = [0u8; 32];
        OsRng::fill_bytes(csprng, &mut a_bytes);
        OsRng::fill_bytes(csprng, &mut b_bytes);
        let sk_a = Scalar::from_bytes_mod_order(a_bytes);
        let sk_b = Scalar::from_bytes_mod_order(b_bytes);
        let pk_a = EdwardsPoint::mul_base(&sk_a);
        let pk_b = EdwardsPoint::mul_base(&sk_b);
        let ss_a = pk_b * sk_a;
        let ss_b = pk_a * sk_b;
        let h_ss_a = hex::encode(&ss_a.compress().to_bytes());
        let h_ss_b = hex::encode(&ss_b.compress().to_bytes());
        let msg = "this is a really long message that will be encrypted by the shared secret";
        let msg_bi = BigInt::from_bytes_le(Sign::Plus, &msg.as_bytes());
        let h = hash_to_scalar(vec![&h_ss_a[..]]);
        let h_bi = BigInt::from_bytes_le(Sign::Plus, h.as_bytes());
        let x = msg_bi + h_bi;
        let x_decoded = big_int_to_string(&x);
        assert_ne!(String::from(msg), x_decoded);
        let h_prime = hash_to_scalar(vec![&h_ss_b[..]]);
        let h_prime_bi = BigInt::from_bytes_le(Sign::Plus, h_prime.as_bytes());
        let m_b = x - h_prime_bi;
        let decoded = big_int_to_string(&m_b);
        assert_eq!(String::from(msg), decoded);
    }
}
