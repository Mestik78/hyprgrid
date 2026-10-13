use std::collections::HashMap;
use std::io::{stdout, Write};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};

use crate::config::Config;
use crate::coords::Coord3D;
use crate::hypr::{self, AnimStyle};

const C_RESET: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";
const C_DIM: &str = "\x1b[2m";
const C_CURRENT: &str = "\x1b[1;38;5;220m"; // yellow bold
const C_SELECT: &str = "\x1b[1;38;5;75m";   // light blue
const C_EXIST: &str = "\x1b[38;5;250m";     // light gray
const C_EMPTY: &str = "\x1b[38;5;238m";     // dark gray
const C_LABEL: &str = "\x1b[38;5;244m";     // gray label
const C_TITLE: &str = "\x1b[1;38;5;255m";   // white bold

struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Result<Self, String> {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = execute!(std::io::stdout(), Show, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            default_hook(info);
        }));

        enable_raw_mode().map_err(|e| format!("enable raw mode: {e}"))?;
        let _ = execute!(stdout(), EnterAlternateScreen, Hide);
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
    }
}

pub fn run(cfg: &Config) -> Result<(), String> {
    let current_ws = hypr::active_workspace()?;
    if current_ws.id <= 0 || current_ws.name.starts_with("special:") {
        return Err("Cannot open workspace map while in a special workspace/scratchpad.".into());
    }

    let _guard = RawModeGuard::new()?;

    let current_id = current_ws.id as u32;
    let current_mon = current_ws.monitor.clone();
    let m = hypr::resolve_monitor_index(cfg, &current_mon);

    let coord = Coord3D::from_ws_id(current_id, cfg);
    let current_t = coord.y;
    let current_u = coord.x.saturating_sub(1); // 0-indexed for cursor

    let all_ws = hypr::workspaces().unwrap_or_default();
    let mut ws_windows: HashMap<u32, u32> = HashMap::new();
    let mut ws_titles: HashMap<u32, String> = HashMap::new();
    let mut existing_ids: Vec<u32> = Vec::new();

    for w in all_ws {
        if w.id <= 0 {
            continue;
        }
        if cfg.monitors.isolate && w.monitor != current_mon {
            continue;
        }
        let id = w.id as u32;
        existing_ids.push(id);
        ws_windows.insert(id, w.windows);
        let title: String = w.last_window_title.chars().take(20).collect();
        ws_titles.insert(id, title);
    }

    // Determine bounding dimensions (0-indexed max)
    let mut max_t = current_t;
    let mut max_u = current_u;

    for &wid in &existing_ids {
        let c = Coord3D::from_ws_id(wid, cfg);
        if c.y > max_t {
            max_t = c.y;
        }
        let u_0 = c.x.saturating_sub(1);
        if u_0 > max_u {
            max_u = u_0;
        }
    }

    let rows = (max_t + 1).min(cfg.grid.rows);
    let cols = (max_u + 1).min(cfg.grid.cols);

    let mut cur_t = current_t;
    let mut cur_u = current_u;

    loop {
        draw_map(
            cfg,
            m,
            &current_mon,
            current_id,
            &existing_ids,
            &ws_windows,
            &ws_titles,
            rows,
            cols,
            cur_t,
            cur_u,
        )?;

        match event::read().map_err(|e| format!("read key: {e}"))? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if cur_t > 0 {
                            cur_t -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if cur_t + 1 < rows {
                            cur_t += 1;
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if cur_u > 0 {
                            cur_u -= 1;
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if cur_u + 1 < cols {
                            cur_u += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let target = Coord3D {
                            m,
                            y: cur_t,
                            x: cur_u + 1,
                        }
                        .to_ws_id(cfg);

                        if cur_t != current_t {
                            let _ = hypr::set_animation(AnimStyle::SlideVert);
                        } else {
                            let _ = hypr::set_animation(AnimStyle::Slide);
                        }
                        let _ = hypr::focus_workspace(target);
                        break;
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        break;
                    }
                    KeyCode::Char('c')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        break;
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {
                continue;
            }
            _ => {}
        }
    }

    Ok(())
}

fn draw_map(
    cfg: &Config,
    m: u32,
    current_mon: &str,
    current_id: u32,
    existing_ids: &[u32],
    ws_windows: &HashMap<u32, u32>,
    ws_titles: &HashMap<u32, String>,
    rows: u32,
    cols: u32,
    cur_t: u32,
    cur_u: u32,
) -> Result<(), String> {
    let mut out = stdout();
    execute!(out, MoveTo(0, 0), Clear(ClearType::All))
        .map_err(|e| format!("terminal write: {e}"))?;

    println!("{C_TITLE}  ╔══════════════════════════════════════╗{C_RESET}");
    println!("{C_TITLE}  ║      Mapa de Workspaces (2D)         ║{C_RESET}");
    println!("{C_TITLE}  ╚══════════════════════════════════════╝{C_RESET}");
    if !current_mon.is_empty() {
        println!("  {C_LABEL}Monitor:{C_RESET} {C_BOLD}{current_mon}{C_RESET} (M{m})");
    }
    println!();

    // Column headers (exactly 6 characters per column: "  %2d  ")
    print!("  {C_LABEL}Y\\X ");
    for u in 0..cols {
        print!("  {:2}  ", u + 1);
    }
    println!("{C_RESET}");

    print!("  {C_LABEL}────");
    for _ in 0..cols {
        print!("──────");
    }
    println!("{C_RESET}");

    // Rows
    for t in 0..rows {
        print!("  {C_LABEL}{:2} │{C_RESET}", t + 1);
        for u in 0..cols {
            let ws_id = Coord3D { m, y: t, x: u + 1 }.to_ws_id(cfg);
            let display_num = t * cfg.grid.cols + u + 1;
            let is_current = ws_id == current_id;
            let is_exist = existing_ids.contains(&ws_id);
            let is_selected = t == cur_t && u == cur_u;

            if is_current && is_selected {
                print!(" {C_CURRENT}[{:2}]{C_RESET} ", display_num);
            } else if is_selected {
                print!(" {C_SELECT}[{:2}]{C_RESET} ", display_num);
            } else if is_current {
                print!(" {C_CURRENT} {:2} {C_RESET} ", display_num);
            } else if is_exist {
                print!(" {C_EXIST} {:2} {C_RESET} ", display_num);
            } else {
                print!(" {C_EMPTY} ·· {C_RESET} ");
            }
        }
        println!();

        // Window count sub-row (aligned to exactly 6 characters per column)
        print!("  {C_LABEL}   │{C_RESET}");
        for u in 0..cols {
            let ws_id = Coord3D { m, y: t, x: u + 1 }.to_ws_id(cfg);
            let is_exist = existing_ids.contains(&ws_id);
            if is_exist {
                let wins = ws_windows.get(&ws_id).copied().unwrap_or(0);
                if t == cur_t && u == cur_u {
                    print!(" {C_SELECT}󰖯 {wins}  {C_RESET}");
                } else if ws_id == current_id {
                    print!(" {C_CURRENT}󰖯 {wins}  {C_RESET}");
                } else {
                    print!(" {C_EXIST}󰖯 {wins}  {C_RESET}");
                }
            } else {
                print!("      ");
            }
        }
        println!();
    }

    println!();
    println!("  {C_LABEL}Flechas / hjkl: mover  │  Enter: ir al workspace  │  q / Esc: salir{C_RESET}");

    let cur_ws = Coord3D {
        m,
        y: cur_t,
        x: cur_u + 1,
    }
    .to_ws_id(cfg);
    let disp_ws = cur_t * cfg.grid.cols + cur_u + 1;
    let is_exist_cur = existing_ids.contains(&cur_ws);

    println!();
    if is_exist_cur {
        let title = ws_titles.get(&cur_ws).cloned().unwrap_or_default();
        println!("  {C_BOLD}Seleccionado:{C_RESET} WS {C_SELECT}{disp_ws}{C_RESET} (ID: {cur_ws})  \"{title}\"");
    } else {
        println!("  {C_BOLD}Seleccionado:{C_RESET} WS {C_DIM}{disp_ws} (vacío, ID: {cur_ws}){C_RESET}");
    }

    let _ = out.flush();
    Ok(())
}
