use crate::secp256r1_ecdsa::{ecrecover, ecrecover_single_shader};
use fuel_crypto::secp256r1::p256::{encode_pubkey, recover, sign_prehashed};
use fuel_crypto::Message;
use num_bigint::{BigUint, RandomBits};
use p256::ecdsa::{SigningKey, VerifyingKey};
use fuel_types::Bytes64;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use crate::precompute::secp256r1_bases;

const NUM_RUNS_PER_TEST: usize = 10;

#[serial_test::serial]
#[tokio::test]
pub async fn test_secp256r1_ecrecover_single() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let scalar_p = crate::moduli::secp256r1_fr_modulus_biguint();
    for log_limb_size in 13..14 {
        let table_limbs = secp256r1_bases(log_limb_size);
        for _ in 0..NUM_RUNS_PER_TEST {
            // Generate a random message
            let signing_key = SigningKey::random(&mut rng);
            let verifying_key = signing_key.verifying_key();

            let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
            let message = Message::new(hex::encode(msg.to_bytes_be()));

            let fuel_signature = sign_prehashed(&signing_key, &message).expect("Couldn't sign");

            let Ok(recovered) = recover(&fuel_signature, &message) else {
                panic!("Failed to recover public key from the message");
            };

            assert_eq!(*recovered, encode_pubkey(*verifying_key));

            do_secp256r1_test(
                &fuel_signature,
                &message,
                &verifying_key,
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
pub async fn test_secp256r1_ecrecover_multi() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let scalar_p = crate::moduli::secp256r1_fr_modulus_biguint();
    for log_limb_size in 13..14 {
        let table_limbs = secp256r1_bases(log_limb_size);
        for _ in 0..NUM_RUNS_PER_TEST {
            // Generate a random message
            let signing_key = SigningKey::random(&mut rng);
            let verifying_key = signing_key.verifying_key();

            let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
            let message = Message::new(hex::encode(msg.to_bytes_be()));

            let fuel_signature = sign_prehashed(&signing_key, &message).expect("Couldn't sign");

            let Ok(recovered) = recover(&fuel_signature, &message) else {
                panic!("Failed to recover public key from the message");
            };

            assert_eq!(*recovered, encode_pubkey(*verifying_key));

            do_secp256r1_test(
                &fuel_signature,
                &message,
                &verifying_key,
                &table_limbs,
                log_limb_size,
                false,
            )
            .await;
        }
    }
}

pub async fn do_secp256r1_test(
    signature: &Bytes64,
    message: &Message,
    verifying_key: &VerifyingKey,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
    invoke_single: bool,
) {
    let pk_affine_bytes = &verifying_key.to_sec1_bytes()[1..65];
    let result = if invoke_single {
        ecrecover_single_shader(vec![*signature], vec![*message], log_limb_size).await
    } else {
        ecrecover(vec![*signature], vec![*message], table_limbs, log_limb_size).await
    };
    assert_eq!(result[0], pk_affine_bytes);
}
