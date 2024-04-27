use std::borrow::Cow;
use std::path::PathBuf;
use crate::gpu::{
    create_sb,
    create_empty_sb,
    get_device_and_queue
};
use num_bigint::BigUint;
use multiprecision::bigint;

#[serial_test::serial]
#[tokio::test]
pub async fn bigint_add_wide() {
    let log_limb_size = 13;
    let num_limbs = 20;
    let a = BigUint::parse_bytes(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 16).unwrap();
    let b = BigUint::parse_bytes(b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 16).unwrap();
    let expected = &a + &b;

    // We are testing add_wide, so the sum should overflow
    assert!(expected > BigUint::from(2u32).pow(256));
    
    let a_limbs = bigint::from_biguint_le(&a, num_limbs, log_limb_size);
    let b_limbs = bigint::from_biguint_le(&b, num_limbs, log_limb_size);
    let expected_limbs = bigint::from_biguint_le(&expected, num_limbs + 1, log_limb_size);
    let expected_limbs_2 = bigint::add_wide(&a_limbs, &b_limbs, log_limb_size);

    assert!(bigint::eq(&expected_limbs, &expected_limbs_2));

    let (device, queue) = get_device_and_queue().await;

    let a_buf = create_sb(&device, &a_limbs);
    let b_buf = create_sb(&device, &b_limbs);
    let result_buf = create_empty_sb(&device, ((num_limbs + 1) * 8 * std::mem::size_of::<u8>()) as u64);

    let path_from_cargo_manifest_dir = "src/tests/";
    let input_filename = "bigint_add_wide.wgsl";
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path_from_cargo_manifest_dir).join(input_filename);
    let source = std::fs::read_to_string(input_path).unwrap();

    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&source)),
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point: "main",
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: a_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: b_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: result_buf.as_entire_binding(),
            },
        ],
    });

    {
    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
    cpass.set_pipeline(&compute_pipeline);
    cpass.set_bind_group(0, &bind_group, &[]);
    cpass.insert_debug_marker("debug marker");
    cpass.dispatch_workgroups(1, 1, 1);
    }

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: result_buf.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_buffer_to_buffer(&result_buf, 0, &staging_buffer, 0, result_buf.size());

    // Submits command encoder for processing
    queue.submit(Some(encoder.finish()));

    // Note that we're not calling `.await` here.
    let buffer_slice = staging_buffer.slice(..);

    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
 
    device.poll(wgpu::Maintain::Wait);

    // Awaits until `buffer_future` can be read from
    if let Some(Ok(())) = receiver.receive().await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap();

        // Returns data from buffer
        let result = bigint::to_biguint_le(&result[0..num_limbs+1].to_vec(), num_limbs + 1, log_limb_size);

        assert_eq!(result, expected);
    } else {
        panic!("failed to run compute on gpu!")
    }
    device.destroy();
}
