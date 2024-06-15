use fuel_crypto::{Message, SecretKey, Signature};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use stopwatch::Stopwatch;

#[serial_test::serial]
#[tokio::test]
pub async fn secp256k1_ecrecover_multiple_benchmarks() {
    let check = false;
    let log_limb_size = 13u32;
    let start = 10;
    let end = 18;

    let mut data = Vec::with_capacity(end - start);
    for i in start..end {
        let num_signatures = 2u32.pow(i as u32) as usize;
        let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures).await;

        data.push((num_signatures, cpu_ms, gpu_ms));
    }

    let table = crate::benchmarks::construct_table(data);
    println!("secp256k1 signature recovery benchmarks: \n{}\n\n", table);
}

#[serial_test::serial]
#[tokio::test]
pub async fn secp256k1_ecrecover_benchmarks() {
    let check = true;
    let log_limb_size = 13u32;
    let num_signatures = 2u32.pow(13u32) as usize;
    //let num_signatures = 255;

    let (cpu_ms, gpu_ms) = do_benchmark(check, log_limb_size, num_signatures).await;

    println!("CPU took {}ms to recover {} secp256k1 ECDSA signatures in serial.", cpu_ms, num_signatures);
    println!("GPU took {}ms to recover {} secp256k1 ECDSA signatures in parallel (including data transfer cost).", gpu_ms, num_signatures);
}

pub async fn do_benchmark(
    check: bool,
    log_limb_size: u32,
    num_signatures: usize,
) -> (u32, u32) {
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(2);

    let mut signatures = Vec::with_capacity(num_signatures);
    let mut messages = Vec::with_capacity(num_signatures);
    let mut expected_pks = Vec::with_capacity(num_signatures);

    for _ in 0..num_signatures {
        // Generate a random message
        let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
        let message = Message::new(hex::encode(msg.to_bytes_be()));
        let secret = SecretKey::random(&mut rng);
        let pk = secret.public_key();

        let fuel_signature = Signature::sign(&secret, &message);

        signatures.push(fuel_signature);
        messages.push(message);
        expected_pks.push(pk.clone());
    }

    // Perform signature recovery using the CPU
    let sw = Stopwatch::start_new();
    for i in 0..num_signatures {
        let _ = signatures[i]
            .recover(&messages[i])
            .expect("Failed to recover PK");
    }
    let cpu_ms = sw.elapsed_ms();

    // Perform signature recovery using the GPU
    let sw = Stopwatch::start_new();
    let recovered = crate::secp256k1_ecdsa::ecrecover(signatures, messages, log_limb_size).await;
    let gpu_ms = sw.elapsed_ms();

    if check {
        for i in 0..num_signatures {
            assert_eq!(recovered[i], expected_pks[i].as_slice());
        }
    }

    (cpu_ms as u32, gpu_ms as u32)
}
