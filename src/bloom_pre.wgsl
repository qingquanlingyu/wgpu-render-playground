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
  let texel = textureSample(image, samp, vs.uv);
  var brightness:f32 = dot(texel.rgb, vec3f(0.2126, 0.7152, 0.0722));
  if (brightness > 1.0) { // 亮度阈值，可根据需要调整
    return vec4f(100.0, 0.0, 0.0, 1.0);
  } else {// 低亮度部分设为黑色
    return vec4f(0.0, 0.0, 0.0, 1.0);
  }
}