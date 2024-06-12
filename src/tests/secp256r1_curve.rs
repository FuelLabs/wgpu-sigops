use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_secp256r1_curve_tests;
use crate::tests::{fq_to_biguint, projectivexyz_to_mont_limbs};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{BigInteger, One, PrimeField};
use ark_secp256r1::{Affine, Fq, Fr, Projective};
use crate::curve_algos::coords;
use crate::curve_algos::secp256r1_curve as curve;
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, mont};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::ops::Mul;

const NUM_RUNS_PER_TEST: usize = 4;

pub fn projective_to_affine_func(x: Fq, y: Fq, z: Fq) -> Affine {
    let p = coords::ProjectiveXYZ { x, y, z };
    curve::projectivexyz_to_affine(&p)
}

#[serial_test::serial]
#[tokio::test]
pub async fn projective_to_affine() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let g = Affine::generator();
    let log_limb_size = 13;

    let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
    let s = Fr::from_be_bytes_mod_order(&s.to_bytes_be());
    let r: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
    let r = Fr::from_be_bytes_mod_order(&r.to_bytes_be());
    let a: Affine = g.mul(s).into_affine();
    let b: Affine = g.mul(r).into_affine();
    let a_proj = curve::affine_to_projectivexyz(&a);
    let b_proj = curve::affine_to_projectivexyz(&b);

    let sum = curve::projective_add_2015_rcb_unsafe(&a_proj, &b_proj);

    do_projective_to_affine_test(
        &sum,
        log_limb_size,
        projective_to_affine_func,
        "secp256r1_curve_tests.wgsl",
        "test_projective_to_affine",
    )
    .await;
}

#[serial_test::serial]
#[tokio::test]
pub async fn projective_add_2015_rcb_unsafe() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for log_limb_size in 11..16 {
        for _ in 0..NUM_RUNS_PER_TEST {
            let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let s = Fr::from_be_bytes_mod_order(&s.to_bytes_be());
            let r: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let r = Fr::from_be_bytes_mod_order(&r.to_bytes_be());

            // a and b are in Jacobian
            let g = Affine::generator();
            let a: Projective = g.mul(s).into_affine().into();
            let b: Projective = g.mul(r).into_affine().into();

            assert_eq!(a.z, Fq::one());
            assert_eq!(b.z, Fq::one());

            let a = coords::ProjectiveXYZ {
                x: a.x,
                y: a.y,
                z: a.z,
            };
            let b = coords::ProjectiveXYZ {
                x: b.x,
                y: b.y,
                z: b.z,
            };
            do_add_test(
                &a,
                &b,
                log_limb_size,
                projective_to_affine_func,
                "secp256r1_curve_tests.wgsl",
                "test_projective_add_2015_rcb_unsafe",
            )
            .await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn projective_dbl_2015_rcb() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for log_limb_size in 11..16 {
        for _ in 0..NUM_RUNS_PER_TEST {
            let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let s = Fr::from_be_bytes_mod_order(&s.to_bytes_be());

            // a and b are in Jacobian
            let g = Affine::generator();
            let a: Projective = g.mul(s).into_affine().into();

            assert_eq!(a.z, Fq::one());

            let a = coords::ProjectiveXYZ {
                x: a.x,
                y: a.y,
                z: a.z,
            };
            do_dbl_test(
                &a,
                log_limb_size,
                projective_to_affine_func,
                "secp256r1_curve_tests.wgsl",
                "test_projective_dbl_2015_rcb",
            )
            .await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn recover_affine_ys() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);

    let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
    let s = Fr::from_be_bytes_mod_order(&s.to_bytes_be());
    let a: Affine = Affine::generator().mul(s).into_affine();

    for log_limb_size in 13..16 {
        do_recover_affine_ys_test(
            &a,
            log_limb_size,
            "secp256r1_curve_recover_affine_ys_tests.wgsl",
            "test_secp256r1_recover_affine_ys",
        )
        .await;
    }
}

pub async fn do_add_test(
    a: &coords::ProjectiveXYZ<Fq>,
    b: &coords::ProjectiveXYZ<Fq>,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);

    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let pt_a_limbs = projectivexyz_to_mont_limbs::<Fq>(&a, &p, log_limb_size);
    let pt_b_limbs = projectivexyz_to_mont_limbs::<Fq>(&b, &p, log_limb_size);

    let a = Projective::new(a.x, a.y, a.z);
    let b = Projective::new(b.x, b.y, b.z);
    let expected_sum_affine = (a + b).into_affine();

    let (device, queue) = get_device_and_queue().await;

    let pt_a_buf = create_sb_with_data(&device, &pt_a_limbs);
    let pt_b_buf = create_sb_with_data(&device, &pt_b_limbs);
    let result_buf = create_empty_sb(&device, pt_a_buf.size());

    let source = render_secp256r1_curve_tests(filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&pt_a_buf, &pt_b_buf, &result_buf],
    );

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
    a: &coords::ProjectiveXYZ<Fq>,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);

    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let pt_a_limbs = projectivexyz_to_mont_limbs(&a, &p, log_limb_size);

    let a = Projective::new(a.x, a.y, a.z);
    let expected_sum_affine = (a + a).into_affine();

    let (device, queue) = get_device_and_queue().await;

    let pt_a_buf = create_sb_with_data(&device, &pt_a_limbs);
    let pt_b_buf = create_empty_sb(&device, pt_a_buf.size());
    let result_buf = create_empty_sb(&device, pt_a_buf.size());

    let source = render_secp256r1_curve_tests(filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&pt_a_buf, &pt_b_buf, &result_buf],
    );

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

pub async fn do_recover_affine_ys_test(
    a: &Affine,
    log_limb_size: u32,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let xr = fq_to_biguint::<Fq>(a.x) * &r % &p;

    let xr_limbs = bigint::from_biguint_le(&xr, num_limbs, log_limb_size);

    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let (device, queue) = get_device_and_queue().await;

    let xr_buf = create_sb_with_data(&device, &xr_limbs);
    let result_0_buf = create_empty_sb(&device, xr_buf.size());
    let result_1_buf = create_empty_sb(&device, xr_buf.size());

    let source = render_secp256r1_curve_tests(filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&xr_buf, &result_0_buf, &result_1_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_0_buf, result_1_buf],
    )
    .await;

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result_x_r = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        let result = &result_x_r * &rinv % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let result_y_0 = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let result_y_1 = convert_result_coord(&results[1][0..num_limbs].to_vec());

    let expected_ys = Affine::get_ys_from_x_unchecked(a.x).unwrap();

    assert!(result_y_0 == expected_ys.0 || result_y_0 == expected_ys.1);
    assert!(result_y_1 == expected_ys.0 || result_y_1 == expected_ys.1);
}

pub async fn do_projective_to_affine_test(
    a: &coords::ProjectiveXYZ<Fq>,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    
    let pt_a_limbs = projectivexyz_to_mont_limbs(&a, &p, log_limb_size);

    let expected_affine = to_affine_func(a.x, a.y, a.z);

    let (device, queue) = get_device_and_queue().await;

    let pt_a_buf = create_sb_with_data(&device, &pt_a_limbs);
    let pt_b_buf = create_empty_sb(&device, pt_a_buf.size());
    let result_buf = create_empty_sb(&device, pt_a_buf.size());

    let source = render_secp256r1_curve_tests(filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&pt_a_buf, &pt_b_buf, &result_buf],
    );

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

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        //let result = &result_x_r * &rinv % &p;
        let result = &result % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let result_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let result_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let result_z = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());

    assert_eq!(result_x, expected_affine.x);
    assert_eq!(result_y, expected_affine.y);
    assert_eq!(result_z, Fq::from(1u32));
}
