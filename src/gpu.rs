use crate::NumberOfState;
use wgpu::util::DeviceExt;

pub fn calc_6x6_gpu() -> NumberOfState<37, 73> {
    pollster::block_on(run())
}

async fn run() -> NumberOfState<37, 73> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .expect("GPU adapter not found");

    let info = adapter.get_info();
    println!("  adapter: {} ({:?})", info.name, info.backend);

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("Failed to create device");

    const HIST_SIZE: usize = 37 * 73; // 2701
    let buffer_bytes = (HIST_SIZE * std::mem::size_of::<u32>()) as u64;

    let zeros = vec![0u8; HIST_SIZE * 4];

    let hist_lo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hist_lo"),
        contents: &zeros,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    let hist_hi = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hist_hi"),
        contents: &zeros,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    let staging_lo = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging_lo"),
        size: buffer_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let staging_hi = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging_hi"),
        size: buffer_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ising_6x6"),
        source: wgpu::ShaderSource::Wgsl(include_str!("ising_6x6.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("ising_6x6"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: hist_lo.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: hist_hi.as_entire_binding(),
            },
        ],
    });

    // Dispatch 1024 x 1024 workgroups
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(1024, 1024, 1);
    }
    encoder.copy_buffer_to_buffer(&hist_lo, 0, &staging_lo, 0, buffer_bytes);
    encoder.copy_buffer_to_buffer(&hist_hi, 0, &staging_hi, 0, buffer_bytes);
    queue.submit(std::iter::once(encoder.finish()));

    // Read back
    let lo_slice = staging_lo.slice(..);
    let hi_slice = staging_hi.slice(..);

    let (tx_lo, rx_lo) = std::sync::mpsc::channel();
    let (tx_hi, rx_hi) = std::sync::mpsc::channel();
    lo_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx_lo.send(r);
    });
    hi_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx_hi.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx_lo.recv().unwrap().unwrap();
    rx_hi.recv().unwrap().unwrap();

    let lo_data = lo_slice.get_mapped_range();
    let hi_data = hi_slice.get_mapped_range();

    let lo_vals: Vec<u32> = lo_data
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let hi_vals: Vec<u32> = hi_data
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    let mut result = NumberOfState::<37, 73>::new();
    for ene in 0..73usize {
        for mag in 0..37usize {
            let idx = ene * 37 + mag;
            let val = (hi_vals[idx] as u64) << 32 | lo_vals[idx] as u64;
            result.data[ene][mag] = val;
        }
    }

    result
}
