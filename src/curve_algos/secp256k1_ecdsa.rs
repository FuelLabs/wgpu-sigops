// https://datatracker.ietf.org/doc/html/rfc6979
#[cfg(test)]
pub mod tests {
    use ark_ec::{AffineRepr, CurveGroup, Group};
    use ark_ff::{BigInteger, Field, PrimeField};
    use ark_secp256k1::{Affine, Fq, Fr, Projective};
    use crypto_bigint::Uint;
    use fuel_crypto::{Message, SecretKey, Signature};
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use sha2::Digest;
    use std::ops::Mul;
    use std::str::FromStr;

    #[test]
    pub fn fuel_ecrecover() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);

        for i in 1..100 {
            let message = Message::new([rng.gen(); 100]);
            let mut i_str = format!("{}", i);
            while i_str.len() < 64 {
                i_str = format!("0{}", i_str);
            }
            let secret = SecretKey::from_str(&i_str).unwrap();
            let public = secret.public_key();
            let fuel_signature = Signature::sign(&secret, &message);

            let recover = fuel_signature
                .recover(&message)
                .expect("Failed to recover PK");
            let (signature, is_y_odd) = decode_signature(&fuel_signature);
            assert_eq!(public, recover);

            // Verify the signature using arkworks
            let z = message.as_slice();
            let r_and_s = signature.as_slice();
            let r = &r_and_s[0..32];
            let s = &r_and_s[32..64];
            let pk_ark = arkworks_recover(r, s, z, is_y_odd, false);
            let pk_hex = format!(
                "{}{}",
                hex::encode(pk_ark.x.into_bigint().to_bytes_be()),
                hex::encode(pk_ark.y.into_bigint().to_bytes_be()),
            );
            assert_eq!(pk_hex, hex::encode(recover.as_slice()));
        }
    }

    fn recover_ys(x: Fq) -> (Fq, Fq) {
        // Copied from
        // https://docs.rs/ark-ec/0.4.0/src/ark_ec/models/short_weierstrass/affine.rs.html#127
        // In secp256k1, a = 0 and b = 7
        // y^2 = x^3 + 7
        let x3 = &x.square() * &x;
        let rhs = x3 + Fq::from_be_bytes_mod_order(&[7u8]);
        let y = rhs.sqrt().unwrap();
        let neg_y = -y;
        match y < neg_y {
            true => (y, neg_y),
            false => (neg_y, y),
        }
    }

    fn arkworks_recover(
        r: &[u8], // x-coord of R
        s: &[u8], // s-value of the signature
        z: &[u8], // message
        is_y_odd: bool,
        is_reduced: bool,
    ) -> Affine {
        // Assume that z < Fr::MODULUS
        let z: Fr = Fr::from_be_bytes_mod_order(z);

        let r_x: Fq = Fq::from_be_bytes_mod_order(r);
        let s = Fr::from_be_bytes_mod_order(s);

        let fr_order_bytes = Fr::MODULUS.to_bytes_be();
        let fr_order: Fq = Fq::from_be_bytes_mod_order(&fr_order_bytes);

        let r_x: Fq = if is_reduced { r_x + fr_order } else { r_x };

        // Recover ys from r_x
        let ys = Affine::get_ys_from_x_unchecked(r_x).unwrap();
        let y0 = ys.0;
        let y1 = ys.1;

        let recovered_ys = recover_ys(r_x);
        assert_eq!(ys, recovered_ys);
        let y0_is_odd = is_odd(&y0);

        let y = if is_y_odd {
            if y0_is_odd {
                y0
            } else {
                y1
            }
        } else {
            // y is even
            if y0_is_odd {
                y1
            } else {
                y0
            }
        };

        let recovered_r = Affine::new(r_x, y);

        let r_x_fr: Fr = Fr::from_bigint(r_x.into_bigint()).unwrap();
        let r_inv: Fr = r_x_fr.inverse().unwrap();

        let u1 = -(r_inv * z);
        let u2 = r_inv * s;

        // Can be done with Shamir's trick
        let recovered_pk = Projective::generator().mul(u1) + recovered_r.mul(u2);

        recovered_pk.into_affine()
    }

    fn decode_signature(signature: &Signature) -> (Signature, bool) {
        let mut sig = signature.clone();
        let is_y_odd = (sig[32] & 0x80) != 0;
        sig.as_mut()[32] &= 0x7f;

        (sig, is_y_odd)
    }

    fn is_odd(val: &Fq) -> bool {
        let bytes = val.into_bigint().to_bytes_be();
        bytes[bytes.len() - 1] % 2u8 != 0u8
    }

    fn message() -> String {
        let message = String::from("A beast can never be as cruel as a human being, so artistically, so picturesquely cruel.");
        message
    }

    fn hash_message(msg: String) -> Vec<u8> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(msg);
        hasher.finalize().to_vec()
    }

    fn k() -> Fr {
        let v = Uint::<4>::from_words([
            16516427254913592388,
            13430632917597294833,
            11381083295555450127,
            10383357845931004348,
        ]);
        let v =
            num_bigint::BigUint::from_bytes_be(hex::decode(format!("{}", v)).unwrap().as_slice());
        let v = ark_ff::BigInt::<4>::try_from(v).unwrap();
        let v = Fr::from_bigint(v).unwrap();
        v
    }

    fn fuel_crypto_sign_and_verify() {
        let message = b"A beast can never be as cruel as a human being, so artistically, so picturesquely cruel.";
        let message = Message::new(message);
        let secret =
            SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let public = secret.public_key();
        let signature = Signature::sign(&secret, &message);
        let recover = signature.recover(&message).expect("Failed to recover PK");
        assert_eq!(public, recover);
    }

    fn arkworks_sign_and_verify() {
        let sk = Fr::from(1u32);
        let g = Affine::generator();
        let pk = g.mul(sk);

        let msghash = hash_message(message());
        let msghash = num_bigint::BigUint::from_bytes_be(msghash.as_slice());
        let msghash = ark_ff::BigInt::<4>::try_from(msghash).unwrap();
        assert!(msghash < Fr::MODULUS);
        let z = Fr::from_bigint(msghash).unwrap();

        // Adapted from https://docs.rs/ecdsa/0.16.9/src/ecdsa/hazmat.rs.html#224
        let k: Fr = k();
        let k_inv: Fr = k.inverse().unwrap();

        let r_pt = g.mul(k).into_affine();

        let r_x: Fq = r_pt.x;
        let r_x_fq_str = format!("{}", r_x);

        let r_x_bigint = r_x.into_bigint();
        let r_x: Fr = Fr::from_bigint(r_x_bigint).unwrap();
        let r_x_fr_str = format!("{}", r_x);

        // is_reduced is true if r_pt.x is greater than the scalar field order
        // since the base field order is greater than the scalar field order,
        // during recovery, is_reduced will tell us whether to add r_x to the scalar field order to
        // recover r_pt.x
        let is_reduced = r_x_fq_str != r_x_fr_str;

        assert!(is_reduced == false);

        let s = k_inv * (z + (r_x * sk));

        let sig_hex = format!(
            "{}{}",
            hex::encode(r_x.into_bigint().to_bytes_be()),
            hex::encode(s.into_bigint().to_bytes_be()),
        );
        assert_eq!(
            sig_hex,
            "46ec716ae185a1d43b537e9ee45e7f178841c9457b5ede4ace9efb585b8ad59f0131dd08f04930d2771de52d2e6aa3f7d12da172ba8af87e963921cd7ed39182"
        );

        // Recover the signer's public key
        // Adapted from https://docs.rs/ecdsa/0.16.9/src/ecdsa/recovery.rs.html#281
        let r_x_bigint = r_x.into_bigint();
        let r_x: Fq = Fq::from_bigint(r_x_bigint).unwrap();

        let fr_order_bytes = Fr::MODULUS.to_bytes_be();
        let fr_order: Fq = Fq::from_be_bytes_mod_order(&fr_order_bytes);

        let r_x: Fq = if is_reduced { r_x + fr_order } else { r_x };

        let is_y_odd = is_odd(&r_pt.y);
        let ys = Affine::get_ys_from_x_unchecked(r_x).unwrap();
        let y0 = ys.0;
        let y1 = ys.1;

        let y = if is_odd(&y0) && is_y_odd {
            y0
        } else if is_odd(&y0) && !is_y_odd {
            y1
        } else if !is_odd(&y1) && is_y_odd {
            y1
        } else {
            y0
        };

        let recovered_r = Affine::new(r_x, y);

        let r_x_fr: Fr = Fr::from_bigint(r_x.into_bigint()).unwrap();
        let r_inv: Fr = r_x_fr.inverse().unwrap();
        let u1 = -(r_inv * z);

        let u2 = r_inv * s;
        let recovered_pk = g.mul(u1) + recovered_r.mul(u2);
        let recovered_pk = recovered_pk.into_affine();

        assert_eq!(recovered_pk, pk);

        let recovered_pk_2 = arkworks_recover(
            &r_x.into_bigint().to_bytes_be(),
            &s.into_bigint().to_bytes_be(),
            &z.into_bigint().to_bytes_be(),
            is_y_odd,
            is_reduced,
        );

        assert_eq!(recovered_pk_2, pk);
    }

    fn k256_sign_and_verify() {
        let sk = Fr::from(1u32);
        // secret key as a k256 SecretKey
        let k_sk = k256::SecretKey::from_slice(&sk.into_bigint().to_bytes_be()).unwrap();
        let k_pk = k256::PublicKey::from_secret_scalar(&k_sk.to_nonzero_scalar());

        assert_eq!(
            hex::encode(k_pk.to_sec1_bytes()),
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
        );

        let k_signingkey: k256::ecdsa::SigningKey = k_sk.into();

        // Hash the message (from Fuel tests)
        let msghash = hash_message(message());

        // Sign the message
        let (signature, recovery_id) = k_signingkey
            .sign_prehash_recoverable(&msghash)
            .expect("Infallible signature operation");

        assert_eq!(
            hex::encode(&msghash),
            "52840c5594968f39c0d7994330b5638405311580b6f9c1b8b3c1f04ca80db7c3"
        );
        assert_eq!(format!("{}", signature), "46EC716AE185A1D43B537E9EE45E7F178841C9457B5EDE4ACE9EFB585B8AD59F0131DD08F04930D2771DE52D2E6AA3F7D12DA172BA8AF87E963921CD7ED39182");

        let signature_as_bytes = signature.to_bytes();

        // Convert the ECDSA signature to a secp256k1 RecoverableSignature
        let recoverable_signature = secp256k1::ecdsa::RecoverableSignature::from_compact(
            &signature_as_bytes,
            secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.to_byte() as i32).unwrap(),
        )
        .unwrap();

        assert_eq!(recovery_id.to_byte(), 0u8);

        // Recover the public key using secp256k1
        let context = secp256k1::Secp256k1::new();
        let vk = context
            .recover_ecdsa(
                &secp256k1::Message::from_slice(&msghash).unwrap(),
                &recoverable_signature,
            )
            .unwrap();

        let k_verifyingkey = k_signingkey.verifying_key();

        assert_eq!(
            hex::encode(k_pk.to_sec1_bytes()),
            hex::encode(k_verifyingkey.to_sec1_bytes())
        );

        assert_eq!(
            hex::encode(vk.serialize_uncompressed()),
            hex::encode(k_verifyingkey.to_encoded_point(false).as_bytes())
        );
    }

    #[test]
    pub fn sign_and_verify() {
        k256_sign_and_verify();
        arkworks_sign_and_verify();
        fuel_crypto_sign_and_verify();
    }
}
