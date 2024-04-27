use std::path::PathBuf;
use crate::gpu::{
    create_empty_sb,
    execute_pipeline,
    create_bind_group,
    create_sb_with_data,
    get_device_and_queue,
    create_compute_pipeline,
    finish_encoder_and_read_from_gpu,
};
use num_bigint::BigUint;
use multiprecision::bigint;

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_add_wide() {
    let log_limb_size = 13;
    let num_limbs = 20;
    let a = BigUint::parse_bytes(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 16).unwrap();
    let b = BigUint::parse_bytes(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 16).unwrap();
    let expected = &a + &b;

    // We are testing add_wide, so the sum should overflow
    assert!(expected > BigUint::from(2u32).pow(256));
    
    let a_limbs = bigint::from_biguint_le(&a, num_limbs, log_limb_size);
    let b_limbs = bigint::from_biguint_le(&b, num_limbs, log_limb_size);
    let expected_limbs = bigint::from_biguint_le(&expected, num_limbs + 1, log_limb_size);
    let expected_limbs_2 = bigint::add_wide(&a_limbs, &b_limbs, log_limb_size);

    assert!(bigint::eq(&expected_limbs, &expected_limbs_2));

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &a_limbs);
    let b_buf = create_sb_with_data(&device, &b_limbs);
    let result_buf = create_empty_sb(&device, ((num_limbs + 1) * 8 * std::mem::size_of::<u8>()) as u64);

    let path_from_cargo_manifest_dir = "src/tests/";
    let input_filename = "bigint_add_wide.wgsl";
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path_from_cargo_manifest_dir).join(input_filename);
    let source = std::fs::read_to_string(input_path).unwrap();

    let compute_pipeline = create_compute_pipeline(
        &device,
        &source,
        "main"
    );

    let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(&results[0][0..num_limbs+1].to_vec(), num_limbs + 1, log_limb_size);

    assert_eq!(result, expected);
}
