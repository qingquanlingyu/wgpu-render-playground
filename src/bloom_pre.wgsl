struct UniformParams {
  img_size: vec2<i32>,
  uv_offset: vec2<i32>,
};

@group(0) @binding(0) var<uniform> params: UniformParams;
@group(0) @binding(1) var from_tex: texture_2d<f32>;
@group(0) @binding(2) var to_tex: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let uv = vec2<i32>(global_id.xy);
  if (uv.x >= params.img_size.x || uv.y >= params.img_size.y) {
    return;
  }

  var weight: array<f32, 5> = array<f32, 5>(0.2, 0.1, 0.10, 0.1, 0.1);
  let uv_max: vec2<i32> = params.img_size - 1;

  var texel = textureLoad(from_tex, uv, 0);
  var brightness:f32 = dot(texel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

  if (brightness > 10.0) { // 亮度阈值，可根据需要调整
    textureStore(to_tex, uv, texel);
  } else {// 低亮度部分设为黑色
    textureStore(to_tex, uv, vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }
}