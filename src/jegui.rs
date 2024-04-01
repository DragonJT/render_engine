use wgpu::util::DeviceExt;

pub struct Jegui{
    mouse_position:egui::Pos2,
    pub scale:f32,
    events:Vec<egui::Event>,
    sizex:u32,
    sizey:u32,
    ctx:egui::Context,
}

impl Jegui{

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

    pub fn run(&mut self,
        event:&winit::event::Event<()>,
        config:&mut wgpu::SurfaceConfiguration, 
        device:&wgpu::Device, 
        surface:&wgpu::Surface,
        queue:&wgpu::Queue,
        camera_buffer:&wgpu::Buffer,
        target:&winit::event_loop::EventLoopWindowTarget<()>,
        fonttex:&crate::JTexture, 
        render_pipeline:&wgpu::RenderPipeline,
        camera_bind_group:&wgpu::BindGroup,
        window:&winit::window::Window,
        run_ui: impl FnOnce(&egui::Context)
    ) {
        if self.update(&event, config, &device, &surface, &queue, &camera_buffer, &target){
            let raw_input = egui::RawInput{
                events:self.events.clone(),
                max_texture_side:Some(crate::MAX_TEXTURE_SIZE as usize),
                screen_rect:Some(egui::Rect{min:egui::pos2(10.0,10.0), max:egui::pos2(200.0,500.0)}),
                ..Default::default()
            };
            self.events.clear();

            let full_output = self.ctx.run(raw_input,run_ui);
            self.render(full_output, &queue, &fonttex, &device, &surface, &render_pipeline, &camera_bind_group, &window);
        }
    }
    
    fn update(
        &mut self,
        event:&winit::event::Event<()>, 
        config:&mut wgpu::SurfaceConfiguration, 
        device:&wgpu::Device, 
        surface:&wgpu::Surface,
        queue:&wgpu::Queue,
        camera_buffer:&wgpu::Buffer,
        target:&winit::event_loop::EventLoopWindowTarget<()>
    )->bool{
        if let winit::event::Event::WindowEvent {
            window_id: _,
            event,
        } = event
        {
            match event {
                winit::event::WindowEvent::CursorMoved { device_id:_, position }=>{
                    self.mouse_position = egui::pos2(position.x as f32/self.scale, position.y as f32/self.scale);
                    self.events.push(egui::Event::PointerMoved(self.mouse_position));
                },
                winit::event::WindowEvent::MouseInput { device_id:_device_id, state, button:_button }=>{
                    self.events.push(egui::Event::PointerButton { 
                        pos: self.mouse_position,
                        button: egui::PointerButton::Primary, 
                        pressed: state.is_pressed(), 
                        modifiers: egui::Modifiers::NONE });
    
                },
                winit::event::WindowEvent::KeyboardInput { device_id:_device_id, event, is_synthetic:_is_synthetic } => {
                    let mut keycode:Option<egui::Key> = None;
                    match event.physical_key{
                        winit::keyboard::PhysicalKey::Code(code)=>{
                            match Jegui::convert_winit_keycode_to_egui_key(code){
                                Some(key)=>{
                                    self.events.push(egui::Event::Key { 
                                        key, 
                                        physical_key: Some(key), 
                                        pressed:event.state.is_pressed() , 
                                        repeat:event.repeat, 
                                        modifiers: egui::Modifiers::NONE });
                                    keycode = Some(key);
                                },
                                None=>{}
                            }
                            
                        },
                        winit::keyboard::PhysicalKey::Unidentified(_native_keycode)=>{}
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
                                self.events.push(egui::Event::Text(text.to_string()));
                            }
                        },
                        None => {}
                    }
                    
                },
                winit::event::WindowEvent::Resized(new_size) => {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(&device, &config);
                    let view = cgmath::ortho(0.0, config.width as f32, config.height as f32, 0.0, -1.0, 1.0)
                        * cgmath::Matrix4::from_scale(self.scale)
                        * crate::OPENGL_TO_WGPU_MATRIX;
                    let camera_uniform = crate::CameraUniform{view:view.into()};
                    queue.write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
                }
                winit::event::WindowEvent::CloseRequested => target.exit(),
                winit::event::WindowEvent::RedrawRequested => return true,
                _=>{}
            }
        }
        false
    }

    pub fn new()->Jegui{
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
        Jegui { mouse_position: egui::pos2(0.0, 0.0), scale: 3.0, events: Vec::new(), sizex: 0, sizey: 0, ctx }
    }

    fn render(
        &mut self, 
        full_output:egui::FullOutput, 
        queue:&wgpu::Queue, 
        fonttex:&crate::JTexture, 
        device:&wgpu::Device, 
        surface:&wgpu::Surface, 
        render_pipeline:&wgpu::RenderPipeline,
        camera_bind_group:&wgpu::BindGroup,
        window:&winit::window::Window
    ){
        for (_id,t) in &full_output.textures_delta.set{
            match &t.image{
                egui::ImageData::Font(font)=>{
                    let dimensions = &font.size;
                    if dimensions[0]>crate::MAX_TEXTURE_SIZE as usize || dimensions[1]>crate::MAX_TEXTURE_SIZE as usize{
                        panic!("font texture size larger than max texture size");
                    }
                    self.sizex = dimensions[0] as u32;
                    self.sizey = dimensions[1] as u32;
                    let mut x = 0;
                    let mut y = 0;
                    for p in &font.pixels{
                        let v = (p*255.0) as u8;
                        let i = (x*4 + y*crate::MAX_TEXTURE_SIZE*4) as usize;
                        unsafe{
                            crate::PIXELS[i] = v;
                            crate::PIXELS[i+1] = v;
                            crate::PIXELS[i+2] = v;
                            crate::PIXELS[i+3] = v;
                        }
                        x+=1;
                        if x>=self.sizex{
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
                            &crate::PIXELS
                        },
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * crate::MAX_TEXTURE_SIZE),
                            rows_per_image: Some(crate::MAX_TEXTURE_SIZE),
                        },
                        fonttex.size,
                    );
                    
                },
                egui::ImageData::Color(_color)=>{
    
                },
            }
        }
    
        let mut vertices:Vec<crate::Vertex> = Vec::new();
        let mut indices:Vec<u16> = Vec::new();
        let mut vertices_id = 0;
        let clipped_primitives = self.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        for cp in &clipped_primitives{
            match &cp.primitive{
                egui::epaint::Primitive::Mesh(mesh)=>{
                    for v in &mesh.vertices{
                        vertices.push(crate::Vertex { 
                            position: [v.pos.x, v.pos.y], 
                            tex_coords: [v.uv.x*self.sizex as f32/crate::MAX_TEXTURE_SIZE as f32, v.uv.y*self.sizey as f32/crate::MAX_TEXTURE_SIZE as f32],
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
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
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
            rpass.set_index_buffer(index_buffer.slice(0..(indices.len()*2) as u64), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
    
        queue.submit(Some(encoder.finish()));
        frame.present();
        window.request_redraw();
    }
}