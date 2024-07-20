use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_ed25519_reduce_fr_tests;
use byteorder::{BigEndian, ByteOrder};
use num_bigint::BigUint;
use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::ops::Shr;

#[serial_test::serial]
#[tokio::test]
pub async fn ed25519_reduce_fr() {
    let mut rng = ChaCha8Rng::seed_from_u64(0 as u64);

    for _ in 0..10 {
        let mut input = [0u8; 64];
        rng.fill_bytes(&mut input);
        let x = BigUint::from_bytes_be(&input);

        do_ed25519_reduce_fr_test(&x, "ed25519_reduce_fr_tests.wgsl", "test_ed25519_reduce_fr")
            .await;
    }
}

pub async fn do_ed25519_reduce_fr_test(input: &BigUint, filename: &str, entrypoint: &str) {
    let p = crate::moduli::ed25519_fr_modulus_biguint();
    let expected = input % &p;

    let (device, queue) = get_device_and_queue().await;
    let input_bytes = input.to_bytes_be();
    let mut input_u32s = Vec::with_capacity(input_bytes.len() / 4);
    for chunk in input_bytes.chunks(4) {
        let value = BigEndian::read_u32(chunk);
        input_u32s.push(value);
    }

    let input_buf = create_sb_with_data(&device, &input_u32s);
    let result_buf = create_empty_sb(&device, input_buf.size() * 2);
    let result_wide_buf = create_empty_sb(&device, input_buf.size() * 4);

    let source = render_ed25519_reduce_fr_tests(filename);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&input_buf, &result_buf, &result_wide_buf]);

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;

    let result = multiprecision::bigint::to_biguint_le(&results[0], 32, 8);

    //println!("{}", hex::encode(&result.to_bytes_be()));
    assert_eq!(result, expected);
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_convert_512_be_to_le() {
    let mut rng = ChaCha8Rng::seed_from_u64(0 as u64);

    for _ in 0..10 {
        let mut input = [0u8; 64];
        rng.fill_bytes(&mut input);
        let x = BigUint::from_bytes_be(&input);

        do_convert_512_be_to_le_test(&x, "ed25519_reduce_fr_tests.wgsl", "test_convert_512_be_to_le")
            .await;
    }
}

pub async fn do_convert_512_be_to_le_test(input: &BigUint, filename: &str, entrypoint: &str) {
    let (device, queue) = get_device_and_queue().await;
    let input_bytes = input.to_bytes_be();
    let mut input_u32s = Vec::with_capacity(input_bytes.len() / 4);
    for chunk in input_bytes.chunks(4) {
        let value = BigEndian::read_u32(chunk);
        input_u32s.push(value);
    }

    let input_buf = create_sb_with_data(&device, &input_u32s);
    let result_buf = create_empty_sb(&device, input_buf.size() * 2);
    let result_wide_buf = create_empty_sb(&device, input_buf.size() * 4);

    let source = render_ed25519_reduce_fr_tests(filename);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&input_buf, &result_buf, &result_wide_buf]);

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;

    let result = multiprecision::bigint::to_biguint_le(&results[0], 32, 16);

    //println!("{}", hex::encode(&result.to_bytes_be()));
    //println!("{}", hex::encode(&input.to_bytes_be()));
    assert_eq!(&result.to_bytes_be(), &input.to_bytes_be());
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_mul_wide() {
    let mut rng = ChaCha8Rng::seed_from_u64(0 as u64);

    for _ in 0..10 {
        let mut input = [0u8; 64];
        rng.fill_bytes(&mut input);
        let x = BigUint::from_bytes_be(&input);

        do_mul_wide_test(&x, "ed25519_reduce_fr_tests.wgsl", "test_mul_wide")
            .await;
    }
}

pub async fn do_mul_wide_test(input: &BigUint, filename: &str, entrypoint: &str) {
    let fr_reduce_r = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffeb2106215d086329a7ed9ce5a30a2c131b",
        16,
    )
    .unwrap();

    let expected = input * &fr_reduce_r;

    let (device, queue) = get_device_and_queue().await;
    let input_bytes = input.to_bytes_be();
    let mut input_u32s = Vec::with_capacity(input_bytes.len() / 4);
    for chunk in input_bytes.chunks(4) {
        let value = BigEndian::read_u32(chunk);
        input_u32s.push(value);
    }

    let input_buf = create_sb_with_data(&device, &input_u32s);
    let result_buf = create_empty_sb(&device, input_buf.size() * 2);
    let result_wide_buf = create_empty_sb(&device, input_buf.size() * 4);

    let source = render_ed25519_reduce_fr_tests(filename);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&input_buf, &result_buf, &result_wide_buf]);

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_wide_buf])
            .await;

    let result = multiprecision::bigint::to_biguint_le(&results[0], 64, 16);

    assert_eq!(&result.to_bytes_be(), &expected.to_bytes_be());
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_shr_512() {
    let mut rng = ChaCha8Rng::seed_from_u64(0 as u64);

    for _ in 0..10 {
        let mut input = [0u8; 64];
        rng.fill_bytes(&mut input);
        let x = BigUint::from_bytes_be(&input);

        do_shr_512_test(&x, "ed25519_reduce_fr_tests.wgsl", "test_shr_512")
            .await;
    }
}

pub async fn do_shr_512_test(input: &BigUint, filename: &str, entrypoint: &str) {
    let fr_reduce_r = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffeb2106215d086329a7ed9ce5a30a2c131b",
        16,
    )
    .unwrap();

    let expected: BigUint = (input * &fr_reduce_r).shr(512);

    let (device, queue) = get_device_and_queue().await;
    let input_bytes = input.to_bytes_be();
    let mut input_u32s = Vec::with_capacity(input_bytes.len() / 4);
    for chunk in input_bytes.chunks(4) {
        let value = BigEndian::read_u32(chunk);
        input_u32s.push(value);
    }

    let input_buf = create_sb_with_data(&device, &input_u32s);
    let result_buf = create_empty_sb(&device, input_buf.size() * 2);
    let result_wide_buf = create_empty_sb(&device, input_buf.size() * 4);

    let source = render_ed25519_reduce_fr_tests(filename);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&input_buf, &result_buf, &result_wide_buf]);

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;

    let result = multiprecision::bigint::to_biguint_le(&results[0], 32, 16);

    assert_eq!(&result.to_bytes_be(), &expected.to_bytes_be());
}

#[serial_test::serial]
#[tokio::test]
pub async fn test_sub_wide() {
    let mut rng = ChaCha8Rng::seed_from_u64(0 as u64);

    for _ in 0..10 {
        let mut input = [0u8; 64];
        rng.fill_bytes(&mut input);
        let x = BigUint::from_bytes_be(&input);

        do_sub_wide_test(&x, "ed25519_reduce_fr_tests.wgsl", "test_sub_wide")
            .await;
    }
}

pub async fn do_sub_wide_test(input: &BigUint, filename: &str, entrypoint: &str) {
    let fr_reduce_r = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffeb2106215d086329a7ed9ce5a30a2c131b",
        16,
    )
    .unwrap();

    let expected: BigUint = input - &fr_reduce_r;

    let (device, queue) = get_device_and_queue().await;
    let input_bytes = input.to_bytes_be();
    let mut input_u32s = Vec::with_capacity(input_bytes.len() / 4);
    for chunk in input_bytes.chunks(4) {
        let value = BigEndian::read_u32(chunk);
        input_u32s.push(value);
    }

    let input_buf = create_sb_with_data(&device, &input_u32s);
    let result_buf = create_empty_sb(&device, input_buf.size() * 2);
    let result_wide_buf = create_empty_sb(&device, input_buf.size() * 4);

    let source = render_ed25519_reduce_fr_tests(filename);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&input_buf, &result_buf, &result_wide_buf]);

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;

    let result = multiprecision::bigint::to_biguint_le(&results[0], 32, 16);

    assert_eq!(&result.to_bytes_be(), &expected.to_bytes_be());
}
