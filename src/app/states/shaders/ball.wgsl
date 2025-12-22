struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) world_pos: vec3<f32>
};

struct Uniforms {
    transform: mat4x4<f32>,
    light_direction: vec3<f32>,
    light_color: vec3<f32>,
    ambient_strength: f32,
};

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>, @location(1) color: vec4<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.position = uniforms.transform * position;
    result.color = color;
    result.normal = normalize(position.xyz);
    result.world_pos = position.xyz;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(vertex.normal);

    let light_dir1 = normalize(uniforms.light_direction);
    let diff1 = max(dot(normal, light_dir1), 0.0);

    let light_dir2 = normalize(vec3<f32>(-0.3, -0.5, 0.4));
    let diff2 = max(dot(normal, light_dir2), 0.0) * 0.3;

    let ambient = uniforms.ambient_strength;
    // let lighting = ambient + diff1 + diff2;

    let light_dir = normalize(uniforms.light_direction);
    // let intencity = max(dot(normal, light_dir), 0.0);
    let intencity = (pow(1.0 + dot(normal, light_dir), 3) + ambient) / (8.0 + ambient);
    let lighting = intencity;
    let lit_color = vertex.color.rgb * lighting * uniforms.light_color;

    // let main_light = vertex.color.rgb * diff1;
    // let fill_light = vertex.color.rgb * diff2 * vec3<f32>(0.8, 0.9, 1.0);
    // let ambient_light = vertex.color.rgb * ambient;
    // let lit_color = main_light + fill_light + ambient_light;
    // let lit_color = fill_light + ambient_light;

    return vec4<f32>(lit_color, vertex.color.a);
}