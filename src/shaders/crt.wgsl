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

fn curve(uv: vec2<f32>) -> vec2<f32> {
        // as we near the edge of our screen apply greater distortion using a cubic function
        let curvature = 4.;
        let scaledUV = (uv - vec2<f32>(0.5, 0.5)) * 2.0;
        let offset = abs(scaledUV.yx) / vec2<f32>(curvature);
        let distortedUV = scaledUV + scaledUV * offset * offset; 
        
        let finalUV = (distortedUV / 2.0) + vec2<f32>(0.5, 0.5);
        
        return finalUV;
}

fn vignette(uv: vec2<f32>,  opacity: f32) -> vec4<f32> {
    let intensity = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y) * 2.;
    let vignette = pow(intensity, opacity);
    return vec4<f32>(vec3<f32>(clamp(vignette, 0.0, 1.0)), 1.0);
}


fn remap(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(cutout.width, cutout.height) + vec2<f32>(cutout.x, cutout.y);
}

fn reverse(uv:vec2<f32>) -> vec2<f32> {
    return (uv -  vec2<f32>(cutout.x, cutout.y)) / vec2<f32>(cutout.width, cutout.height);

}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {

    

    let curved = remap(curve(tex_coord));


    let sampled_color = textureSample(r_tex_color, r_tex_sampler, curved);

    let s_count = 100.;
    let i = (curved.y - r_locals.time * 0.05) * s_count * tau / cutout.height;
    let s = (sin(i) * 0.25 + 0.75);
    let scan = vec4<f32>(vec3<f32>(pow(s, 0.5)), 1.0);
    
    
    


    /*if (tex_coord.x < (0.1) && tex_coord.y < (0.1)) {
        return vec4<f32>(0., 255., 0., 1.);
    }

    if (tex_coord.x < (0.1) && tex_coord.y > (0.9)) {
        return vec4<f32>(255., 0., 0., 1.);
    }

    if (tex_coord.x > (0.9) && tex_coord.y > (0.9)) {
        return vec4<f32>(0., 0., 255., 1.);
    }*/

    if (curved.x < 0.0 || curved.x > 1.0 ||
        curved.y < 0.0 || curved.y > 1.0) {
        // Return black for out-of-bounds pixels
        discard;
    }


    let color = vec4<f32>(sampled_color.r * (cos(i) * 0.25 + 0.75), sampled_color.g * (sin(i) * 0.25 + 0.75), sampled_color.b * (cos(i) * 0.25 + 0.75), sampled_color.a);

    /*let edge_fade = vec4<f32>(smoothstep(0.0, 0.05, curved.x) *
                smoothstep(0.0, 0.05, curved.y) *
                smoothstep(1.0, 0.95, curved.x) *
                smoothstep(1.0, 0.95, curved.y), 1.);
*/

    //let noise_color = vec3<f32>((sin(15.*r_locals.time)*0.5) + 0.5); 
    //let noise_color = vec3<f32>(min(r_locals.time, 1.)); //vec3<f32>(random_vec2(tex_coord.xy * vec2<f32>(r_locals.time % tau + bias)));

    //return vec4<f32>(sampled_color);

    return vec4<f32>(color * scan * vignette(reverse(curved), 0.4)); //* edge_fade);
}
