use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_buffer_test;
use num_bigint::BigUint;

/// This test shows how to pass a slice of bytes to the GPU.
/// 1. Convert it to a slice of u32s using bytemuck
/// 2. Read the result using finish_encoder_and_read_from_gpu() which uses bytemuck to convert
///    bytes to u32s
#[serial_test::serial]
#[tokio::test]
pub async fn test_buffer() {
    let p = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
        16,
    )
    .unwrap();

    let p_bytes = p.to_bytes_be();
    assert_eq!(p_bytes.len(), 32);
    let p_u32s: Vec<u32> = bytemuck::cast_slice(&p_bytes).to_vec();

    let expected: &[u32] = &[
        4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4278190079,
        805109759,
    ];
    assert_eq!(p_u32s, expected);

    let (device, queue) = get_device_and_queue().await;

    let p_buf = create_sb_with_data(&device, &p_u32s);
    let result_buf = create_empty_sb(&device, (1 * std::mem::size_of::<u32>()) as u64);

    let source = render_buffer_test("buffer_tests.wgsl");
    let compute_pipeline = create_compute_pipeline(&device, &source, "test_buffer");
    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(&device, &compute_pipeline, 0, &[&p_buf, &result_buf]);

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

    assert_eq!(results[0][0], p_u32s[0]);
    assert_eq!(results[0].len(), 1);
}
