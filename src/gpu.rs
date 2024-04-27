use wgpu::util::DeviceExt;

pub async fn get_device_and_queue() -> (wgpu::Device, wgpu::Queue) {
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

pub fn create_sb(
    device: &wgpu::Device,
    contents: &[u32],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(contents),
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
