// Vertex shader bindings

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

struct CutoutRegion {
    x: f32,      // X offset (normalized)
    y: f32,       // Y offset (normalized)
    width: f32,   // Width (normalized)
    height: f32,
    b_width: f32,
    b_height: f32,  // Height (normalized)
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
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return (c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y));
};


fn remap(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(cutout.width, cutout.height) + vec2<f32>(cutout.x, cutout.y);
}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {

    let color = textureSample(r_tex_color, r_tex_sampler, remap(tex_coord));

    // Create the green filter effect
    // Amplify the green channel and reduce red/blue

    let avg = (color.r + color.g + color.b) / 3.;
    let filterx = vec3<f32>(0.3339, 0.3731, 0.0);
    let filter_color =  vec3<f32>(0.3039, 0.3431, 0.0);
    let filter_color2 = vec3<f32>(0.0263, 0.1063, 0.0);


    let pixel_coord = vec2<f32>(
        tex_coord.x * cutout.b_width,
        tex_coord.y * cutout.b_height,
    );


    if (tex_coord.x < 0. || tex_coord.x > 1. ||
        tex_coord.y < 0. || tex_coord.y > 1.) {
        // Return black for out-of-bounds pixels
        discard;
    };

    // Calculate edge brightness effect

    let from_center = vec2<f32>(
        pixel_coord.x - floor(pixel_coord.x) - 0.5,
        pixel_coord.y - floor(pixel_coord.y) - 0.5
    );
    let distance = max(abs(from_center.x), abs(from_center.y));

    // Use smoothstep to adjust alpha based on distance from the center

    let filtered = vec3<f32>(avg * 0.5 + 0.25) * filterx;

    let shade = vec4<f32>((filterx), smoothstep(0., 0.9, (1. - distance)));
    
    // Apply the custom color filte


    // Return the filtered color with alpha unchanged


    return mix(vec4<f32>(mix(filter_color2, filter_color, avg)  , 1.), shade,  1. - shade.a);
    
    
}



