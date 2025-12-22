// Build script for embedding Windows resources (icon, version info)
// This only runs on Windows targets

fn main() {
    // Check required assets
    check_assets();

    // Only compile resources for Windows builds
    // Use CARGO_CFG_TARGET_OS to check the target (not host) platform
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        #[cfg(windows)]
        embed_windows_resources();
    }
}

fn check_assets() {
    println!("cargo:rerun-if-changed=3d/box.glb");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = std::path::Path::new(&manifest_dir).join("3d/box.glb");
    
    // If the file doesn't exist, we might be in a clean checkout or something, 
    // but for this project structure it should be there.
    // We'll panic if it's missing because the app depends on it.
    let (doc, buffers, _images) = gltf::import(&path)
        .unwrap_or_else(|e| panic!("Failed to import {:?} as glTF/GLB: {e}", path));

    let required_animations = ["LidOpening", "LidClosing", "LidIdleClosed", "LidIdleOpened"];
    let found_animations: std::collections::HashSet<_> = doc
        .animations()
        .filter_map(|anim| anim.name().map(|s| s.to_string()))
        .collect();

    for anim in required_animations {
        if !found_animations.contains(anim) {
            panic!(
                "Missing required animation '{}' in {:?}. Found animations: {:?}",
                anim, path, found_animations
            );
        }
    }

    // Validate that the lid animations only contain keys in the intended short range.
    // The spec says the authored frame range is 1..=10. In glTF, key times are seconds,
    // so we infer a likely FPS from common values.
    for anim in required_animations {
        let (min_t, max_t) = animation_time_range_seconds(&doc, &buffers, anim)
            .unwrap_or_else(|| panic!("Animation '{}' exists but has no input keyframe times", anim));

        // Single-key clips are OK (no time range).
        let duration = (max_t - min_t).max(0.0);
        if duration <= 1e-6 {
            continue;
        }

        let fps = infer_common_fps_for_10_frames(duration).unwrap_or_else(|| {
            panic!(
                "Animation '{}' has duration {:.6}s (min={:.6}, max={:.6}) which doesn't match ~10 frames at common FPS values",
                anim, duration, min_t, max_t
            )
        });

        // Frame 1..=10 implies 9 frame intervals.
        let expected = 9.0 / fps;
        let epsilon = 0.0025_f32.max(expected * 0.02);
        if (duration - expected).abs() > epsilon {
            panic!(
                "Animation '{}' has duration {:.6}s, expected ~{:.6}s for frames 1..=10 at {}fps (min={:.6}, max={:.6})",
                anim, duration, expected, fps, min_t, max_t
            );
        }
    }
}

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

            // glTF animation sampler inputs are scalar times in seconds.
            // Read them manually from the referenced buffer.
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
    // Accept a small epsilon to handle floating-point + export rounding.
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

#[cfg(windows)]
fn embed_windows_resources() {
    // Get package info from Cargo environment variables (set by Cargo during build)
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "dndgamerolls".to_string());
    let description =
        std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_else(|_| "DnD Game Rolls".to_string());
    let authors =
        std::env::var("CARGO_PKG_AUTHORS").unwrap_or_else(|_| "Edgar Sanchez".to_string());

    // Parse version for Windows VERSIONINFO (major.minor.patch.0)
    let version_parts: Vec<&str> = version.split('.').collect();
    let major = version_parts.first().unwrap_or(&"0");
    let minor = version_parts.get(1).unwrap_or(&"0");
    let patch = version_parts.get(2).unwrap_or(&"0");

    let mut res = winresource::WindowsResource::new();

    // Try to find rc.exe in Windows SDK paths if not in PATH
    let sdk_paths = [
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64",
        r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.18362.0\x64",
    ];

    for sdk_path in sdk_paths {
        let rc_path = std::path::Path::new(sdk_path).join("rc.exe");
        if rc_path.exists() {
            res.set_toolkit_path(sdk_path);
            println!("cargo:warning=Found Windows SDK at: {}", sdk_path);
            break;
        }
    }

    // Set the application icon - path is relative to the crate root (dndgamerolls/)
    // The icon is in the parent repo's assets folder
    res.set_icon("../assets/icon.ico");

    // Set version information
    res.set("FileVersion", &format!("{}.{}.{}.0", major, minor, patch));
    res.set("ProductVersion", &version);
    res.set("ProductName", "DnD Game Rolls");
    res.set("FileDescription", &description);
    res.set("OriginalFilename", &format!("{}.exe", name));
    res.set("CompanyName", "M2IAB");
    res.set("LegalCopyright", &format!("Copyright © 2024 {}", authors));

    // Compile the resources
    match res.compile() {
        Ok(_) => println!("cargo:warning=Successfully compiled Windows resources with icon"),
        Err(e) => println!("cargo:warning=Failed to compile Windows resources: {}", e),
    }
}
