use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::{render_ed25519_eddsa_tests, render_ed25519_utils_tests};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ed25519::{EdwardsAffine as Affine, EdwardsProjective as Projective, Fq, Fr};
use ark_ff::{BigInteger, PrimeField};
use byteorder::{BigEndian, ByteOrder};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use fuel_algos::coords;
use fuel_algos::ed25519_eddsa::{
    ark_ecverify, compressed_y_to_eteprojective, compute_hash, conditional_assign,
    conditional_negate, decompress_to_ete_unsafe, is_negative, sqrt_ratio_i,
};
use fuel_crypto::Message;
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, mont};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sha2::Digest;
use std::ops::Mul;

#[serial_test::serial]
#[tokio::test]
pub async fn verify() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(1);

    for log_limb_size in 13..14 {
        for _ in 0..1 {
            let mut message = [0u8; 100];
            rng.fill_bytes(&mut message);
            let message = Message::new(&message);
            let message = message.as_slice();

            let signing_key: SigningKey = SigningKey::generate(&mut rng);
            let verifying_key = signing_key.verifying_key();
            let signature: Signature = signing_key.sign(&message);

            assert!(verifying_key.verify(&message, &signature).is_ok());

            do_eddsa_test(&verifying_key, &signature, &message, &p, log_limb_size).await;
        }
    }
}

pub async fn do_eddsa_test(
    verifying_key: &VerifyingKey,
    signature: &Signature,
    message: &[u8],
    p: &BigUint,
    log_limb_size: u32,
) {
    let s_bytes = signature.s_bytes();
    let r_bytes = signature.r_bytes();
    let a_bytes = verifying_key.as_bytes();
    let m_bytes = message;
    let mut preimage_bytes = Vec::<u8>::with_capacity(96);
    preimage_bytes.extend(r_bytes);
    preimage_bytes.extend(a_bytes);
    preimage_bytes.extend(m_bytes);
    let mut hasher = sha2::Sha512::new();
    hasher.update(&preimage_bytes);
    let hash0 = hasher.finalize();

    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let expected = ark_ecverify(&verifying_key, &signature, &message);

    let hash1 = compute_hash(&verifying_key, &signature, &message);
    assert_eq!(hash0.as_slice(), hash1.as_slice());

    let a = compressed_y_to_eteprojective(*a_bytes);
    assert_eq!(a.t, a.x * a.y);
    assert_eq!(a.z, Fq::from(1u32));

    let (device, queue) = get_device_and_queue().await;

    let mut s_bytes_le = s_bytes.as_slice().to_vec();
    s_bytes_le.reverse();
    let s_u32s: Vec<u32> = bytemuck::cast_slice(&s_bytes_le).to_vec();

    let mut k_u32s = Vec::with_capacity(preimage_bytes.len() / 4);
    for chunk in preimage_bytes.chunks(4) {
        let value = BigEndian::read_u32(chunk);
        k_u32s.push(value);
    }

    let s_buf = create_sb_with_data(&device, &s_u32s);
    let k_buf = create_sb_with_data(&device, &k_u32s);
    let result_buf = create_empty_sb(&device, (num_limbs * 4 * std::mem::size_of::<u32>()) as u64);

    let source = render_ed25519_eddsa_tests("src/wgsl/", "ed25519_eddsa_tests.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_verify");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&s_buf, &k_buf, &result_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result_r = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        let result = &result_r * &rinv % p;
        //let result = &result_r;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let results =
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;

    let recovered_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let recovered_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let recovered_t = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());
    let recovered_z = convert_result_coord(&results[0][(num_limbs * 3)..(num_limbs * 4)].to_vec());

    //println!("r.x: {}", hex::encode(&recovered_x.into_bigint().to_bytes_be()));
    //println!("r.y: {}", hex::encode(&recovered_y.into_bigint().to_bytes_be()));
    //println!("r.t: {}", hex::encode(&recovered_t.into_bigint().to_bytes_be()));
    //println!("r.z: {}", hex::encode(&recovered_z.into_bigint().to_bytes_be()));

    let recovered =
        Projective::new(recovered_x, recovered_y, recovered_t, recovered_z).into_affine();
    assert_eq!(recovered, expected);
}

#[serial_test::serial]
#[tokio::test]
pub async fn is_negative_test() {
    let a_val = BigUint::parse_bytes(
        b"7525073331273976790771568375528135302506060854772922661176563997672455312353",
        10,
    )
    .unwrap();
    let expected = is_negative(Fq::from_be_bytes_mod_order(&a_val.to_bytes_be()));
    assert!(expected);

    let log_limb_size = 13;
    let num_limbs = 20;

    let a_val_limbs = bigint::from_biguint_le(&a_val, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let a_val_buf = create_sb_with_data(&device, &a_val_limbs);
    let b_val_buf = create_empty_sb(&device, a_val_buf.size());
    let result_buf = create_empty_sb(&device, a_val_buf.size());
    let result2_buf = create_empty_sb(&device, a_val_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_is_negative");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_val_buf, &b_val_buf, &result_buf, &result2_buf],
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

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size);
    let expected = if expected {
        BigUint::from(1u32)
    } else {
        BigUint::from(0u32)
    };
    assert_eq!(result, expected);
}

#[serial_test::serial]
#[tokio::test]
pub async fn conditional_assign_test() {
    do_conditional_assign_test(true).await;
    do_conditional_assign_test(false).await;
}

#[serial_test::serial]
#[tokio::test]
pub async fn conditional_negate_test() {
    do_conditional_negate_test(true).await;
    do_conditional_negate_test(false).await;
}

#[serial_test::serial]
#[tokio::test]
pub async fn pow_p58_test() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(2);

    for log_limb_size in 11..15 {
        for _ in 0..10 {
            let x: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            do_pow_p58_test(&x, &p, log_limb_size).await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn sqrt_ratio_i_test() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(3);

    for log_limb_size in 11..15 {
        for _ in 0..20 {
            let u: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            let v: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &p;
            do_sqrt_ratio_i_test(&u, &v, &p, log_limb_size).await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn reconstruct_ete_point_from_y() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(1);

    for log_limb_size in 11..15 {
        for _ in 0..30 {
            let signing_key: SigningKey = SigningKey::generate(&mut rng);
            let verifying_key = signing_key.verifying_key();
            let a_bytes = verifying_key.as_bytes();
            let a = compressed_y_to_eteprojective(*a_bytes);

            let a = coords::ETEProjective::<Fq> {
                x: a.x,
                y: a.y,
                t: a.t,
                z: a.z,
            };
            assert_eq!(a.t, a.x * a.y);
            assert_eq!(a.z, Fq::from(1u32));

            do_reconstruct_ete_point_from_y_test(&a, &p, log_limb_size).await;
        }
    }
}

#[serial_test::serial]
#[tokio::test]
pub async fn reconstruct_ete_point_from_y_invalid() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(1);

    for log_limb_size in 11..15 {
        for _ in 0..10 {
            let s: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256));
            let s = Fr::from_be_bytes_mod_order(&s.to_bytes_be());
            let g = Affine::generator();
            let a: Projective = g.mul(s).into_affine().into();
            let a = coords::ETEProjective::<Fq> {
                x: a.x,
                y: a.y,
                t: a.t,
                z: a.z,
            };
            assert_eq!(a.t, a.x * a.y);
            assert_eq!(a.z, Fq::from(1u32));

            // Search for an invalid y-coordinate
            let mut i = 1u32;
            let mut new_y: Fq;
            let mut x_sign: u8;

            loop {
                new_y = a.y + Fq::from(i);
                let mut y_bytes = new_y.into_bigint().to_bytes_le();
                x_sign = y_bytes[31] >> 7u8;
                y_bytes[31] &= 0x7f;

                new_y = Fq::from_le_bytes_mod_order(&y_bytes);

                let new_pt_unsafe =
                    decompress_to_ete_unsafe(y_bytes.as_slice().try_into().unwrap());
                if !new_pt_unsafe.0 {
                    break;
                }

                i += 1u32;
            }
            do_reconstruct_ete_point_from_y_invalid_test(&new_y, x_sign, &p, log_limb_size).await;
        }
    }
}

pub async fn do_reconstruct_ete_point_from_y_invalid_test(
    y: &Fq,
    x_sign: u8,
    p: &BigUint,
    log_limb_size: u32,
) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);

    let y_bigint: BigUint = y.into_bigint().into();
    let yr_limbs = bigint::from_biguint_le(&(y_bigint * &r % p), num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let yr_buf = create_sb_with_data(&device, &yr_limbs);
    let x_sign_buf = create_sb_with_data(&device, &[x_sign as u32]);
    let result_buf = create_empty_sb(&device, yr_buf.size() * 4);
    let is_valid_buf = create_empty_sb(&device, x_sign_buf.size());

    let source = render_ed25519_utils_tests(
        "src/wgsl/",
        "ed25519_reconstruct_ete_from_y_tests.wgsl",
        log_limb_size,
    );
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_reconstruct_ete_from_y");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&yr_buf, &x_sign_buf, &result_buf, &is_valid_buf],
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
        &[is_valid_buf],
    )
    .await;
    assert!(results[0][0] == 0);
}

pub async fn do_reconstruct_ete_point_from_y_test(
    a: &coords::ETEProjective<Fq>,
    p: &BigUint,
    log_limb_size: u32,
) {
    let x_sign = if is_negative(a.x) { 1u32 } else { 0u32 };

    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let ay: BigUint = a.y.into_bigint().into();
    let yr_limbs = bigint::from_biguint_le(&(ay * &r % p), num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let yr_buf = create_sb_with_data(&device, &yr_limbs);
    let x_sign_buf = create_sb_with_data(&device, &[x_sign]);
    let result_buf = create_empty_sb(&device, yr_buf.size() * 4);
    let is_valid_buf = create_empty_sb(&device, x_sign_buf.size());

    let source = render_ed25519_utils_tests(
        "src/wgsl/",
        "ed25519_reconstruct_ete_from_y_tests.wgsl",
        log_limb_size,
    );
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_reconstruct_ete_from_y");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&yr_buf, &x_sign_buf, &result_buf, &is_valid_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result_r = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        let result = &result_r * &rinv % p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf, is_valid_buf],
    )
    .await;

    let is_valid_y_coord = results[1][0] == 1;
    assert!(is_valid_y_coord);

    let recovered_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let recovered_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let recovered_t = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());
    let recovered_z = convert_result_coord(&results[0][(num_limbs * 3)..(num_limbs * 4)].to_vec());

    assert_eq!(recovered_x, a.x);
    assert_eq!(recovered_y, a.y);
    assert_eq!(recovered_t, a.t);
    assert_eq!(recovered_z, a.z);
}

pub async fn do_sqrt_ratio_i_test(u: &BigUint, v: &BigUint, p: &BigUint, log_limb_size: u32) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let u_fq = Fq::from_be_bytes_mod_order(&u.to_bytes_be());
    let v_fq = Fq::from_be_bytes_mod_order(&v.to_bytes_be());
    let expected = sqrt_ratio_i(&u_fq, &v_fq);

    let ur_limbs = bigint::from_biguint_le(&(u * &r % p), num_limbs, log_limb_size);
    let vr_limbs = bigint::from_biguint_le(&(v * &r % p), num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let ur_buf = create_sb_with_data(&device, &ur_limbs);
    let vr_buf = create_sb_with_data(&device, &vr_limbs);
    let result_buf = create_empty_sb(&device, ur_buf.size());
    let result2_buf = create_empty_sb(&device, ur_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_sqrt_ratio_i");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&ur_buf, &vr_buf, &result_buf, &result2_buf],
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
        &[result_buf, result2_buf],
    )
    .await;

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size) * rinv % p;
    let was_nonzero_square = bigint::to_biguint_le(&results[1], num_limbs, log_limb_size);

    let was_nonzero_square = was_nonzero_square == BigUint::from(1u32);

    assert_eq!(result, expected.1.into_bigint().into());
    assert_eq!(was_nonzero_square, expected.0);
}

pub async fn do_pow_p58_test(x: &BigUint, p: &BigUint, log_limb_size: u32) {
    let p58_exponent = BigUint::parse_bytes(
        b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd",
        16,
    )
    .unwrap();

    let expected = x.modpow(&p58_exponent, p);

    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);

    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let xr = x * r % p;
    let xr_limbs = bigint::from_biguint_le(&xr, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let xr_buf = create_sb_with_data(&device, &xr_limbs);
    let b_val_buf = create_empty_sb(&device, xr_buf.size());
    let result_buf = create_empty_sb(&device, xr_buf.size());
    let result2_buf = create_empty_sb(&device, xr_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_pow_p58");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&xr_buf, &b_val_buf, &result_buf, &result2_buf],
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

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size) * rinv % p;
    assert_eq!(result, expected);
}

pub async fn do_conditional_assign_test(choice: bool) {
    let a_val = BigUint::parse_bytes(b"123", 10).unwrap();
    let b_val = BigUint::parse_bytes(b"456", 10).unwrap();
    let a = Fq::from_be_bytes_mod_order(&a_val.to_bytes_be());
    let b = Fq::from_be_bytes_mod_order(&b_val.to_bytes_be());
    let expected = conditional_assign(a, b, choice);

    let log_limb_size = 13;
    let num_limbs = 20;

    let a_val_limbs = bigint::from_biguint_le(&a_val, num_limbs, log_limb_size);
    let b_val_limbs = bigint::from_biguint_le(&b_val, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let a_val_buf = create_sb_with_data(&device, &a_val_limbs);
    let b_val_buf = create_sb_with_data(&device, &b_val_limbs);
    let result_buf = create_empty_sb(&device, a_val_buf.size());
    let result2_buf = create_empty_sb(&device, a_val_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let entrypoint = if choice {
        "test_conditional_assign_true"
    } else {
        "test_conditional_assign_false"
    };
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_val_buf, &b_val_buf, &result_buf, &result2_buf],
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

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size);
    assert_eq!(result, expected.into_bigint().into());
}

pub async fn do_conditional_negate_test(choice: bool) {
    let a_val = BigUint::parse_bytes(b"123", 10).unwrap();
    let a = Fq::from_be_bytes_mod_order(&a_val.to_bytes_be());
    let expected = conditional_negate(a, choice);

    let log_limb_size = 13;
    let num_limbs = 20;

    let a_val_limbs = bigint::from_biguint_le(&a_val, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let a_val_buf = create_sb_with_data(&device, &a_val_limbs);
    let b_val_buf = create_empty_sb(&device, a_val_buf.size());
    let result_buf = create_empty_sb(&device, a_val_buf.size());
    let result2_buf = create_empty_sb(&device, a_val_buf.size());

    let source = render_ed25519_utils_tests("src/wgsl/", "ed25519_utils_tests.wgsl", log_limb_size);
    let entrypoint = if choice {
        "test_conditional_negate_true"
    } else {
        "test_conditional_negate_false"
    };
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&a_val_buf, &b_val_buf, &result_buf, &result2_buf],
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

    let result = bigint::to_biguint_le(&results[0], num_limbs, log_limb_size);
    assert_eq!(result, expected.into_bigint().into());
}

#[serial_test::serial]
#[tokio::test]
pub async fn compressed_y_to_eteprojective_test() {
    let p = crate::moduli::ed25519_fq_modulus_biguint();

    let mut rng = ChaCha8Rng::seed_from_u64(1);

    for log_limb_size in 11..15 {
        for _ in 0..10 {
            let signing_key: SigningKey = SigningKey::generate(&mut rng);
            let verifying_key = signing_key.verifying_key();
            let a_bytes = verifying_key.as_bytes();

            do_compressed_y_to_eteprojective_test(*a_bytes, &p, log_limb_size).await;
        }
    }
}

pub async fn do_compressed_y_to_eteprojective_test(
    a_bytes: [u8; 32],
    p: &BigUint,
    log_limb_size: u32,
) {
    let a = compressed_y_to_eteprojective(a_bytes);
    let a = coords::ETEProjective::<Fq> {
        x: a.x,
        y: a.y,
        t: a.t,
        z: a.z,
    };
    assert_eq!(a.t, a.x * a.y);
    assert_eq!(a.z, Fq::from(1u32));

    let expected = Affine::new(a.x, a.y);

    let y_u32s: Vec<u32> = bytemuck::cast_slice(&a_bytes).to_vec();

    let num_limbs = calc_num_limbs(log_limb_size, 256);
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let (device, queue) = get_device_and_queue().await;
    let y_buf = create_sb_with_data(&device, &y_u32s);
    let result_buf = create_empty_sb(&device, (num_limbs * 8 * std::mem::size_of::<u32>()) as u64);
    let is_valid_buf = create_empty_sb(&device, (std::mem::size_of::<u32>()) as u64);

    let source = render_ed25519_utils_tests(
        "src/wgsl/",
        "ed25519_compressed_y_to_eteprojective_tests.wgsl",
        log_limb_size,
    );
    let compute_pipeline =
        create_compute_pipeline(&device, &source, "test_compressed_y_to_eteprojective");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&y_buf, &result_buf, &is_valid_buf],
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
        &[result_buf, is_valid_buf],
    )
    .await;

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result_r = bigint::to_biguint_le(&data, num_limbs, log_limb_size);
        let result = &result_r * &rinv % p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let recovered_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let recovered_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let recovered_t = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());
    let recovered_z = convert_result_coord(&results[0][(num_limbs * 3)..(num_limbs * 4)].to_vec());

    let recovered =
        Projective::new(recovered_x, recovered_y, recovered_t, recovered_z).into_affine();
    assert_eq!(recovered, expected);

    let is_valid = results[1][0];
    assert_eq!(is_valid, 1);
}
