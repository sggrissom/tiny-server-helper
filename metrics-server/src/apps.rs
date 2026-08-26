use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

/// How many releases of an app's deploy history to report. `bin/deploy` prunes
/// to the last five, so asking for more only ever returns fewer.
const RELEASE_HISTORY: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub name: String,
    pub sha: String,
    pub deployed_at: Option<DateTime<Utc>>,
    pub current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupInfo {
    pub registered: bool,
    pub last_success: Option<DateTime<Utc>>,
    pub age_seconds: Option<u64>,
    pub size_kb: Option<u64>,
}

/// What `backupctl` publishes for readers that are not root. Its private cache
/// under `/var/lib/tiny-server-helper/backup/` is mode 700 and stays that way.
#[derive(Deserialize)]
struct BackupStatusFile {
    last_success: Option<i64>,
    size_kb: Option<u64>,
}

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

/// The last few releases, newest first, with the one `current` points at marked.
///
/// The timestamp comes from the release directory rather than from the name it
/// was given: `bin/deploy` builds that name from the *deploying* machine's
/// clock, in whatever zone that machine is in, while the directory was created
/// here.
pub fn release_history(app_dir: &Path) -> Vec<ReleaseInfo> {
    let Ok(entries) = std::fs::read_dir(app_dir.join("releases")) else {
        return Vec::new();
    };

    let current = std::fs::read_link(app_dir.join("current"))
        .ok()
        .and_then(|target| target.file_name().map(|n| n.to_string_lossy().into_owned()));

    let mut releases: Vec<(String, Option<SystemTime>)> = entries
        .flatten()
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_owned();
            Some((name, entry.metadata().and_then(|m| m.modified()).ok()))
        })
        .collect();

    // Release names begin YYYY-MM-DD_HHMMSS, so lexical order is chronological
    // order even when two deploys share a second.
    releases.sort_by(|a, b| b.0.cmp(&a.0));
    releases.truncate(RELEASE_HISTORY);

    releases
        .into_iter()
        .map(|(name, modified)| ReleaseInfo {
            sha: name.rsplit('_').next().unwrap_or_default().to_owned(),
            current: current.as_deref() == Some(name.as_str()),
            deployed_at: modified.map(DateTime::<Utc>::from),
            name,
        })
        .collect()
}

/// Backup state for one app, from the two places that own it: the filesystem
/// registry (`shared/backup.conf` exists) and the status file `backupctl`
/// writes after a run restic reported as successful.
///
/// Registered with no success is the interesting state, and it is deliberately
/// representable: `registered` true, `last_success` absent.
pub fn backup_info(app_dir: &Path, status_dir: &Path, app: &str) -> BackupInfo {
    let mut info = BackupInfo {
        registered: app_dir.join("shared/backup.conf").is_file(),
        ..Default::default()
    };

    let Ok(raw) = std::fs::read_to_string(status_dir.join(format!("backup-{app}.json"))) else {
        return info;
    };
    let Ok(status) = serde_json::from_str::<BackupStatusFile>(&raw) else {
        return info;
    };

    info.size_kb = status.size_kb;
    if let Some(epoch) = status.last_success {
        if let Some(at) = DateTime::from_timestamp(epoch, 0) {
            info.age_seconds = Some((Utc::now() - at).num_seconds().max(0) as u64);
            info.last_success = Some(at);
        }
    }
    info
}

fn dir_size_bytes(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        // file_type does not follow the link, which is what keeps `current`
        // from counting the release it points at a second time.
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            total += dir_size_bytes(&entry.path());
        } else if let Ok(meta) = entry.metadata() {
            total += meta.len();
        }
    }
    total
}
