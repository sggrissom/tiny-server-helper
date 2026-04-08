use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::metrics::TrafficWindow;

#[derive(Deserialize)]
struct CaddyLine {
    ts: f64,
    status: u16,
}

struct LogState {
    reader: BufReader<File>,
    inode: u64,
    entries: VecDeque<(f64, u16)>,
}

pub struct LogReader {
    states: HashMap<String, LogState>,
    log_dir: PathBuf,
    window_seconds: u64,
}

impl LogReader {
    pub fn new(log_dir: &str, window_seconds: u64) -> Self {
        Self {
            states: HashMap::new(),
            log_dir: PathBuf::from(log_dir),
            window_seconds,
        }
    }

    pub fn update(&mut self, apps: &[String]) -> HashMap<String, TrafficWindow> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let cutoff = now - self.window_seconds as f64;

        let mut result = HashMap::new();

        for app in apps {
            let path = self.log_dir.join(format!("access-{app}.log"));
            let traffic = self.read_app_traffic(app, &path, cutoff);
            result.insert(app.clone(), traffic);
        }

        result
    }

    fn read_app_traffic(&mut self, app: &str, path: &Path, cutoff: f64) -> TrafficWindow {
        // Check if file exists; get inode
        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return TrafficWindow { window_seconds: self.window_seconds, ..Default::default() },
        };
        let current_inode = meta.ino();

        // Detect rotation: if inode changed, drop state so we re-open
        if let Some(state) = self.states.get(app) {
            if state.inode != current_inode {
                self.states.remove(app);
            }
        }

        // Open file if no state yet
        if !self.states.contains_key(app) {
            match File::open(path) {
                Ok(f) => {
                    self.states.insert(app.to_owned(), LogState {
                        reader: BufReader::new(f),
                        inode: current_inode,
                        entries: VecDeque::new(),
                    });
                }
                Err(_) => return TrafficWindow { window_seconds: self.window_seconds, ..Default::default() },
            }
        }

        let state = self.states.get_mut(app).unwrap();

        // Read newly appended lines
        let mut line = String::new();
        loop {
            line.clear();
            match state.reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(entry) = serde_json::from_str::<CaddyLine>(line.trim_end()) {
                        state.entries.push_back((entry.ts, entry.status));
                    }
                }
                Err(_) => break,
            }
        }

        // Evict entries outside the window
        while state.entries.front().map(|(ts, _)| *ts < cutoff).unwrap_or(false) {
            state.entries.pop_front();
        }

        compute_traffic(&state.entries, self.window_seconds)
    }
}

fn compute_traffic(entries: &VecDeque<(f64, u16)>, window_seconds: u64) -> TrafficWindow {
    let requests_total = entries.len() as u64;
    let requests_per_min = requests_total as f64 / (window_seconds as f64 / 60.0);
    let error_4xx = entries.iter().filter(|(_, s)| *s >= 400 && *s < 500).count() as u64;
    let error_5xx = entries.iter().filter(|(_, s)| *s >= 500).count() as u64;
    let error_pct = if requests_total > 0 {
        error_5xx as f64 / requests_total as f64 * 100.0
    } else {
        0.0
    };
    TrafficWindow { window_seconds, requests_total, requests_per_min, error_4xx, error_5xx, error_pct }
}
