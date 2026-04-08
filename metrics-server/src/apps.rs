use std::path::Path;

pub fn scan_apps(apps_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(apps_dir) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_owned());
            }
        }
    }
    names.sort();
    names
}

pub fn disk_usage_kb(app_dir: &Path) -> u64 {
    dir_size_bytes(app_dir) / 1024
}

fn dir_size_bytes(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(meta) = entry.metadata() {
            if meta.is_dir() {
                total += dir_size_bytes(&path);
            } else {
                total += meta.len();
            }
        }
    }
    total
}
