use crate::precompute::ed25519_bases;
use crate::ed25519_eddsa::{ecverify, ecverify_single};
use crate::curve_algos::ed25519_eddsa::curve25519_ecverify;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use fuel_crypto::Message;
use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;

const START: usize = 10;
const END: usize = 18;

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_multiple_benchmarks_multi_shader() {
    let check = false;
    let log_limb_size = 13u32;
    let table_limbs = ed25519_bases(log_limb_size);

    let (signatures, messages, verifying_keys) = gen_test_data(2u32.pow(END as u32) as usize);

    let mut data = Vec::with_capacity(END - START);
    for i in START..END {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, &signatures, &messages, &verifying_keys, false).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!("ed25519 signature verification benchmarks (multiple shaders): \n{}\n\n", table);
}

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_multiple_benchmarks_single_shader() {
    let check = false;
    let log_limb_size = 13u32;
    let table_limbs = ed25519_bases(log_limb_size);

    let (signatures, messages, verifying_keys) = gen_test_data(2u32.pow(END as u32) as usize);

    let mut data = Vec::with_capacity(END - START);
    for i in START..END {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, &signatures, &messages, &verifying_keys, true).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!("ed25519 signature verification benchmarks (single shader): \n{}\n\n", table);
}

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_ecverify_benchmarks_multi_shader() {
    let check = true;
    let log_limb_size = 13u32;
    let table_limbs = ed25519_bases(log_limb_size);
    let num_signatures = 2u32.pow(13u32) as usize;

    let (signatures, messages, verifying_keys) = gen_test_data(num_signatures);

    let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, &signatures, &messages, &verifying_keys, false).await;

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
    let table_limbs = ed25519_bases(log_limb_size);
    let num_signatures = 2u32.pow(13u32) as usize;

    let (signatures, messages, verifying_keys) = gen_test_data(num_signatures);

    let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, &signatures, &messages, &verifying_keys, true).await;

    println!(
        "CPU took {}ms to recover {} ed25519 EdDSA signatures in serial.",
        cpu_ms, num_signatures
    );
    println!("GPU took {}ms to recover {} ed25519 EdDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

pub async fn do_benchmark(
    check: bool,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
    num_signatures: usize,
    signatures: &Vec<Signature>,
    messages: &Vec<Message>,
    verifying_keys: &Vec<VerifyingKey>,
    invoke_single: bool,
) -> (u32, u32) {
    let signatures = signatures[0..num_signatures].to_vec();
    let messages = messages[0..num_signatures].to_vec();
    let verifying_keys = verifying_keys[0..num_signatures].to_vec();

    // Perform signature recovery using the CPU
    let sw = Stopwatch::start_new();
    for i in 0..num_signatures {
        let _ = curve25519_ecverify(&verifying_keys[i], &signatures[i], &messages[i].as_slice());
    }
    let cpu_ms = sw.elapsed_ms();

    // Start the GPU stopwatch
    let sw = Stopwatch::start_new();
    let all_is_valid = if invoke_single {
        ecverify_single(&signatures, &messages, &verifying_keys, log_limb_size).await
    } else {
        ecverify(&signatures, &messages, &verifying_keys, table_limbs, log_limb_size).await
    };

    if all_is_valid.is_err() {
        panic!("Shader failed");
    }

    let all_is_valid = all_is_valid.unwrap();

    let gpu_ms = sw.elapsed_ms();

    if check {
        for i in 0..num_signatures {
            assert!(all_is_valid[i]);
        }
    }

    (cpu_ms as u32, gpu_ms as u32)
}

pub fn gen_test_data(
    num_signatures: usize,
) -> (Vec<Signature>, Vec<Message>, Vec<VerifyingKey>) {
    let mut rng = ChaCha8Rng::seed_from_u64(2);

    let mut signatures = Vec::with_capacity(num_signatures);
    let mut verifying_keys = Vec::with_capacity(num_signatures);
    let mut messages = Vec::with_capacity(num_signatures);

    for _ in 0..num_signatures {
        let signing_key: SigningKey = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let mut message = [0u8; 100];
        rng.fill_bytes(&mut message);

        let message = Message::new(&message);
        messages.push(message);

        let message = message.as_slice();

        let signature: Signature = signing_key.sign(&message);

        signatures.push(signature);
        verifying_keys.push(verifying_key);
    }
    (signatures, messages, verifying_keys)
}
