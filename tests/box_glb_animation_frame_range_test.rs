use std::path::Path;

fn animation_time_range_seconds(
    doc: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    animation_name: &str,
) -> Option<(f32, f32)> {
    let mut min_time = f32::INFINITY;
    let mut max_time = f32::NEG_INFINITY;

    for anim in doc.animations() {
        if anim.name() != Some(animation_name) {
            continue;
        }

        for channel in anim.channels() {
            let sampler = channel.sampler();
            let input = sampler.input();

            if input.data_type() != gltf::accessor::DataType::F32 {
                continue;
            }
            if input.dimensions() != gltf::accessor::Dimensions::Scalar {
                continue;
            }
            let Some(view) = input.view() else {
                continue;
            };

            let buffer_index = view.buffer().index();
            let data: &[u8] = &buffers[buffer_index].0;
            let stride = view.stride().unwrap_or(4);
            let start = view.offset() + input.offset();
            let count = input.count();

            for i in 0..count {
                let off = start + i * stride;
                if off + 4 > data.len() {
                    break;
                }
                let t = f32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
                min_time = min_time.min(t);
                max_time = max_time.max(t);
            }
        }
    }

    if min_time.is_finite() && max_time.is_finite() {
        Some((min_time, max_time))
    } else {
        None
    }
}

fn infer_common_fps_for_10_frames(duration_seconds: f32) -> Option<f32> {
    // Frames 1..=10 => 9 intervals.
    // Exporters may introduce tiny rounding differences.
    let candidates = [
        23.976_f32, 24.0, 25.0, 29.97_f32, 30.0, 48.0, 50.0, 59.94_f32, 60.0,
    ];

    for fps in candidates {
        let frames = duration_seconds * fps;
        if (frames - 9.0).abs() <= 0.10 {
            return Some(fps);
        }
    }

    None
}

#[test]
fn box_glb_lid_animations_only_span_frames_1_to_10() {
    let path = Path::new("3d/box.glb");
    assert!(
        path.exists(),
        "Expected {} to exist (repo should include the box model)",
        path.display()
    );

    let (doc, buffers, _images) =
        gltf::import(path).unwrap_or_else(|e| panic!("Failed to import {}: {e}", path.display()));

    let required = ["LidOpening", "LidClosing", "LidIdleOpened", "LidIdleClosed"];

    for anim in required {
        let (min_t, max_t) = animation_time_range_seconds(&doc, &buffers, anim)
            .unwrap_or_else(|| panic!("Animation '{anim}' exists but has no input keyframe times"));

        let duration = (max_t - min_t).max(0.0);
        if duration <= 1e-6 {
            // Single-key clip: no span.
            continue;
        }

        let fps = infer_common_fps_for_10_frames(duration).unwrap_or_else(|| {
            panic!(
                "Animation '{anim}' duration is {duration:.6}s (min={min_t:.6}, max={max_t:.6}); \
                 expected ~10 frames (9 intervals) at a common FPS"
            )
        });

        let expected = 9.0 / fps;
        let epsilon = 0.0025_f32.max(expected * 0.02);

        assert!(
            (duration - expected).abs() <= epsilon,
            "Animation '{anim}' duration is {duration:.6}s; expected ~{expected:.6}s for frames 1..=10 at {fps}fps (min={min_t:.6}, max={max_t:.6})"
        );
    }
}
