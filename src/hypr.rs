use serde::Deserialize;
use std::io::{BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::config::Config;

// ── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Monitor {
    pub id: i32,
    pub name: String,
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: WsRef,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct WsRef {
    pub id: i32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Workspace {
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub monitor: String,
    #[serde(default)]
    pub windows: u32,
    #[serde(default, rename = "lastwindowtitle")]
    pub last_window_title: String,
}

#[derive(Debug, Deserialize)]
pub struct Client {
    pub address: String,
    pub workspace: WsRef,
    #[serde(default)]
    pub pinned: bool,
}

// ── Socket Directory Discovery ──────────────────────────────────────────────

pub fn hypr_socket_dir() -> Result<PathBuf, String> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("Uid:\t"))?
                        .split_whitespace()
                        .nth(1)?
                        .parse::<u32>()
                        .ok()
                })
                .unwrap_or(1000);
            PathBuf::from(format!("/run/user/{uid}"))
        });

    // Check environment signature or scan for active socket directory
    if let Ok(instance) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        let p = runtime_dir.join("hypr").join(instance);
        if p.exists() {
            return Ok(p);
        }
    }

    let hypr_dir = runtime_dir.join("hypr");
    if let Ok(entries) = std::fs::read_dir(&hypr_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".socket.sock").exists() {
                return Ok(path);
            }
        }
    }

    Err("Hyprland socket directory not found".into())
}

// ── IPC queries (Direct .socket.sock with hyprctl fallback) ─────────────────

fn hypr_cmd(req: &str) -> Result<String, String> {
    let socket_path = hypr_socket_dir()?.join(".socket.sock");
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| format!("connect .socket.sock: {e}"))?;

    stream
        .set_read_timeout(Some(Duration::from_millis(600)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_millis(600)))
        .map_err(|e| e.to_string())?;

    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("write socket: {e}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("read socket: {e}"))?;

    Ok(response)
}

fn hyprctl(args: &[&str]) -> Result<String, String> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .map_err(|e| format!("hyprctl spawn: {e}"))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("hyprctl failed: {}", err.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn monitors() -> Result<Vec<Monitor>, String> {
    let json = hypr_cmd("j/monitors").or_else(|_| hyprctl(&["monitors", "-j"]))?;
    serde_json::from_str(&json).map_err(|e| format!("parse monitors: {e}"))
}

pub fn workspaces() -> Result<Vec<Workspace>, String> {
    let json = hypr_cmd("j/workspaces").or_else(|_| hyprctl(&["workspaces", "-j"]))?;
    serde_json::from_str(&json).map_err(|e| format!("parse workspaces: {e}"))
}

pub fn active_workspace() -> Result<Workspace, String> {
    let json = hypr_cmd("j/activeworkspace").or_else(|_| hyprctl(&["activeworkspace", "-j"]))?;
    serde_json::from_str(&json).map_err(|e| format!("parse activeworkspace: {e}"))
}

pub fn clients() -> Result<Vec<Client>, String> {
    let json = hypr_cmd("j/clients").or_else(|_| hyprctl(&["clients", "-j"]))?;
    serde_json::from_str(&json).map_err(|e| format!("parse clients: {e}"))
}

// ── IPC commands ────────────────────────────────────────────────────────────

pub fn eval(expr: &str) -> Result<(), String> {
    let cmd = format!("eval {expr}");
    let _ = hypr_cmd(&cmd).or_else(|_| hyprctl(&["eval", expr]))?;
    Ok(())
}

#[allow(dead_code)]
pub fn dispatch(cmd: &str) -> Result<(), String> {
    let full = format!("dispatch {cmd}");
    let _ = hypr_cmd(&full).or_else(|_| {
        let args: Vec<&str> = std::iter::once("dispatch")
            .chain(cmd.split_whitespace())
            .collect();
        hyprctl(&args)
    })?;
    Ok(())
}

// ── Animation ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum AnimStyle {
    Slide,
    SlideVert,
}

pub fn set_animation(style: AnimStyle) -> Result<(), String> {
    let s = match style {
        AnimStyle::Slide => "slide",
        AnimStyle::SlideVert => "slidevert",
    };
    eval(&format!(
        "hl.animation({{ leaf = 'workspaces', enabled = true, speed = 7, bezier = 'menu_decel', style = '{s}' }})"
    ))
}

// ── Dispatch helpers ────────────────────────────────────────────────────────

pub fn focus_workspace(id: u32) -> Result<(), String> {
    eval(&format!("hl.dispatch(hl.dsp.focus({{workspace = {id}}}))"))
}

pub fn move_window_to(id: u32) -> Result<(), String> {
    eval(&format!(
        "hl.dispatch(hl.dsp.window.move({{workspace = {id}}}))"
    ))
}

pub fn move_window_addr_to(addr: &str, id: u32) -> Result<(), String> {
    eval(&format!(
        "hl.dispatch(hl.dsp.window.move({{address = '{addr}', workspace = {id}}}))"
    ))
}

// ── Monitor helper ──────────────────────────────────────────────────────────

/// Resolve the monitor index for the given monitor name.
pub fn resolve_monitor_index(cfg: &Config, name: &str) -> u32 {
    if !cfg.monitors.isolate {
        return 0;
    }
    // Check config order
    if let Some(idx) = cfg.monitors.order.iter().position(|n| n == name) {
        return idx as u32;
    }
    // Fallback: query Hyprland
    monitors()
        .ok()
        .and_then(|mons| {
            mons.iter()
                .find(|m| m.name == name)
                .map(|m| m.id.max(0) as u32)
        })
        .unwrap_or(0)
}

/// Get workspace IDs filtered by monitor (respects isolate setting).
pub fn monitor_workspaces(cfg: &Config, monitor: &str) -> Result<Vec<u32>, String> {
    let all = workspaces()?;
    Ok(all
        .iter()
        .filter(|w| {
            w.id > 0
                && if cfg.monitors.isolate {
                    w.monitor == monitor
                } else {
                    true
                }
        })
        .map(|w| w.id as u32)
        .collect())
}

// ── Event stream ────────────────────────────────────────────────────────────

/// Connect to Hyprland's event socket (.socket2.sock) for live event streaming.
pub fn event_stream() -> Result<BufReader<UnixStream>, String> {
    let socket_path = hypr_socket_dir()?.join(".socket2.sock");
    let stream =
        UnixStream::connect(&socket_path).map_err(|e| format!("connect event socket: {e}"))?;
    Ok(BufReader::new(stream))
}

// ── Notifications ───────────────────────────────────────────────────────────

/// Send notification asynchronously and reap child process to prevent zombies.
pub fn notify(msg: &str) {
    if let Ok(mut child) = Command::new("notify-send")
        .args(["-t", "3000", "hyprgrid", msg])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}
