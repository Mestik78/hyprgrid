use std::io::{self, BufRead, Write};
use std::time::Duration;

use crate::config::Config;
use crate::coords::Coord3D;
use crate::hypr;

pub fn print_status(cfg: &Config) -> Result<(), String> {
    let target_mon = std::env::var("WAYBAR_OUTPUT_NAME").unwrap_or_default();
    let target_mon = if target_mon.is_empty() {
        hypr::active_workspace().map(|w| w.monitor).unwrap_or_default()
    } else {
        target_mon
    };

    let m = hypr::resolve_monitor_index(cfg, &target_mon);

    // Get current workspace for this specific monitor
    let mut current_id = None;
    if !target_mon.is_empty() {
        if let Ok(mons) = hypr::monitors() {
            if let Some(mon) = mons.iter().find(|mon| mon.name == target_mon) {
                if mon.active_workspace.id > 0 {
                    current_id = Some(mon.active_workspace.id as u32);
                }
            }
        }
    }
    if current_id.is_none() {
        if let Ok(act) = hypr::active_workspace() {
            if act.id > 0 {
                current_id = Some(act.id as u32);
            }
        }
    }
    let current_id = current_id.unwrap_or_else(|| Coord3D { m, y: 0, x: 1 }.to_ws_id(cfg));

    // Decode in 3D
    let coord = Coord3D::from_ws_id(current_id, cfg);
    let t = coord.y;
    let u = coord.x;
    let display_t = t + 1;

    let start = Coord3D { m, y: t, x: 1 }.to_ws_id(cfg);
    let end = Coord3D {
        m,
        y: t,
        x: cfg.grid.cols,
    }
    .to_ws_id(cfg);

    let all_existing = hypr::monitor_workspaces(cfg, &target_mon).unwrap_or_default();

    let mut existing: Vec<u32> = all_existing
        .iter()
        .copied()
        .filter(|&w| w >= start && w <= end)
        .collect();

    if !existing.contains(&current_id) {
        existing.push(current_id);
    }
    existing.sort_unstable();

    // Render workspaces according to user's rice template
    let ws_items: Vec<String> = existing
        .iter()
        .map(|w| {
            let w_coord = Coord3D::from_ws_id(*w, cfg);
            let w_u = w_coord.x;
            let tmpl = if *w == current_id {
                &cfg.waybar.active_format
            } else {
                &cfg.waybar.occupied_format
            };
            tmpl.replace("{col}", &w_u.to_string())
                .replace("{ws}", &w.to_string())
                .replace("{row}", &display_t.to_string())
        })
        .collect();

    let ws_rendered = ws_items.join(&cfg.waybar.separator);

    let text = cfg
        .waybar
        .format
        .replace("{row}", &display_t.to_string())
        .replace("{workspaces}", &ws_rendered)
        .replace("{active}", &u.to_string())
        .replace("{monitor}", &target_mon);

    // Dynamic classes for rice styling in Waybar style.css
    let classes = vec![
        format!("m{m}"),
        format!("row-{display_t}"),
        format!("col-{u}"),
        "active".to_string(),
    ];

    // Minimal Bounding Box Grid Tooltip
    let mut eval_set = all_existing.clone();
    if !eval_set.contains(&current_id) {
        eval_set.push(current_id);
    }

    let mut min_x = cfg.grid.cols;
    let mut max_x = 1;
    let mut min_y = cfg.grid.rows;
    let mut max_y = 1;

    for &w in &eval_set {
        let c = Coord3D::from_ws_id(w, cfg);
        let row = c.y + 1;
        let col = c.x;
        if col < min_x {
            min_x = col;
        }
        if col > max_x {
            max_x = col;
        }
        if row < min_y {
            min_y = row;
        }
        if row > max_y {
            max_y = row;
        }
    }

    let mut tooltip = String::from("<span font_family='monospace'>");
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let id = Coord3D {
                m,
                y: y - 1,
                x,
            }
            .to_ws_id(cfg);
            let display_x = format!("{x:2}");

            if id == current_id {
                tooltip.push_str(&format!("<span color='#ffffff'><b>{display_x}</b></span> "));
            } else if all_existing.contains(&id) {
                tooltip.push_str(&format!("<span color='#999999'>{display_x}</span> "));
            } else {
                tooltip.push_str("   ");
            }
        }
        tooltip.push('\n');
    }
    if tooltip.ends_with('\n') {
        tooltip.pop();
    }
    tooltip.push_str("</span>");

    let json = serde_json::json!({
        "text": text,
        "tooltip": if cfg.waybar.show_tooltip { &tooltip } else { "" },
        "class": classes,
    });

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{json}").map_err(|e| format!("stdout write: {e}"))?;
    handle.flush().map_err(|e| format!("stdout flush: {e}"))?;
    Ok(())
}

pub fn run_stream(cfg: &Config) -> Result<(), String> {
    let mut backoff_ms = 500;

    loop {
        // Initial state emission
        if let Err(e) = print_status(cfg) {
            if e.contains("Broken pipe") {
                return Ok(());
            }
        }

        // Establish connection to socket2 with retry backoff
        let mut reader = match hypr::event_stream() {
            Ok(r) => {
                backoff_ms = 500;
                r
            }
            Err(e) => {
                eprintln!("hyprgrid: event stream connection failed: {e}. Retrying in {backoff_ms}ms...");
                std::thread::sleep(Duration::from_millis(backoff_ms));
                backoff_ms = (backoff_ms * 2).min(5000);
                continue;
            }
        };

        let mut line = String::with_capacity(256);
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // EOF: Hyprland restarted or closed socket
                    eprintln!("hyprgrid: event socket disconnected (EOF). Reconnecting...");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with("workspace>>")
                        || trimmed.starts_with("createworkspace>>")
                        || trimmed.starts_with("destroyworkspace>>")
                        || trimmed.starts_with("focusedmon>>")
                        || trimmed.starts_with("moveworkspace>>")
                        || trimmed.starts_with("movewindow>>")
                        || trimmed.starts_with("movewindowv2>>")
                        || trimmed.starts_with("monitoradded>>")
                        || trimmed.starts_with("monitorremoved>>")
                        || trimmed.starts_with("renameworkspace>>")
                    {
                        if let Err(e) = print_status(cfg) {
                            if e.contains("Broken pipe") {
                                return Ok(());
                            }
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    eprintln!("hyprgrid: event stream error: {e}. Reconnecting...");
                    break;
                }
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}
