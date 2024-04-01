struct CameraUniform {
    view: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) viewport: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) viewport: vec4<f32>,
    @location(3) viewport_position:vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    var position = camera.view * vec4<f32>(model.position, 0.0, 1.0);
    out.viewport_position = position.xy/position.w;
    out.position = position;
    out.viewport = vec4<f32>(
        (camera.view * vec4<f32>(model.viewport.xy, 0.0, 1.0)).xy,
        (camera.view * vec4<f32>(model.viewport.zw, 0.0, 1.0)).xy);
    return out;
}
//===================================

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

fn between(x:f32, min:f32, max:f32) -> bool{
    if(min<max){
        return x>min && x<max;
    }else{
        return x>max && x<min;
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos = in.viewport_position;
    var vp = in.viewport;
    if(between(pos.x, vp.x ,vp.z) && between(pos.y, vp.y, vp.w)){
        return textureSample(t_diffuse, s_diffuse, in.tex_coords) * in.color;
    }
    discard;
}