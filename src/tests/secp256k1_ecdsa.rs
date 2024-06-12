use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_secp256k1_ecdsa_tests;
use ark_ff::{BigInteger, PrimeField};
use ark_secp256k1::{Affine, Fq};
use fuel_crypto::{Message, SecretKey, Signature};
use multiprecision::utils::calc_num_limbs;
use multiprecision::bigint;
use num_bigint::{BigUint, RandomBits};
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[serial_test::serial]
#[tokio::test]
pub async fn test_secp256k1_ecrecover() {
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let scalar_p = crate::moduli::secp256k1_fr_modulus_biguint();
    for log_limb_size in 13..14 {
        for _ in 0..10 {
            // Generate a random message
            let msg: BigUint = rng.sample::<BigUint, RandomBits>(RandomBits::new(256)) % &scalar_p;
            let message = Message::new(hex::encode(msg.to_bytes_be()));

            let secret = SecretKey::random(&mut rng);
            let pk = secret.public_key();

            let fuel_signature = Signature::sign(&secret, &message);
            let recovered = fuel_signature
                .recover(&message)
                .expect("Failed to recover PK");
            let (_decoded_sig, _is_y_odd) =
                crate::tests::fuel_decode_signature(&fuel_signature.clone());
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

            do_secp256k1_test(
                &sig_bytes,
                &msg_bytes,
                &pk,
                log_limb_size,
                "secp256k1_ecdsa_tests.wgsl",
                "test_secp256k1_recover",
            )
            .await;
        }
    }
}

pub async fn do_secp256k1_test(
    sig_bytes: &[u8],
    msg_bytes: &[u8],
    expected_pk: &Affine,
    log_limb_size: u32,
    filename: &str,
    entrypoint: &str,
) {
    let num_limbs = calc_num_limbs(log_limb_size, 256);

    let p = BigUint::from_bytes_be(&Fq::MODULUS.to_bytes_be());

    let _msg = BigUint::from_bytes_be(&msg_bytes);
    let _sig = BigUint::from_bytes_be(sig_bytes);

    let (device, queue) = get_device_and_queue().await;
    let source = render_secp256k1_ecdsa_tests(filename, log_limb_size);
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
        let result = &result % &p;
        Fq::from_be_bytes_mod_order(&result.to_bytes_be())
    };

    let result_x = convert_result_coord(&results[0][0..num_limbs].to_vec());
    let result_y = convert_result_coord(&results[0][num_limbs..(num_limbs * 2)].to_vec());
    let result_z = convert_result_coord(&results[0][(num_limbs * 2)..(num_limbs * 3)].to_vec());

    assert_eq!(result_x, expected_pk.x);
    assert_eq!(result_y, expected_pk.y);
    assert_eq!(result_z, Fq::from(1u32));
}
