
use wgpu::*;
use wgpu::util::*;
use futures::executor::block_on;
use std::borrow::Cow;
mod jegui;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
    viewport:[f32; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const MAX_TEXTURE_SIZE:u32 = 4096;

static mut PIXELS:[u8;(MAX_TEXTURE_SIZE*MAX_TEXTURE_SIZE*4) as usize] = [0; (MAX_TEXTURE_SIZE*MAX_TEXTURE_SIZE*4) as usize];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CameraUniform{
    view:[[f32;4];4],
}

unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}





struct JTexture{
    texture:Texture,
    bind_group:BindGroup,
    bind_group_layout:BindGroupLayout,
    size:Extent3d,
}

fn create_texture(device:&Device, width:u32, height:u32)->JTexture{
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(
        &wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("font_texture"),
            view_formats: &[],
        }
    );    
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        }
    );

    JTexture{texture, bind_group, bind_group_layout, size}
}

fn camera_uniform(device:&Device, view:cgmath::Matrix4<f32>)->(BindGroup, BindGroupLayout, Buffer){
    let camera_uniform = CameraUniform{view:view.into()};
    let camera_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("camera_bind_group_layout"),
    });
    
    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }
        ],
        label: Some("camera_bind_group"),
    });
    (camera_bind_group, camera_bind_group_layout, camera_buffer)
}

fn create_shader(device:&Device, file:&str)->ShaderModule{
    device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(file)),
    })
}

fn create_window(width:f64, height:f64)->(winit::event_loop::EventLoop<()>, winit::window::Window, Instance, winit::dpi::PhysicalSize<u32>){
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_position(winit::dpi::Position::Logical(winit::dpi::LogicalPosition{x:25.0, y:25.0}))
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize{width, height}))
        .build(&event_loop)
        .unwrap();
    //GL loads faster
    let instance = Instance::new(InstanceDescriptor{ backends:Backends::GL, ..Default::default()});
    let size = window.inner_size();
    (event_loop, window, instance, size)
}

fn create_surface(instance:&Instance, surface:&Surface, size:winit::dpi::PhysicalSize<u32>)->(Device, Queue, SurfaceConfiguration){
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(), 
        compatible_surface: Some(&surface), 
        force_fallback_adapter: false 
    })).unwrap();
    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor { label: None, required_features: Features::empty(), required_limits: Limits::default()}, 
        None)).unwrap();
    let mut config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
    let view_format = config.format.add_srgb_suffix();
    config.view_formats.push(view_format);
    surface.configure(&device, &config);
    (device, queue, config)
}

fn create_render_pipeline(device:&Device, bind_group_layouts:&[&BindGroupLayout], shader:&ShaderModule, config:&SurfaceConfiguration) -> RenderPipeline{
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    let vertex_buffer_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
        ]
    };

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_buffer_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.view_formats[0],
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    })
}

fn main() {
    env_logger::init();
    let (event_loop, window, instance, size) = create_window(1200.0, 800.0);
    let surface = instance.create_surface(&window).unwrap();
    let (device, queue, mut config) = create_surface(&instance, &surface, size);
    let shader = create_shader(&device, include_str!("shader.wgsl"));

    let mut egui = jegui::Jegui::new();
    let view = cgmath::ortho(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0)
            * cgmath::Matrix4::from_scale(egui.scale)
            * OPENGL_TO_WGPU_MATRIX;
    let (camera_bind_group, camera_bindgroup_layout, camera_buffer) = camera_uniform(&device, view);
    let fonttex = create_texture(&device, MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE);
    let render_pipeline = create_render_pipeline(&device, &[&fonttex.bind_group_layout, &camera_bindgroup_layout], &shader, &config);

    let mut text = "".to_owned();
    let mut code = "".to_owned();
    let mut checked = false;

    let window = &window;
    event_loop.run(move |event, target| {
        egui.run(&event, &mut config, &device, &surface, &queue, &camera_buffer, &target, &fonttex, &render_pipeline, &camera_bind_group, &window, |ctx|{
            egui::CentralPanel::default().show(&ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui|{
                    ui.heading("Special heading");
                    ui.text_edit_singleline(&mut text);
                    ui.code_editor(&mut code);
                    ui.checkbox(&mut checked, "Checkbox");
                    if ui.button("Click me").clicked() {
                        println!("HERE");
                    }
                });
            });
        });
    }).unwrap();
}

