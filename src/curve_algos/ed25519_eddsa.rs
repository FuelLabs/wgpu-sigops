use num_bigint::BigUint;
use ark_ff::{BigInteger, Field};
use crate::curve_algos::coords::ETEProjective;
use sha2::Digest;
use ed25519_dalek::{VerifyingKey, Signature};
use curve25519_dalek::Scalar;
use curve25519_dalek::edwards::CompressedEdwardsY;
use curve25519_dalek::edwards::EdwardsPoint;
use ark_ec::CurveGroup;
use std::ops::Neg;
use ark_ec::AffineRepr;
use ark_ec::twisted_edwards::TECurveConfig;
use ark_ff::PrimeField;
use ark_ed25519::{EdwardsAffine, EdwardsConfig, Fq, Fr};

pub fn ark_ecverify(
    verifying_key: &VerifyingKey,
    signature: &Signature,
    message: &[u8],
) -> EdwardsAffine {
    // signature contains scalar s and the y-coordinate of the point R
    // verifying key = point A
    // k = hash(r, a, msg)
    // return sG - kA
    // 
    // sG - kA should equal R.
    let s_bytes = signature.s_bytes();
    let a_bytes = verifying_key.as_bytes();

    let hash = compute_hash(verifying_key, signature, message);
    let s = Fr::from_le_bytes_mod_order(s_bytes);
    let k = Fr::from_le_bytes_mod_order(&hash);

    let g = EdwardsAffine::generator();
    let gs: EdwardsAffine = (g * s).into_affine();

    let a_pt = compressed_y_to_eteprojective(*a_bytes);

    let a_pt = EdwardsAffine::new(a_pt.x, a_pt.y);

    let neg_a_pt = a_pt.neg();
    let k_neg_a_pt = (neg_a_pt * k).into_affine();

    let ark_recovered = (gs + k_neg_a_pt).into_affine();

    ark_recovered
}

pub fn curve25519_ecverify(
    verifying_key: &VerifyingKey,
    signature: &Signature,
    message: &[u8],
) -> Vec<u8> {
    let s_bytes = signature.s_bytes();
    let a_bytes = verifying_key.as_bytes();

    let hash = compute_hash(verifying_key, signature, message);

    let a_pt = CompressedEdwardsY::from_slice(a_bytes).unwrap().decompress().unwrap();

    let s = Scalar::from_bytes_mod_order(*s_bytes);
    let k = Scalar::from_bytes_mod_order_wide(hash.as_slice().try_into().unwrap());

    let g = curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
    let gs = g * s;
    let k_neg_a = k * (-a_pt);

    let cd_recovered = EdwardsPoint::vartime_double_scalar_mul_basepoint(&k, &(-a_pt), &s).compress();

    assert_eq!((gs + k_neg_a).compress(), cd_recovered);

    Vec::<u8>::from(cd_recovered.to_bytes())
}

pub fn decompress_to_ete_unsafe(r_bytes: [u8; 32]) -> (bool, ETEProjective::<Fq>) {
    let compressed_sign_bit = r_bytes[31] >> 7;

    let mut y_bytes = r_bytes;
    y_bytes[31] &= 0x7f;
    let y = Fq::from_le_bytes_mod_order(&y_bytes);

    let z = Fq::from(1u32);
    let yy = &y * &y;
    let u = &yy - &z;
    let v = &(&yy * &EdwardsConfig::COEFF_D) + &z;

    let (is_valid_y_coord, mut x) = sqrt_ratio_i(&u, &v);

    x = conditional_negate(x, compressed_sign_bit == 1u8);

    let t = x * y;

    (is_valid_y_coord, ETEProjective::<Fq> { x, y, t, z })
}

pub fn compressed_y_to_eteprojective(r_bytes: [u8; 32]) -> ETEProjective::<Fq> {
    let r = decompress_to_ete_unsafe(r_bytes);
    assert!(r.0);
    r.1
}

pub fn compute_hash(
    verifying_key: &VerifyingKey,
    signature: &Signature,
    message: &[u8],
    ) -> Vec<u8> {
    let r_bytes = signature.r_bytes();
    let a_bytes = verifying_key.as_bytes();
    let m_bytes = &message;
    let mut hasher = sha2::Sha512::new();
    hasher.update(&r_bytes);
    hasher.update(&a_bytes);
    hasher.update(&m_bytes);
    let result = hasher.finalize();
    Vec::<u8>::from(result.as_slice())
}


pub fn compress_ark_projective(pt: EdwardsAffine) -> Vec<u8> {
    let mut y_bytes = pt.y.into_bigint().to_bytes_be();
    let neg_bit = if is_negative(pt.x) {
        1u8
    } else {
        0u8
    };

    y_bytes[0] ^= neg_bit << 7u8;

    Vec::<u8>::from(y_bytes)
}

pub fn conditional_assign(
    a: Fq,
    b: Fq,
    choice: bool
    ) -> Fq {
    if choice {
        return b;
    }
    a
}

pub fn pow_p58(a: Fq) -> Fq {
    // Raise this field element to the power (p-5)/8 = 2^252 -3.
    let power = BigUint::parse_bytes(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd", 16).unwrap();
    a.pow(power.to_u64_digits())
}

pub fn is_negative(a: Fq) -> bool {
    let bytes = a.into_bigint().to_bytes_le();
    bytes[0] & 1u8 == 1u8
}

pub fn conditional_negate(
    a: Fq,
    choice: bool
    ) -> Fq {
    if choice {
        return -a;
    }
    a
}

pub fn sqrt_ratio_i(u: &Fq, v: &Fq) -> (bool, Fq) {
    let v3 = &v.square() * v;
    let v7 = &v3.square() * v;

    let mut r = *&(u * &v3) * pow_p58(u * &v7);
    let check = v * &r.square();

    let i = Fq::from(-1i32).sqrt().unwrap();

    let correct_sign_sqrt = check == *u;
    let flipped_sign_sqrt = check == -*u;
    let flipped_sign_sqrt_i = check == -*u * i;

    let r_prime = &i * &r;

    r = conditional_assign(r, r_prime, flipped_sign_sqrt | flipped_sign_sqrt_i);

    let r_is_negative = is_negative(r);

    r = conditional_negate(r, r_is_negative);

    let was_nonzero_square = correct_sign_sqrt | flipped_sign_sqrt;

    (was_nonzero_square, r)
}

#[cfg(test)]
pub mod tests {
    use ed25519_dalek::{ SigningKey, Signature, Signer, Verifier };
    use rand::RngCore;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use crate::curve_algos::ed25519_eddsa::{curve25519_ecverify, ark_ecverify, compress_ark_projective};

    // Lots of code in this file is ported from curve25519_dalek
    // Also ported from verify_strict():
    // https://docs.rs/ed25519-dalek/latest/src/ed25519_dalek/verifying.rs.html#402-425

    #[test]
    pub fn ecverify() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);

        for _i in 0..50 {
            let mut message = [0u8; 100];
            rng.fill_bytes(&mut message);

            let signing_key: SigningKey = SigningKey::generate(&mut rng);
            let verifying_key = signing_key.verifying_key();

            let signature: Signature = signing_key.sign(&message);

            assert!(verifying_key.verify(&message, &signature).is_ok());

            let cd_recovered_bytes = curve25519_ecverify(&verifying_key, &signature, &message);

            let mut ark_recovered_bytes = compress_ark_projective(ark_ecverify(&verifying_key, &signature, &message));
            ark_recovered_bytes.reverse();

            assert_eq!(ark_recovered_bytes, cd_recovered_bytes);

            let sig_r_bytes = signature.r_bytes();
            assert_eq!(*sig_r_bytes, *cd_recovered_bytes);
        }
    }
}
