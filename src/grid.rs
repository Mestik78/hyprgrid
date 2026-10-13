use crate::config::Config;
use crate::coords::Coord3D;
use crate::hypr::{self, AnimStyle};
use crate::state;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Focus,
    Move,
    MoveAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Absolute(u32),
    Next,
    Prev,
}

pub fn run(cfg: &Config, action: Action, axis: Axis, target: Target) -> Result<(), String> {
    let current_ws = hypr::active_workspace()?;
    let current_id = (current_ws.id.max(1)) as u32;
    let m = hypr::resolve_monitor_index(cfg, &current_ws.monitor);

    let coord = Coord3D::from_ws_id(current_id, cfg);
    let t = coord.y; // 0-indexed row
    let u = coord.x; // 1-indexed col

    let new_id = match (axis, target) {
        // ── Absolute X (column jump) ─────────────────────────────────────────
        (Axis::X, Target::Absolute(val)) => {
            let new_u = val.clamp(1, cfg.grid.cols);
            state::save_column(m, t, new_u);
            Coord3D { m, y: t, x: new_u }.to_ws_id(cfg)
        }

        // ── Absolute Y (row jump with column memory) ─────────────────────────
        (Axis::Y, Target::Absolute(val)) => {
            let new_t = val.clamp(1, cfg.grid.rows) - 1;
            state::save_column(m, t, u);

            let remembered_u = state::load_column(m, new_t)
                .filter(|&c| c >= 1 && c <= cfg.grid.cols)
                .unwrap_or(1);

            Coord3D {
                m,
                y: new_t,
                x: remembered_u,
            }
            .to_ws_id(cfg)
        }

        // ── Relative X (cycle through occupied cols in current row) ──────────
        (Axis::X, Target::Next) | (Axis::X, Target::Prev) => {
            let is_next = matches!(target, Target::Next);
            let existing = hypr::monitor_workspaces(cfg, &current_ws.monitor)?;

            let row_start = Coord3D { m, y: t, x: 1 }.to_ws_id(cfg);
            let row_end = Coord3D {
                m,
                y: t,
                x: cfg.grid.cols,
            }
            .to_ws_id(cfg);

            let mut cands: Vec<u32> = existing
                .into_iter()
                .filter(|&w| w >= row_start && w <= row_end)
                .collect();

            if !cands.contains(&current_id) {
                cands.push(current_id);
            }
            cands.sort_unstable();

            if cands.is_empty() {
                return Ok(());
            }

            let idx = cands.iter().position(|&w| w == current_id).unwrap_or(0);
            let len = cands.len();
            let next_idx = if is_next {
                (idx + 1) % len
            } else {
                (idx + len - 1) % len
            };
            cands[next_idx]
        }

        // ── Relative Y (cycle through occupied rows on this monitor) ─────────
        (Axis::Y, Target::Next) | (Axis::Y, Target::Prev) => {
            let is_next = matches!(target, Target::Next);
            state::save_column(m, t, u);

            let existing = hypr::monitor_workspaces(cfg, &current_ws.monitor)?;

            let mut rows: Vec<u32> = existing
                .iter()
                .map(|&w| Coord3D::from_ws_id(w, cfg).y)
                .collect();

            if !rows.contains(&t) {
                rows.push(t);
            }
            rows.sort_unstable();
            rows.dedup();

            if rows.len() <= 1 {
                return Ok(());
            }

            let idx = rows.iter().position(|&r| r == t).unwrap_or(0);
            let len = rows.len();
            let next_idx = if is_next {
                (idx + 1) % len
            } else {
                (idx + len - 1) % len
            };
            let next_t = rows[next_idx];

            if next_t == t {
                return Ok(());
            }

            // Try to restore last visited column in that row
            let mut target_id = None;
            if let Some(rem_u) = state::load_column(m, next_t) {
                if rem_u >= 1 && rem_u <= cfg.grid.cols {
                    let cand_id = Coord3D {
                        m,
                        y: next_t,
                        x: rem_u,
                    }
                    .to_ws_id(cfg);
                    if existing.contains(&cand_id) {
                        target_id = Some(cand_id);
                    }
                }
            }

            // Fallback: first existing workspace in that row
            if target_id.is_none() {
                for &w in &existing {
                    if Coord3D::from_ws_id(w, cfg).y == next_t {
                        target_id = Some(w);
                        break;
                    }
                }
            }

            target_id.unwrap_or_else(|| {
                Coord3D {
                    m,
                    y: next_t,
                    x: 1,
                }
                .to_ws_id(cfg)
            })
        }
    };

    if new_id == current_id {
        return Ok(());
    }

    // Set animation style matching movement axis
    match axis {
        Axis::Y => hypr::set_animation(AnimStyle::SlideVert)?,
        Axis::X => hypr::set_animation(AnimStyle::Slide)?,
    }

    // Dispatch action
    match action {
        Action::Focus => {
            hypr::focus_workspace(new_id)?;
        }
        Action::Move => {
            hypr::move_window_to(new_id)?;
        }
        Action::MoveAll => {
            let clients = hypr::clients()?;
            for c in clients {
                if c.workspace.id == current_id as i32 {
                    let _ = hypr::move_window_addr_to(&c.address, new_id);
                }
            }
            hypr::focus_workspace(new_id)?;
        }
    }

    Ok(())
}
