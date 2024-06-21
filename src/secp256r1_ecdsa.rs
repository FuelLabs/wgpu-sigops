use crate::benchmarks::compute_num_workgroups;
use crate::gpu::{
    create_bind_group, create_command_encoder, create_compute_pipeline, create_empty_sb,
    create_sb_with_data, create_ub_with_data, execute_pipeline,
    finish_encoder_and_read_bytes_from_gpu, get_device_and_queue,
};
use crate::shader::render_secp256r1_ecdsa;
use multiprecision::utils::calc_num_limbs;
use fuel_crypto::Message;
use fuel_types::Bytes64;

pub fn init(
    signatures: &Vec<Bytes64>,
    messages: &Vec<Message>,
    log_limb_size: u32,
) ->
    (usize, usize, usize, Vec<u32>, Vec<u32>, (u32, u32, u32))
{
    let num_limbs = calc_num_limbs(log_limb_size, 256);

    let num_signatures = signatures.len();
    assert_eq!(num_signatures, messages.len());
    assert!(num_signatures <= 256 * 256 * 256 * 64);

    // Compute the next power of 2
    let next_pow_2 = (2u32.pow((num_signatures as f32).log2().ceil() as u32)) as usize;

    assert!(num_signatures <= next_pow_2);

    let mut all_sig_bytes = Vec::<u8>::with_capacity(next_pow_2 * 64);
    let mut all_msg_bytes = Vec::<u8>::with_capacity(next_pow_2 * 32);
    for sig in signatures {
        let sig_bytes = sig.as_slice();
        all_sig_bytes.extend(sig_bytes);
    }

    for msg in messages {
        let msg_bytes = msg.as_slice();
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
    let all_msg_u32s: Vec<u32> = bytemuck::cast_slice(&all_msg_bytes).to_vec();

    let params = (
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    );
    (num_signatures, next_pow_2, num_limbs, all_sig_u32s, all_msg_u32s, params)
}

pub async fn ecrecover(
    signatures: &Vec<Bytes64>,
    messages: &Vec<Message>,
    table_limbs: &Vec<u32>,
    log_limb_size: u32,
) -> Vec<Vec<u8>> {
    let (num_signatures, next_pow_2, num_limbs, all_sig_u32s, all_msg_u32s, params_t) = init(&signatures, &messages, log_limb_size);
    let (num_x_workgroups, num_y_workgroups, num_z_workgroups) = params_t;
    let params = &[num_x_workgroups, num_y_workgroups, num_z_workgroups];

    if num_signatures == 0 {
        return vec![];
    }

    let (device, queue) = get_device_and_queue().await;
    let mut command_encoder = create_command_encoder(&device);

    // Stage 0
    let source = render_secp256r1_ecdsa("secp256r1_ecdsa_main_0.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "secp256r1_recover_0");

    let sig_buf = create_sb_with_data(&device, &all_sig_u32s);
    let msg_buf = create_sb_with_data(&device, &all_msg_u32s);
    let u1_buf = create_empty_sb(&device, (num_limbs * next_pow_2 * std::mem::size_of::<u32>()) as u64);
    let u2_buf = create_empty_sb(&device, (num_limbs * next_pow_2 * std::mem::size_of::<u32>()) as u64);
    let recovered_r_buf = create_empty_sb(&device, (num_limbs * 3 * next_pow_2 * std::mem::size_of::<u32>()) as u64);
    let params_buf = create_ub_with_data(&device, params);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sig_buf, &msg_buf, &u1_buf, &u2_buf, &recovered_r_buf, &params_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    );

    // Stage 1
    let source = render_secp256r1_ecdsa("secp256r1_ecdsa_main_1.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "secp256r1_recover_1");

    let table_buf = create_sb_with_data(&device, table_limbs);
    let u1g_buf = create_empty_sb(&device, (num_limbs * 3 * next_pow_2 * std::mem::size_of::<u32>()) as u64);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&table_buf, &u1_buf, &u1g_buf, &params_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    );

    // Stage 2
    let source = render_secp256r1_ecdsa("secp256r1_ecdsa_main_2.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "secp256r1_recover_2");

    let u2r_buf = create_empty_sb(&device, (num_limbs * 3 * next_pow_2 * std::mem::size_of::<u32>()) as u64);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&u2_buf, &recovered_r_buf, &u2r_buf, &params_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    );

    // Stage 3
    let source = render_secp256r1_ecdsa("secp256r1_ecdsa_main_3.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "secp256r1_recover_3");

    let sum_buf = create_empty_sb(&device, (num_limbs * 3 * next_pow_2 * std::mem::size_of::<u32>()) as u64);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&u1g_buf, &u2r_buf, &sum_buf, &params_buf],
    );

    execute_pipeline(
        &mut command_encoder,
        &compute_pipeline,
        &bind_group,
        num_x_workgroups as u32,
        num_y_workgroups as u32,
        num_z_workgroups as u32,
    );

    // Stage 4
    let source = render_secp256r1_ecdsa("secp256r1_ecdsa_main_4.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "secp256r1_recover_4");

    let result_buf = create_empty_sb(&device, (64 * next_pow_2 * std::mem::size_of::<u32>()) as u64);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sum_buf, &result_buf, &params_buf],
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
        &[result_buf],
    )
    .await;

    let mut all_recovered: Vec<Vec<u8>> = Vec::with_capacity(num_signatures * 64);
    for i in 0..num_signatures {
        let result_bytes = &results[0][i * 64..i * 64 + 64];
        all_recovered.push(result_bytes.to_vec());
    }
    all_recovered
}

pub async fn ecrecover_single_shader(
    signatures: &Vec<Bytes64>,
    messages: &Vec<Message>,
    log_limb_size: u32,
) -> Vec<Vec<u8>> {
    let (num_signatures, next_pow_2, _num_limbs, all_sig_u32s, all_msg_u32s, params_t) = init(&signatures, &messages, log_limb_size);
    let (num_x_workgroups, num_y_workgroups, num_z_workgroups) = params_t;
    let params = &[num_x_workgroups, num_y_workgroups, num_z_workgroups];

    if num_signatures == 0 {
        return vec![];
    }

    let (device, queue) = get_device_and_queue().await;
    let mut command_encoder = create_command_encoder(&device);

    let source = render_secp256r1_ecdsa("secp256r1_ecdsa_main.wgsl", log_limb_size);
    let compute_pipeline = create_compute_pipeline(&device, &source, "secp256r1_recover_main");

    let sig_buf = create_sb_with_data(&device, &all_sig_u32s);
    let msg_buf = create_sb_with_data(&device, &all_msg_u32s);
    let result_buf = create_empty_sb(&device, (64 * next_pow_2 * std::mem::size_of::<u32>()) as u64);
    let params_buf = create_ub_with_data(&device, params);

    let bind_group = create_bind_group(
        &device,
        &compute_pipeline,
        0,
        &[&sig_buf, &msg_buf, &result_buf, &params_buf],
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
        &[result_buf],
    )
    .await;

    let mut all_recovered: Vec<Vec<u8>> = Vec::with_capacity(num_signatures * 64);
    for i in 0..num_signatures {
        let result_bytes = &results[0][i * 64..i * 64 + 64];
        all_recovered.push(result_bytes.to_vec());
    }
    all_recovered
}
