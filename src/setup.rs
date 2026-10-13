use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::hypr;

pub fn generate_conf(mod_x: &str, mod_y: &str) -> String {
    let mut out = String::new();
    out.push_str("# ─────────────────────────────────────────────────────────────────────────────\n");
    out.push_str("# hyprgrid keybindings (Monitor × Row × Column)\n");
    out.push_str("# Generated automatically by `hyprgrid setup`\n");
    out.push_str("# ─────────────────────────────────────────────────────────────────────────────\n\n");

    out.push_str(&format!("$modX = {mod_x}\n"));
    out.push_str(&format!("$modY = {mod_y}\n\n"));

    out.push_str("# ── Relative Navigation (Tab / Shift+Tab) ────────────────────────────────────\n");
    out.push_str("bind = $modX, Tab, exec, hyprgrid grid focus x next\n");
    out.push_str("bind = $modX SHIFT, Tab, exec, hyprgrid grid focus x prev\n");
    out.push_str("bind = $modY, Tab, exec, hyprgrid grid focus y next\n");
    out.push_str("bind = $modY SHIFT, Tab, exec, hyprgrid grid focus y prev\n\n");

    out.push_str("# ── Workspace Creation & Shifting ────────────────────────────────────────────\n");
    out.push_str("bind = $modX, N, exec, hyprgrid create x empty\n");
    out.push_str("bind = $modY, N, exec, hyprgrid create y empty\n");
    out.push_str("bind = $modX CTRL, N, exec, hyprgrid create x right\n");
    out.push_str("bind = $modX SHIFT, N, exec, hyprgrid create x left\n");
    out.push_str("bind = $modY CTRL, N, exec, hyprgrid create y right\n");
    out.push_str("bind = $modY SHIFT, N, exec, hyprgrid create y left\n\n");

    out.push_str("# ── Interactive 2D Workspace Map ─────────────────────────────────────────────\n");
    out.push_str("bind = $modY, G, exec, hyprgrid map\n\n");

    out.push_str("# ── Absolute Navigation (1..10) ──────────────────────────────────────────────\n");
    out.push_str("# Columns (X axis)\n");
    for i in 1..=10 {
        let key = i % 10;
        out.push_str(&format!("bind = $modX, {key}, exec, hyprgrid grid focus x {i}\n"));
        out.push_str(&format!("bind = $modX SHIFT, {key}, exec, hyprgrid grid move x {i}\n"));
    }

    out.push_str("\n# Rows (Y axis)\n");
    for i in 1..=10 {
        let key = i % 10;
        out.push_str(&format!("bind = $modY, {key}, exec, hyprgrid grid focus y {i}\n"));
        out.push_str(&format!("bind = $modY SHIFT, {key}, exec, hyprgrid grid move y {i}\n"));
    }

    out.push_str("\n# Move all windows in workspace to row\n");
    for i in 1..=10 {
        let key = i % 10;
        out.push_str(&format!("bind = $modX $modY, {key}, exec, hyprgrid grid move_all y {i}\n"));
    }

    out
}

pub fn generate_lua(mod_x: &str, mod_y: &str) -> String {
    let mut out = String::new();
    out.push_str("-- hyprgrid keybindings (Lua format for hyprland-lua)\n");
    out.push_str(&format!("local xMod = \"{mod_x}\"\n"));
    out.push_str(&format!("local yMod = \"{mod_y}\"\n\n"));

    out.push_str("-- Relative cycling\n");
    out.push_str("hl.bind(xMod .. \"+Tab\", hl.dsp.exec_cmd(\"hyprgrid grid focus x next\"))\n");
    out.push_str("hl.bind(xMod .. \"+SHIFT+Tab\", hl.dsp.exec_cmd(\"hyprgrid grid focus x prev\"))\n");
    out.push_str("hl.bind(yMod .. \"+Tab\", hl.dsp.exec_cmd(\"hyprgrid grid focus y next\"))\n");
    out.push_str("hl.bind(yMod .. \"+SHIFT+Tab\", hl.dsp.exec_cmd(\"hyprgrid grid focus y prev\"))\n\n");

    out.push_str("-- Creation and shifting\n");
    out.push_str("hl.bind(xMod .. \"+N\", hl.dsp.exec_cmd(\"hyprgrid create x empty\"))\n");
    out.push_str("hl.bind(yMod .. \"+N\", hl.dsp.exec_cmd(\"hyprgrid create y empty\"))\n");
    out.push_str("hl.bind(\"CTRL+\" .. xMod .. \"+N\", hl.dsp.exec_cmd(\"hyprgrid create x right\"))\n");
    out.push_str("hl.bind(\"SHIFT+\" .. xMod .. \"+N\", hl.dsp.exec_cmd(\"hyprgrid create x left\"))\n");
    out.push_str("hl.bind(\"CTRL+\" .. yMod .. \"+N\", hl.dsp.exec_cmd(\"hyprgrid create y right\"))\n");
    out.push_str("hl.bind(\"SHIFT+\" .. yMod .. \"+N\", hl.dsp.exec_cmd(\"hyprgrid create y left\"))\n\n");

    out.push_str("-- 2D Map\n");
    out.push_str("hl.bind(yMod .. \"+G\", hl.dsp.exec_cmd(\"hyprgrid map\"))\n\n");

    out.push_str("-- Absolute navigation\n");
    out.push_str("for i = 1, 10 do\n");
    out.push_str("    local key = tostring(i % 10)\n");
    out.push_str("    local val = tostring(i)\n");
    out.push_str("    hl.bind(xMod .. \"+\" .. key, hl.dsp.exec_cmd(\"hyprgrid grid focus x \" .. val))\n");
    out.push_str("    hl.bind(xMod .. \"+SHIFT+\" .. key, hl.dsp.exec_cmd(\"hyprgrid grid move x \" .. val))\n");
    out.push_str("    hl.bind(yMod .. \"+\" .. key, hl.dsp.exec_cmd(\"hyprgrid grid focus y \" .. val))\n");
    out.push_str("    hl.bind(yMod .. \"+SHIFT+\" .. key, hl.dsp.exec_cmd(\"hyprgrid grid move y \" .. val))\n");
    out.push_str("    hl.bind(xMod .. \"+\" .. yMod .. \"+\" .. key, hl.dsp.exec_cmd(\"hyprgrid grid move_all y \" .. val))\n");
    out.push_str("end\n");

    out
}

pub fn run(args: &[String]) -> Result<(), String> {
    let is_print = args.iter().any(|a| a == "--print" || a == "-p");
    let is_lua = args.iter().any(|a| a == "--lua");

    let mut mod_x = "ALT";
    let mut mod_y = "SUPER";

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--mod-x" && i + 1 < args.len() {
            mod_x = &args[i + 1];
            i += 1;
        } else if args[i] == "--mod-y" && i + 1 < args.len() {
            mod_y = &args[i + 1];
            i += 1;
        }
        i += 1;
    }

    let content = if is_lua {
        generate_lua(mod_x, mod_y)
    } else {
        generate_conf(mod_x, mod_y)
    };

    if is_print {
        print!("{content}");
        return Ok(());
    }

    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let hypr_dir = PathBuf::from(&home).join(".config/hypr");
    if !hypr_dir.exists() {
        let _ = fs::create_dir_all(&hypr_dir);
    }

    let target_file = hypr_dir.join("hyprgrid.conf");
    fs::write(&target_file, &content)
        .map_err(|e| format!("failed to write {}: {e}", target_file.display()))?;

    println!("✨ hyprgrid keybindings written to: {}", target_file.display());

    // Check hyprland.conf and add source line if missing
    let main_conf = hypr_dir.join("hyprland.conf");
    let source_line = "source = ~/.config/hypr/hyprgrid.conf";

    if main_conf.exists() {
        let main_content = fs::read_to_string(&main_conf).unwrap_or_default();
        if !main_content.contains("hyprgrid.conf") {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&main_conf)
                .map_err(|e| format!("failed to open hyprland.conf for appending: {e}"))?;

            writeln!(file, "\n# 3D Workspace Grid Manager\n{source_line}")
                .map_err(|e| format!("failed to append source line: {e}"))?;

            println!("🔗 Added '{}' to {}", source_line, main_conf.display());
        } else {
            println!("ℹ️  Source line already present in {}", main_conf.display());
        }
    } else {
        println!("ℹ️  No hyprland.conf found at {}. Add '{}' to your main config.", main_conf.display(), source_line);
    }

    // Attempt to reload Hyprland configuration
    let _ = hypr::eval("hl.reload()");

    println!("\n🚀 Done! Hyprland keybindings are ready to use.");
    println!("\nQuick reference:");
    println!("  {mod_x} + 1..0        Focus columns (1-10)");
    println!("  {mod_y} + 1..0        Focus rows (1-10)");
    println!("  {mod_x} + Tab         Next occupied column");
    println!("  {mod_y} + Tab         Next occupied row");
    println!("  {mod_y} + G           Open interactive 2D workspace map\n");

    Ok(())
}
