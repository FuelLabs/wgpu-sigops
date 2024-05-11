use ark_ff::{ PrimeField, BigInteger };
use ark_secp256k1::{ Affine, Fq };
use ark_ec::AffineRepr;
use num_bigint::{ BigUint, RandomBits };
use multiprecision::utils::calc_num_limbs;
use multiprecision::{ mont, bigint };
use fuel_crypto::{ Message, Signature, SecretKey };
use std::str::FromStr;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use crate::shader::render_secp256k1_ecdsa_tests;
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
use crate::tests::secp256k1_curve::projective_to_affine_func;

#[serial_test::serial]
#[tokio::test]
pub async fn test_secp256k1_ecrecover() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();
    for log_limb_size in 13..14 {
        for i in 1..100 {
            // Generate a random message
            let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
            //println!("{}", hex::encode(msg.to_bytes_be()));
            let message = Message::new(hex::encode(msg.to_bytes_be()));
            //let message = Message::new(b"aA beast can never be as cruel as a human being, so artistically, so picturesquely cruel.");
            //let message = Message::new(b"db7ab4303b0a72c2ca0c574308434c198c65f8c985b46530e19f8bc9318a1bc5");
 
            let mut i_str = format!("{}", i);
            while i_str.len() < 64 {
                i_str = format!("0{}", i_str);
            }
            let secret = SecretKey::from_str(&i_str).unwrap();
            let pk = secret.public_key();
            let fuel_signature = Signature::sign(&secret, &message);
            let recovered = fuel_signature.recover(&message).expect("Failed to recover PK");
            let (_decoded_sig, _is_y_odd) = crate::tests::fuel_decode_signature(&fuel_signature.clone());
            assert_eq!(recovered, pk);

            let msg_bytes = message.as_slice();
            let sig_bytes = fuel_signature.as_slice();
            let pk_affine_bytes = pk.as_slice();

            let pk_x = pk_affine_bytes[0..32].to_vec();
            let pk_y = pk_affine_bytes[32..64].to_vec();
            let pk_x = BigUint::from_bytes_be(&pk_x);
            let pk_y = BigUint::from_bytes_be(&pk_y);
            let pk_x = Fq::from_be_bytes_mod_order(&pk_x.to_bytes_be());
            let pk_y = Fq::from_be_bytes_mod_order(&pk_y.to_bytes_be());
            let pk = Affine::new(pk_x, pk_y);
            
            let msg = BigUint::from_bytes_be(&msg_bytes);

            do_secp256k1_test(i, &sig_bytes, &msg, &pk, log_limb_size, projective_to_affine_func, "secp256k1_ecdsa_tests.wgsl", "test_secp256k1_recover").await;
        }
    }
}

pub async fn do_secp256k1_test(
    i: u32,
    sig_bytes: &[u8],
    msg: &BigUint,
    expected_pk: &Affine,
    log_limb_size: u32,
    to_affine_func: fn(Fq, Fq, Fq) -> Affine,
    filename: &str,
    entrypoint: &str,
) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);

    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());
    let r = mont::calc_mont_radix(num_limbs, log_limb_size);
    let res = mont::calc_rinv_and_n0(&p, &r, log_limb_size);
    let rinv = res.0;

    let _sig = BigUint::from_bytes_be(sig_bytes);
    let msg_limbs = bigint::from_biguint_le(&msg, num_limbs, log_limb_size);

    let (device, queue) = get_device_and_queue().await;
    let source = render_secp256k1_ecdsa_tests("src/wgsl/", filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let sig_u32s: Vec<u32> = bytemuck::cast_slice(&sig_bytes).to_vec();
    let sig_buf = create_sb_with_data(&device, &sig_u32s);
    let msg_buf = create_sb_with_data(&device, &msg_limbs);
    let result_buf = create_empty_sb(&device, msg_buf.size() * 3);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sig_buf, &msg_buf, &result_buf],
    );

    execute_pipeline(&mut command_encoder, &compute_pipeline, &bind_group, 1, 1, 1);

    let results = finish_encoder_and_read_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[result_buf],
    ).await;

    let convert_result_coord = |data: &Vec<u32>| -> Fq {
        let result = bigint::to_biguint_le(&data, num_limbs, log_limb_size);

        //println!("{}", result);
        let result = &result * &rinv % &p;

        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let result_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let result_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let result_z = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());

    let result_affine = to_affine_func(result_x, result_y, result_z);

    if result_affine.is_zero() {
        println!("i: {}", i);
    }
    assert_eq!(result_affine, *expected_pk);
}