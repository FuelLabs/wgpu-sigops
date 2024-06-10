use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_bytes_to_limbs_test;
use crate::tests::get_secp256k1_b;
use multiprecision::bigint;
use multiprecision::utils::calc_num_limbs;
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[serial_test::serial]
#[tokio::test]
pub async fn test_bytes_be_to_limbs_le_shader() {
    let mut rng = ChaCha8Rng::seed_from_u64(33);
    let p = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
        16,
    )
    .unwrap();
    for log_limb_size in 11..16 {
        for _ in 0..10 {
            let val: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));

            do_test_bytes_be_to_limbs_le(&val, &p, log_limb_size).await;
        }
    }
}

pub async fn do_test_bytes_be_to_limbs_le(val: &BigUint, p: &BigUint, log_limb_size: u32) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let bytes = val.to_bytes_be();

    let (device, queue) = get_device_and_queue().await;

    let bytes_u32s: Vec<u32> = bytemuck::cast_slice(&bytes).to_vec();
    let bytes_buf = create_sb_with_data(&device, &bytes_u32s);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u8>()) as u64);

    let source = render_bytes_to_limbs_test(
        "bytes_be_to_limbs_le_tests.wgsl",
        &p,
        &get_secp256k1_b(),
        log_limb_size,
    );
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_bytes_be_to_limbs_le");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&bytes_buf, &result_buf]);

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

    let result =
        bigint::to_biguint_le(&results[0][0..num_limbs].to_vec(), num_limbs, log_limb_size);

    assert_eq!(result, *val);
}
