use num_bigint::BigUint;
use multiprecision::bigint;

#[cfg(test)]
pub mod bigint_and_ff;
#[cfg(test)]
pub mod mont;

use crate::shader::render_tests;
use crate::gpu::{
    create_empty_sb,
    execute_pipeline,
    create_bind_group,
    create_sb_with_data,
    get_device_and_queue,
    create_command_encoder,
    create_compute_pipeline,
    finish_encoder_and_read_from_gpu,
};

pub async fn do_test(
    a: BigUint,
    b: BigUint,
    p: BigUint,
    log_limb_size: u32,
    num_limbs: usize,
    result_len: usize,
    func: fn(&Vec<u32>, &Vec<u32>, &Vec<u32>, u32) -> Vec<u32>,
    biguint_func: fn(&BigUint, &BigUint, &BigUint) -> BigUint,
    filename: &str,
    entrypoint: &str,
) {
    let expected = biguint_func(&a, &b, &p);
    let p_limbs = bigint::from_biguint_le(&p, num_limbs, log_limb_size);
    let a_limbs = bigint::from_biguint_le(&a, num_limbs, log_limb_size);
    let b_limbs = bigint::from_biguint_le(&b, num_limbs, log_limb_size);
    let expected_limbs = bigint::from_biguint_le(&expected, result_len, log_limb_size);
    let expected_limbs_2 = func(&a_limbs, &b_limbs, &p_limbs, log_limb_size);

    assert!(bigint::eq(&expected_limbs, &expected_limbs_2));

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb_with_data(&device, &a_limbs);
    let b_buf = create_sb_with_data(&device, &b_limbs);
    let result_buf = create_empty_sb(&device, (result_len * 8 * std::mem::size_of::<u8>()) as u64);
    let p_buf = create_sb_with_data(&device, &p_limbs);

    let source = render_tests("src/wgsl/", filename, &p, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_buf, &b_buf, &p_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let result = bigint::to_biguint_le(
        &results[0][0..result_len].to_vec(),
        result_len,
        log_limb_size,
    );

    assert_eq!(result, expected);
}
