const PI: f32 = 3.1415926535897932384626433832795;

struct Camera {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
};
@group(1) @binding(0) 
var<uniform> camera: Camera;
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3f,
    @location(3) tangent: vec3f,
    @location(4) bitangent: vec3f,
    @location(5) scale:f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tangent_position: vec3f,
    @location(2) tangent_light_position: vec3f,
    @location(3) tangent_view_position: vec3f,
    @location(4) world_normal: vec3f,
    @location(5) world_position: vec3f,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    //这里没有乘法线矩阵，因为只有一个模型，且没有非位移变换
    let world_normal = normalize(model.normal);
    let world_tangent = normalize(model.tangent);
    let world_bitangent = normalize(model.bitangent);
    let tangent_matrix = transpose(mat3x3f(
        world_tangent,
        world_bitangent,
        world_normal,
    ));
    //let world_position = vec4f(model.position, 1.0);
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position * model.scale, 1.0);
    out.tangent_position = tangent_matrix * model.position * model.scale;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.tangent_light_position = tangent_matrix * light.position;

    out.world_normal = world_normal;
    out.world_position = model.position * model.scale;
    return out;
}

// 即uniform
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0)@binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

struct Light {
    position: vec3f,
    color: vec3f,
    proj: mat4x4<f32>,
}
@group(2) @binding(0)
var<uniform> light: Light;
@group(2) @binding(1)
var t_shadow: texture_depth_2d;
@group(2) @binding(2)
var sampler_shadow: sampler_comparison;

fn fetch_shadow(homogeneous_coords: vec4f) -> f32 
{
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }
    // compensate for the Y-flip difference between the NDC and texture coordinates
    let flip_correction = vec2f(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    // homogeneous_coords.z * proj_correction

    return textureSampleCompareLevel(t_shadow, sampler_shadow, light_local, homogeneous_coords.z * proj_correction);
}

fn DistributionGGX(N:vec3f, H:vec3f, roughness:f32)->f32
{
    let a      = roughness*roughness;
    let a2     = a*a;
    let NdotH  = max(dot(N, H), 0.0);
    let NdotH2 = NdotH*NdotH;

    let nom   = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    let denom2 = PI * denom * denom;

    return nom / denom2;
}
fn GeometrySchlickGGX(NdotV:f32, roughness:f32) -> f32{
    let r:f32 = (roughness + 1.0);
    let k:f32 = (r*r) / 8.0;

    let nom:f32   = NdotV;
    let denom:f32 = NdotV * (1.0 - k) + k;

    return nom / denom;
}
fn GeometrySmith(N:vec3f, nDotV:f32, nDotL:f32, roughness:f32)->f32{
    let ggx2:f32  = GeometrySchlickGGX(nDotV, roughness);
    let ggx1:f32  = GeometrySchlickGGX(nDotL, roughness);

    return ggx1 * ggx2;
}

fn fresnelSchlick(cosTheta:f32, F0:vec3f)->vec3f{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
} 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture
    let roughness:f32 = 0.5;
    let metallic:f32 = 0.0;
    let F0 = vec3f(0.04);

    let object_color: vec4f = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    let normal = normalize(object_normal.xyz * 2.0 - 1.0);
    //let normal = vec3f(0.0,0.0,1.0);
    //let normal = in.world_normal;

    // 这个模型不能用法线空间，地面UV疑似有问题😅

    let light_dir = normalize(in.tangent_light_position - in.tangent_position);
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);
    //let light_dir = normalize(light.position - in.world_position);
    //let view_dir = normalize(camera.view_pos.xyz - in.world_position);

    let half_dir = normalize(view_dir + light_dir);
    let radianceIn = light.color;
    let nDotV = max(dot(normal, view_dir), 0.0);
    let nDotL = max(dot(normal, light_dir), 0.0);

    //Cook-Torrance BRDF
    let NDF = DistributionGGX(normal, half_dir, roughness);
    let G = GeometrySmith(normal, nDotV, nDotL, roughness);
    let F = fresnelSchlick(max(dot(half_dir, view_dir), 0.0), F0);

    let Ks = F;
    let Kd = (vec3f(1.0,1.0,1.0) - Ks) * (1.0 - metallic);
    let numerator = NDF * G * F;
    let denominator = 4.0 * nDotV * nDotL;
    let specular = numerator / max(denominator, 0.0001);
    let radiance = (Kd * object_color.rgb / PI + specular) * radianceIn.rgb * nDotL;

    let ambient_strength = 0.01;
    let ambient_color = light.color * object_color.rgb* ambient_strength;

    let shadow = fetch_shadow(light.proj * vec4<f32>(in.world_position, 1.0));
/*
    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    let diffuse_color = light.color * object_color.xyz* diffuse_strength;
    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;
*/
    let result = ambient_color + shadow * radiance;

    return vec4<f32>(result, object_color.a);
}
