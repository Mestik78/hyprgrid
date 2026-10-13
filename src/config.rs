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

        // Fall back to legacy monitors_order file only if config file was absent or order was empty
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
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
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

    fn strip_comment(line: &str) -> &str {
        let mut in_quote = false;
        let mut quote_char = '"';
        for (i, c) in line.char_indices() {
            if (c == '"' || c == '\'') && (!in_quote || c == quote_char) {
                if in_quote {
                    in_quote = false;
                } else {
                    in_quote = true;
                    quote_char = c;
                }
            } else if c == '#' && !in_quote {
                return &line[..i];
            }
        }
        line
    }

    pub fn parse(content: &str) -> Self {
        let mut cfg = Config::default();
        let mut section = String::new();
        let mut accumulating_key = None;
        let mut array_buffer = String::new();

        for line in content.lines() {
            let line = Self::strip_comment(line).trim();
            if line.is_empty() {
                continue;
            }

            // Handle multi-line array accumulation
            if let Some((sec, key)) = accumulating_key.take() {
                array_buffer.push_str(line);
                if line.contains(']') {
                    if sec == "monitors" && key == "order" {
                        cfg.monitors.order = Self::parse_array(&array_buffer);
                    }
                    array_buffer.clear();
                } else {
                    accumulating_key = Some((sec, key));
                }
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_string();
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // If starting a multi-line array that isn't closed on this line
                if value.starts_with('[') && !value.contains(']') {
                    accumulating_key = Some((section.clone(), key.to_string()));
                    array_buffer.clear();
                    array_buffer.push_str(value);
                    continue;
                }

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
                        cfg.monitors.isolate = value.eq_ignore_ascii_case("true");
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
        let start = match s.find('[') {
            Some(i) => i + 1,
            None => return Vec::new(),
        };
        let end = match s.rfind(']') {
            Some(i) => i,
            None => s.len(),
        };
        if start > end {
            return Vec::new();
        }

        s[start..end]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_with_inline_comments_and_multiline_array() {
        let toml = r#"
            [grid]
            rows = 8 # custom row count
            cols = 12 # custom col count

            [monitors]
            isolate = true # enable 3d
            order = [
                "eDP-1",
                "HDMI-A-2", # external
            ]
        "#;
        let cfg = Config::parse(toml);
        assert_eq!(cfg.grid.rows, 8);
        assert_eq!(cfg.grid.cols, 12);
        assert!(cfg.monitors.isolate);
        assert_eq!(cfg.monitors.order, vec!["eDP-1", "HDMI-A-2"]);
    }
}
