use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, execute_pipeline, finish_encoder_and_read_from_gpu, get_device_and_queue,
};
use crate::shader::render_simple;

#[serial_test::serial]
#[tokio::test]
pub async fn multi_stage_test() {
    let (device, queue) = get_device_and_queue().await;

    // Stage 1
    let val = vec![123];

    let val_buf = create_sb_with_data(&device, &val);
    let result_buf = create_empty_sb(&device, (std::mem::size_of::<u32>()) as u64);

    let source = render_simple("multi_stage_1_test.wgsl");
    let compute_pipeline = create_compute_pipeline(&device, &source, "stage_1");
    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&val_buf, &result_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        1,
        1,
        1,
    );

    // Stage 2
    let b = vec![4];
    let b_buf = create_sb_with_data(&device, &b);
    let result_2_buf = create_empty_sb(&device, (std::mem::size_of::<u32>()) as u64);

    let source = render_simple("multi_stage_2_test.wgsl");
    let compute_pipeline = create_compute_pipeline(&device, &source, "stage_2");
    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&result_buf, &b_buf, &result_2_buf],
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
        finish_encoder_and_read_from_gpu(&device, &queue, Box::new(command_encoder), &[result_2_buf])
            .await;

    assert_eq!(results[0][0], val[0] + 1 + b[0]);
}
