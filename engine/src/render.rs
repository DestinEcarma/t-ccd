use std::mem;
use std::{iter, sync::Arc};

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::circle::{Circle, InstanceRaw, MAX_INSTANCES};
use crate::mesh::{QUAD_INDICES, QUAD_VERTICES, QuadVertex};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    screen_wh: [f32; 2],
    _pad: [f32; 2],
}

pub struct Renderer {
    device: Device,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    queue: Queue,
    pipeline: RenderPipeline,

    quad_vb: Buffer,
    quad_ib: Buffer,

    globals_buffer: Buffer,
    globals_bg: BindGroup,

    instance_buffer: Buffer,
    num_instances: usize,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        PhysicalSize { width, height }: PhysicalSize<u32>,
    ) -> anyhow::Result<Self> {
        let instance = Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("WGPU Device"),
                required_features: Features::empty(),
                required_limits: Limits {
                    max_texture_dimension_2d: 4096,
                    ..Limits::downlevel_defaults()
                },
                memory_hints: MemoryHints::default(),
                trace: Trace::Off,
            })
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: if caps.present_modes.contains(&PresentMode::Fifo) {
                PresentMode::Fifo
            } else {
                caps.present_modes[0]
            },
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let globals = Globals {
            screen_wh: [width as f32, height as f32],
            _pad: [0.0; 2],
        };

        let raw_size = mem::size_of::<Globals>() as BufferAddress;
        let aligned_size = (raw_size + 15) & !15;
        let globals_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Globals UBO"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: aligned_size,
            mapped_at_creation: false,
        });

        queue.write_buffer(&globals_buffer, 0, bytemuck::bytes_of(&globals));

        let globals_bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Globals BGL"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Globals BG"),
            layout: &globals_bgl,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&globals_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[QuadVertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview: None,
            cache: None,
        });

        let quad_vb = DeviceExt::create_buffer_init(
            &device,
            &BufferInitDescriptor {
                label: Some("Quad VB"),
                contents: bytemuck::cast_slice(QUAD_VERTICES),
                usage: BufferUsages::VERTEX,
            },
        );
        let quad_ib = DeviceExt::create_buffer_init(
            &device,
            &BufferInitDescriptor {
                label: Some("Quad IB"),
                contents: bytemuck::cast_slice(QUAD_INDICES),
                usage: BufferUsages::INDEX,
            },
        );

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (MAX_INSTANCES * mem::size_of::<InstanceRaw>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            device,
            surface,
            config,
            queue,
            pipeline,

            quad_vb,
            quad_ib,

            globals_buffer,
            globals_bg,

            instance_buffer,
            num_instances: 0,
        })
    }

    pub fn resize(&mut self, PhysicalSize { width, height }: PhysicalSize<u32>) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        let globals = Globals {
            screen_wh: [width as f32, height as f32],
            _pad: [0.0; 2],
        };

        self.queue
            .write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));
    }

    pub fn upload_instances<C>(&mut self, instances: &[C])
    where
        C: Into<Circle> + Copy,
    {
        let data = instances
            .iter()
            .take(MAX_INSTANCES)
            .map(|c| InstanceRaw::from((*c).into()))
            .collect::<Vec<InstanceRaw>>();

        self.num_instances = data.len();

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&data[..self.num_instances]),
        );
    }

    pub fn render(&self) -> Result<(), SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.set_vertex_buffer(0, self.quad_vb.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.quad_ib.slice(..), IndexFormat::Uint16);
            pass.draw_indexed(0..6, 0, 0..(self.num_instances as u32));
        }

        self.queue.submit(iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}
