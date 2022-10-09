use std::num::NonZeroU32;

use wgpu::util::{DeviceExt, BufferInitDescriptor};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, ShaderStages, BufferUsages, BindGroupEntry, include_wgsl};
use winit::event::{WindowEvent, Event};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{WindowBuilder, Window};

struct Graphics {
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    surface: Surface,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Material {
    colour: [f32; 4],
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut graphics = pollster::block_on(setup(&window));

    let materials = vec![[1.0, 1.0, 0.0, 1.0], [0.3; 4], [1.0; 4]];

    let buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Materials Buffer"),
        contents: bytemuck::cast_slice(&materials),
        usage: BufferUsages::STORAGE,
    });

    let layout = graphics.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: NonZeroU32::new(3),
            }
        ]
    });

    let bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind group"),
        layout: &layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }]
    });

    let shader = graphics.device.create_shader_module(include_wgsl!("shader.wgsl"));
    let pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[
            &layout,
        ],
        push_constant_ranges: &[],
    });


    let pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: graphics.config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                graphics.config.width = size.width;
                graphics.config.height = size.height;
                graphics.surface.configure(&graphics.device, &graphics.config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let frame = graphics.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.draw(0..3, 0..1);
                }

                graphics.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    })
}

async fn setup(window: &Window) -> Graphics {
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }).await.unwrap();

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: 
                wgpu::Features::TEXTURE_BINDING_ARRAY
                | wgpu::Features::BUFFER_BINDING_ARRAY 
                | wgpu::Features::PUSH_CONSTANTS 
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING 
                | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
            limits: adapter.limits(),
            label: None,
        }, None
    ).await.unwrap();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        // Fifo is essentially vsync
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto, 
    };
    surface.configure(&device, &config);

    Graphics {
        config,
        device,
        queue,
        surface,
    }
}
