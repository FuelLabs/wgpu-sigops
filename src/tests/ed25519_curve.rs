use ark_ed25519::{ EdwardsProjective as Projective, EdwardsAffine as Affine, Fr, Fq };
use ark_ff::{ PrimeField, BigInteger, One };
use ark_ec::{ CurveGroup, AffineRepr };
use std::ops::{ Mul };
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use num_bigint::{ BigUint, RandomBits };
use multiprecision::utils::calc_num_limbs;
use multiprecision::{ mont, bigint };
use fuel_algos::ed25519_curve as curve;
use fuel_algos::coords;
use crate::tests::{ fq_to_biguint, eteprojective_to_mont_limbs };
use crate::shader::render_ed25519_curve_tests;
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

const NUM_RUNS_PER_TEST: usize = 4;

pub fn projective_to_affine_func(x: Fq, y: Fq, t: Fq, z: Fq) -> Affine {
    let p = coords::ETEProjective {x, y, t, z};
    curve::projective_to_affine(&p)
}

#[serial_test::serial]
#[tokio::test]
pub async fn ete_add_2008_hwcd_3() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for log_limb_size in 11..16 {
        for _ in 0..NUM_RUNS_PER_TEST {

            let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let s = Fr::from_be_bytes_mod_order(&s.to_bytes_be());
            let r: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let r = Fr::from_be_bytes_mod_order(&r.to_bytes_be());

            let g = Affine::generator();
            let a: Projective = g.mul(s).into_affine().into();
            let b: Projective = g.mul(r).into_affine().into();

            assert_eq!(a.z, Fq::one());
            assert_eq!(b.z, Fq::one());

            let a = coords::ETEProjective {x: a.x, y: a.y, t: a.t, z: a.z };
            let b = coords::ETEProjective {x: b.x, y: b.y, t: b.t, z: b.z };
            do_add_test(&a, &b, log_limb_size, projective_to_affine_func, "ed25519_curve_tests.wgsl", "test_ete_add_2008_hwcd_3").await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn ete_dbl_2008_hwcd() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for log_limb_size in 11..16 {
        for _ in 0..NUM_RUNS_PER_TEST {

            let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let s = Fr::from_be_bytes_mod_order(&s.to_bytes_be());

            let g = Affine::generator();
            let a: Projective = g.mul(s).into_affine().into();

            assert_eq!(a.z, Fq::one());

            let a = coords::ETEProjective {x: a.x, y: a.y, t: a.t, z: a.z };
            do_dbl_test(&a, log_limb_size, projective_to_affine_func, "ed25519_curve_tests.wgsl", "test_ete_dbl_2008_hwcd").await;
        }
    }
}

pub async fn do_add_test(
    a: &coords::ETEProjective<Fq>,
    b: &coords::ETEProjective<Fq>,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);

    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let pt_a_limbs = eteprojective_to_mont_limbs::<Fq>(&a, &p, log_limb_size);
    let pt_b_limbs = eteprojective_to_mont_limbs::<Fq>(&b, &p, log_limb_size);

    let a = Projective::new(a.x, a.y, a.t, a.z);
    let b = Projective::new(b.x, b.y, a.t, b.z);
    let expected_sum_affine = (a + b).into_affine();

    let (device, queue) = get_device_and_queue().await;

    let pt_a_buf = create_sb_with_data(&device, &pt_a_limbs);
    let pt_b_buf = create_sb_with_data(&device, &pt_b_limbs);
    let result_buf = create_empty_sb(&device, pt_a_buf.size());

    let source = render_ed25519_curve_tests("src/wgsl/", filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&pt_a_buf, &pt_b_buf, &result_buf],
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
    let result_t = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());
    let result_z = convert_result_coord(&results[0][(num_limbs * 3)..(num_limbs * 4)].to_vec());
    let result_affine = to_affine_func(result_x, result_y, result_t, result_z);

    assert_eq!(result_affine, expected_sum_affine);
}

pub async fn do_dbl_test(
    a: &coords::ETEProjective<Fq>,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);

    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let pt_a_limbs = eteprojective_to_mont_limbs::<Fq>(&a, &p, log_limb_size);

    let a = Projective::new(a.x, a.y, a.t, a.z);
    let expected_sum_affine = (a + a).into_affine();

    let (device, queue) = get_device_and_queue().await;

    let pt_a_buf = create_sb_with_data(&device, &pt_a_limbs);
    let pt_b_buf = create_empty_sb(&device, pt_a_buf.size());
    let result_buf = create_empty_sb(&device, pt_a_buf.size());

    let source = render_ed25519_curve_tests("src/wgsl/", filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&pt_a_buf, &pt_b_buf, &result_buf],
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
    let result_t = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());
    let result_z = convert_result_coord(&results[0][(num_limbs * 3)..(num_limbs * 4)].to_vec());
    let result_affine = to_affine_func(result_x, result_y, result_t, result_z);

    assert_eq!(result_affine, expected_sum_affine);
}
