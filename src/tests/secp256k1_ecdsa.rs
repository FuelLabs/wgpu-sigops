use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_bytes_from_gpu, get_device_and_queue,
};
use crate::shader::render_secp256k1_ecdsa_tests;
use fuel_crypto::{Message, SecretKey, Signature};
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

            //let msg_bytes = vec![0u8; msg_bytes.len()];
            //let sig_bytes = vec![0u8; sig_bytes.len()];
            //let pk_affine_bytes = vec![0u8; 64];

            do_secp256k1_test(
                &sig_bytes,
                &msg_bytes,
                &pk_affine_bytes,
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
    pk_affine_bytes: &[u8],
    log_limb_size: u32,
    filename: &str,
    entrypoint: &str,
) {
    let (device, queue) = get_device_and_queue().await;
    let source = render_secp256k1_ecdsa_tests(filename, log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let sig_u32s: Vec<u32> = bytemuck::cast_slice(&sig_bytes).to_vec();
    let msg_u32s: Vec<u32> = bytemuck::cast_slice(&msg_bytes).to_vec();

    let sig_buf = create_sb_with_data(&device, &sig_u32s);
    let msg_buf = create_sb_with_data(&device, &msg_u32s);
    let result_buf = create_empty_sb(&device, (16 * std::mem::size_of::<u32>()) as u64);

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
        finish_encoder_and_read_bytes_from_gpu(&device, &queue, Box::new(command_encoder), &[result_buf])
            .await;

    assert_eq!(results[0], pk_affine_bytes);
}
