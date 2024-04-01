use egui::Modifiers;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::PhysicalKey;
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

fn convert_winit_keycode_to_egui_key(winit_keycode:winit::keyboard::KeyCode) -> Option<egui::Key>{
    match winit_keycode {
        winit::keyboard::KeyCode::Backquote=>None,
        winit::keyboard::KeyCode::Backslash=>Some(egui::Key::Backslash),
        winit::keyboard::KeyCode::BracketLeft=>Some(egui::Key::OpenBracket),
        winit::keyboard::KeyCode::BracketRight=>Some(egui::Key::CloseBracket),
        winit::keyboard::KeyCode::Comma=>Some(egui::Key::Comma),
        winit::keyboard::KeyCode::Digit0=>Some(egui::Key::Num0),
        winit::keyboard::KeyCode::Digit1=>Some(egui::Key::Num1),
        winit::keyboard::KeyCode::Digit2=>Some(egui::Key::Num2),
        winit::keyboard::KeyCode::Digit3=>Some(egui::Key::Num3),
        winit::keyboard::KeyCode::Digit4=>Some(egui::Key::Num4),
        winit::keyboard::KeyCode::Digit5=>Some(egui::Key::Num5),
        winit::keyboard::KeyCode::Digit6=>Some(egui::Key::Num6),
        winit::keyboard::KeyCode::Digit7=>Some(egui::Key::Num7),
        winit::keyboard::KeyCode::Digit8=>Some(egui::Key::Num8),
        winit::keyboard::KeyCode::Digit9=>Some(egui::Key::Num9),
        winit::keyboard::KeyCode::Equal=>Some(egui::Key::Equals),
        winit::keyboard::KeyCode::IntlBackslash=>Some(egui::Key::Backslash),
        winit::keyboard::KeyCode::IntlRo=>None,
        winit::keyboard::KeyCode::IntlYen=>None,
        winit::keyboard::KeyCode::KeyA=>Some(egui::Key::A),
        winit::keyboard::KeyCode::KeyB=>Some(egui::Key::B),
        winit::keyboard::KeyCode::KeyC=>Some(egui::Key::C),
        winit::keyboard::KeyCode::KeyD=>Some(egui::Key::D),
        winit::keyboard::KeyCode::KeyE=>Some(egui::Key::E),
        winit::keyboard::KeyCode::KeyF=>Some(egui::Key::F),
        winit::keyboard::KeyCode::KeyG=>Some(egui::Key::G),
        winit::keyboard::KeyCode::KeyH=>Some(egui::Key::H),
        winit::keyboard::KeyCode::KeyI=>Some(egui::Key::I),
        winit::keyboard::KeyCode::KeyJ=>Some(egui::Key::J),
        winit::keyboard::KeyCode::KeyK=>Some(egui::Key::K),
        winit::keyboard::KeyCode::KeyL=>Some(egui::Key::L),
        winit::keyboard::KeyCode::KeyM=>Some(egui::Key::M),
        winit::keyboard::KeyCode::KeyN=>Some(egui::Key::N),
        winit::keyboard::KeyCode::KeyO=>Some(egui::Key::O),
        winit::keyboard::KeyCode::KeyP=>Some(egui::Key::P),
        winit::keyboard::KeyCode::KeyQ=>Some(egui::Key::Q),
        winit::keyboard::KeyCode::KeyR=>Some(egui::Key::R),
        winit::keyboard::KeyCode::KeyS=>Some(egui::Key::S),
        winit::keyboard::KeyCode::KeyT=>Some(egui::Key::T),
        winit::keyboard::KeyCode::KeyU=>Some(egui::Key::U),
        winit::keyboard::KeyCode::KeyV=>Some(egui::Key::V),
        winit::keyboard::KeyCode::KeyW=>Some(egui::Key::W),
        winit::keyboard::KeyCode::KeyX=>Some(egui::Key::X),
        winit::keyboard::KeyCode::KeyY=>Some(egui::Key::Y),
        winit::keyboard::KeyCode::KeyZ=>Some(egui::Key::Z),
        winit::keyboard::KeyCode::Enter=>Some(egui::Key::Enter),
        winit::keyboard::KeyCode::Backspace=>Some(egui::Key::Backspace),
        winit::keyboard::KeyCode::Tab=>Some(egui::Key::Tab),
        winit::keyboard::KeyCode::ArrowLeft=>Some(egui::Key::ArrowLeft),
        winit::keyboard::KeyCode::ArrowRight=>Some(egui::Key::ArrowRight),
        winit::keyboard::KeyCode::ArrowUp=>Some(egui::Key::ArrowUp),
        winit::keyboard::KeyCode::ArrowDown=>Some(egui::Key::ArrowDown),
        _ => None,
    }
}

struct EguiData{
    mouse_position:egui::Pos2,
    scale:f32,
    egui_events:Vec<egui::Event>,
    sizex:u32,
    sizey:u32,
    ctx:egui::Context,
}

fn update_events(
    event:&Event<()>, 
    ed:&mut EguiData, 
    config:&mut SurfaceConfiguration, 
    device:&Device, 
    surface:&Surface,
    queue:&Queue,
    camera_buffer:&Buffer,
    target:&EventLoopWindowTarget<()>
)->bool{
    if let Event::WindowEvent {
        window_id: _,
        event,
    } = event
    {
        match event {
            WindowEvent::CursorMoved { device_id:_, position }=>{
                ed.mouse_position = egui::pos2(position.x as f32/ed.scale, position.y as f32/ed.scale);
                ed.egui_events.push(egui::Event::PointerMoved(ed.mouse_position));
            },
            WindowEvent::MouseInput { device_id:_device_id, state, button:_button }=>{
                ed.egui_events.push(egui::Event::PointerButton { 
                    pos: ed.mouse_position,
                    button: egui::PointerButton::Primary, 
                    pressed: state.is_pressed(), 
                    modifiers: egui::Modifiers::NONE });

            },
            WindowEvent::KeyboardInput { device_id:_device_id, event, is_synthetic:_is_synthetic } => {
                let mut keycode:Option<egui::Key> = None;
                match event.physical_key{
                    PhysicalKey::Code(code)=>{
                        match convert_winit_keycode_to_egui_key(code){
                            Some(key)=>{
                                ed.egui_events.push(egui::Event::Key { 
                                    key, 
                                    physical_key: Some(key), 
                                    pressed:event.state.is_pressed() , 
                                    repeat:event.repeat, 
                                    modifiers: Modifiers::NONE });
                                keycode = Some(key);
                            },
                            None=>{}
                        }
                        
                    },
                    PhysicalKey::Unidentified(_native_keycode)=>{}
                }
                match &event.text{
                    Some(text) => {
                        let do_event = match keycode{
                            Some(key) => {
                                match key{
                                    egui::Key::Backspace=>false,
                                    egui::Key::Enter=>false,
                                    egui::Key::Tab=>false,
                                    _=>true,
                                }
                            }
                            None => true,
                        };
                        if do_event{
                            ed.egui_events.push(egui::Event::Text(text.to_string()));
                        }
                    },
                    None => {}
                }
                
            },
            WindowEvent::Resized(new_size) => {
                config.width = new_size.width.max(1);
                config.height = new_size.height.max(1);
                surface.configure(&device, &config);
                let view = cgmath::ortho(0.0, config.width as f32, config.height as f32, 0.0, -1.0, 1.0)
                    * cgmath::Matrix4::from_scale(ed.scale)
                    * OPENGL_TO_WGPU_MATRIX;
                let camera_uniform = CameraUniform{view:view.into()};
                queue.write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
            }
            WindowEvent::CloseRequested => target.exit(),
            WindowEvent::RedrawRequested => return true,
            _=>{}
        }
    }
    false
}

fn render_egui(
    full_output:egui::FullOutput, 
    ed:&mut EguiData, 
    queue:&Queue, 
    fonttex:&JTexture, 
    device:&Device, 
    surface:&Surface, 
    render_pipeline:&RenderPipeline,
    camera_bind_group:&BindGroup,
    window:&winit::window::Window
){
    for (_id,t) in &full_output.textures_delta.set{
        match &t.image{
            egui::ImageData::Font(font)=>{
                let dimensions = &font.size;
                if dimensions[0]>MAX_TEXTURE_SIZE as usize || dimensions[1]>MAX_TEXTURE_SIZE as usize{
                    panic!("font texture size larger than max texture size");
                }
                ed.sizex = dimensions[0] as u32;
                ed.sizey = dimensions[1] as u32;
                let mut x = 0;
                let mut y = 0;
                for p in &font.pixels{
                    let v = (p*255.0) as u8;
                    let i = (x*4 + y*MAX_TEXTURE_SIZE*4) as usize;
                    unsafe{
                    PIXELS[i] = v;
                    PIXELS[i+1] = v;
                    PIXELS[i+2] = v;
                    PIXELS[i+3] = v;
                    }
                    x+=1;
                    if x>=ed.sizex{
                        x=0;
                        y+=1;
                    }

                }
                
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &fonttex.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    unsafe {
                        &PIXELS
                    },
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * MAX_TEXTURE_SIZE),
                        rows_per_image: Some(MAX_TEXTURE_SIZE),
                    },
                    fonttex.size,
                );
                
            },
            egui::ImageData::Color(_color)=>{

            },
        }
    }

    let mut vertices:Vec<Vertex> = Vec::new();
    let mut indices:Vec<u16> = Vec::new();
    let mut vertices_id = 0;
    let clipped_primitives = ed.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
    for cp in &clipped_primitives{
        match &cp.primitive{
            egui::epaint::Primitive::Mesh(mesh)=>{
                for v in &mesh.vertices{
                    vertices.push(Vertex { 
                        position: [v.pos.x, v.pos.y], 
                        tex_coords: [v.uv.x*ed.sizex as f32/MAX_TEXTURE_SIZE as f32, v.uv.y*ed.sizey as f32/MAX_TEXTURE_SIZE as f32],
                        color: [
                            (v.color[0] as f32)/256.0, 
                            (v.color[1] as f32)/256.0, 
                            (v.color[2] as f32)/256.0, 
                            (v.color[3] as f32)/256.0
                        ],
                        viewport: [cp.clip_rect.min.x, cp.clip_rect.min.y, cp.clip_rect.max.x, cp.clip_rect.max.y],
                    });
                }
                for i in &mesh.indices{
                    indices.push(*i as u16 + vertices_id);
                }
            },
            egui::epaint::Primitive::Callback(_callback)=>{

            },
        }
        vertices_id = vertices.len() as u16;
    }

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

        rpass.set_bind_group(0, &fonttex.bind_group, &[]);
        rpass.set_bind_group(1, &camera_bind_group, &[]);

        rpass.set_vertex_buffer(0, vertex_buffer.slice(0..(vertices.len()*24)as u64));
        rpass.set_index_buffer(index_buffer.slice(0..(indices.len()*2) as u64), IndexFormat::Uint16);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    frame.present();
    window.request_redraw();
}
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

fn create_window(width:f64, height:f64)->(EventLoop<()>, winit::window::Window, Instance, winit::dpi::PhysicalSize<u32>){
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
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

fn create_egui_data()->EguiData{
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
    EguiData { mouse_position: egui::pos2(0.0, 0.0), scale: 3.0, egui_events: Vec::new(), sizex: 0, sizey: 0, ctx }
}

fn main() {
    env_logger::init();
    let (event_loop, window, instance, size) = create_window(1200.0, 800.0);
    let surface = instance.create_surface(&window).unwrap();
    let (device, queue, mut config) = create_surface(&instance, &surface, size);
    let shader = create_shader(&device, include_str!("shader.wgsl"));

    let mut ed = create_egui_data();
    let view = cgmath::ortho(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0)
            * cgmath::Matrix4::from_scale(ed.scale)
            * OPENGL_TO_WGPU_MATRIX;
    let (camera_bind_group, camera_bindgroup_layout, camera_buffer) = camera_uniform(&device, view);
    let fonttex = create_texture(&device, MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE);
    let render_pipeline = create_render_pipeline(&device, &[&fonttex.bind_group_layout, &camera_bindgroup_layout], &shader, &config);

    let mut text = "".to_owned();
    let mut code = "".to_owned();
    let mut checked = false;
    let mut checked2 = true;

    let window = &window;
    event_loop.run(move |event, target| {
            if update_events(&event, &mut ed, &mut config, &device, &surface, &queue, &camera_buffer, &target){
                let raw_input = egui::RawInput{
                    events:ed.egui_events.clone(),
                    max_texture_side:Some(MAX_TEXTURE_SIZE as usize),
                    screen_rect:Some(egui::Rect{min:egui::pos2(10.0,10.0), max:egui::pos2(200.0,500.0)}),
                    ..Default::default()
                };
                ed.egui_events.clear();
    
                let full_output = ed.ctx.run(raw_input, |ctx| {
                    egui::CentralPanel::default().show(&ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui|{
                            ui.heading("Special heading");
                            ui.text_edit_singleline(&mut text);
                            ui.code_editor(&mut code);
                            ui.checkbox(&mut checked, "Checkbox");
                            ui.checkbox(&mut checked2, "Checkbox2");
                            if ui.button("Click me").clicked() {
                                println!("HERE");
                            }
                        });
                        
                    });
                });
                render_egui(full_output, &mut ed, &queue, &fonttex, &device, &surface, &render_pipeline, &camera_bind_group, &window);
            }
    })
    .unwrap();
}

