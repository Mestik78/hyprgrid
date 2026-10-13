use serde::Deserialize;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;

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
}

// ── IPC queries ─────────────────────────────────────────────────────────────

fn hyprctl(args: &[&str]) -> Result<String, String> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .map_err(|e| format!("hyprctl: {e}"))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn monitors() -> Result<Vec<Monitor>, String> {
    let json = hyprctl(&["monitors", "-j"])?;
    serde_json::from_str(&json).map_err(|e| format!("parse monitors: {e}"))
}

pub fn workspaces() -> Result<Vec<Workspace>, String> {
    let json = hyprctl(&["workspaces", "-j"])?;
    serde_json::from_str(&json).map_err(|e| format!("parse workspaces: {e}"))
}

pub fn active_workspace() -> Result<Workspace, String> {
    let json = hyprctl(&["activeworkspace", "-j"])?;
    serde_json::from_str(&json).map_err(|e| format!("parse activeworkspace: {e}"))
}

pub fn clients() -> Result<Vec<Client>, String> {
    let json = hyprctl(&["clients", "-j"])?;
    serde_json::from_str(&json).map_err(|e| format!("parse clients: {e}"))
}

// ── IPC commands ────────────────────────────────────────────────────────────

pub fn eval(expr: &str) -> Result<(), String> {
    let _ = hyprctl(&["eval", expr])?;
    Ok(())
}

#[allow(dead_code)]
pub fn dispatch(cmd: &str) -> Result<(), String> {
    let args: Vec<&str> = std::iter::once("dispatch")
        .chain(cmd.split_whitespace())
        .collect();
    let _ = hyprctl(&args)?;
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
    eval(&format!("hl.dispatch(hl.dsp.focus({{workspace = {id}}}))"))?;
    Ok(())
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
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".into());
    let instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set".to_string())?;
    let path = PathBuf::from(runtime_dir)
        .join("hypr")
        .join(instance)
        .join(".socket2.sock");
    let stream =
        UnixStream::connect(&path).map_err(|e| format!("connect event socket: {e}"))?;
    Ok(BufReader::new(stream))
}

// ── Notifications ───────────────────────────────────────────────────────────

pub fn notify(msg: &str) {
    let _ = Command::new("notify-send")
        .args(["-t", "3000", "hyprgrid", msg])
        .spawn();
}
