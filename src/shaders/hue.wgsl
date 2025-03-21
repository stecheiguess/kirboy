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

fn hsv_to_rgb(c: vec3<f32>) -> vec3<f32> {
    /* stores all the needed constants into a single 4 float vector, using swizzling to 
    transform to needed values. */
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return (c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y));
};


fn remap(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(cutout.width, cutout.height) + vec2<f32>(cutout.x, cutout.y);
}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {

    // Sample the texture with the displaced coordinates
    let sampled_color = textureSample(r_tex_color, r_tex_sampler, remap(tex_coord));
    
    // ensuring that the darker the color, the more saturated the result would be. 
    // e.g. white -> white, while black -> the most saturated color depending on the time
    let saturation = 1. -  max(max(sampled_color.r, sampled_color.g), sampled_color.b); 
    
    if (tex_coord.x < 0. || tex_coord.x > 1. ||
        tex_coord.y < 0. || tex_coord.y > 1.) {
        // Return black for out-of-bounds pixels
        discard;
    };
    // Calculate a cycling hue based on time
    let cycling_hue = fract(r_locals.time); // Hue cycles over time (0 to 1)

    // Convert the cycling hue, saturation, and value to RGB.
    // value is given 1, to ensure that brightness is the same. 
    let cycling_rgb = hsv_to_rgb(vec3<f32>(cycling_hue, saturation , 1.));


    return vec4<f32>(cycling_rgb, 1.0);

}


