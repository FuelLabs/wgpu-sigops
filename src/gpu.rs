use std::boxed::Box;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

pub async fn get_device_and_queue(
) -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await.unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    //let info = adapter.get_info();
    //println!("{:?}", info);
    (device, queue)
}

pub fn create_command_encoder(
    device: &wgpu::Device,
) -> wgpu::CommandEncoder {
    device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { label: None }
    )
}

pub fn create_sb_with_data(
    device: &wgpu::Device,
    data: &[u32],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(data),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    })
}

pub fn create_empty_sb(
    device: &wgpu::Device,
    size: u64,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        mapped_at_creation: false,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    })
}

pub fn create_compute_pipeline(
    device: &wgpu::Device,
    code: &str,
    entry_point: &str,
) -> wgpu::ComputePipeline {
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&code)),
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point,
    })
}

pub fn execute_pipeline(
    command_encoder: &mut wgpu::CommandEncoder,
    compute_pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
    num_x_workgroups: u32,
    num_y_workgroups: u32,
    num_z_workgroups: u32,
) {
    let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
    cpass.set_pipeline(compute_pipeline);
    cpass.set_bind_group(0, &bind_group, &[]);
    cpass.insert_debug_marker("debug marker");
    cpass.dispatch_workgroups(num_x_workgroups, num_y_workgroups, num_z_workgroups);
}

pub fn create_bind_group(
    device: &wgpu::Device,
    compute_pipeline: &wgpu::ComputePipeline,
    binding_idx: u32,
    buffers: &[&wgpu::Buffer],
) -> wgpu::BindGroup {
    let entries: Vec::<wgpu::BindGroupEntry> = 
        buffers
        .iter()
        .enumerate()
        .map(
            |(i, buf)| 
            wgpu::BindGroupEntry{ binding: i as u32, resource: buf.as_entire_binding() }
        )
        .collect();

    let bind_group_layout = compute_pipeline.get_bind_group_layout(binding_idx);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &entries,
    })
}

pub async fn finish_encoder_and_read_from_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut command_encoder: Box<wgpu::CommandEncoder>,
    buffers: &[wgpu::Buffer],
) -> Vec<Vec<u32>> {
    let mut results = Vec::<Vec<u32>>::with_capacity(buffers.len());
    let mut staging_buffers  = Vec::<wgpu::Buffer>::with_capacity(buffers.len());

    for buffer in buffers {
        let size = buffer.size();
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        (*command_encoder).copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, size);
        staging_buffers.push(staging_buffer);
    }

    queue.submit(Some((*command_encoder).finish()));

    for staging_buffer in staging_buffers {
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(wgpu::Maintain::Wait);

        if let Some(Ok(())) = receiver.receive().await {
            let data = buffer_slice.get_mapped_range();
            let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
            drop(data);
            staging_buffer.unmap();
            results.push(result);
        } else {
            panic!("failed to run compute on gpu!")
        }
    }
    device.destroy();

    results
}

#[cfg(test)]
pub mod tests {
    use crate::gpu::get_device_and_queue;

    #[tokio::test]
    pub async fn test_get_device_and_queue() {
        let (device, _queue) = get_device_and_queue().await;
        let poll_result = device.poll(wgpu::Maintain::Poll);
        assert!(poll_result.is_queue_empty());
    }
}
