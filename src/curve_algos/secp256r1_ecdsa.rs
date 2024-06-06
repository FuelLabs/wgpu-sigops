// https://datatracker.ietf.org/doc/html/rfc6979
#[cfg(test)]
pub mod tests {
    use ark_ec::{CurveGroup, Group};
    use ark_ff::{BigInteger, Field, PrimeField};
    use ark_secp256r1::{Affine, Fq, Fr, Projective};
    use fuel_crypto::secp256r1::p256::{encode_pubkey, recover, sign_prehashed};
    use fuel_crypto::Message;
    use p256::ecdsa::SigningKey;
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use sha2::Digest;
    use std::ops::Mul;

    fn decode_signature(signature: Vec<u8>) -> (Vec<u8>, bool) {
        let mut sig = signature.clone();
        let is_y_odd = (sig[32] & 0x80) != 0;
        sig[32] &= 0x7f;

        (sig, is_y_odd)
    }

    #[test]
    pub fn fuel_ecrecover() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);

        //let message = b"A beast can never be as cruel as a human being, so artistically, so picturesquely cruel.";
        for _ in 1..100 {
            let signing_key = SigningKey::random(&mut rng);
            let verifying_key = signing_key.verifying_key();
            let message = Message::new([rng.gen(); 100]);

            let fuel_signature = sign_prehashed(&signing_key, &message).expect("Couldn't sign");

            let Ok(recovered) = recover(&fuel_signature, &message) else {
                panic!("Failed to recover public key from the message");
            };

            assert_eq!(*recovered, encode_pubkey(*verifying_key));

            let (signature, is_y_odd) = decode_signature(fuel_signature.as_slice().to_vec());

            let pk_bytes = &verifying_key.to_sec1_bytes()[1..65];
            assert_eq!(hex::encode(pk_bytes), hex::encode(recovered));

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

            assert_eq!(pk_hex, hex::encode(pk_bytes));
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

        //let recovered_ys = recover_ys(r_x);
        //assert_eq!(ys, recovered_ys);
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

    pub fn is_odd(val: &Fq) -> bool {
        let bytes = val.into_bigint().to_bytes_be();
        bytes[bytes.len() - 1] % 2u8 != 0u8
    }

    pub fn message() -> String {
        let message = String::from("A beast can never be as cruel as a human being, so artistically, so picturesquely cruel.");
        message
    }

    pub fn hash_message(msg: String) -> Vec<u8> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(msg);
        hasher.finalize().to_vec()
    }
}
