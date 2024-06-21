use crate::precompute::ed25519_bases;
use crate::ed25519_eddsa::{ecverify, ecverify_single};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use fuel_crypto::Message;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use rand::RngCore;

#[serial_test::serial]
#[tokio::test]
pub async fn test_ed25519_ecverify_single() {
    let mut rng = ChaCha8Rng::seed_from_u64(1);

    for log_limb_size in 13..14 {
        let table_limbs = ed25519_bases(log_limb_size);
        for _ in 0..10 {
            let mut message = [0u8; 100];
            rng.fill_bytes(&mut message);
            let message_m = Message::new(&message);
            let message = message_m.as_slice();

            let signing_key: SigningKey = SigningKey::generate(&mut rng);
            let verifying_key = signing_key.verifying_key();
            let signature: Signature = signing_key.sign(&message);

            assert!(verifying_key.verify(&message, &signature).is_ok());

            do_eddsa_test(&verifying_key, &signature, &message_m, &table_limbs, log_limb_size, true).await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_ed25519_ecverify_multi() {
    let mut rng = ChaCha8Rng::seed_from_u64(1);

    for log_limb_size in 13..14 {
        let table_limbs = ed25519_bases(log_limb_size);
        for _ in 0..10 {
            let mut message = [0u8; 100];
            rng.fill_bytes(&mut message);
            let message_m = Message::new(&message);
            let message = message_m.as_slice();

            let signing_key: SigningKey = SigningKey::generate(&mut rng);
            let verifying_key = signing_key.verifying_key();
            let signature: Signature = signing_key.sign(&message);

            assert!(verifying_key.verify(&message, &signature).is_ok());

            do_eddsa_test(&verifying_key, &signature, &message_m, &table_limbs, log_limb_size, false).await;
        }
    }
}

pub async fn do_eddsa_test(
    verifying_key: &VerifyingKey,
    signature: &Signature,
    message: &Message,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
    invoke_single: bool,
) {
    let result = if invoke_single {
        ecverify_single(&vec![*signature], &vec![*message], &vec![*verifying_key], log_limb_size).await
    } else {
        ecverify(&vec![*signature], &vec![*message], &vec![*verifying_key], table_limbs, log_limb_size).await
    };
    for r in result {
        assert!(r);
    }
}
