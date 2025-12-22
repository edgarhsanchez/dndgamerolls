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
var atomic_noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(108)
var atomic_noise_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(109)
var atomic_ramp_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(110)
var atomic_ramp_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(111)
var atomic_mask_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(112)
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

    // Vertical plume factor: stronger near the die center, fades upward.
    let base = clamp(1.10 - y * 0.95, 0.0, 1.0);

    // Radial falloff from die center.
    let r = length(lateral);
    let radial = clamp(1.0 - r * 1.55, 0.0, 1.0);

    // Texture-driven turbulence, stable around each die.
    let uv_base = vec2<f32>(dot(lateral, right), dot(lateral, fwd));
    let uv = fract(uv_base * 0.85 + vec2<f32>(0.0, -fx.time * 0.12 + y * 0.08));
    let fire_n = textureSample(fire_noise_tex, fire_noise_samp, uv).r;
    let fire_m = textureSample(fire_mask_tex, fire_mask_samp, uv * 1.2).r;
    let atomic_n = textureSample(atomic_noise_tex, atomic_noise_samp, uv).r;
    let atomic_m = textureSample(atomic_mask_tex, atomic_mask_samp, uv * 1.2).r;

    let n = mix(fire_n, atomic_n, step(0.5, fx.kind));
    let m = mix(fire_m, atomic_m, step(0.5, fx.kind));

    // Fire vs atomic behavior.
    let is_atomic = step(0.5, fx.kind);

    // Fire: flickery tongues.
    let fire_shape = smoothstep(0.20, 0.98, radial * base + n * 0.55) * smoothstep(0.10, 0.95, m);

    // Atomic: brighter core + pulsing shell.
    let pulse = 0.65 + 0.35 * sin(fx.time * 7.0);
    let atomic_core = smoothstep(0.18, 0.90, radial * base) * (0.6 + 0.6 * pulse) * (0.25 + 0.75 * smoothstep(0.10, 0.95, m));

    let shape = mix(fire_shape, atomic_core, is_atomic);

    // Alpha controls how much of the plume is visible.
    let alpha = clamp(shape * fx.intensity, 0.0, 1.0) * 0.85;

    let ramp_color = mix(
        sample_ramp(fire_ramp_tex, fire_ramp_samp, n),
        sample_ramp(atomic_ramp_tex, atomic_ramp_samp, n),
        is_atomic,
    );
    let emissive = ramp_color * (1.2 + 2.6 * shape) * fx.intensity;

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
