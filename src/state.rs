use std::fs;
use std::path::PathBuf;

fn state_dir() -> PathBuf {
    PathBuf::from("/tmp/hyprgrid")
}

fn state_file(m: u32, y: u32) -> PathBuf {
    state_dir().join(format!("{m}_{y}"))
}

/// Save the current column for a (monitor, row) pair.
pub fn save_column(m: u32, y: u32, x: u32) {
    let dir = state_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(state_file(m, y), x.to_string());
}

/// Load the remembered column for a (monitor, row) pair.
pub fn load_column(m: u32, y: u32) -> Option<u32> {
    fs::read_to_string(state_file(m, y))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
