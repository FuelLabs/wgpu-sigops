use crate::secp256k1_ecdsa::{ecrecover, ecrecover_single_shader};
use fuel_crypto::{Message, SecretKey, Signature, PublicKey};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use crate::precompute::secp256k1_bases;

const NUM_RUNS_PER_TEST: usize = 100;

#[serial_test::serial]
#[tokio::test]
pub async fn test_secp256k1_ecrecover_single() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();
    for log_limb_size in 13..14 {
        let table_limbs = secp256k1_bases(log_limb_size);

        for _ in 0..NUM_RUNS_PER_TEST {
            // Generate a random message
            let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
            let message = Message::new(hex::encode(msg.to_bytes_be()));

            let secret = SecretKey::random(&mut rng);
            let pk = secret.public_key();

            let fuel_signature = Signature::sign(&secret, &message);
            let recovered = fuel_signature
                .recover(&message)
                .expect("Failed to recover PK");
            let (_decoded_sig, _is_y_odd) =
                crate::tests::fuel_decode_signature(&fuel_signature.clone());
            assert_eq!(recovered, pk);

            do_secp256k1_test(
                &fuel_signature,
                &message,
                &pk,
                &table_limbs,
                log_limb_size,
                true,
            )
            .await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_secp256k1_ecrecover_multi() {
    let mut rng = ChaCha8Rng::seed_from_u64(3);
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();

    for log_limb_size in 13..14 {
        let table_limbs = secp256k1_bases(log_limb_size);

        for _ in 0..NUM_RUNS_PER_TEST {
            // Generate a random message
            let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
            let message = Message::new(hex::encode(msg.to_bytes_be()));

            let secret = SecretKey::random(&mut rng);
            let pk = secret.public_key();

            let fuel_signature = Signature::sign(&secret, &message);
            let recovered = fuel_signature
                .recover(&message)
                .expect("Failed to recover PK");
            let (_decoded_sig, _is_y_odd) =
                crate::tests::fuel_decode_signature(&fuel_signature.clone());
            assert_eq!(recovered, pk);

            do_secp256k1_test(
                &fuel_signature,
                &message,
                &pk,
                &table_limbs,
                log_limb_size,
                false,
            )
            .await;
        }
    }
}

pub async fn do_secp256k1_test(
    signature: &Signature,
    message: &Message,
    verifying_key: &PublicKey,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
    invoke_single: bool,
) {
    let pk_affine_bytes = verifying_key.as_slice();
    let result = if invoke_single {
        ecrecover_single_shader(vec![*signature], vec![*message], log_limb_size).await
    } else {
        ecrecover(vec![*signature], vec![*message], table_limbs, log_limb_size).await
    };
    assert_eq!(result[0], pk_affine_bytes);
}
