struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0)
@binding(1)
var<uniform> is_edge: u32;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.position = transform * position;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    if (is_edge == 1) {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    }
    return vec4<f32>(0.0, 1.0, 1.0, 1.0);
}