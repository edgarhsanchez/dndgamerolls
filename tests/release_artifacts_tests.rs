use std::fs;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_string(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel))
        .unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

fn cargo_version() -> String {
    let cargo_toml = read_repo_string("Cargo.toml");
    for line in cargo_toml.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("version = ") {
            let rest = rest.trim();
            if let Some(v) = rest.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
                return v.to_string();
            }
        }
    }

    panic!("could not determine version from Cargo.toml");
}

fn list_markdown_files() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    let mut stack = vec![root.clone()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Avoid huge folders we don't treat as docs artifacts.
                if path.file_name().and_then(|s| s.to_str()) == Some("target") {
                    continue;
                }
                stack.push(path);
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }

    out
}

fn extract_versions_from_text(text: &str) -> Vec<String> {
    // We only care about version strings used to pin release artifacts.
    // Examples:
    // - .../releases/tag/v0.2.17
    // - .../releases/download/v0.2.17/...
    // - release/v0.2.17
    let mut versions = Vec::new();
    for token in ["releases/tag/v", "releases/download/v", "release/v"] {
        let mut rest = text;
        while let Some(idx) = rest.find(token) {
            rest = &rest[idx + token.len()..];
            let ver: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if ver.split('.').count() == 3 && !ver.is_empty() {
                versions.push(ver);
            }
        }
    }
    versions
}

fn extract_relative_artifact_links(text: &str, exts: &[&str]) -> Vec<String> {
    // Very small parser for markdown inline links to local files.
    // We intentionally avoid pulling in regex deps for tests.
    //
    // Matches both images and links:
    // - ![alt](path/to/file.png)
    // - [text](docs/foo.md)
    let mut out = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("](") {
        // Move to the start of the link path (right after "](").
        rest = &rest[start + 2..];
        let Some(end) = rest.find(')') else {
            break;
        };
        let raw = rest[..end].trim();
        rest = &rest[end + 1..];

        if raw.starts_with("http://") || raw.starts_with("https://") || raw.starts_with("data:") {
            continue;
        }

        let path = raw.split('#').next().unwrap_or("").trim();
        if path.is_empty() {
            continue;
        }

        let lower = path.to_ascii_lowercase();
        if exts.iter().any(|ext| lower.ends_with(&format!(".{ext}"))) {
            out.push(path.to_string());
        }
    }

    out
}

fn normalize_display_path(p: &Path) -> String {
    // Prefer a repo-relative path in assertion messages.
    let root = repo_root();
    if let Ok(stripped) = p.strip_prefix(&root) {
        return stripped.to_string_lossy().replace('\\', "/");
    }
    p.to_string_lossy().replace('\\', "/")
}

fn resolve_relative(from_file: &Path, rel: &str) -> PathBuf {
    let base = from_file.parent().unwrap_or_else(|| Path::new("."));
    base.join(rel)
}

#[test]
fn docs_versions_match_cargo_version() {
    let expected = cargo_version();

    // Only enforce version pinning inside this crate's markdown files.
    // If a doc doesn't contain any pinned versions, it's fine.
    for md_path in list_markdown_files() {
        // Avoid scanning dependency repos in a multi-root workspace.
        if !md_path.starts_with(repo_root()) {
            continue;
        }

        let text = fs::read_to_string(&md_path).unwrap_or_default();
        let versions = extract_versions_from_text(&text);
        for found in versions {
            assert_eq!(
                found, expected,
                "{path} pins version v{found}, but Cargo.toml is {expected}",
                path = md_path.display()
            );
        }
    }
}

#[test]
fn markdown_image_links_exist() {
    let (_expected, missing) = collect_release_artifacts();
    assert!(
        missing.is_empty(),
        "Missing release artifacts:\n{}",
        missing.join("\n")
    );
}

fn collect_release_artifacts() -> (BTreeSet<String>, Vec<String>) {
    // "Release artifacts" are files referenced by our markdown docs that should exist
    // on a given release branch/tag (screenshots, docs, media snippets, etc.).
    let exts = ["png", "jpg", "jpeg", "gif", "mp4", "webm", "md", "txt"];
    let mut expected: BTreeSet<String> = BTreeSet::new();
    let mut missing: Vec<String> = Vec::new();

    for md_path in list_markdown_files() {
        if !md_path.starts_with(repo_root()) {
            continue;
        }
        let text = fs::read_to_string(&md_path).unwrap_or_default();
        let links = extract_relative_artifact_links(&text, &exts);
        for rel in links {
            let resolved = resolve_relative(&md_path, &rel);
            let display = format!(
                "- {} -> {}",
                normalize_display_path(&md_path),
                rel
            );
            expected.insert(display);
            if !resolved.exists() {
                missing.push(format!(
                    "- missing: {} (resolved {})",
                    normalize_display_path(&md_path),
                    normalize_display_path(&resolved)
                ));
            }
        }
    }

    (expected, missing)
}

#[test]
fn list_release_artifacts_and_report_missing() {
    let (expected, missing) = collect_release_artifacts();

    // This prints only if you run with `-- --nocapture`.
    println!("Release artifacts referenced by markdown ({}):", expected.len());
    for item in &expected {
        println!("{item}");
    }

    if !missing.is_empty() {
        panic!(
            "Missing release artifacts ({}):\n{}",
            missing.len(),
            missing.join("\n")
        );
    }
}
