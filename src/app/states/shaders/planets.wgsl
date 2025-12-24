struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) world_pos: vec3<f32>,
    @location(4) uv: vec2<f32>,
};

struct Uniforms {
    transform: mat4x4<f32>,
    light_origin: vec3<f32>,
    light_color: vec3<f32>,
    ambient_strength: f32,
};

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var planet_texture: texture_2d<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>, 
    @location(1) color: vec4<f32>, 
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.position = uniforms.transform * position;
    result.color = color;
    result.normal = normalize(normal);
    result.world_pos = position.xyz;
    result.uv = uv;

    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(planet_texture, texture_sampler, vertex.uv);
    
    let normal = normalize(vertex.normal);
    let light_direction = (uniforms.light_origin - vertex.world_pos);
    let light_dir = normalize(light_direction);
    
    let ambient = uniforms.ambient_strength;
    let intensity = (pow(1.0 + dot(normal, light_dir), 2.0) + ambient) / (4.0 + ambient);
    
    let lit_color = texture_color.rgb * intensity * uniforms.light_color;
    
    return vec4<f32>(lit_color, texture_color.a);
}