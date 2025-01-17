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

 // Speed of the rainbow cycling
    let color = textureSample(r_tex_color, r_tex_sampler, remap(tex_coord));

    // Create the green filter effect
    // Amplify the green channel and reduce red/blue
    let filter_color =  vec3<f32>(0.56, 0.73, 0.14); // Example: purple filter

    // Define the filter strength
    // Full strength

    let s_count = 160.;
    let i = sin(remap(tex_coord).x * s_count * tau / cutout.height);
    let s = (i * 0.25 + 0.75);
    let scan = vec4<f32>(vec3<f32>(pow(s, 0.5)), 1.0);
    

    // Apply the custom color filter
    let filtered = color.rgb * filter_color;


    // Return the filtered color with alpha unchanged
    return vec4<f32>(filtered.rgb, color.a) * scan;

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


