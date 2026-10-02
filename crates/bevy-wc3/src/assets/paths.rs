use std::path::{Component, Path};

/// Resolve beside the model first, then from the Bevy asset root.
pub(super) fn texture_paths(model_path: &Path, name: &str) -> Vec<String> {
    let normalized = name.replace('\\', "/");
    if normalized.is_empty() {
        return Vec::new();
    }
    let path = Path::new(&normalized);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Vec::new();
    }
    let local = model_path.parent().unwrap_or(Path::new("")).join(path);
    let local = local.to_string_lossy().replace('\\', "/");
    if local == normalized {
        vec![normalized]
    } else {
        vec![local, normalized]
    }
}

/// Model lookup uses the same locations as textures. Try a real MDL before
/// falling back to the MDX commonly shipped for a Warcraft `.mdl` reference.
pub(super) fn model_paths(model_path: &Path, name: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    for path in texture_paths(model_path, name) {
        let extension = Path::new(&path)
            .extension()
            .and_then(|value| value.to_str());
        if extension.is_some_and(|value| value.eq_ignore_ascii_case("mdl")) {
            let mdx = Path::new(&path)
                .with_extension("mdx")
                .to_string_lossy()
                .into_owned();
            candidates.push(path);
            candidates.push(mdx);
        } else if extension.is_some_and(|value| value.eq_ignore_ascii_case("mdx")) {
            candidates.push(path);
        }
    }
    candidates
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
