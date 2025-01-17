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

@group(0) @binding(0) var tex_color: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
struct Locals {
    time: f32
}
@group(0) @binding(2) var<uniform> r_locals: Locals;

const tau = 6.283185307179586476925286766559;

fn remap(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(cutout.width, cutout.height) + vec2<f32>(cutout.x, cutout.y);
}

fn avg(vec: vec2<f32>) -> f32{
    // Compute the average of the components
    let avg = (vec.x + vec.y) / 3.0;

    // Create a new vector where each component is the average
    return avg;
}


@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {

    let sampled = textureSample(tex_color, tex_sampler, remap(vec2<f32>(tex_coord.x,tex_coord.y)));

    let red_sampled = textureSample(tex_color, tex_sampler, remap(vec2<f32>(tex_coord.x - 0.01,tex_coord.y)));
    let blue_sampled = textureSample(tex_color, tex_sampler, remap(vec2<f32>(tex_coord.x + 0.01,tex_coord.y)));


    if (tex_coord.y < 0. || tex_coord.x > 1.) {
        discard;
    }

    let blue = vec4<f32>(blue_sampled.rg,1., 1.-avg(blue_sampled.gb));
    let red = vec4<f32>(1., red_sampled.gb, 1.-avg(blue_sampled.gb));

    return red;
}


/*@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    // Sample the texture color at the original UV
    let color = textureSample(tex_color, tex_sampler, remap(tex_coord));

    // Separate brightness (luminance) using standard grayscale conversion
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));

    // Create red channel (shifted right)
    let red_shift_uv = tex_coord + vec2<f32>(0.01, 0.0); // Shift right
    let red_color = textureSample(tex_color, tex_sampler, remap(red_shift_uv)).rgb;
    let red_channel = vec4<f32>(vec3<f32>(luminance, 0.0, 0.0), red_color.r);

    // Create blue channel (shifted left)
    let blue_shift_uv = tex_coord - vec2<f32>(0.01, 0.0); // Shift left
    let blue_color = textureSample(tex_color, tex_sampler, remap(blue_shift_uv)).rgb;
    let blue_channel = vec4<f32>(vec3<f32>(0.0, 0.0, luminance), blue_color.b);

    // Handle transparency for white pixels (make them transparent)
    let red_alpha = step(0.001, red_channel.r); // Keep alpha for red pixels
    let blue_alpha = step(0.001, blue_channel.b); // Keep alpha for blue pixels
    let red_transparency = vec4<f32>(red_channel.rgb, red_alpha * red_channel.a);
    let blue_transparency = vec4<f32>(blue_channel.rgb, blue_alpha * blue_channel.a);

    // Combine red and blue channels
    let combined = red_transparency + blue_transparency;

    // Intersection of red and blue becomes black
    let intersection = step(0.001, red_channel.a) * step(0.001, blue_channel.a);
    if (intersection > 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black where red and blue overlap
    }

    return combined;
}
*/