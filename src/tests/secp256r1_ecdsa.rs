use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_secp256r1_ecdsa_tests;
use crate::tests::secp256r1_curve::projective_to_affine_func;
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, PrimeField};
use ark_secp256r1::{Affine, Fq};
use fuel_crypto::secp256r1::p256::{encode_pubkey, recover, sign_prehashed};
use p256::ecdsa::SigningKey;
use fuel_crypto::Message;
use multiprecision::utils::calc_num_limbs;
use multiprecision::{bigint, mont};
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[serial_test::serial]
#[tokio::test]
pub async fn test_secp256r1_ecrecover() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let scalar_p = crate::moduli::secp256r1_fr_modulus_biguint();
    for log_limb_size in 13..14 {
        for i in 1..10 {
            // Generate a random message
            let signing_key = SigningKey::random(&mut rng);
            let verifying_key = signing_key.verifying_key();

            let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
            let message = Message::new(hex::encode(msg.to_bytes_be()));

            let fuel_signature = sign_prehashed(&signing_key, &message).expect("Couldn't sign");

            let Ok(recovered) = recover(&fuel_signature, &message) else {
                panic!("Failed to recover public key from the message");
            };

            assert_eq!(*recovered, encode_pubkey(*verifying_key));

            let msg_bytes = message.as_slice();
            let sig_bytes = fuel_signature.as_slice();
            let pk_affine_bytes = &verifying_key.to_sec1_bytes()[1..65];

            let pk_x = pk_affine_bytes[0..32].to_vec();
            let pk_x = BigUint::from_bytes_be(&pk_x);
            let pk_x = Fq::from_be_bytes_mod_order(&pk_x.to_bytes_be());

            let pk_y = pk_affine_bytes[32..64].to_vec();
            let pk_y = BigUint::from_bytes_be(&pk_y);
            let pk_y = Fq::from_be_bytes_mod_order(&pk_y.to_bytes_be());

            let pk = Affine::new(pk_x, pk_y);

            do_secp256r1_test(
                i,
                &sig_bytes,
                &msg_bytes,
                &pk,
                log_limb_size,
                projective_to_affine_func,
                "secp256r1_ecdsa_tests.wgsl",
                "test_secp256r1_recover",
            )
            .await;
        }
    }
}

pub async fn do_secp256r1_test(
    i: u32,
    sig_bytes: &[u8],
    msg_bytes: &[u8],
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
    let _msg = BigUint::from_bytes_be(&msg_bytes);

    let (device, queue) = get_device_and_queue().await;
    let source = render_secp256r1_ecdsa_tests(filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let sig_u32s: Vec<u32> = bytemuck::cast_slice(&sig_bytes).to_vec();
    let msg_u32s: Vec<u32> = bytemuck::cast_slice(&msg_bytes).to_vec();

    let sig_buf = create_sb_with_data(&device, &sig_u32s);
    let msg_buf = create_sb_with_data(&device, &msg_u32s);
    let result_buf = create_empty_sb(&device, (num_limbs * 3 * std::mem::size_of::<u32>()) as u64);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sig_buf, &msg_buf, &result_buf],
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

        let result = &result * &rinv % &p;
        //println!("{}", result);

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
