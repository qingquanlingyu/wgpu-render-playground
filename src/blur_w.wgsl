struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@group(0)
@binding(0)
var image: texture_2d<f32>;
@group(0)
@binding(1)
var samp: sampler;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
  // 双线性采样
  var weight: array<f32, 3> = array<f32, 3>(0.2270270270, 0.3162162162, 0.0702702703);
  var offset: array<f32, 3> = array<f32, 3>(0.0, 1.3846153846, 3.2307692308);

  var texel = textureSample(image, samp, vs.uv) * weight[0];

  let tex_size = textureDimensions(image);
  let offset_base = vec2f(1.0, 0.0) /  vec2f(f32(tex_size.x), f32(tex_size.y));

  for (var i: i32 = 1; i <= 2; i += 1) {
    let uv_offset = offset_base * offset[i];
    texel += textureSample(image, samp, vs.uv + uv_offset) * weight[i];
    texel += textureSample(image, samp, vs.uv - uv_offset) * weight[i];
  }
  return texel;
}