
mod jegui;
mod jwgpu;
const MAX_TEXTURE_SIZE:u32 = 4096;

struct MyGame{
    text:String,
    anglex:f32,
    angley:f32,
    origin:cgmath::Vector3<f32>,
    pipeline:jwgpu::JRenderPipeline,
    egui:jegui::Core,
}

fn awake(jwgpu_core:&mut jwgpu::Core)->MyGame{
    let shader = jwgpu::create_shader(&jwgpu_core.device, include_str!("shader.wgsl"));
    let egui = jegui::Core::new();
    let view = cgmath::ortho(0.0, jwgpu_core.size.width as f32, jwgpu_core.size.height as f32, 0.0, -1.0, 1.0)
            * cgmath::Matrix4::from_scale(egui.scale)
            * jwgpu::OPENGL_TO_WGPU_MATRIX;
    let camera = jwgpu::JCamera::new(&jwgpu_core.device, view);
    let fonttex = jwgpu::JTexture::new(&jwgpu_core.device, MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE);
    let render_pipeline = jwgpu::create_render_pipeline(&jwgpu_core.device, &[&fonttex.bind_group_layout, &camera.bind_group_layout], &shader, &jwgpu_core.config);
    let pipeline = jwgpu::JRenderPipeline { texture:fonttex, camera, render_pipeline};

    MyGame{
        text:"".to_owned(),
        anglex:180.0,
        angley:180.0,
        origin:cgmath::Vector3 { x: 160.0, y: 160.0, z: 180.0 },
        egui,
        pipeline,
    }
}

fn update(jwgpu_core:&mut jwgpu::Core, mygame:&mut MyGame){
    mygame.egui.run(jwgpu_core, &mut mygame.pipeline, |ctx|{
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.heading("HelloWorld");
            ui.text_edit_singleline(&mut mygame.text);
            ui.add(egui::Slider::new(&mut mygame.anglex, 0.0..=360.0).text("AngleX"));
            ui.add(egui::Slider::new(&mut mygame.angley, 0.0..=360.0).text("AngleY"));
            ui.add(egui::Slider::new(&mut mygame.origin.x, 0.0..=360.0).text("OriginX"));
            ui.add(egui::Slider::new(&mut mygame.origin.y, 0.0..=360.0).text("OriginY"));
            ui.add(egui::Slider::new(&mut mygame.origin.z, 0.0..=360.0).text("OriginZ"));
        });
    });
}
fn main() {
    jwgpu::run(1200.0, 800.0, awake, update);
}

