# 🚀 hyprgrid

Ultra-fast, lightweight 3D workspace grid manager for **Hyprland**, written in **Rust**. Organizes your workspaces into three independent dimensions: **Monitor × Row × Column**, featuring absolute jumps and relative cycling, per-monitor column memory, a live reactive Waybar module, and an interactive 2D terminal map (TUI).

Packaged in a single compiled binary of just **~530 KB** with zero background runtime dependencies.

---

## 📐 3D Mathematical Model

```
            Monitor 1 (HDMI-A-2)  ──>  Workspaces 101 .. 200
         ┌───────────────────────┐
         │ (1,1) (1,2) ... (1,C) │  Row 1 (SUPER + 1)
         │ (2,1) (2,2) ... (2,C) │  Row 2 (SUPER + 2)
         │  ...   ...  ...  ...  │
         │ (R,1) (R,2) ... (R,C) │  Row R (SUPER + 0)
         └───────────────────────┘
      Monitor 0 (eDP-1)  ──>  Workspaces 1 .. 100
   ┌───────────────────────┐
   │ (1,1) (1,2) ... (1,C) │
   │ (2,1) (2,2) ... (2,C) │
   │ (R,1) (R,2) ... (R,C) │
   └───────────────────────┘
       ALT+1 ALT+2 ... ALT+0  (Columns X)
```

$$\text{Workspace ID} = M \times (\text{Rows} \times \text{Cols}) + (Y - 1) \times \text{Cols} + X$$

* **$M$ (Z Dimension - Monitor):** Isolated numeric workspace pool per physical screen.
* **$Y$ (Y Dimension - Row / Tens):** Rows $1..R$ on each monitor (`SUPER + 1..0` or `SUPER + Tab`).
* **$X$ (X Dimension - Column / Units):** Columns $1..C$ within each row (`ALT + 1..0` or `ALT + Tab`).

Default dimensions: $10 \times 10$ ($R=10, C=10$). Both dimensions and monitor isolation are fully configurable.

---

## ✨ Features

* **Instant Performance (<1ms IPC):** Direct Unix domain socket communication (`.socket.sock`). Zero child process overhead.
* **Ultra-Lightweight Binary (~530 KB):** No heavy async runtime (zero tokio overhead) and no bloated TUI frameworks. Built with LTO, `opt-level = "z"`, and stripped symbols.
* **Unified Navigation:** Supports both absolute jumps to specific rows/columns and relative cycling (`next` / `prev`) through a single command.
* **Per-Monitor Column Memory:** When switching rows (`SUPER + N`), automatically remembers and restores the last visited column for that specific row and monitor.
* **Dynamic Animations on the Fly:** Seamlessly triggers vertical slide (`slidevert`) for the $Y$ axis and horizontal slide (`slide`) for the $X$ axis.
* **Smart Creation & Shifting:**
  * Jump directly to the first available empty column in the current row or first unoccupied row group.
  * Shift workspaces and windows left/right or up/down without clobbering pinned windows or scratchpads.
* **Live Waybar Module:**
  * Real-time event streaming via Hyprland's broadcast socket (`.socket2.sock`) with automatic reconnection backoff.
  * Per-monitor status isolation matching `$WAYBAR_OUTPUT_NAME`.
  * Compact 2D monospace bounding box tooltip adapting cleanly to both light and dark themes.
* **Interactive TUI Map (`hyprgrid map`):**
  * Clean 2D terminal grid with arrow keys or `hjkl` navigation.
  * Live open window count and active window title preview.
  * Instant focus jump on `Enter`.
* **Configurable:** Simple TOML configuration at `~/.config/hyprgrid/config.toml` to customize grid size, monitor ordering, or disable the Z dimension for a single shared 2D grid.

---

## 🛠️ Installation

### Install from AUR (Arch Linux)

```bash
yay -S hyprgrid-git
# or
paru -S hyprgrid-git
```

### Build and Install from Source

```bash
git clone https://github.com/Mestik78/hyprgrid.git
cd hyprgrid
./install.sh
```

The install script compiles the binary in `--release` mode, copies it to `~/.local/bin/hyprgrid` (and `~/.cargo/bin/hyprgrid`), and generates a default configuration at `~/.config/hyprgrid/config.toml`.

---

## ⚙️ Configuration (`~/.config/hyprgrid/config.toml`)

```toml
[grid]
# Number of rows (Y dimension) per monitor (1..100, default: 10)
rows = 10

# Number of columns (X dimension) per row (1..100, default: 10)
cols = 10

[monitors]
# Enable 3D monitor isolation (Z axis)
# If set to false, all monitors share a single flat 2D grid 1..N
isolate = true

# Deterministic monitor ordering (indices 0, 1, 2, ...)
# Falls back to ~/.config/hypr/monitors_order if omitted
order = [
    "eDP-1",
    "HDMI-A-2"
]

[waybar]
# Main format string
# Variables: {row}, {workspaces}, {active}, {monitor}
format = " {row} {workspaces}"

# Active workspace template ({col}, {ws}, {row})
active_format = "<b><u>{col}</u></b>"

# Inactive/occupied workspace template
occupied_format = "<span alpha='60%'>{col}</span>"

# Separator between workspace numbers
separator = " "

# Show 2D workspace bounding-box grid tooltip
show_tooltip = true
```

### 🎨 Rice & Theme Customization Examples

**Minimalist / Dot-style rice:**
```toml
[waybar]
format = "R{row}  {workspaces}"
active_format = "●"
occupied_format = "○"
separator = " "
```

**Pure CSS-controlled (No Pango):**
```toml
[waybar]
format = "{row} | {workspaces}"
active_format = "[{col}]"
occupied_format = "{col}"
```

**Custom colors / Catppuccin / Tokyo Night:**
```toml
[waybar]
format = " {row}  {workspaces}"
active_format = "<span color='#89b4fa'><b>{col}</b></span>"
occupied_format = "<span color='#6c7086'>{col}</span>"
```

Waybar automatically receives dynamic CSS classes on `#custom-ws_grid`:
`.m0`, `.m1`, `.row-1`, `.col-2`, `.active`, allowing deep styling in `style.css`.

---

## ⌨️ Hyprland Keybindings

### In Lua (`~/.config/hypr/conf/keybinds.lua`)

```lua
local xMod = "ALT"
local yMod = "SUPER"

-- Relative cycling (Tab / Shift+Tab)
hl.bind(yMod .. "+Tab", hl.dsp.exec_cmd("hyprgrid grid focus y next"))
hl.bind(yMod .. "+SHIFT+Tab", hl.dsp.exec_cmd("hyprgrid grid focus y prev"))
hl.bind(xMod .. "+Tab", hl.dsp.exec_cmd("hyprgrid grid focus x next"))
hl.bind(xMod .. "+SHIFT+Tab", hl.dsp.exec_cmd("hyprgrid grid focus x prev"))

-- Workspace creation and shifting
hl.bind(xMod .. "+N", hl.dsp.exec_cmd("hyprgrid create x empty"))
hl.bind(yMod .. "+N", hl.dsp.exec_cmd("hyprgrid create y empty"))
hl.bind("CTRL+" .. xMod .. "+N", hl.dsp.exec_cmd("hyprgrid create x right"))
hl.bind("SHIFT+" .. xMod .. "+N", hl.dsp.exec_cmd("hyprgrid create x left"))
hl.bind("CTRL+" .. yMod .. "+N", hl.dsp.exec_cmd("hyprgrid create y right"))
hl.bind("SHIFT+" .. yMod .. "+N", hl.dsp.exec_cmd("hyprgrid create y left"))

-- Absolute navigation (1..10)
for i = 1, 10 do
    local key = tostring(i % 10)
    local val = tostring(i)

    -- Columns (X axis)
    hl.bind(xMod .. "+" .. key, hl.dsp.exec_cmd("hyprgrid grid focus x " .. val))
    hl.bind(xMod .. "+SHIFT+" .. key, hl.dsp.exec_cmd("hyprgrid grid move x " .. val))

    -- Rows (Y axis)
    hl.bind(yMod .. "+" .. key, hl.dsp.exec_cmd("hyprgrid grid focus y " .. val))
    hl.bind(yMod .. "+SHIFT+" .. key, hl.dsp.exec_cmd("hyprgrid grid move y " .. val))

    -- Move all windows in workspace to row
    hl.bind(xMod .. "+" .. yMod .. "+" .. key, hl.dsp.exec_cmd("hyprgrid grid move_all y " .. val))
end

-- Interactive TUI map
hl.bind(yMod .. "+G", hl.dsp.exec_cmd("ghostty --class=hyprgrid-map -e hyprgrid map"))
```

### Standard `hyprland.conf`

```ini
# X Axis (Columns)
bind = ALT, 1, exec, hyprgrid grid focus x 1
bind = ALT SHIFT, 1, exec, hyprgrid grid move x 1
bind = ALT, Tab, exec, hyprgrid grid focus x next
bind = ALT SHIFT, Tab, exec, hyprgrid grid focus x prev

# Y Axis (Rows)
bind = SUPER, 1, exec, hyprgrid grid focus y 1
bind = SUPER SHIFT, 1, exec, hyprgrid grid move y 1
bind = SUPER, Tab, exec, hyprgrid grid focus y next
bind = SUPER SHIFT, Tab, exec, hyprgrid grid focus y prev
```

---

## 📊 Waybar Module

Add the following to your `~/.config/waybar/config`:

```json
"custom/ws_grid": {
    "format": "{}",
    "return-type": "json",
    "exec": "hyprgrid status",
    "tooltip": true
}
```

---

## 📖 CLI Command Reference

```bash
# Grid navigation
hyprgrid grid focus x 3          # Focus column 3
hyprgrid grid move y 2           # Move active window to row 2
hyprgrid grid move_all y 4       # Move all windows in current ws to row 4
hyprgrid grid focus x next       # Next occupied column in row
hyprgrid grid focus y prev       # Previous occupied row on monitor

# Shorthand syntax:
hyprgrid grid x next             # Same as: focus x next
hyprgrid grid y 3 move           # Same as: move y 3

# Workspace creation & shifting
hyprgrid create x empty          # Jump to first empty column in row
hyprgrid create y empty          # Jump to first completely unoccupied row
hyprgrid create x right          # Insert workspace and shift columns right
hyprgrid create x left           # Insert workspace and shift columns left
hyprgrid create y right          # Insert workspace and shift rows down
hyprgrid create y left           # Insert workspace and shift rows up

# Waybar status stream
hyprgrid status                  # Real-time event stream over socket2
hyprgrid status --once           # Print single JSON snapshot and exit

# Interactive TUI map
hyprgrid map                     # Launch interactive 2D terminal map
```

---

## 📜 License

MIT © [mestik78](https://github.com/mestik78)
