

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

//static mut PIXELS:[u8;(MAX_TEXTURE_SIZE*MAX_TEXTURE_SIZE*4) as usize] = [0; (MAX_TEXTURE_SIZE*MAX_TEXTURE_SIZE*4) as usize];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CameraUniform{
    view:[[f32;4];4],
}

unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}





struct JTexture{
    texture:wgpu::Texture,
    bind_group:wgpu::BindGroup,
    bind_group_layout:wgpu::BindGroupLayout,
    size:wgpu::Extent3d,
    pixels:Vec<u8>,
    width:u32,
    height:u32,
}

impl JTexture{
    fn new(device:&wgpu::Device, width:u32, height:u32)->Self{
        let pixels = vec![0; (width*height*4) as usize];
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
    
        Self{texture, bind_group, bind_group_layout, size, pixels, width, height}
    }

    fn write_texture(&self, queue:&wgpu::Queue){
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            self.size,
        );
    }
}


struct JCamera{
    bind_group:wgpu::BindGroup,
    bind_group_layout:wgpu::BindGroupLayout,
    buffer:wgpu::Buffer,
}

impl JCamera{
    fn new(device:&wgpu::Device, view:cgmath::Matrix4<f32>)->Self{
        let camera_uniform = CameraUniform{view:view.into()};
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
    
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        Self{bind_group, bind_group_layout, buffer}
    }
}


fn create_shader(device:&wgpu::Device, file:&str)->wgpu::ShaderModule{
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(file)),
    })
}

fn create_window(width:f64, height:f64)->(winit::event_loop::EventLoop<()>, winit::window::Window, wgpu::Instance, winit::dpi::PhysicalSize<u32>){
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_position(winit::dpi::Position::Logical(winit::dpi::LogicalPosition{x:25.0, y:25.0}))
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize{width, height}))
        .build(&event_loop)
        .unwrap();
    //GL loads faster
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{ backends:wgpu::Backends::GL, ..Default::default()});
    let size = window.inner_size();
    (event_loop, window, instance, size)
}

fn create_surface(instance:&wgpu::Instance, surface:&wgpu::Surface, size:winit::dpi::PhysicalSize<u32>)->(wgpu::Device, wgpu::Queue, wgpu::SurfaceConfiguration){
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(), 
        compatible_surface: Some(&surface), 
        force_fallback_adapter: false 
    })).unwrap();
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor { label: None, required_features: wgpu::Features::empty(), required_limits: wgpu::Limits::default()}, 
        None)).unwrap();
    let mut config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
    let view_format = config.format.add_srgb_suffix();
    config.view_formats.push(view_format);
    surface.configure(&device, &config);
    (device, queue, config)
}

fn create_render_pipeline(device:&wgpu::Device, bind_group_layouts:&[&wgpu::BindGroupLayout], shader:&wgpu::ShaderModule, config:&wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline{
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    let vertex_buffer_layout = wgpu::VertexBufferLayout {
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

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
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
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
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
    let camera = JCamera::new(&device, view);
    let mut fonttex = JTexture::new(&device, MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE);
    let render_pipeline = create_render_pipeline(&device, &[&fonttex.bind_group_layout, &camera.bind_group_layout], &shader, &config);

    let mut origin = cgmath::vec3(180.0, 170.0, 160.0);
    let mut anglex = 180.0;
    let mut angley = 180.0;
    let mut text = "".to_owned();

    let window = &window;
    event_loop.run(move |event, target| {
        egui.run(&event, &mut config, &device, &surface, &queue, &camera, &target, &mut fonttex, &render_pipeline, &window, |ctx|{
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.heading("HelloWorld");
                ui.text_edit_singleline(&mut text);
                ui.add(egui::Slider::new(&mut anglex, 0.0..=360.0).text("AngleX"));
                ui.add(egui::Slider::new(&mut angley, 0.0..=360.0).text("AngleY"));
                ui.add(egui::Slider::new(&mut origin.x, 0.0..=360.0).text("OriginX"));
                ui.add(egui::Slider::new(&mut origin.y, 0.0..=360.0).text("OriginY"));
                ui.add(egui::Slider::new(&mut origin.z, 0.0..=360.0).text("OriginZ"));
            });
        });
    }).unwrap();
}

