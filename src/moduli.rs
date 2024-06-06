use ark_ff::{BigInteger, PrimeField};
use num_bigint::BigUint;

pub fn secp256k1_fq_modulus_biguint() -> BigUint {
    BigUint::from_bytes_be(&ark_secp256k1::Fq::MODULUS.to_bytes_be())
}

pub fn secp256k1_fq_modulus() -> ark_secp256k1::Fq {
    ark_secp256k1::Fq::from_be_bytes_mod_order(&ark_secp256k1::Fq::MODULUS.to_bytes_be())
}

pub fn secp256k1_fr_modulus_biguint() -> BigUint {
    BigUint::from_bytes_be(&ark_secp256k1::Fr::MODULUS.to_bytes_be())
}

pub fn secp256k1_fr_modulus() -> ark_secp256k1::Fr {
    ark_secp256k1::Fr::from_be_bytes_mod_order(&ark_secp256k1::Fr::MODULUS.to_bytes_be())
}

pub fn secp256r1_fq_modulus_biguint() -> BigUint {
    BigUint::from_bytes_be(&ark_secp256r1::Fq::MODULUS.to_bytes_be())
}

pub fn secp256r1_fq_modulus() -> ark_secp256r1::Fq {
    ark_secp256r1::Fq::from_be_bytes_mod_order(&ark_secp256r1::Fq::MODULUS.to_bytes_be())
}

pub fn secp256r1_fr_modulus_biguint() -> BigUint {
    BigUint::from_bytes_be(&ark_secp256r1::Fr::MODULUS.to_bytes_be())
}

pub fn secp256r1_fr_modulus() -> ark_secp256r1::Fr {
    ark_secp256r1::Fr::from_be_bytes_mod_order(&ark_secp256r1::Fr::MODULUS.to_bytes_be())
}

pub fn ed25519_fq_modulus_biguint() -> BigUint {
    BigUint::from_bytes_be(&ark_ed25519::Fq::MODULUS.to_bytes_be())
}

pub fn ed25519_fq_modulus() -> ark_ed25519::Fq {
    ark_ed25519::Fq::from_be_bytes_mod_order(&ark_ed25519::Fq::MODULUS.to_bytes_be())
}

pub fn ed25519_fr_modulus_biguint() -> BigUint {
    BigUint::from_bytes_be(&ark_ed25519::Fr::MODULUS.to_bytes_be())
}

pub fn ed25519_fr_modulus() -> ark_ed25519::Fr {
    ark_ed25519::Fr::from_be_bytes_mod_order(&ark_ed25519::Fr::MODULUS.to_bytes_be())
}
