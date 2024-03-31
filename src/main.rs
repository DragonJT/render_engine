use winit::{
    event::*,
    event_loop::EventLoop,
    window::WindowBuilder,
};
use wgpu::*;
use wgpu::util::*;
use futures::executor::block_on;
use std::borrow::Cow;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

fn main() {
    let mut vertices:Vec<Vertex> = Vec::new();
    let mut indices:Vec<u16> = Vec::new();

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_position(winit::dpi::Position::Logical(winit::dpi::LogicalPosition{x:25.0, y:25.0}))
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize{width:1000.0, height:800.0}))
        .build(&event_loop)
        .unwrap();
    let size = window.inner_size();
    //GL loads faster
    let instance = Instance::new(InstanceDescriptor{ backends:Backends::GL, ..Default::default()});
    let surface = instance.create_surface(&window).unwrap();
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

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    //let swapchain_capabilities = surface.get_capabilities(&adapter);
    //let swapchain_format = swapchain_capabilities.formats[0];

    
    let ctx = egui::Context::default();
    ctx.set_pixels_per_point(7.5);
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "RedditMono".to_owned(),
        egui::FontData::from_static(include_bytes!("RedditMono-Medium.ttf")).tweak(
            egui::FontTweak {
                scale: 0.95, 
                ..Default::default()
            },
        ),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "RedditMono".to_owned());
    ctx.set_fonts(fonts);

    let window = &window;
    event_loop.run(move |event, target| {
        let _ = (&instance, &adapter, &shader);

        if let Event::WindowEvent {
            window_id: _,
            event,
        } = event
        {

            match event {
                WindowEvent::Resized(new_size) => {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(&device, &config);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    
                    vertices.clear();
                    indices.clear();

                    let raw_input = egui::RawInput{..Default::default()};
                    let full_output = ctx.run(raw_input, |ctx| {
                        egui::CentralPanel::default().show(&ctx, |ui| {
                            ui.label("Hello world!");
                            if ui.button("Click me").clicked() {
                            }
                            if ui.button("Click me").clicked() {
                            }
                        });
                    });
                    let clipped_primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                    for cp in &clipped_primitives{
                        match &cp.primitive{
                            egui::epaint::Primitive::Mesh(mesh)=>{
                                for v in &mesh.vertices{
                                    vertices.push(Vertex { 
                                        position: [v.pos.x/250.0, v.pos.y/250.0, 0.0], 
                                        tex_coords: [v.uv.x, v.uv.y],
                                        color: [
                                            (v.color[0] as f32)/256.0, 
                                            (v.color[1] as f32)/256.0, 
                                            (v.color[2] as f32)/256.0, 
                                            (v.color[3] as f32)/256.0
                                        ] 
                                    });
                                }
                                for i in &mesh.indices{
                                    indices.push(*i as u16);
                                }
                                
                            },
                            egui::epaint::Primitive::Callback(_callback)=>{
            
                            },
                        }
                    }

                    
                    let set_textures = full_output.textures_delta.set;
                    for (_id,t) in &set_textures{
                        match &t.image{
                            egui::ImageData::Font(font)=>{
                                let mut pixels:Vec<u8> = Vec::new();
                                for p in &font.pixels{
                                    let v = (p*255.0) as u8;
                                    pixels.push(v);
                                    pixels.push(v);
                                    pixels.push(v);
                                    pixels.push(v);
                                }
                                let dimensions = &font.size;
                                let texture_size = wgpu::Extent3d {
                                    width: dimensions[0] as u32,
                                    height: dimensions[1] as u32,
                                    depth_or_array_layers: 1,
                                };
                                let font_texture = device.create_texture(
                                    &wgpu::TextureDescriptor {
                                        size: texture_size,
                                        mip_level_count: 1,
                                        sample_count: 1,
                                        dimension: wgpu::TextureDimension::D2,
                                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                                        label: Some("font_texture"),
                                        view_formats: &[],
                                    }
                                );
                                queue.write_texture(
                                    wgpu::ImageCopyTexture {
                                        texture: &font_texture,
                                        mip_level: 0,
                                        origin: wgpu::Origin3d::ZERO,
                                        aspect: wgpu::TextureAspect::All,
                                    },
                                    &pixels,
                                    wgpu::ImageDataLayout {
                                        offset: 0,
                                        bytes_per_row: Some(4 * dimensions[0] as u32),
                                        rows_per_image: Some(dimensions[1] as u32),
                                    },
                                    texture_size,
                                );
                                let diffuse_texture_view = font_texture.create_view(&wgpu::TextureViewDescriptor::default());
                                let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                                    mag_filter: wgpu::FilterMode::Linear,
                                    min_filter: wgpu::FilterMode::Nearest,
                                    mipmap_filter: wgpu::FilterMode::Nearest,
                                    ..Default::default()
                                });
                                let texture_bind_group_layout =
                                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                                let font_bind_group = device.create_bind_group(
                                    &wgpu::BindGroupDescriptor {
                                        layout: &texture_bind_group_layout,
                                        entries: &[
                                            wgpu::BindGroupEntry {
                                                binding: 0,
                                                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                                            },
                                            wgpu::BindGroupEntry {
                                                binding: 1,
                                                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                                            }
                                        ],
                                        label: Some("diffuse_bind_group"),
                                    }
                                );

                                let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                                    label: None,
                                    bind_group_layouts: &[&texture_bind_group_layout],
                                    push_constant_ranges: &[],
                                });
                            
                                let vertex_buffer_layout = VertexBufferLayout {
                                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                                    step_mode: wgpu::VertexStepMode::Vertex,
                                    attributes: &[
                                        wgpu::VertexAttribute {
                                            offset: 0,
                                            shader_location: 0,
                                            format: wgpu::VertexFormat::Float32x3,
                                        },
                                        wgpu::VertexAttribute {
                                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                            shader_location: 1,
                                            format: wgpu::VertexFormat::Float32x2,
                                        },
                                        wgpu::VertexAttribute {
                                            offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                                            shader_location: 2,
                                            format: wgpu::VertexFormat::Float32x4,
                                        }
                                    ]
                                };
                            
                                let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
                                });
                            

                                let vertex_buffer = device.create_buffer_init(
                                    &BufferInitDescriptor {
                                        label: Some("Vertex Buffer"),
                                        contents: bytemuck::cast_slice(&vertices),
                                        usage: BufferUsages::VERTEX,
                                    }
                                );
                            
                                let index_buffer = device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Index Buffer"),
                                        contents: bytemuck::cast_slice(&indices),
                                        usage: wgpu::BufferUsages::INDEX,
                                    }
                                );
            
                                let frame = surface
                                    .get_current_texture()
                                    .expect("Failed to acquire next swap chain texture");
                                let view = frame
                                    .texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());
                                let mut encoder =
                                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                        label: None,
                                    });
                                {
                                    let mut rpass =
                                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: None,
                                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                                view: &view,
                                                resolve_target: None,
                                                ops: wgpu::Operations {
                                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                                    store: wgpu::StoreOp::Store,
                                                },
                                            })],
                                            depth_stencil_attachment: None,
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                        });
                                    rpass.set_pipeline(&render_pipeline);
                                    rpass.set_bind_group(0, &font_bind_group, &[]); // NEW!
                                    rpass.set_vertex_buffer(0, vertex_buffer.slice(0..(vertices.len()*24)as u64));
                                    rpass.set_index_buffer(index_buffer.slice(0..(indices.len()*2) as u64), IndexFormat::Uint16);
                                    rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                                }
            
                                queue.submit(Some(encoder.finish()));
                                frame.present();
                                
                            },
                            egui::ImageData::Color(_color)=>{

                            },
                        }
                    }
                }
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            };
        }
    })
    .unwrap();
}

