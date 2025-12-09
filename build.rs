// Build script for embedding Windows resources (icon, version info)
// This only runs on Windows targets

fn main() {
    // Only compile resources for Windows builds
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        embed_windows_resources();
    }
}

fn embed_windows_resources() {
    // Get package info from Cargo environment variables (set by Cargo during build)
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "dndgamerolls".to_string());
    let description = std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_else(|_| "DnD Game Rolls".to_string());
    let authors = std::env::var("CARGO_PKG_AUTHORS").unwrap_or_else(|_| "Edgar Sanchez".to_string());

    // Parse version for Windows VERSIONINFO (major.minor.patch.0)
    let version_parts: Vec<&str> = version.split('.').collect();
    let major = version_parts.get(0).unwrap_or(&"0");
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
