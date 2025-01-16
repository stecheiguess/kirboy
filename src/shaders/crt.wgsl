// Vertex shader bindings

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = fma(position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    out.position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

// Fragment shader bindings

@group(0) @binding(0) var r_tex_color: texture_2d<f32>;
@group(0) @binding(1) var r_tex_sampler: sampler;
struct Locals {
    time: f32,
}
@group(0) @binding(2) var<uniform> r_locals: Locals;

const tau = 6.283185307179586476925286766559;

fn curveRemapUV(uv: vec2<f32>) -> vec2<f32> {
        // as we near the edge of our screen apply greater distortion using a cubic function
        let curvature = 4.;
        let scaledUV = uv * 2.0 - vec2<f32>(1.0, 1.0);
        let offset = abs(scaledUV.yx) / vec2<f32>(curvature,curvature);
        let distortedUV = scaledUV + scaledUV * offset * offset;
        return distortedUV * 0.5 + vec2<f32>(0.5, 0.5);
}

fn scanLineIntensity(uv: f32, resolution: f32, opacity: f32) -> vec4<f32> {
    // Calculate the intensity of the scanline based on UV and resolution
    let intensity = sin(uv * resolution * tau);
    let adjusted_intensity = ((0.5 * intensity) + 0.5) * 0.9 + 0.1;
    // Apply the opacity as a power to the intensity
    let final_intensity = pow(adjusted_intensity, opacity);
    // Return the intensity as an RGBA color (greyscale)
    return vec4<f32>(vec3<f32>(final_intensity), 1.0);
}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {

    let curved = curveRemapUV(tex_coord);

    let sampled_color = textureSample(r_tex_color, r_tex_sampler, curved);

    if (curved.x < 0.0 || curved.x > 1.0 ||
        curved.y < 0.0 || curved.y > 1.0) {
        // Return black for out-of-bounds pixels
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let s_count = 20.;
    let i = sin((curved.y - r_locals.time * 0.1) * s_count * tau);
    let s = (i * 0.5 + 0.5) * 0.9 + 0.1;
    let scan = vec4<f32>(vec3<f32>(pow(s, 0.5)), 1.0);

    //let noise_color = vec3<f32>((sin(15.*r_locals.time)*0.5) + 0.5); 
    //let noise_color = vec3<f32>(min(r_locals.time, 1.)); //vec3<f32>(random_vec2(tex_coord.xy * vec2<f32>(r_locals.time % tau + bias)));

    return vec4<f32>(sampled_color * scan);
}
