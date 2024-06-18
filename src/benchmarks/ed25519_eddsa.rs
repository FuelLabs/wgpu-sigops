use crate::ed25519_eddsa::{ecverify, ecverify_single};
use crate::curve_algos::ed25519_eddsa::{ark_ecverify, curve25519_ecverify};
use ed25519_dalek::{Signature, Signer, SigningKey};
use fuel_crypto::Message;
use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;

const START: usize = 10;
const END: usize = 18;

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_multiple_benchmarks() {
    let check = false;
    let log_limb_size = 13u32;

    let mut data = Vec::with_capacity(END - START);
    for i in START..END {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures, false).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!("ed25519 signature verification benchmarks (multiple shaders): \n{}\n\n", table);
}

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_multiple_benchmarks_single() {
    let check = false;
    let log_limb_size = 13u32;

    let mut data = Vec::with_capacity(END - START);
    for i in START..END {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures, true).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!("ed25519 signature verification benchmarks (single shader): \n{}\n\n", table);
}

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_benchmarks() {
    let check = true;
    let log_limb_size = 13u32;
    let num_signatures = 2u32.pow(13u32) as usize;

    let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures, false).await;

    println!(
        "CPU took {}ms to recover {} ed25519 EdDSA signatures in serial.",
        cpu_ms, num_signatures
    );
    println!("GPU took {}ms to recover {} ed25519 EdDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_benchmarks_single() {
    let check = true;
    let log_limb_size = 13u32;
    let num_signatures = 2u32.pow(13u32) as usize;

    let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures, true).await;

    println!(
        "CPU took {}ms to recover {} ed25519 EdDSA signatures in serial.",
        cpu_ms, num_signatures
    );
    println!("GPU took {}ms to recover {} ed25519 EdDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

pub async fn do_benchmark(
    check: bool,
    log_limb_size: u32,
    num_signatures: usize,
    invoke_single: bool,
) -> (u32, u32) {
    let mut rng = ChaCha8Rng::seed_from_u64(2);

    let mut signatures = Vec::with_capacity(num_signatures);
    let mut verifying_keys = Vec::with_capacity(num_signatures);
    let mut messages = Vec::with_capacity(num_signatures);
    let mut expected_pks = Vec::with_capacity(num_signatures);

    for _ in 0..num_signatures {
        let signing_key: SigningKey = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let mut message = [0u8; 100];
        rng.fill_bytes(&mut message);

        let message = Message::new(&message);
        messages.push(message);

        let message = message.as_slice();

        let signature: Signature = signing_key.sign(&message);

        let expected_pk = ark_ecverify(&verifying_key, &signature, &message);

        signatures.push(signature);
        verifying_keys.push(verifying_key);
        expected_pks.push(expected_pk);
    }

    let sw = Stopwatch::start_new();
    for i in 0..num_signatures {
        let _ = curve25519_ecverify(&verifying_keys[i], &signatures[i], &messages[i].as_slice());
    }
    let cpu_ms = sw.elapsed_ms();

    // Start the GPU stopwatch
    let sw = Stopwatch::start_new();
    let all_is_valid = if invoke_single {
        ecverify_single(signatures, messages, verifying_keys, log_limb_size).await
    } else {
        ecverify(signatures, messages, verifying_keys, log_limb_size).await
    };
    let gpu_ms = sw.elapsed_ms();

    if check {
        for i in 0..num_signatures {
            assert!(all_is_valid[i]);
        }
    }

    (cpu_ms as u32, gpu_ms as u32)
}
