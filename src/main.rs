
mod jegui;
mod jwgpu;

struct MyGame{
    text:String,
    anglex:f32,
    angley:f32,
    origin:cgmath::Vector3<f32>,
    egui:jegui::Core,
}

fn awake(jwgpu_core:&mut jwgpu::Core)->MyGame{
    let egui = jegui::Core::new(jwgpu_core, 3.0);
    MyGame{
        text:"".to_owned(),
        anglex:180.0,
        angley:180.0,
        origin:cgmath::Vector3 { x: 160.0, y: 160.0, z: 180.0 },
        egui,
    }
}

fn update(jwgpu_core:&mut jwgpu::Core, mygame:&mut MyGame){
    mygame.egui.run(jwgpu_core, |ctx|{
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

