use crate::config::Config;
use crate::coords::Coord3D;
use crate::grid::Axis;
use crate::hypr::{self, AnimStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Empty,
    Right,
    Left,
}

fn move_all_windows(src: u32, dst: u32) -> Result<(), String> {
    let clients = hypr::clients()?;
    for c in clients {
        if c.workspace.id == src as i32 && !c.pinned {
            let _ = hypr::move_window_addr_to(&c.address, dst);
        }
    }
    Ok(())
}

pub fn run(cfg: &Config, axis: Axis, direction: Direction) -> Result<(), String> {
    let current_ws = hypr::active_workspace()?;
    if current_ws.id <= 0 || current_ws.name.starts_with("special:") {
        return Err("Cannot perform workspace creation or shifting from a special workspace.".into());
    }

    let current_id = current_ws.id as u32;
    let current_mon = &current_ws.monitor;
    let m = hypr::resolve_monitor_index(cfg, current_mon);

    let coord = Coord3D::from_ws_id(current_id, cfg);
    let t = coord.y; // 0-indexed row
    let u = coord.x; // 1-indexed col

    // Animation style
    match axis {
        Axis::Y => hypr::set_animation(AnimStyle::SlideVert)?,
        Axis::X => hypr::set_animation(AnimStyle::Slide)?,
    }

    let all_existing = hypr::monitor_workspaces(cfg, current_mon)?;
    let cols = cfg.grid.cols.max(1);
    let rows = cfg.grid.rows.max(1);

    if direction == Direction::Empty {
        match axis {
            Axis::X => {
                for i in 1..=cols {
                    let cand = Coord3D { m, y: t, x: i }.to_ws_id(cfg);
                    if !all_existing.contains(&cand) && cand != current_id {
                        hypr::focus_workspace(cand)?;
                        return Ok(());
                    }
                }
                hypr::notify("Row is full.");
            }
            Axis::Y => {
                for check_t in 0..rows {
                    let mut is_empty = true;
                    for check_u in 1..=cols {
                        let cand = Coord3D {
                            m,
                            y: check_t,
                            x: check_u,
                        }
                        .to_ws_id(cfg);
                        if all_existing.contains(&cand) || cand == current_id {
                            is_empty = false;
                            break;
                        }
                    }
                    if is_empty {
                        let new_id = Coord3D {
                            m,
                            y: check_t,
                            x: 1,
                        }
                        .to_ws_id(cfg);
                        hypr::focus_workspace(new_id)?;
                        return Ok(());
                    }
                }
                hypr::notify("All row groups are occupied.");
            }
        }
        return Ok(());
    }

    match axis {
        Axis::X => {
            if cols <= 1 {
                return Ok(());
            }

            let mut count = 0;
            for i in 1..=cols {
                let cand = Coord3D { m, y: t, x: i }.to_ws_id(cfg);
                if all_existing.contains(&cand) || cand == current_id {
                    count += 1;
                }
            }

            if count >= cols {
                hypr::notify("Error: Row is full, cannot shift.");
                return Err("Row full".into());
            }

            let mut found_empty_u = None;
            if direction == Direction::Right {
                for step in 1..cols {
                    let check_u = (u - 1 + step) % cols + 1;
                    let cand = Coord3D { m, y: t, x: check_u }.to_ws_id(cfg);
                    if !all_existing.contains(&cand) {
                        found_empty_u = Some(check_u);
                        break;
                    }
                }

                let empty_u = match found_empty_u {
                    Some(v) => v,
                    None => {
                        hypr::notify("Error: No free slot available to shift.");
                        return Err("No empty slot found".into());
                    }
                };

                let target_u_0 = u % cols;
                let mut curr_step_0 = empty_u - 1;
                while curr_step_0 != target_u_0 {
                    let prev_step_0 = (curr_step_0 + cols - 1) % cols;
                    let src = Coord3D { m, y: t, x: prev_step_0 + 1 }.to_ws_id(cfg);
                    let dst = Coord3D { m, y: t, x: curr_step_0 + 1 }.to_ws_id(cfg);
                    move_all_windows(src, dst)?;
                    curr_step_0 = prev_step_0;
                }
                let target_ws = Coord3D { m, y: t, x: target_u_0 + 1 }.to_ws_id(cfg);
                hypr::focus_workspace(target_ws)?;
            } else if direction == Direction::Left {
                for step in 1..cols {
                    let check_u = (u - 1 + cols - (step % cols)) % cols + 1;
                    let cand = Coord3D { m, y: t, x: check_u }.to_ws_id(cfg);
                    if !all_existing.contains(&cand) {
                        found_empty_u = Some(check_u);
                        break;
                    }
                }

                let empty_u = match found_empty_u {
                    Some(v) => v,
                    None => {
                        hypr::notify("Error: No free slot available to shift.");
                        return Err("No empty slot found".into());
                    }
                };

                let target_u_0 = (u + cols - 2) % cols;
                let mut curr_step_0 = empty_u - 1;
                while curr_step_0 != target_u_0 {
                    let prev_step_0 = (curr_step_0 + 1) % cols;
                    let src = Coord3D { m, y: t, x: prev_step_0 + 1 }.to_ws_id(cfg);
                    let dst = Coord3D { m, y: t, x: curr_step_0 + 1 }.to_ws_id(cfg);
                    move_all_windows(src, dst)?;
                    curr_step_0 = prev_step_0;
                }
                let target_ws = Coord3D { m, y: t, x: target_u_0 + 1 }.to_ws_id(cfg);
                hypr::focus_workspace(target_ws)?;
            }
        }

        Axis::Y => {
            if rows <= 1 {
                return Ok(());
            }

            let mut count = 0;
            for check_t in 0..rows {
                let mut is_occ = false;
                for check_u in 1..=cols {
                    let cand = Coord3D {
                        m,
                        y: check_t,
                        x: check_u,
                    }
                    .to_ws_id(cfg);
                    if all_existing.contains(&cand) || cand == current_id {
                        is_occ = true;
                        break;
                    }
                }
                if is_occ {
                    count += 1;
                }
            }

            if count >= rows {
                hypr::notify("Error: All rows are occupied.");
                return Err("All rows full".into());
            }

            let mut found_empty_t = None;
            if direction == Direction::Right {
                for step in 1..rows {
                    let check_t = (t + step) % rows;
                    let mut is_occ = false;
                    for check_u in 1..=cols {
                        let cand = Coord3D {
                            m,
                            y: check_t,
                            x: check_u,
                        }
                        .to_ws_id(cfg);
                        if all_existing.contains(&cand) {
                            is_occ = true;
                            break;
                        }
                    }
                    if !is_occ {
                        found_empty_t = Some(check_t);
                        break;
                    }
                }

                let empty_t = match found_empty_t {
                    Some(v) => v,
                    None => {
                        hypr::notify("Error: No free row available to shift.");
                        return Err("No empty row found".into());
                    }
                };

                let target_t = (t + 1) % rows;
                let mut curr_step = empty_t;
                while curr_step != target_t {
                    let prev_step = (curr_step + rows - 1) % rows;
                    for step_u in 1..=cols {
                        let src = Coord3D { m, y: prev_step, x: step_u }.to_ws_id(cfg);
                        let dst = Coord3D { m, y: curr_step, x: step_u }.to_ws_id(cfg);
                        move_all_windows(src, dst)?;
                    }
                    curr_step = prev_step;
                }
                let target_ws = Coord3D { m, y: target_t, x: u }.to_ws_id(cfg);
                hypr::focus_workspace(target_ws)?;
            } else if direction == Direction::Left {
                for step in 1..rows {
                    let check_t = (t + rows - (step % rows)) % rows;
                    let mut is_occ = false;
                    for check_u in 1..=cols {
                        let cand = Coord3D {
                            m,
                            y: check_t,
                            x: check_u,
                        }
                        .to_ws_id(cfg);
                        if all_existing.contains(&cand) {
                            is_occ = true;
                            break;
                        }
                    }
                    if !is_occ {
                        found_empty_t = Some(check_t);
                        break;
                    }
                }

                let empty_t = match found_empty_t {
                    Some(v) => v,
                    None => {
                        hypr::notify("Error: No free row available to shift.");
                        return Err("No empty row found".into());
                    }
                };

                let target_t = (t + rows - 1) % rows;
                let mut curr_step = empty_t;
                while curr_step != target_t {
                    let prev_step = (curr_step + 1) % rows;
                    for step_u in 1..=cols {
                        let src = Coord3D { m, y: prev_step, x: step_u }.to_ws_id(cfg);
                        let dst = Coord3D { m, y: curr_step, x: step_u }.to_ws_id(cfg);
                        move_all_windows(src, dst)?;
                    }
                    curr_step = prev_step;
                }
                let target_ws = Coord3D { m, y: target_t, x: u }.to_ws_id(cfg);
                hypr::focus_workspace(target_ws)?;
            }
        }
    }

    Ok(())
}
