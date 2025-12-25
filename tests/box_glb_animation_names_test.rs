use std::collections::HashSet;
use std::path::Path;

#[test]
fn box_glb_contains_required_lid_animations() {
    let path = Path::new("3d/box.glb");
    assert!(
        path.exists(),
        "Expected {} to exist (repo should include the box model)",
        path.display()
    );

    let gltf = gltf::Gltf::open(path)
        .unwrap_or_else(|e| panic!("Failed to parse {} as glTF/GLB: {e}", path.display()));

    let names: HashSet<String> = gltf
        .animations()
        .filter_map(|a| a.name().map(str::to_string))
        .collect();

    assert!(
        names.contains("LidOpening"),
        "Missing required animation 'LidOpening' in {}. Found animations: {:?}",
        path.display(),
        names
    );
    assert!(
        names.contains("LidClosing"),
        "Missing required animation 'LidClosing' in {}. Found animations: {:?}",
        path.display(),
        names
    );

    assert!(
        names.contains("LidIdleOpened"),
        "Missing required animation 'LidIdleOpened' in {}. Found animations: {:?}",
        path.display(),
        names
    );
    assert!(
        names.contains("LidIdleClosed"),
        "Missing required animation 'LidIdleClosed' in {}. Found animations: {:?}",
        path.display(),
        names
    );
}
