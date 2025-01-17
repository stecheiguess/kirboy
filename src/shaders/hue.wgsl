// Vertex shader bindings

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

struct CutoutRegion {
    x: f32,      // X offset (normalized)
    y: f32,       // Y offset (normalized)
    width: f32,   // Width (normalized)
    height: f32,  // Height (normalized)
};

@group(0) @binding(3) var<uniform> cutout: CutoutRegion;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = fma(position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    out.position = vec4<f32>((position * vec2<f32>(cutout.width, cutout.height)), 0.0, 1.0);
    return out;
}

// Fragment shader bindings

@group(0) @binding(0) var r_tex_color: texture_2d<f32>;
@group(0) @binding(1) var r_tex_sampler: sampler;
struct Locals {
    time: f32
}
@group(0) @binding(2) var<uniform> r_locals: Locals;

const tau = 6.283185307179586476925286766559;

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return (c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y));
};


fn remap(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(cutout.width, cutout.height) + vec2<f32>(cutout.x, cutout.y);
}

fn reverse(uv:vec2<f32>) -> vec2<f32> {
    return (uv -  vec2<f32>(cutout.x, cutout.y)) / vec2<f32>(cutout.width, cutout.height);

}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {

    // Sample the texture with the displaced coordinates
    let sampled_color = textureSample(r_tex_color, r_tex_sampler, remap(tex_coord));
 // Speed of the rainbow cycling
    let value = max(max(sampled_color.r, sampled_color.g), sampled_color.b); // Brightness (value)
    
    // Calculate a cycling hue based on time
    let cycling_hue = fract(r_locals.time); // Hue cycles over time (0 to 1)

    // Convert the cycling hue, saturation, and value to RGB
    let cycling_rgb = hsv_to_rgb(vec3<f32>(cycling_hue, 1. - value, 1.));

    return vec4<f32>(cycling_rgb, 1.0);

    //
    /*let edge_fade = vec4<f32>(smoothstep(0.0, 0.05, curved.x) *
                smoothstep(0.0, 0.05, curved.y) *
                smoothstep(1.0, 0.95, curved.x) *
                smoothstep(1.0, 0.95, curved.y), 1.);
*/

    //let noise_color = vec3<f32>((sin(15.*r_locals.time)*0.5) + 0.5); 
    //let noise_color = vec3<f32>(min(r_locals.time, 1.)); //vec3<f32>(random_vec2(tex_coord.xy * vec2<f32>(r_locals.time % tau + bias)));

    //return vec4<f32>(sampled_color);
}


