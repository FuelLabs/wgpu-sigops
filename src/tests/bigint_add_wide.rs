use crate::shader::render;
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
use multiprecision::utils::calc_num_limbs;

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_add_wide() {
    for log_limb_size in 11..15 {
        let num_limbs = calc_num_limbs(log_limb_size, 256);

        for i in 0..4 {
            let x = BigUint::from((i + 1 * 824234) as u32);
            let y = BigUint::from((i + 1 * 223234) as u32);
            let a = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000", 16).unwrap() + x;
            let b = BigUint::parse_bytes(b"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee000000000", 16).unwrap() + y;
            do_test(a, b, log_limb_size, num_limbs).await;
        }
    }
}

async fn do_test(
    a: BigUint,
    b: BigUint,
    log_limb_size: u32,
    num_limbs: usize,
) {
    let p = BigUint::parse_bytes(b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141", 16).unwrap();
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

    let source = render("src/tests/", "bigint_add_wide.wgsl", &p, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "main");

    let mut command_encoder = device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { label: None }
    );

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

    let result = bigint::to_biguint_le(
        &results[0][0..num_limbs+1].to_vec(),
        num_limbs + 1,
        log_limb_size,
    );

    assert_eq!(result, expected);
}
