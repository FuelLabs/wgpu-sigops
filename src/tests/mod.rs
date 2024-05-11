#[cfg(test)]
pub mod bigint_and_ff;
#[cfg(test)]
pub mod mont;
#[cfg(test)]
pub mod secp256k1_curve;
#[cfg(test)]
pub mod secp256k1_ecdsa;
#[cfg(test)]
pub mod secp256r1_curve;
#[cfg(test)]
pub mod secp256r1_ecdsa;
#[cfg(test)]
pub mod bytes_to_limbs;

use ark_ff::BigInteger;
use ark_ff::fields::PrimeField;
use num_bigint::BigUint;
use fuel_algos::coords;
use fuel_crypto::Signature;
use multiprecision::utils::calc_num_limbs;
use multiprecision::bigint;

pub fn get_secp256k1_b() -> BigUint {
    BigUint::from(7u32)
}

pub fn get_secp256r1_a() -> BigUint {
    BigUint::parse_bytes(b"ffffffff00000001000000000000000000000000fffffffffffffffffffffffc", 16).unwrap()
}

pub fn get_secp256r1_b() -> BigUint {
    BigUint::parse_bytes(b"5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b", 16).unwrap()
}

pub fn fq_to_biguint<F: PrimeField>(val: F) -> BigUint {
    let b = val.into_bigint().to_bytes_be();
    BigUint::from_bytes_be(&b)
}


pub fn projectivexyz_to_mont_limbs<F: PrimeField>(
    a: &coords::ProjectiveXYZ<F>,
    p: &BigUint,
    log_limb_size: u32,
) -> Vec<u32> {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = multiprecision::mont::calc_mont_radix(num_limbs, log_limb_size);
    let a_x_r = fq_to_biguint::<F>(a.x) * &r % p;
    let a_y_r = fq_to_biguint::<F>(a.y) * &r % p;
    let a_z_r = fq_to_biguint::<F>(a.z) * &r % p;
    let a_x_r_limbs = bigint::from_biguint_le(&a_x_r, num_limbs, log_limb_size);
    let a_y_r_limbs = bigint::from_biguint_le(&a_y_r, num_limbs, log_limb_size);
    let a_z_r_limbs = bigint::from_biguint_le(&a_z_r, num_limbs, log_limb_size);
    let mut pt_a_limbs = Vec::<u32>::with_capacity(num_limbs * 3);
    pt_a_limbs.extend_from_slice(&a_x_r_limbs);
    pt_a_limbs.extend_from_slice(&a_y_r_limbs);
    pt_a_limbs.extend_from_slice(&a_z_r_limbs);
    pt_a_limbs
}

pub fn fuel_decode_signature(signature: &Signature) -> (Signature, bool) {
    let mut sig = signature.clone();
    let is_y_odd = (sig[32] & 0x80) != 0;
    sig.as_mut()[32] &= 0x7f;
    (sig, is_y_odd )
}

