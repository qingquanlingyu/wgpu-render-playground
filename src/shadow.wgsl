struct LightView {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) 
var<uniform> lightview:LightView;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3f,
    @location(3) tangent: vec3f,
    @location(4) bitangent: vec3f,
    @location(5) scale:f32,
};

@vertex
fn vs_bake(model: VertexInput) -> @builtin(position) vec4<f32> {
    return lightview.view_proj * vec4<f32>(model.position * model.scale, 1.0);
}