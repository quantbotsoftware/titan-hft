use bytemuck::{Pod, Zeroable};
use rand::Rng;
use std::sync::{Arc, Mutex};
use tokio::sync::OnceCell;
use wgpu::util::DeviceExt;

use crate::error::TitanError;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct DotInput {
    pub len: u32,
    pub total_threads: u32,
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct IndicatorParams {
    pub len: u32,
    pub ema_fast: u32,
    pub ema_slow: u32,
    pub rsi_period: u32,
}

pub struct GpuBacktestInputs {
    pub ema_fast: Vec<f32>,
    pub ema_slow: Vec<f32>,
    pub rsi: Vec<f32>,
}

pub struct GpuBatchOutputs {
    pub ema_fast: Vec<f32>,
    pub ema_slow: Vec<f32>,
    pub rsi: Vec<f32>,
    pub offsets: Vec<u32>,
    pub lengths: Vec<u32>,
}

#[derive(Default)]
struct GpuBufferPool {
    closes: Option<ReusableBuffer>,
    offsets: Option<ReusableBuffer>,
    lengths: Option<ReusableBuffer>,
    ema_fast: Option<ReusableBuffer>,
    ema_slow: Option<ReusableBuffer>,
    rsi: Option<ReusableBuffer>,
    ema_fast_staging: Option<ReusableBuffer>,
    ema_slow_staging: Option<ReusableBuffer>,
    rsi_staging: Option<ReusableBuffer>,
}

struct ReusableBuffer {
    buffer: Arc<wgpu::Buffer>,
    size: u64,
}

fn ensure_buffer(
    slot: &mut Option<ReusableBuffer>,
    device: &wgpu::Device,
    size: u64,
    usage: wgpu::BufferUsages,
    label: &str,
) -> Arc<wgpu::Buffer> {
    let needs_new = slot.as_ref().is_none_or(|b| b.size < size);
    if needs_new {
        let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        }));
        *slot = Some(ReusableBuffer { buffer, size });
    }
    slot.as_ref()
        .expect("buffer just initialized")
        .buffer
        .clone()
}

struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    dot_bind_group_layout: wgpu::BindGroupLayout,
    dot_pipeline: wgpu::ComputePipeline,
    indicator_bind_group_layout: wgpu::BindGroupLayout,
    indicator_pipeline: wgpu::ComputePipeline,
    indicator_batch_bind_group_layout: wgpu::BindGroupLayout,
    indicator_batch_pipeline: wgpu::ComputePipeline,
    buffer_pool: Mutex<GpuBufferPool>,
}

static GPU_CONTEXT: OnceCell<Arc<GpuContext>> = OnceCell::const_new();

async fn get_gpu_context() -> Result<Arc<GpuContext>, TitanError> {
    let ctx = GPU_CONTEXT
        .get_or_try_init(|| async {
            let instance = wgpu::Instance::default();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .ok_or_else(|| TitanError::Gpu("No GPU adapter".into()))?;

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
                .map_err(|e| TitanError::Gpu(format!("GPU device error: {:?}", e)))?;

            let dot_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("dot-shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("gpu_dot.wgsl").into()),
            });
            let dot_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("dot-bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
            let dot_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("dot-pl"),
                    bind_group_layouts: &[&dot_bind_group_layout],
                    push_constant_ranges: &[],
                });
            let dot_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("dot-pipe"),
                layout: Some(&dot_pipeline_layout),
                module: &dot_shader,
                entry_point: "main",
            });

            let indicator_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("indicator-shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("gpu_indicators.wgsl").into()),
            });
            let indicator_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("indicator-bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
            let indicator_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("indicator-pl"),
                    bind_group_layouts: &[&indicator_bind_group_layout],
                    push_constant_ranges: &[],
                });
            let indicator_pipeline =
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("indicator-pipe"),
                    layout: Some(&indicator_pipeline_layout),
                    module: &indicator_shader,
                    entry_point: "main",
                });

            let indicator_batch_shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("indicator-batch-shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("gpu_indicators_batch.wgsl").into(),
                    ),
                });
            let indicator_batch_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("indicator-batch-bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
            let indicator_batch_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("indicator-batch-pl"),
                    bind_group_layouts: &[&indicator_batch_bind_group_layout],
                    push_constant_ranges: &[],
                });
            let indicator_batch_pipeline =
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("indicator-batch-pipe"),
                    layout: Some(&indicator_batch_pipeline_layout),
                    module: &indicator_batch_shader,
                    entry_point: "main",
                });

            Ok::<Arc<GpuContext>, TitanError>(Arc::new(GpuContext {
                device,
                queue,
                dot_bind_group_layout,
                dot_pipeline,
                indicator_bind_group_layout,
                indicator_pipeline,
                indicator_batch_bind_group_layout,
                indicator_batch_pipeline,
                buffer_pool: Mutex::new(GpuBufferPool::default()),
            }))
        })
        .await?;
    Ok(ctx.clone())
}

pub async fn gpu_dot(a: &[f32], b: &[f32]) -> Result<f32, TitanError> {
    if a.len() != b.len() || a.is_empty() {
        return Err(TitanError::Gpu("Invalid vectors".into()));
    }

    let ctx = get_gpu_context().await?;
    let device = &ctx.device;
    let queue = &ctx.queue;

    let a_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("a"),
        contents: bytemuck::cast_slice(a),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let b_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("b"),
        contents: bytemuck::cast_slice(b),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let workgroup_size: u32 = 256;
    let len = a.len() as u32;
    let mut num_workgroups = len.div_ceil(workgroup_size);
    let max_workgroups: u32 = 1024;
    if num_workgroups == 0 {
        num_workgroups = 1;
    } else if num_workgroups > max_workgroups {
        num_workgroups = max_workgroups;
    }
    let total_threads = num_workgroups * workgroup_size;
    let out_byte_len = (num_workgroups as u64) * 4;

    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("out"),
        size: out_byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let params = DotInput {
        len,
        total_threads,
        _pad: [0; 2],
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("dot-bg"),
        layout: &ctx.dot_bind_group_layout,
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
                resource: out_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: out_byte_len,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("dot-enc"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("dot-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.dot_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, out_byte_len);
    queue.submit(Some(encoder.finish()));

    let partials = map_read_f32(&ctx, &staging, num_workgroups as usize).await?;
    let sum: f64 = partials.iter().map(|v| *v as f64).sum();
    Ok(sum as f32)
}

async fn map_read_f32(
    ctx: &Arc<GpuContext>,
    buffer: &wgpu::Buffer,
    len: usize,
) -> Result<Vec<f32>, TitanError> {
    let slice = buffer.slice(..);
    let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
    slice.map_async(wgpu::MapMode::Read, move |v| {
        let _ = tx.send(v);
    });
    let ctx_clone = Arc::clone(ctx);
    tokio::task::spawn_blocking(move || {
        ctx_clone.device.poll(wgpu::Maintain::Wait);
    })
    .await
    .map_err(|e| TitanError::Gpu(format!("GPU poll task panicked: {e}")))?;
    rx.receive()
        .await
        .ok_or_else(|| TitanError::Gpu("Map failed".into()))?
        .map_err(|_| TitanError::Gpu("Map error".into()))?;
    let data = slice.get_mapped_range();
    let mut out = bytemuck::cast_slice::<u8, f32>(&data).to_vec();
    out.truncate(len);
    drop(data);
    buffer.unmap();
    Ok(out)
}

pub async fn gpu_backtest_inputs(
    closes: &[f32],
    ema_fast: u32,
    ema_slow: u32,
    rsi_period: u32,
) -> Result<GpuBacktestInputs, TitanError> {
    if closes.is_empty() {
        return Err(TitanError::Gpu("Empty closes".into()));
    }
    if closes.len() < 128 {
        return Err(TitanError::Gpu("Not enough data for GPU".into()));
    }
    if rsi_period == 0 {
        return Err(TitanError::Gpu("rsi_period must be > 0".into()));
    }
    if ema_fast == 0 || ema_slow == 0 {
        return Err(TitanError::Gpu("EMA periods must be > 0".into()));
    }

    let len = closes.len();
    let byte_len = std::mem::size_of_val(closes) as u64;

    let ctx = get_gpu_context().await?;
    let device = &ctx.device;
    let queue = &ctx.queue;

    let closes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("closes"),
        contents: bytemuck::cast_slice(closes),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let ema_fast_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ema-fast"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let ema_slow_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ema-slow"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let rsi_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rsi"),
        size: byte_len,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let params = IndicatorParams {
        len: len as u32,
        ema_fast,
        ema_slow,
        rsi_period,
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indicator-params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("indicator-bg"),
        layout: &ctx.indicator_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: closes_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: ema_fast_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: ema_slow_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: rsi_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    let ema_fast_staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ema-fast-staging"),
        size: byte_len,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let ema_slow_staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ema-slow-staging"),
        size: byte_len,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let rsi_staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rsi-staging"),
        size: byte_len,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("indicator-enc"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("indicator-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.indicator_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&ema_fast_buf, 0, &ema_fast_staging, 0, byte_len);
    encoder.copy_buffer_to_buffer(&ema_slow_buf, 0, &ema_slow_staging, 0, byte_len);
    encoder.copy_buffer_to_buffer(&rsi_buf, 0, &rsi_staging, 0, byte_len);
    queue.submit(Some(encoder.finish()));

    let ema_fast_out = map_read_f32(&ctx, &ema_fast_staging, len).await?;
    let ema_slow_out = map_read_f32(&ctx, &ema_slow_staging, len).await?;
    let rsi_out = map_read_f32(&ctx, &rsi_staging, len).await?;

    Ok(GpuBacktestInputs {
        ema_fast: ema_fast_out,
        ema_slow: ema_slow_out,
        rsi: rsi_out,
    })
}

static BATCH_GPU_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn gpu_backtest_inputs_batch(
    series: &[Vec<f32>],
    ema_fast: u32,
    ema_slow: u32,
    rsi_period: u32,
) -> Result<GpuBatchOutputs, TitanError> {
    let _guard = BATCH_GPU_LOCK.lock().await;
    if series.is_empty() {
        return Err(TitanError::Gpu("Empty series".into()));
    }
    if rsi_period == 0 {
        return Err(TitanError::Gpu("rsi_period must be > 0".into()));
    }
    if ema_fast == 0 || ema_slow == 0 {
        return Err(TitanError::Gpu("EMA periods must be > 0".into()));
    }

    let mut closes_flat: Vec<f32> = Vec::new();
    let mut offsets: Vec<u32> = Vec::with_capacity(series.len());
    let mut lengths: Vec<u32> = Vec::with_capacity(series.len());
    let mut cursor: u32 = 0;

    for closes in series {
        if closes.is_empty() {
            offsets.push(cursor);
            lengths.push(0);
            continue;
        }
        offsets.push(cursor);
        let len = closes.len() as u32;
        lengths.push(len);
        closes_flat.extend_from_slice(closes);
        cursor = cursor.saturating_add(len);
    }

    if closes_flat.is_empty() {
        return Err(TitanError::Gpu("Empty closes".into()));
    }

    let total_len = closes_flat.len();
    let byte_len = (total_len * std::mem::size_of::<f32>()) as u64;
    let series_count = series.len() as u32;
    let dispatch_groups = series_count.max(1);

    let ctx = get_gpu_context().await?;
    let device = &ctx.device;
    let queue = &ctx.queue;

    let offsets_byte_len = (offsets.len() * std::mem::size_of::<u32>()) as u64;
    let lengths_byte_len = (lengths.len() * std::mem::size_of::<u32>()) as u64;
    let (
        closes_buf,
        offsets_buf,
        lengths_buf,
        ema_fast_buf,
        ema_slow_buf,
        rsi_buf,
        ema_fast_staging,
        ema_slow_staging,
        rsi_staging,
    ) = {
        let mut pool = ctx.buffer_pool.lock().unwrap_or_else(|e| e.into_inner());
        let closes_buf = ensure_buffer(
            &mut pool.closes,
            device,
            byte_len,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "closes-batch",
        );
        let offsets_buf = ensure_buffer(
            &mut pool.offsets,
            device,
            offsets_byte_len,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "offsets",
        );
        let lengths_buf = ensure_buffer(
            &mut pool.lengths,
            device,
            lengths_byte_len,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "lengths",
        );
        let ema_fast_buf = ensure_buffer(
            &mut pool.ema_fast,
            device,
            byte_len,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            "ema-fast-batch",
        );
        let ema_slow_buf = ensure_buffer(
            &mut pool.ema_slow,
            device,
            byte_len,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            "ema-slow-batch",
        );
        let rsi_buf = ensure_buffer(
            &mut pool.rsi,
            device,
            byte_len,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            "rsi-batch",
        );
        let ema_fast_staging = ensure_buffer(
            &mut pool.ema_fast_staging,
            device,
            byte_len,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            "ema-fast-batch-staging",
        );
        let ema_slow_staging = ensure_buffer(
            &mut pool.ema_slow_staging,
            device,
            byte_len,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            "ema-slow-batch-staging",
        );
        let rsi_staging = ensure_buffer(
            &mut pool.rsi_staging,
            device,
            byte_len,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            "rsi-batch-staging",
        );
        (
            closes_buf,
            offsets_buf,
            lengths_buf,
            ema_fast_buf,
            ema_slow_buf,
            rsi_buf,
            ema_fast_staging,
            ema_slow_staging,
            rsi_staging,
        )
    };

    queue.write_buffer(closes_buf.as_ref(), 0, bytemuck::cast_slice(&closes_flat));
    queue.write_buffer(offsets_buf.as_ref(), 0, bytemuck::cast_slice(&offsets));
    queue.write_buffer(lengths_buf.as_ref(), 0, bytemuck::cast_slice(&lengths));

    let params = IndicatorParams {
        len: series_count,
        ema_fast,
        ema_slow,
        rsi_period,
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indicator-batch-params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("indicator-batch-bg"),
        layout: &ctx.indicator_batch_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: closes_buf.as_ref().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: offsets_buf.as_ref().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: lengths_buf.as_ref().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: ema_fast_buf.as_ref().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: ema_slow_buf.as_ref().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: rsi_buf.as_ref().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: params_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("indicator-batch-enc"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("indicator-batch-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.indicator_batch_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch_groups, 1, 1);
    }
    encoder.copy_buffer_to_buffer(
        ema_fast_buf.as_ref(),
        0,
        ema_fast_staging.as_ref(),
        0,
        byte_len,
    );
    encoder.copy_buffer_to_buffer(
        ema_slow_buf.as_ref(),
        0,
        ema_slow_staging.as_ref(),
        0,
        byte_len,
    );
    encoder.copy_buffer_to_buffer(rsi_buf.as_ref(), 0, rsi_staging.as_ref(), 0, byte_len);
    queue.submit(Some(encoder.finish()));

    let ema_fast_out = map_read_f32(&ctx, ema_fast_staging.as_ref(), total_len).await?;
    let ema_slow_out = map_read_f32(&ctx, ema_slow_staging.as_ref(), total_len).await?;
    let rsi_out = map_read_f32(&ctx, rsi_staging.as_ref(), total_len).await?;

    Ok(GpuBatchOutputs {
        ema_fast: ema_fast_out,
        ema_slow: ema_slow_out,
        rsi: rsi_out,
        offsets,
        lengths,
    })
}

pub async fn gpu_warmup() -> Result<(), TitanError> {
    let _ = get_gpu_context().await?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
//  GPU NEXUS — Multi-Agent Consensus Engine (150,000 Agents)
// ═══════════════════════════════════════════════════════════════════
//
//  Architecture:
//    - 150k agents stored in a single StorageBuffer (32B × 150k = 4.8MB)
//    - Each tick: write MarketState uniform, reset IntentBuffer, dispatch compute
//    - Read back only 8 bytes: atomic buy_weight + sell_weight
//    - Agent state persists across ticks (capital evolves, survival of fittest)
//
// ═══════════════════════════════════════════════════════════════════

pub const NEXUS_AGENT_COUNT: u32 = 150_000;
const NEXUS_WORKGROUP_SIZE: u32 = 64;

/// 32-byte aligned agent for GPU (matches nexus_kernel.wgsl exactly)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NexusAgent {
    pub risk_dna: f32,      // Risk threshold (0.0 → 1.0)
    pub strategy_type: u32, // 0=momentum, 1=mean-rev, 2=breakout
    pub capital_0: f32,     // Position size
    pub capital_1: f32,     // Cash reserves
    pub action_state: u32,  // 0=HOLD, 1=BUY, 2=SELL
    pub _pad: [u32; 3],     // Padding to 32B total
}

// Verify 32-byte alignment at compile time
const _: () = assert!(std::mem::size_of::<NexusAgent>() == 32);

/// Market state uniform — pushed to GPU per tick (32 bytes, aligned)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NexusMarketState {
    pub current_price: f32,
    pub price_velocity: f32,
    pub volatility: f32,
    pub tick_count: u32,
    pub prev_price: f32,
    pub _pad: [u32; 3],
}

/// Raw intent readback (8 bytes, mapped from GPU)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct NexusIntentRaw {
    pub buy_weight: u32,
    pub sell_weight: u32,
}

/// Public result returned from dispatch
#[derive(Debug, Clone)]
pub struct NexusDispatchResult {
    pub buy_weight: u32,
    pub sell_weight: u32,
    pub buy_pct: f64,
    pub sell_pct: f64,
    pub consensus: String,
    pub dispatch_us: u64,
}

struct NexusContext {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    agent_buffer: wgpu::Buffer,
    intent_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    market_buffer: wgpu::Buffer,
    agent_count_buffer: wgpu::Buffer,
}

static NEXUS_CONTEXT: OnceCell<Arc<Mutex<NexusContext>>> = OnceCell::const_new();

async fn get_nexus_context() -> Result<Arc<Mutex<NexusContext>>, TitanError> {
    let ctx = NEXUS_CONTEXT
        .get_or_try_init(|| async {
            let gpu = get_gpu_context().await?;
            let device = &gpu.device;

            // — Shader —
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nexus-kernel"),
                source: wgpu::ShaderSource::Wgsl(include_str!("nexus_kernel.wgsl").into()),
            });

            // — Bind Group Layout —
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("nexus-bgl"),
                    entries: &[
                        // @binding(0) agents — read_write storage
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
                        // @binding(1) market — uniform
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // @binding(2) intent — read_write storage (atomics)
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // @binding(3) agent_count — uniform
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

            // — Pipeline —
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("nexus-pl"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("nexus-pipe"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
            });

            // — Initialize 150k agents with diverse strategies —
            let mut agents = vec![NexusAgent::zeroed(); NEXUS_AGENT_COUNT as usize];
            let mut rng = rand::thread_rng();
            for (i, agent) in agents.iter_mut().enumerate() {
                // Distribution: 40% momentum, 35% mean-reversion, 25% breakout
                agent.strategy_type = if i % 100 < 40 {
                    0
                } else if i % 100 < 75 {
                    1
                } else {
                    2
                };
                // Randomized risk threshold (0.01 → 0.5)
                agent.risk_dna = 0.01 + rng.gen::<f32>() * 0.49;
                // Starting capital (0.5 → 2.0)
                agent.capital_0 = 0.0; // No position initially
                agent.capital_1 = 0.5 + rng.gen::<f32>() * 1.5;
                agent.action_state = 0; // HOLD
                agent._pad = [0; 3];
            }

            let agent_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("nexus-agents"),
                contents: bytemuck::cast_slice(&agents),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            });

            // Intent buffer (8 bytes: 2 × u32 atomics)
            let intent_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nexus-intent"),
                size: 8,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Staging buffer for CPU readback (8 bytes)
            let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nexus-staging"),
                size: 8,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Market state uniform (16 bytes)
            let market_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("nexus-market"),
                size: std::mem::size_of::<NexusMarketState>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Agent count uniform (4 bytes)
            let agent_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("nexus-agent-count"),
                contents: bytemuck::bytes_of(&NEXUS_AGENT_COUNT),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            Ok::<Arc<Mutex<NexusContext>>, TitanError>(Arc::new(Mutex::new(NexusContext {
                pipeline,
                bind_group_layout,
                agent_buffer,
                intent_buffer,
                staging_buffer,
                market_buffer,
                agent_count_buffer,
            })))
        })
        .await?;
    Ok(ctx.clone())
}

/// Stateless dispatch for cross-symbol scoring (AI Scanner).
/// Sets prev_price = current_price so agent evolution is neutralized
/// (price_delta = 0, no capital changes). Prevents cross-symbol contamination
/// when scoring multiple independent symbols in sequence.
pub async fn nexus_dispatch_stateless(
    price: f32,
    velocity: f32,
    volatility: f32,
) -> Result<NexusDispatchResult, TitanError> {
    nexus_dispatch(price, velocity, volatility, 0, price).await
}

/// Dispatch one tick to the GPU Nexus engine.
/// Writes market state, resets intent counters, runs compute, reads back 8 bytes.
/// Structured to drop MutexGuard before any .await (Send-safe).
pub async fn nexus_dispatch(
    price: f32,
    velocity: f32,
    volatility: f32,
    tick_count: u32,
    prev_price: f32,
) -> Result<NexusDispatchResult, TitanError> {
    let start = std::time::Instant::now();

    let gpu = get_gpu_context().await?;
    let nexus = get_nexus_context().await?;

    // Phase 1: Submit GPU work (sync, holding lock), then drop lock before async poll
    let (raw, dispatch_us) = {
        let nctx = nexus
            .lock()
            .map_err(|e| TitanError::Gpu(format!("Nexus lock: {}", e)))?;

        let device = &gpu.device;
        let queue = &gpu.queue;

        // 1. Write MarketState uniform
        let market_state = NexusMarketState {
            current_price: price,
            price_velocity: velocity,
            volatility,
            tick_count,
            prev_price,
            _pad: [0; 3],
        };
        queue.write_buffer(&nctx.market_buffer, 0, bytemuck::bytes_of(&market_state));

        // 2. Reset intent counters to zero
        let zero_intent = [0u32; 2];
        queue.write_buffer(&nctx.intent_buffer, 0, bytemuck::cast_slice(&zero_intent));

        // 3. Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nexus-bg"),
            layout: &nctx.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: nctx.agent_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: nctx.market_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: nctx.intent_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: nctx.agent_count_buffer.as_entire_binding(),
                },
            ],
        });

        // 4. Dispatch compute
        let num_workgroups = NEXUS_AGENT_COUNT.div_ceil(NEXUS_WORKGROUP_SIZE);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nexus-enc"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("nexus-pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&nctx.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(num_workgroups, 1, 1);
        }

        // 5. Copy intent to staging
        encoder.copy_buffer_to_buffer(&nctx.intent_buffer, 0, &nctx.staging_buffer, 0, 8);
        queue.submit(Some(encoder.finish()));

        // 6. Map staging buffer for reading
        let slice = nctx.staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |v| {
            let _ = tx.send(v);
        });

        // Synchronous poll -- still inside lock so staging buffer is stable
        device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|_| TitanError::Gpu("Nexus map channel error".into()))?
            .map_err(|_| TitanError::Gpu("Nexus staging map error".into()))?;

        let data = slice.get_mapped_range();
        let raw: NexusIntentRaw = *bytemuck::from_bytes(&data[..8]);
        drop(data);
        nctx.staging_buffer.unmap();

        (raw, start.elapsed().as_micros() as u64)
        // nctx lock drops here
    };

    // 7. Compute consensus
    let total = (raw.buy_weight as f64) + (raw.sell_weight as f64);
    let (buy_pct, sell_pct) = if total > 0.0 {
        (
            raw.buy_weight as f64 / total * 100.0,
            raw.sell_weight as f64 / total * 100.0,
        )
    } else {
        (50.0, 50.0)
    };

    let consensus = if buy_pct > 70.0 {
        "STRONG_BUY".to_string()
    } else if buy_pct > 55.0 {
        "BUY".to_string()
    } else if sell_pct > 70.0 {
        "STRONG_SELL".to_string()
    } else if sell_pct > 55.0 {
        "SELL".to_string()
    } else {
        "NEUTRAL".to_string()
    };

    Ok(NexusDispatchResult {
        buy_weight: raw.buy_weight,
        sell_weight: raw.sell_weight,
        buy_pct,
        sell_pct,
        consensus,
        dispatch_us,
    })
}
