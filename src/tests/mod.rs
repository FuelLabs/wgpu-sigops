#[cfg(test)]
pub mod bigint_and_ff;
#[cfg(test)]
pub mod mont;
#[cfg(test)]
pub mod secp256k1_curve;
#[cfg(test)]
pub mod secp256k1_ecdsa;
#[cfg(test)]
pub mod secp256r1_curve;
#[cfg(test)]
pub mod secp256r1_ecdsa;
#[cfg(test)]
pub mod bytes_to_limbs;

use ark_ff::BigInteger;
use ark_ff::fields::PrimeField;
use num_bigint::BigUint;
use fuel_algos::coords;
use multiprecision::utils::calc_num_limbs;
use multiprecision::bigint;
use crate::shader::render_bigint_ff_mont_tests;
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

pub fn get_secp256k1_b() -> BigUint {
    BigUint::from(7u32)
}

pub fn get_secp256r1_b() -> BigUint {
    BigUint::parse_bytes(b"5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b", 16).unwrap()
}

pub fn fq_to_biguint<F: PrimeField>(val: F) -> BigUint {
    let b = val.into_bigint().to_bytes_be();
    BigUint::from_bytes_be(&b)
}


pub fn projectivexyz_to_mont_limbs<F: PrimeField>(
    a: &coords::ProjectiveXYZ<F>,
    p: &BigUint,
    log_limb_size: u32,
) -> Vec<u32> {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = multiprecision::mont::calc_mont_radix(num_limbs, log_limb_size);
    let a_x_r = fq_to_biguint::<F>(a.x) * &r % p;
    let a_y_r = fq_to_biguint::<F>(a.y) * &r % p;
    let a_z_r = fq_to_biguint::<F>(a.z) * &r % p;
    let a_x_r_limbs = bigint::from_biguint_le(&a_x_r, num_limbs, log_limb_size);
    let a_y_r_limbs = bigint::from_biguint_le(&a_y_r, num_limbs, log_limb_size);
    let a_z_r_limbs = bigint::from_biguint_le(&a_z_r, num_limbs, log_limb_size);
    let mut pt_a_limbs = Vec::<u32>::with_capacity(num_limbs * 3);
    pt_a_limbs.extend_from_slice(&a_x_r_limbs);
    pt_a_limbs.extend_from_slice(&a_y_r_limbs);
    pt_a_limbs.extend_from_slice(&a_z_r_limbs);
    pt_a_limbs
}


pub async fn do_test(
    a: &BigUint,
    b: &BigUint,
    p: &BigUint,
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

    let source = render_bigint_ff_mont_tests("src/wgsl/", filename, &p, &get_secp256k1_b(), log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

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
        &results[0][0..result_len].to_vec(),
        result_len,
        log_limb_size,
    );

    assert_eq!(result, expected);
}
