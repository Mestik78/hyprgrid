use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn get_current_uid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            for line in status.lines() {
                if let Some(rest) = line.strip_prefix("Uid:\t") {
                    return rest.split_whitespace().next()?.parse().ok();
                }
            }
            None
        })
        .unwrap_or(1000)
}

fn state_dir() -> PathBuf {
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime).join("hyprgrid")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/state/hyprgrid")
    } else {
        let uid = get_current_uid();
        PathBuf::from(format!("/tmp/hyprgrid-{uid}"))
    }
}

fn state_file(m: u32, y: u32) -> PathBuf {
    state_dir().join(format!("{m}_{y}"))
}

/// Save the current column for a (monitor, row) pair atomically.
pub fn save_column(m: u32, y: u32, x: u32) {
    let dir = state_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }

    let target = state_file(m, y);
    let tmp = dir.join(format!("{m}_{y}.tmp.{}", std::process::id()));

    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)
    {
        if write!(file, "{x}").is_ok() && file.flush().is_ok() {
            let _ = fs::rename(&tmp, &target);
            return;
        }
    }
    let _ = fs::remove_file(&tmp);
}

/// Load the remembered column for a (monitor, row) pair.
pub fn load_column(m: u32, y: u32) -> Option<u32> {
    fs::read_to_string(state_file(m, y))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
