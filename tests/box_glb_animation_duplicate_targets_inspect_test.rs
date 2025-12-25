use std::collections::HashMap;
use std::path::Path;

#[test]
fn inspect_box_glb_duplicate_animation_targets() {
    let path = Path::new("3d/box.glb");
    assert!(path.exists(), "Expected {} to exist", path.display());

    let (doc, _buffers, _images) =
        gltf::import(path).unwrap_or_else(|e| panic!("Failed to import {}: {e}", path.display()));

    // Build a readable node label map.
    let mut node_labels: HashMap<usize, String> = HashMap::new();
    for (i, node) in doc.nodes().enumerate() {
        let label = node
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("(unnamed node #{i})"));
        node_labels.insert(i, label);
    }

    for (anim_index, anim) in doc.animations().enumerate() {
        let name = anim
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("(unnamed animation #{anim_index})"));

        // target key: (node_index, property kind)
        // (Property itself isn't Hash in gltf 1.4.x)
        let mut seen: HashMap<(usize, std::mem::Discriminant<gltf::animation::Property>), usize> =
            HashMap::new();
        let mut duplicates: Vec<(usize, usize, usize, gltf::animation::Property)> = Vec::new();

        for (channel_index, channel) in anim.channels().enumerate() {
            let target = channel.target();
            let node_index = target.node().index();
            let prop = target.property();
            let key = (node_index, std::mem::discriminant(&prop));

            if let Some(first_channel) = seen.get(&key).copied() {
                duplicates.push((first_channel, channel_index, node_index, prop));
            } else {
                seen.insert(key, channel_index);
            }
        }

        if !duplicates.is_empty() {
            println!("Animation #{anim_index} '{name}' has duplicate channel targets:");
            for (first, dup, node_index, prop) in duplicates {
                let label = node_labels
                    .get(&node_index)
                    .cloned()
                    .unwrap_or_else(|| "(unknown node)".to_string());
                println!(
                    "  - channels {first} and {dup} both target node #{node_index} '{label}' property {:?}",
                    prop
                );
            }
        }
    }
}
