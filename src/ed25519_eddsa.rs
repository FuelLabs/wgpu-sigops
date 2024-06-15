use crate::benchmarks::compute_num_workgroups;
use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, create_ub_with_data, execute_pipeline,
    finish_encoder_and_read_bytes_from_gpu, get_device_and_queue,
};
use crate::shader::render_ed25519_eddsa_tests;
use ed25519_dalek::{Signature, VerifyingKey};
use fuel_crypto::Message;

pub async fn ecverify(
    signatures: Vec<Signature>,
    messages: Vec<Message>,
    verifying_keys: Vec<VerifyingKey>,
    log_limb_size: u32,
) -> Vec<bool> {
    let num_signatures = signatures.len();
    assert_eq!(num_signatures, messages.len());
    assert!(num_signatures <= 256 * 256 * 256 * 64);

    if num_signatures == 0 {
        return vec![];
    }

    // Compute the next power of 2
    let next_pow_2 = (2u32.pow((num_signatures as f32).log2().ceil() as u32)) as usize;

    assert!(num_signatures <= next_pow_2);

    // Set up data for the input buffers

    let mut all_sig_bytes = Vec::with_capacity(num_signatures * 64 * 8);
    let mut all_pk_bytes = Vec::with_capacity(num_signatures * 32 * 8);
    let mut all_msg_bytes: Vec<u8> = Vec::with_capacity(num_signatures * 32 * 8);

    for i in 0..num_signatures {
        let sig_bytes_be = signatures[i].to_bytes();
        let pk_bytes_be = verifying_keys[i].to_bytes();
        let msg_bytes = messages[i].as_slice();

        all_sig_bytes.extend(sig_bytes_be);
        all_pk_bytes.extend(pk_bytes_be);
        all_msg_bytes.extend(msg_bytes);
    }

    if num_signatures < next_pow_2 {
        all_sig_bytes.resize(next_pow_2 * 64, 0);
        all_msg_bytes.resize(next_pow_2 * 32, 0);
    }

    let workgroup_size = 256;
    let (num_x_workgroups, num_y_workgroups, num_z_workgroups) =
        compute_num_workgroups(next_pow_2, workgroup_size);

    let all_sig_u32s: Vec<u32> = bytemuck::cast_slice(&all_sig_bytes).to_vec();
    let all_pk_u32s: Vec<u32> = bytemuck::cast_slice(&all_pk_bytes).to_vec();
    let all_msg_u32s: Vec<u32> = bytemuck::cast_slice(&all_msg_bytes).to_vec();

    let (device, queue) = get_device_and_queue().await;
    let params = &[
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    ];

    let sig_buf = create_sb_with_data(&device, &all_sig_u32s);
    let pk_buf = create_sb_with_data(&device, &all_pk_u32s);
    let msg_buf = create_sb_with_data(&device, &all_msg_u32s);
    let is_valid_buf = create_empty_sb(&device, (next_pow_2 * std::mem::size_of::<u32>()) as u64);
    let params_buf = create_ub_with_data(&device, params);

    let source = render_ed25519_eddsa_tests("ed25519_eddsa_benchmarks.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "benchmark_verify");

    let mut command_encoder = create_command_encoder(&device);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sig_buf, &pk_buf, &msg_buf, &is_valid_buf, &params_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    );

    let results = finish_encoder_and_read_bytes_from_gpu(
        &device,
        &queue,
        Box::new(command_encoder),
        &[is_valid_buf],
    )
    .await;

    let mut all_is_valid: Vec<bool> = Vec::with_capacity(num_signatures);
    for i in 0..num_signatures {
        all_is_valid.push(results[0][i * 4] == 1);
    }

    all_is_valid
}