use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_sha512_96_tests;
use byteorder::{BigEndian, ByteOrder};
use rand::RngCore;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sha2::Digest;

#[serial_test::serial]
#[tokio::test]
pub async fn sha512_96() {
    let mut rng = ChaCha8Rng::seed_from_u64(1 as u64);

    for _ in 0..50 {
        let mut input = [0u8; 96];
        rng.fill_bytes(&mut input);

        do_sha512_96_test(&input, "sha512_96_tests.wgsl", "test_sha512_96").await;
    }
}

pub async fn do_sha512_96_test(input_bytes: &[u8], filename: &str, entrypoint: &str) {
    let mut hasher = sha2::Sha512::new();
    hasher.update(input_bytes);
    let expected = hasher.finalize();

    let (device, queue) = get_device_and_queue().await;
    let mut input_u32s = Vec::with_capacity(input_bytes.len() / 4);
    for chunk in input_bytes.chunks(4) {
        let value = BigEndian::read_u32(chunk);
        input_u32s.push(value);
    }
    let input_buf = create_sb_with_data(&device, &input_u32s);
    let source = render_sha512_96_tests(filename);
    let compute_pipeline = create_compute_pipeline(&device, &source, entrypoint);

    let result_buf = create_empty_sb(&device, (64 * std::mem::size_of::<u8>()) as u64);

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&input_buf, &result_buf]);

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

    let result_bytes: Vec<u8> = flip_endianness(&results[0]);

    assert_eq!(
        hex::encode(result_bytes.as_slice()),
        hex::encode(expected.as_slice())
    );
}

pub fn flip_endianness(slice: &[u32]) -> Vec<u8> {
    let flipped: Vec<u32> = slice.iter().map(|&num| num.to_be()).collect();
    bytemuck::cast_slice(&flipped).to_vec()
}
