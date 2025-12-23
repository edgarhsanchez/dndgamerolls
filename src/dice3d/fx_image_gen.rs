use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::path::{Path, PathBuf};

use crate::dice3d::types::{CustomDiceFxSetting, FxCurvePointSetting};

pub struct GeneratedFxImages {
    pub source_path: PathBuf,
    pub noise_path: PathBuf,
    pub ramp_path: PathBuf,
    pub mask_path: PathBuf,

    pub source_image: Image,
    pub noise_image: Image,
    pub ramp_image: Image,
    pub mask_image: Image,
}

fn bevy_image_from_rgba8(width: u32, height: u32, rgba: Vec<u8>) -> Image {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                | bevy::render::render_resource::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);
    image.data = Some(rgba);
    image
}

fn sort_points(points: &mut [FxCurvePointSetting]) {
    points.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
}

fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let omt = 1.0 - u;
    (omt * omt * omt) * p0
        + (3.0 * omt * omt * u) * p1
        + (3.0 * omt * u * u) * p2
        + (u * u * u) * p3
}

fn cubic_bezier_derivative(p0: f32, p1: f32, p2: f32, p3: f32, u: f32) -> f32 {
    let omt = 1.0 - u;
    3.0 * omt * omt * (p1 - p0) + 6.0 * omt * u * (p2 - p1) + 3.0 * u * u * (p3 - p2)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Sample a curve mapping x in [0..1] -> y in [0..1].
fn sample_curve01(points: &[FxCurvePointSetting], t: f32) -> f32 {
    if points.is_empty() {
        return t.clamp(0.0, 1.0);
    }
    if points.len() == 1 {
        return points[0].value.clamp(0.0, 1.0);
    }

    let t = t.clamp(0.0, 1.0);

    // Work on a sorted copy if needed.
    let mut points_sorted: std::borrow::Cow<'_, [FxCurvePointSetting]> =
        std::borrow::Cow::Borrowed(points);
    if !points.windows(2).all(|w| w[0].t <= w[1].t) {
        let mut tmp = points.to_vec();
        sort_points(&mut tmp);
        points_sorted = std::borrow::Cow::Owned(tmp);
    }
    let points = points_sorted.as_ref();

    if t <= points[0].t {
        return points[0].value.clamp(0.0, 1.0);
    }
    if t >= points[points.len() - 1].t {
        return points[points.len() - 1].value.clamp(0.0, 1.0);
    }

    for w in points.windows(2) {
        let a = &w[0];
        let b = &w[1];
        if t >= a.t && t <= b.t {
            let dt = (b.t - a.t).max(0.0001);
            let initial_u = ((t - a.t) / dt).clamp(0.0, 1.0);

            // Resolve control points (Bezier in (t,value) space).
            let mut p1 = a
                .out_handle
                .map(|h| Vec2::new(h[0], h[1]))
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 1.0 / 3.0), a.value));
            let mut p2 = b
                .in_handle
                .map(|h| Vec2::new(h[0], h[1]))
                .unwrap_or(Vec2::new(lerp(a.t, b.t, 2.0 / 3.0), b.value));

            // Clamp handles within domain/range.
            p1.x = p1.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p2.x = p2.x.clamp(a.t.min(b.t), a.t.max(b.t));
            p1.y = p1.y.clamp(0.0, 1.0);
            p2.y = p2.y.clamp(0.0, 1.0);

            // Invert x(u) to find u s.t. x(u) == t.
            let mut u = initial_u;
            for _ in 0..8 {
                let x = cubic_bezier(a.t, p1.x, p2.x, b.t, u);
                let dx = cubic_bezier_derivative(a.t, p1.x, p2.x, b.t, u);
                if dx.abs() < 1e-5 {
                    break;
                }
                u = (u - (x - t) / dx).clamp(0.0, 1.0);
            }

            return cubic_bezier(a.value, p1.y, p2.y, b.value, u).clamp(0.0, 1.0);
        }
    }

    t
}

fn xorshift32(mut x: u32) -> impl FnMut() -> u32 {
    move || {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    }
}

pub fn generate_custom_fx_textures(
    source_image_path: &Path,
    out_dir: &Path,
    cfg: &CustomDiceFxSetting,
) -> Result<GeneratedFxImages, String> {
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("Failed to create fx dir {:?}: {}", out_dir, e))?;

    // Load source image
    let dyn_img = image::open(source_image_path)
        .map_err(|e| format!("Failed to open image {:?}: {}", source_image_path, e))?;
    let src = dyn_img.to_rgba8();

    // UI preview: keep a modest 256x256 copy so large user images don't bloat GPU memory.
    let preview_img =
        image::imageops::resize(&src, 256, 256, image::imageops::FilterType::Triangle);
    let source_image = bevy_image_from_rgba8(256, 256, preview_img.into_raw());

    // Copy source into app data as a canonical PNG so we don't depend on the original file.
    let source_out = out_dir.join("custom_source.png");
    dyn_img
        .save(&source_out)
        .map_err(|e| format!("Failed to save source png {:?}: {}", source_out, e))?;

    // Seed noise from source pixels (deterministic for this image).
    let mut seed: u32 = 0x1234_5678;
    for p in src.pixels().take(2048) {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223)
            ^ (p.0[0] as u32)
            ^ ((p.0[1] as u32) << 8)
            ^ ((p.0[2] as u32) << 16);
    }

    // Noise: 256x256 grayscale
    let noise_w = 256u32;
    let noise_h = 256u32;
    let mut rng = xorshift32(seed.max(1));
    let mut noise_rgba = vec![0u8; (noise_w * noise_h * 4) as usize];
    for y in 0..noise_h {
        for x in 0..noise_w {
            let r = (rng() & 0xFF) as u8;
            let raw = (r as f32) / 255.0;
            let v = sample_curve01(&cfg.curve_points_noise, raw);
            let b = (v * 255.0).round().clamp(0.0, 255.0) as u8;
            let i = ((y * noise_w + x) * 4) as usize;
            noise_rgba[i] = b;
            noise_rgba[i + 1] = b;
            noise_rgba[i + 2] = b;
            noise_rgba[i + 3] = 255;
        }
    }

    // Mask: derived from source luminance (resized to 256x256)
    let mask_img = image::imageops::resize(&src, 256, 256, image::imageops::FilterType::Triangle);
    let mut mask_rgba = vec![0u8; (256 * 256 * 4) as usize];
    for (i, p) in mask_img.pixels().enumerate() {
        let r = p.0[0] as f32 / 255.0;
        let g = p.0[1] as f32 / 255.0;
        let b = p.0[2] as f32 / 255.0;
        let luma = (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 1.0);
        let v = sample_curve01(&cfg.curve_points_mask, luma);
        let out = (v * 255.0).round().clamp(0.0, 255.0) as u8;
        let o = i * 4;
        mask_rgba[o] = out;
        mask_rgba[o + 1] = out;
        mask_rgba[o + 2] = out;
        mask_rgba[o + 3] = 255;
    }

    // Ramp: 256x1 gradient from source palette sorted by luminance
    let mut pixels: Vec<(f32, [u8; 4])> = src
        .pixels()
        .map(|p| {
            let r = p.0[0] as f32 / 255.0;
            let g = p.0[1] as f32 / 255.0;
            let b = p.0[2] as f32 / 255.0;
            let luma = (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 1.0);
            (luma, p.0)
        })
        .collect();
    pixels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut ramp_rgba = vec![0u8; 256 * 4];
    let n = pixels.len().max(1);
    for i in 0..256usize {
        let u = (i as f32) / 255.0;
        let u2 = sample_curve01(&cfg.curve_points_ramp, u);
        let idx = ((u2 * ((n - 1) as f32)).round() as usize).min(n - 1);
        let c = pixels[idx].1;
        ramp_rgba[i * 4] = c[0];
        ramp_rgba[i * 4 + 1] = c[1];
        ramp_rgba[i * 4 + 2] = c[2];
        ramp_rgba[i * 4 + 3] = 255;
    }

    // Save PNGs
    let noise_out = out_dir.join("custom_noise.png");
    let mask_out = out_dir.join("custom_mask.png");
    let ramp_out = out_dir.join("custom_ramp.png");

    image::RgbaImage::from_raw(noise_w, noise_h, noise_rgba.clone())
        .ok_or_else(|| "Failed to build noise image".to_string())?
        .save(&noise_out)
        .map_err(|e| format!("Failed to save noise png {:?}: {}", noise_out, e))?;

    image::RgbaImage::from_raw(256, 256, mask_rgba.clone())
        .ok_or_else(|| "Failed to build mask image".to_string())?
        .save(&mask_out)
        .map_err(|e| format!("Failed to save mask png {:?}: {}", mask_out, e))?;

    image::RgbaImage::from_raw(256, 1, ramp_rgba.clone())
        .ok_or_else(|| "Failed to build ramp image".to_string())?
        .save(&ramp_out)
        .map_err(|e| format!("Failed to save ramp png {:?}: {}", ramp_out, e))?;

    Ok(GeneratedFxImages {
        source_path: source_out,
        noise_path: noise_out,
        ramp_path: ramp_out,
        mask_path: mask_out,
        source_image,
        noise_image: bevy_image_from_rgba8(noise_w, noise_h, noise_rgba),
        ramp_image: bevy_image_from_rgba8(256, 1, ramp_rgba),
        mask_image: bevy_image_from_rgba8(256, 256, mask_rgba),
    })
}
