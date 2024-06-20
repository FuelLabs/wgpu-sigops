use crate::secp256r1_ecdsa::{ecrecover, ecrecover_single_shader};
use fuel_crypto::secp256r1::p256::{recover, sign_prehashed};
use fuel_crypto::Message;
use fuel_types::Bytes64;
use num_bigint::{BigUint, RandomBits};
use p256::ecdsa::SigningKey;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;
use crate::precompute::secp256r1_bases;

const START: usize = 8;
const END: usize = 18;

#[serial_test::serial]
#[tokio::test]
pub async fn secp256r1_ecrecover_multiple_benchmarks_multi_shader() {
    let check = false;
    let log_limb_size = 13u32;

    let mut data = Vec::with_capacity(END - START);
    let table_limbs = secp256r1_bases(log_limb_size);
    for i in START..END {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, false).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!(
        "secp256r1 signature verification benchmarks (multiple shaders): \n{}\n\n",
        table
    );
}

#[serial_test::serial]
#[tokio::test]
pub async fn secp256r1_ecrecover_multiple_benchmarks_single_shader() {
    let check = false;
    let log_limb_size = 13u32;

    let mut data = Vec::with_capacity(END - START);
    let table_limbs = secp256r1_bases(log_limb_size);
    for i in START..END {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, true).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!(
        "secp256r1 signature verification benchmarks (single shader): \n{}\n\n",
        table
    );
}

#[serial_test::serial]
#[tokio::test]
pub async fn secp256r1_ecrecover_benchmarks_multi_shader() {
    let check = true;
    let log_limb_size = 13u32;
    let table_limbs = secp256r1_bases(log_limb_size);
    let num_signatures = 2u32.pow(13u32) as usize;

    let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, false).await;

    println!(
        "CPU took {}ms to recover {} secp256r1 ECDSA signatures in serial.",
        cpu_ms, num_signatures
    );
    println!("GPU took {}ms to recover {} secp256r1 ECDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

#[serial_test::serial]
#[tokio::test]
pub async fn secp256r1_ecrecover_benchmarks_single_shader() {
    let check = true;
    let log_limb_size = 13u32;
    let table_limbs = secp256r1_bases(log_limb_size);
    let num_signatures = 2u32.pow(13u32) as usize;

    let (cpu_ms, gpu_ms) = do_benchmark(check, &table_limbs, log_limb_size, num_signatures, true).await;

    println!(
        "CPU took {}ms to recover {} secp256r1 ECDSA signatures in serial.",
        cpu_ms, num_signatures
    );
    println!("GPU took {}ms to recover {} secp256r1 ECDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

pub async fn do_benchmark(
    check: bool,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
    num_signatures: usize,
    invoke_single: bool,
) -> (u32, u32) {
    let scalar_p = crate::moduli::secp256r1_fr_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(2);

    let mut signatures = Vec::with_capacity(num_signatures);
    let mut messages = Vec::with_capacity(num_signatures);
    let mut expected_pks = Vec::with_capacity(num_signatures);

    for _ in 0..num_signatures {
        // Generate a random message
        let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
        let message = Message::new(hex::encode(msg.to_bytes_be()));
        let signing_key = SigningKey::random(&mut rng);
        let verifying_key = signing_key.verifying_key();

        let pk_affine_bytes = &verifying_key.to_sec1_bytes()[1..65];
        let fuel_signature: Bytes64 =
            sign_prehashed(&signing_key, &message).expect("Couldn't sign");

        signatures.push(fuel_signature);
        messages.push(message);
        expected_pks.push(pk_affine_bytes.to_vec());
    }

    // Perform signature recovery using the CPU
    let sw = Stopwatch::start_new();
    for i in 0..num_signatures {
        let _ = recover(&signatures[i], &messages[i]);
    }
    let cpu_ms = sw.elapsed_ms();

    // Perform signature recovery using the GPU
    let sw = Stopwatch::start_new();
    let recovered = if invoke_single {
        ecrecover_single_shader(signatures, messages, log_limb_size).await
    } else {
        ecrecover(signatures, messages, table_limbs, log_limb_size).await
    };
    let gpu_ms = sw.elapsed_ms();

    if check {
        for i in 0..num_signatures {
            assert_eq!(recovered[i], expected_pks[i].as_slice());
        }
    }

    (cpu_ms as u32, gpu_ms as u32)
}
