use num_bigint::BigUint;
use ark_ff::{ PrimeField, BigInteger, One };
use ark_secp256k1::{ Projective, Affine, Fr, Fq };
use ark_ec::{ CurveGroup, AffineRepr };
use std::ops::{ Mul };
use multiprecision::utils::calc_num_limbs;
use multiprecision::{ mont, bigint };
use fuel_algos::curve;
use crate::shader::render_curve_tests;
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

pub fn fq_to_biguint(val: Fq) -> BigUint {
    let b = val.into_bigint().to_bytes_be();
    BigUint::from_bytes_be(&b)
}

fn jacobian_to_affine_func(x: Fq, y: Fq, z: Fq) -> Affine {
    Projective::new(x, y, z).into_affine()
}

fn projective_to_affine_func(x: Fq, y: Fq, z: Fq) -> Affine {
    let p = curve::ProjectiveXYZ {x, y, z };
    curve::projectivexyz_to_affine(&p)
}

#[serial_test::serial]
#[tokio::test]
pub async fn projective_add_2007_bl_unsafe() {
    let g = Affine::generator();

    // a and b are in Jacobian
    let a: Projective = g.mul(Fr::from(2u32)).into_affine().into();
    let b: Projective = g.mul(Fr::from(3u32)).into_affine().into();

    assert_eq!(a.z, Fq::one());
    assert_eq!(b.z, Fq::one());

    let a = curve::ProjectiveXYZ {x: a.x, y: a.y, z: a.z };
    let b = curve::ProjectiveXYZ {x: b.x, y: b.y, z: b.z };

    for log_limb_size in 11..16 {
        do_add_test(&a, &b, log_limb_size, projective_to_affine_func, "curve_tests.wgsl", "test_projective_add_2007_bl_unsafe").await;
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn projective_dbl_2007_bl_unsafe() {
    let g = Affine::generator();

    // a is in Jacobian
    let a: Projective = g.mul(Fr::from(2u32)).into_affine().into();

    assert_eq!(a.z, Fq::one());

    let a = curve::ProjectiveXYZ {x: a.x, y: a.y, z: a.z };

    for log_limb_size in 11..16 {
        do_dbl_test(&a, log_limb_size, projective_to_affine_func, "curve_tests.wgsl", "test_projective_dbl_2007_bl_unsafe").await;
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn jacobian_add_2007_bl_unsafe() {
    let g = Affine::generator();
    let a: Projective = g.mul(Fr::from(2u32));
    let b: Projective = g.mul(Fr::from(3u32));
    let a = curve::ProjectiveXYZ {x: a.x, y: a.y, z: a.z };
    let b = curve::ProjectiveXYZ {x: b.x, y: b.y, z: b.z };

    for log_limb_size in 11..16 {
        do_add_test(&a, &b, log_limb_size, jacobian_to_affine_func, "curve_tests.wgsl", "test_jacobian_add_2007_bl_unsafe").await;
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn jacobian_dbl_2009_l() {
    let g = Affine::generator();
    let a: Projective = g.mul(Fr::from(2u32));
    let a = curve::ProjectiveXYZ {x: a.x, y: a.y, z: a.z };

    for log_limb_size in 11..16 {
        do_dbl_test(&a, log_limb_size, jacobian_to_affine_func, "curve_tests.wgsl", "test_jacobian_dbl_2009_l").await;
    }
}

pub async fn do_add_test(
    a: &curve::ProjectiveXYZ,
    b: &curve::ProjectiveXYZ,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let a_x_r = fq_to_biguint(a.x) * &r % &p;
    let a_y_r = fq_to_biguint(a.y) * &r % &p;
    let a_z_r = fq_to_biguint(a.z) * &r % &p;
    let b_x_r = fq_to_biguint(b.x) * &r % &p;
    let b_y_r = fq_to_biguint(b.y) * &r % &p;
    let b_z_r = fq_to_biguint(b.z) * &r % &p;

    let p_limbs = bigint::from_biguint_le(&p, num_limbs, log_limb_size);
    let a_x_r_limbs = bigint::from_biguint_le(&a_x_r, num_limbs, log_limb_size);
    let a_y_r_limbs = bigint::from_biguint_le(&a_y_r, num_limbs, log_limb_size);
    let a_z_r_limbs = bigint::from_biguint_le(&a_z_r, num_limbs, log_limb_size);
    let b_x_r_limbs = bigint::from_biguint_le(&b_x_r, num_limbs, log_limb_size);
    let b_y_r_limbs = bigint::from_biguint_le(&b_y_r, num_limbs, log_limb_size);
    let b_z_r_limbs = bigint::from_biguint_le(&b_z_r, num_limbs, log_limb_size);

   let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
   let rinv = res.0;

   let a = Projective::new(a.x, a.y, a.z);
   let b = Projective::new(b.x, b.y, b.z);
   let expected_sum_affine = (a + b).into_affine();

   let (device, queue) = get_device_and_queue().await;

   let mut pt_a_limbs = Vec::<u32>::with_capacity(num_limbs * 3);
   pt_a_limbs.extend_from_slice(&a_x_r_limbs);
   pt_a_limbs.extend_from_slice(&a_y_r_limbs);
   pt_a_limbs.extend_from_slice(&a_z_r_limbs);

   let mut pt_b_limbs = Vec::<u32>::with_capacity(num_limbs * 3);
   pt_b_limbs.extend_from_slice(&b_x_r_limbs);
   pt_b_limbs.extend_from_slice(&b_y_r_limbs);
   pt_b_limbs.extend_from_slice(&b_z_r_limbs);

   let p_buf = create_sb_with_data(&device, &p_limbs);
   let pt_a_buf = create_sb_with_data(&device, &pt_a_limbs);
   let pt_b_buf = create_sb_with_data(&device, &pt_b_limbs);
   let result_buf = create_empty_sb(&device, pt_a_buf.size());

   let source = render_curve_tests("src/wgsl/", filename, &p, log_limb_size);
   let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

   let mut command_encoder = create_command_encoder(&device);

   let bind_group = create_bind_group(
       &device,
       &compute_pipeline,
       0,
       &[&p_buf, &pt_a_buf, &pt_b_buf, &result_buf],
   );

   execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result_x_r = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        let result = &result_x_r * &rinv % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let result_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let result_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let result_z = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());
    let result_affine = to_affine_func(result_x, result_y, result_z);

    assert_eq!(result_affine, expected_sum_affine);
}

pub async fn do_dbl_test(
    a: &curve::ProjectiveXYZ,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let a_x_r = fq_to_biguint(a.x) * &r % &p;
    let a_y_r = fq_to_biguint(a.y) * &r % &p;
    let a_z_r = fq_to_biguint(a.z) * &r % &p;

    let p_limbs = bigint::from_biguint_le(&p, num_limbs, log_limb_size);
    let a_x_r_limbs = bigint::from_biguint_le(&a_x_r, num_limbs, log_limb_size);
    let a_y_r_limbs = bigint::from_biguint_le(&a_y_r, num_limbs, log_limb_size);
    let a_z_r_limbs = bigint::from_biguint_le(&a_z_r, num_limbs, log_limb_size);

   let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
   let rinv = res.0;

   let a = Projective::new(a.x, a.y, a.z);
   let expected_sum_affine = (a + a).into_affine();

   let (device, queue) = get_device_and_queue().await;

   let mut pt_a_limbs = Vec::<u32>::with_capacity(num_limbs * 3);
   pt_a_limbs.extend_from_slice(&a_x_r_limbs);
   pt_a_limbs.extend_from_slice(&a_y_r_limbs);
   pt_a_limbs.extend_from_slice(&a_z_r_limbs);

   let p_buf = create_sb_with_data(&device, &p_limbs);
   let pt_a_buf = create_sb_with_data(&device, &pt_a_limbs);
   let pt_b_buf = create_empty_sb(&device, pt_a_buf.size());
   let result_buf = create_empty_sb(&device, pt_a_buf.size());

   let source = render_curve_tests("src/wgsl/", filename, &p, log_limb_size);
   let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

   let mut command_encoder = create_command_encoder(&device);

   let bind_group = create_bind_group(
       &device,
       &compute_pipeline,
       0,
       &[&p_buf, &pt_a_buf, &pt_b_buf, &result_buf],
   );

   execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result_x_r = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        let result = &result_x_r * &rinv % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let result_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let result_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let result_z = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());
    let result_affine = to_affine_func(result_x, result_y, result_z);

    assert_eq!(result_affine, expected_sum_affine);
}
