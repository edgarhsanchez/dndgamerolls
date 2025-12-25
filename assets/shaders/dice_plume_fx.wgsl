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

struct DicePlumeFxParams {
    time: f32,
    started_at: f32,
    duration: f32,
    intensity: f32,
    kind: f32,
    color: vec4<f32>,

    origin_ws: vec3<f32>,
    _pad0: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> fx: DicePlumeFxParams;

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

@group(#{MATERIAL_BIND_GROUP}) @binding(107)
var electric_noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(108)
var electric_noise_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(109)
var electric_ramp_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(110)
var electric_ramp_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(111)
var electric_mask_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(112)
var electric_mask_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(113)
var atomic_noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(114)
var atomic_noise_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(115)
var atomic_ramp_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(116)
var atomic_ramp_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(117)
var atomic_mask_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(118)
var atomic_mask_samp: sampler;

fn sample_ramp(tex: texture_2d<f32>, samp: sampler, t: f32) -> vec3<f32> {
    let u = clamp(t, 0.0, 1.0);
    return textureSample(tex, samp, vec2<f32>(u, 0.5)).rgb;
}


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Work in world-space but relative to the owning die center.
    let wp = in.world_position.xyz;
    let p = wp - fx.origin_ws;
    // World-up is hardcoded to +Y so dice rotation never affects the plume.
    let y = p.y;
    let lateral = vec3<f32>(p.x, 0.0, p.z);
    let right = vec3<f32>(1.0, 0.0, 0.0);
    let fwd = vec3<f32>(0.0, 0.0, 1.0);

    let local_t = max(0.0, fx.time - fx.started_at);
    let dur = max(0.001, fx.duration);
    let u = clamp(local_t / dur, 0.0, 1.0);
    let fade = pow(1.0 - u, 1.5);

    // Common geometry factors.
    let r = length(lateral);
    let dist = length(p);
    let uv_base = vec2<f32>(dot(lateral, right), dot(lateral, fwd));

    // Keep plumes visually "coming out" of the die by gating to above the die center.
    // This assumes fx.origin_ws is the die's center.
    let above = smoothstep(-0.04, 0.14, y);

    // Kind codes (set from CPU): 0=fire, 1=fireworks, 2=explosion, 3=electric.
    let k = fx.kind;

    var shape: f32 = 0.0;
    var alpha: f32 = 0.0;
    var emissive: vec3<f32> = vec3<f32>(0.0);
    // Hanabi examples rely on HDR-bright colors (often > 1.0) to drive bloom.
    // Our CPU side supplies 0..1 tints; scale here into HDR range.
    let tint = clamp(fx.color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)) * 4.0;

    if (k < 0.5) {
        // Fire: flickery tongues, stronger near base.
        let base = clamp(1.10 - y * 0.95, 0.0, 1.0);
        let radial = clamp(1.0 - r * 1.55, 0.0, 1.0);
        let uv = fract(uv_base * 0.85 + vec2<f32>(0.0, -fx.time * 0.12 + y * 0.08));
        let n = textureSample(fire_noise_tex, fire_noise_samp, uv).r;
        let m = textureSample(fire_mask_tex, fire_mask_samp, uv * 1.2).r;
        shape = smoothstep(0.20, 0.98, radial * base + n * 0.60) * smoothstep(0.12, 0.98, m);
        shape *= above;
        alpha = clamp(shape * fx.intensity, 0.0, 1.0) * 0.85;
        let ramp_color = sample_ramp(fire_ramp_tex, fire_ramp_samp, n);
        emissive = (ramp_color * tint) * (1.4 + 2.8 * shape) * fx.intensity;
    } else if (k < 1.5) {
        // Fireworks: expanding spark burst (more like a disc/fountain than a sphere).
        let ring_r = u * 1.10;
        let dist_fw = length(vec2<f32>(r, y * 0.35));
        let ring = exp(-pow(abs(dist_fw - ring_r) * 6.5, 2.0));
        let uv = fract(uv_base * 1.45 + vec2<f32>(fx.time * 0.28, -fx.time * 0.21));
        let n = textureSample(atomic_noise_tex, atomic_noise_samp, uv).r;
        let m = textureSample(atomic_mask_tex, atomic_mask_samp, uv * 1.7).r;
        let sparks = smoothstep(0.86, 0.999, n) * smoothstep(0.32, 0.98, m);
        shape = ring * sparks * above;
        alpha = clamp(shape * fx.intensity * fade, 0.0, 1.0) * 0.95;
        let ramp_color = sample_ramp(atomic_ramp_tex, atomic_ramp_samp, n);
        emissive = (ramp_color * tint) * (12.0 * shape) * fx.intensity * fade;
    } else if (k < 2.5) {
        // Explosion: bright flash with a fast decaying core.
        let core = exp(-dist * 2.6) * pow(fade, 2.2);
        let uv = fract(uv_base * 0.95 + vec2<f32>(fx.time * 0.12, -fx.time * 0.10));
        let n = textureSample(atomic_noise_tex, atomic_noise_samp, uv).r;
        let m = textureSample(atomic_mask_tex, atomic_mask_samp, uv * 1.3).r;
        let grit = smoothstep(0.10, 0.98, n) * smoothstep(0.15, 0.98, m);
        shape = core * grit;
        alpha = clamp(shape * fx.intensity, 0.0, 1.0) * 0.9;
        let ramp_color = sample_ramp(atomic_ramp_tex, atomic_ramp_samp, n);
        emissive = (ramp_color * tint) * (14.0 * shape) * fx.intensity;
    } else {
        // Electricity: tight crackle halo.
        let halo = clamp(1.0 - r * 2.2, 0.0, 1.0);
        let uv = fract(uv_base * 1.35 + vec2<f32>(fx.time * 0.35, -fx.time * 0.28 + y * 0.10));
        let n = textureSample(electric_noise_tex, electric_noise_samp, uv).r;
        let m = textureSample(electric_mask_tex, electric_mask_samp, uv * 1.4).r;
        let arcs = smoothstep(0.60, 0.995, n) * smoothstep(0.35, 0.98, m);
        let flicker = 0.7 + 0.3 * sin(fx.time * 24.0 + n * 10.0);
        shape = halo * arcs * flicker * above;
        alpha = clamp(shape * fx.intensity, 0.0, 1.0) * 0.9;
        let ramp_color = sample_ramp(electric_ramp_tex, electric_ramp_samp, n);
        emissive = (ramp_color * tint) * (8.0 * shape) * fx.intensity;
    }

    pbr_input.material.emissive += vec4<f32>(emissive, 0.0);

    // Make the plume's geometry invisible unless the effect says otherwise.
    pbr_input.material.base_color = vec4<f32>(0.0, 0.0, 0.0, alpha);
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
