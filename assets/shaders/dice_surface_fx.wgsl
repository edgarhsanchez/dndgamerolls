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
    fire: f32,
    atomic_fx: f32,
    electric: f32,

    origin_ws: vec3<f32>,
    custom: f32,
    custom_noise: f32,
    custom_mask: f32,
    custom_hue: f32,
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
var custom_noise_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(120)
var custom_noise_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(121)
var custom_ramp_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(122)
var custom_ramp_samp: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(123)
var custom_mask_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(124)
var custom_mask_samp: sampler;

fn hue_rotate(rgb: vec3<f32>, hue01: f32) -> vec3<f32> {
    let a = hue01 * 6.28318530718;
    let c = cos(a);
    let s = sin(a);

    let m00 = 0.213 + c * 0.787 - s * 0.213;
    let m01 = 0.715 - c * 0.715 - s * 0.715;
    let m02 = 0.072 - c * 0.072 + s * 0.928;

    let m10 = 0.213 - c * 0.213 + s * 0.143;
    let m11 = 0.715 + c * 0.285 + s * 0.140;
    let m12 = 0.072 - c * 0.072 - s * 0.283;

    let m20 = 0.213 - c * 0.213 - s * 0.787;
    let m21 = 0.715 - c * 0.715 + s * 0.715;
    let m22 = 0.072 + c * 0.928 + s * 0.072;

    return clamp(
        vec3<f32>(
            m00 * rgb.x + m01 * rgb.y + m02 * rgb.z,
            m10 * rgb.x + m11 * rgb.y + m12 * rgb.z,
            m20 * rgb.x + m21 * rgb.y + m22 * rgb.z,
        ),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
}

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

    // Electric: crackle streaks around the surface, driven by noise+mask textures.
    let e = clamp(fx.electric, 0.0, 1.0);
    let e_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 0.65 + vec2<f32>(fx.time * 0.12, -fx.time * 0.08));
    let e_noise = textureSample(electric_noise_tex, electric_noise_samp, e_uv).r;
    let e_mask = textureSample(electric_mask_tex, electric_mask_samp, e_uv * 1.3).r;
    let e_lines = smoothstep(0.55, 0.95, e_noise) * smoothstep(0.25, 0.95, e_mask);
    let e_fres = pow(1.0 - abs(dot(n, normalize(vec3<f32>(0.3, 1.0, 0.2)))), 2.0);
    let e_intensity = e * (0.75 * e_lines + 0.55 * e_fres);

    // Fire: warm shimmer glow (surface-only; plume is a separate mesh), driven by noise+mask.
    let f = clamp(fx.fire, 0.0, 1.0);
    let f_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 0.45 + vec2<f32>(0.0, -fx.time * 0.10));
    let f_noise = textureSample(fire_noise_tex, fire_noise_samp, f_uv).r;
    let f_mask = textureSample(fire_mask_tex, fire_mask_samp, f_uv * 1.1).r;
    let f_shape = smoothstep(0.15, 0.95, f_noise) * smoothstep(0.10, 0.90, f_mask);
    let f_intensity = f * (0.28 + 0.85 * f_shape);

    // Atomic: intense glow pulse, driven by noise+mask.
    let a = clamp(fx.atomic_fx, 0.0, 1.0);
    let pulse = 0.6 + 0.4 * sin(fx.time * 9.0);
    let a_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 0.40 + vec2<f32>(fx.time * 0.06, -fx.time * 0.04));
    let a_noise = textureSample(atomic_noise_tex, atomic_noise_samp, a_uv).r;
    let a_mask = textureSample(atomic_mask_tex, atomic_mask_samp, a_uv * 1.2).r;
    let a_shape = smoothstep(0.20, 0.95, a_noise) * smoothstep(0.10, 0.95, a_mask);
    let a_intensity = a * (0.55 + 1.05 * pulse) * (0.35 + 0.65 * a_shape);

    // Custom: user-provided textures using the same basic patterning as electric.
    let c = clamp(fx.custom, 0.0, 1.0);
    let c_uv = fract(vec2<f32>(dot(p, right), dot(p, fwd)) * 0.55 + vec2<f32>(fx.time * 0.10, -fx.time * 0.06));
    let c_noise_raw = textureSample(custom_noise_tex, custom_noise_samp, c_uv).r;
    let c_mask_raw = textureSample(custom_mask_tex, custom_mask_samp, c_uv * 1.25).r;

    // Curve-driven shaping over time.
    let cn = clamp(fx.custom_noise, 0.0, 1.0);
    let cm = clamp(fx.custom_mask, 0.0, 1.0);
    let c_noise = smoothstep(mix(0.35, 0.55, cn), mix(0.80, 0.98, cn), c_noise_raw);
    let c_mask = smoothstep(mix(0.10, 0.35, cm), mix(0.80, 0.98, cm), c_mask_raw);

    let c_lines = c_noise * c_mask;
    let c_fres = pow(1.0 - abs(dot(n, normalize(vec3<f32>(0.2, 1.0, 0.15)))), 2.0);
    let c_intensity = c * (0.80 * c_lines + 0.45 * c_fres);

    let electric_color = sample_ramp(electric_ramp_tex, electric_ramp_samp, e_noise);
    let fire_color = sample_ramp(fire_ramp_tex, fire_ramp_samp, f_noise);
    let atomic_color = sample_ramp(atomic_ramp_tex, atomic_ramp_samp, a_noise);
    let custom_color_raw = sample_ramp(custom_ramp_tex, custom_ramp_samp, c_noise_raw);
    let custom_color = hue_rotate(custom_color_raw, clamp(fx.custom_hue, 0.0, 1.0));

    let emissive_rgb =
        electric_color * e_intensity * 4.0 +
        fire_color * f_intensity * 3.2 +
        atomic_color * a_intensity * 5.5 +
        custom_color * c_intensity * 4.2;

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
