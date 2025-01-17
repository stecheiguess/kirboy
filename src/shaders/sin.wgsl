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


fn remap(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(cutout.width, cutout.height) + vec2<f32>(cutout.x, cutout.y);
}

fn reverse(uv:vec2<f32>) -> vec2<f32> {
    return (uv -  vec2<f32>(cutout.x, cutout.y)) / vec2<f32>(cutout.width, cutout.height);

}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {



    let wave_frequency: f32 = 10.0; // Number of waves along the x-axis
    let wave_amplitude: f32 = 0.05; // Amplitude of the wave (displacement)
    let wave_speed: f32 = 1.0; // Speed of the wave animation

    // Offset the y-coordinate based on a sine wave
    let wave_offset = wave_amplitude * sin(tex_coord.x * wave_frequency + r_locals.time * wave_speed);
    let displaced_tex_coord = vec2<f32>(tex_coord.x, tex_coord.y + wave_offset);

    // Sample the texture with the displaced coordinates
    let sampled_color = textureSample(r_tex_color, r_tex_sampler, remap(displaced_tex_coord));

    if (displaced_tex_coord.x < 0. || displaced_tex_coord.x > 1. ||
        displaced_tex_coord.y < 0. || displaced_tex_coord.y > 1.) {
        // Return black for out-of-bounds pixels
        discard;
    };


    // Return the sampled color
    return vec4<f32>(sampled_color);

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


