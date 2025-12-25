#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct DiceSurfaceFxParams {
    time: f32,
    started_at: f32,
    duration: f32,
    fire: f32,
    electric: f32,

    fireworks: f32,
    explosion: f32,

    origin_ws: vec3<f32>,
    _pad0: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> fx: DiceSurfaceFxParams;

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var fire_noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102)
var fire_noise_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(103)
var fire_ramp_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(104)
var fire_ramp_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(105)
var fire_mask_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(106)
var fire_mask_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(113)
var electric_noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(114)
var electric_noise_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(115)
var electric_ramp_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(116)
var electric_ramp_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(117)
var electric_mask_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(118)
var electric_mask_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(119)
var atomic_noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(120)
var atomic_noise_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(121)
var atomic_ramp_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(122)
var atomic_ramp_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(123)
var atomic_mask_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(124)
var atomic_mask_samp: sampler;

fn sample_ramp(tex: texture_2d<f32>, samp: sampler, t: f32) -> vec3<f32> {
    let u = clamp(t, 0.0, 1.0);
    // Sample in the middle row so both 1D and 2D ramps work.
    return textureSample(tex, samp, vec2<f32>(u, 0.5)).rgb;
}


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Use world-space, but centered on each die so patterns don't "stick" to world origin.
    let wp = in.world_position.xyz;
    let p = wp - fx.origin_ws;
    let n = normalize(in.world_normal);
    // World-up is hardcoded to +Y so dice rotation never affects the effect.
    let right = vec3<f32>(1.0, 0.0, 0.0);
    let fwd = vec3<f32>(0.0, 0.0, 1.0);

    let local_t = max(0.0, fx.time - fx.started_at);
    let dur = max(0.001, fx.duration);
    let u = clamp(local_t / dur, 0.0, 1.0);
    let fade = pow(1.0 - u, 1.5);

    // Electric: crackle streaks around the surface, driven by noise+mask textures.
    let e = clamp(fx.electric, 0.0, 1.0);
    let e_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 0.65 + vec2<f32>(fx.time * 0.12, -fx.time * 0.08));
    let e_noise = textureSample(electric_noise_tex, electric_noise_samp, e_uv).r;
    let e_mask = textureSample(electric_mask_tex, electric_mask_samp, e_uv * 1.3).r;
    let e_lines = smoothstep(0.55, 0.95, e_noise) * smoothstep(0.25, 0.95, e_mask);
    let e_fres = pow(1.0 - abs(dot(n, normalize(vec3<f32>(0.3, 1.0, 0.2)))), 2.0);
    let e_pulse = 0.75 + 0.25 * sin(fx.time * 16.0 + e_noise * 8.0);
    let e_intensity = e * (0.80 * e_lines + 0.60 * e_fres) * e_pulse;

    // Fire: warm shimmer glow (surface-only; plume is a separate mesh), driven by noise+mask.
    let f = clamp(fx.fire, 0.0, 1.0);
    let f_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 0.45 + vec2<f32>(0.0, -fx.time * 0.10));
    let f_noise = textureSample(fire_noise_tex, fire_noise_samp, f_uv).r;
    let f_mask = textureSample(fire_mask_tex, fire_mask_samp, f_uv * 1.1).r;
    let f_shape = smoothstep(0.15, 0.95, f_noise) * smoothstep(0.10, 0.90, f_mask);
    let f_intensity = f * (0.28 + 0.85 * f_shape);

    // Fireworks: short burst of bright sparkles.
    let fw = clamp(fx.fireworks, 0.0, 1.0);
    let fw_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 1.15 + vec2<f32>(fx.time * 0.25, -fx.time * 0.21));
    let fw_noise = textureSample(atomic_noise_tex, atomic_noise_samp, fw_uv).r;
    let fw_mask = textureSample(atomic_mask_tex, atomic_mask_samp, fw_uv * 1.6).r;
    let fw_sparks = smoothstep(0.90, 0.999, fw_noise) * smoothstep(0.45, 0.98, fw_mask);
    let fw_flicker = 0.55 + 0.45 * sin(fx.time * 30.0 + fw_noise * 14.0);
    let fw_attack = smoothstep(0.00, 0.08, u);
    let fw_intensity = fw * fw_sparks * fw_flicker * fw_attack * fade * 2.0;

    // Explosion: hot flash that quickly decays, plus a subtle expanding ring.
    let ex = clamp(fx.explosion, 0.0, 1.0);
    let ex_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 0.75 + vec2<f32>(fx.time * 0.12, -fx.time * 0.09));
    let ex_noise = textureSample(atomic_noise_tex, atomic_noise_samp, ex_uv).r;
    let ex_mask = textureSample(atomic_mask_tex, atomic_mask_samp, ex_uv * 1.25).r;
    let ex_shape = smoothstep(0.10, 0.98, ex_noise) * smoothstep(0.15, 0.98, ex_mask);
    let ex_flash = pow(fade, 2.0);
    let ex_ring_r = u * 0.85;
    let ex_ring = exp(-pow(abs(length(p) - ex_ring_r) * 5.5, 2.0));
    let ex_intensity = ex * (ex_shape * ex_flash * 3.2 + ex_ring * fade * 1.6);

    let electric_color = sample_ramp(electric_ramp_tex, electric_ramp_samp, e_noise);
    let fire_color = sample_ramp(fire_ramp_tex, fire_ramp_samp, f_noise);
    let fireworks_color = sample_ramp(atomic_ramp_tex, atomic_ramp_samp, fw_noise);
    let explosion_color = sample_ramp(atomic_ramp_tex, atomic_ramp_samp, ex_noise);

    let emissive_rgb =
        electric_color * e_intensity * 4.4 +
        fire_color * f_intensity * 3.6 +
        fireworks_color * fw_intensity * 7.2 +
        explosion_color * ex_intensity * 7.0;

    pbr_input.material.emissive += vec4<f32>(emissive_rgb, 0.0);

    // Keep alpha controlled by the base material.
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
