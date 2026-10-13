use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub grid: GridConfig,
    pub monitors: MonitorConfig,
}

#[derive(Debug, Clone)]
pub struct GridConfig {
    pub rows: u32,
    pub cols: u32,
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub isolate: bool,
    pub order: Vec<String>,
}

impl GridConfig {
    /// Total workspace slots per monitor.
    pub fn capacity(&self) -> u32 {
        self.rows * self.cols
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid: GridConfig { rows: 10, cols: 10 },
            monitors: MonitorConfig {
                isolate: true,
                order: Vec::new(),
            },
        }
    }
}

impl Config {
    /// Load configuration from `~/.config/hyprgrid/config.toml`.
    /// Falls back to defaults + legacy `monitors_order` file if not found.
    pub fn load() -> Self {
        let config_path = Self::config_path();
        let mut cfg = if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            Self::parse(&content)
        } else {
            Config::default()
        };

        // Fall back to legacy monitors_order file
        if cfg.monitors.order.is_empty() {
            if let Some(legacy) = Self::legacy_monitors_path() {
                if legacy.exists() {
                    if let Ok(content) = fs::read_to_string(&legacy) {
                        cfg.monitors.order = content
                            .lines()
                            .map(|l| l.trim().to_string())
                            .filter(|l| !l.is_empty() && !l.starts_with('#'))
                            .collect();
                    }
                }
            }
        }

        cfg
    }

    fn config_path() -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME")
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                format!("{home}/.config")
            });
        PathBuf::from(base).join("hyprgrid/config.toml")
    }

    fn legacy_monitors_path() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config/hypr/monitors_order"))
    }

    fn parse(content: &str) -> Self {
        let mut cfg = Config::default();
        let mut section = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_string();
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match (section.as_str(), key) {
                    ("grid", "rows") => {
                        if let Ok(v) = value.parse::<u32>() {
                            cfg.grid.rows = v.clamp(1, 100);
                        }
                    }
                    ("grid", "cols") => {
                        if let Ok(v) = value.parse::<u32>() {
                            cfg.grid.cols = v.clamp(1, 100);
                        }
                    }
                    ("monitors", "isolate") => {
                        cfg.monitors.isolate = value == "true";
                    }
                    ("monitors", "order") => {
                        cfg.monitors.order = Self::parse_array(value);
                    }
                    _ => {}
                }
            }
        }
        cfg
    }

    fn parse_array(s: &str) -> Vec<String> {
        let s = s.trim();
        if !s.starts_with('[') || !s.ends_with(']') {
            return Vec::new();
        }
        s[1..s.len() - 1]
            .split(',')
            .map(|item| {
                item.trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect()
    }
}
