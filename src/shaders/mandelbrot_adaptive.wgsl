struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
};

struct Uniforms {
    zoom: f32,              // Relative scale
    aspect: f32,
    offset: vec2<f32>,
    iter_count: u32,
};

// BINDING 0: The Uniforms
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// BINDING 1: The Reference Orbit calculated by Rug
@group(0) @binding(1)
var<storage, read> reference_orbit: array<vec2<f32>>;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.coord = model.position.xy;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.coord;
    uv.x = uv.x * uniforms.aspect;

    // Delta C calculation:
    // We add the offset (Camera - Reference) to the pixel coordinate
    let delta_c = (uv + uniforms.offset) / uniforms.zoom;

    var dz = vec2<f32>(0.0, 0.0);
    let max_iter = uniforms.iter_count;
    var final_iter = 0u;
    var escaped = false;


    // --- Perturbation Loop ---
    for (var i = 0u; i < max_iter; i = i + 1u) {
        let z_ref = reference_orbit[i];

        // Perturbation Math
        // dz = 2*Z*dz + dz^2 + dc
        let term1_x = 2.0 * (z_ref.x * dz.x - z_ref.y * dz.y);
        let term1_y = 2.0 * (z_ref.x * dz.y + z_ref.y * dz.x);
        let term2_x = dz.x * dz.x - dz.y * dz.y;
        let term2_y = 2.0 * dz.x * dz.y;

        dz.x = term1_x + term2_x + delta_c.x;
        dz.y = term1_y + term2_y + delta_c.y;

        // Check absolute escape
        let z_x = z_ref.x + dz.x;
        let z_y = z_ref.y + dz.y;
        
        // Escape Threshold 4.0
        if (z_x * z_x + z_y * z_y > 4.0) {
            final_iter = i;
            escaped = true;
            break;
        }
    }

    if (escaped) {
        // --- Smooth Sine Coloring ---
        // This palette depends on the absolute iteration count, 
        // so it won't flicker when the reference orbit length changes.
        let f_iter = f32(final_iter);
        let freq = 0.1; 
        
        let r = 0.5 + 0.5 * sin(freq * f_iter + 0.0);
        let g = 0.5 + 0.5 * sin(freq * f_iter + 2.09); // +120 deg
        let b = 0.5 + 0.5 * sin(freq * f_iter + 4.18); // +240 deg

        return vec4<f32>(r, g, b, 1.0);
    } else {
        // Inside the set (Black)
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}