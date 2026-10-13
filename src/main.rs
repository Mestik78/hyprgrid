mod config;
mod coords;
mod create;
mod grid;
mod hypr;
mod map;
mod setup;
mod state;
mod waybar;

use std::env;
use std::process::ExitCode;

use config::Config;
use create::Direction;
use grid::{Action, Axis, Target};

fn print_help() {
    eprintln!(
        r#"hyprgrid — 3D Workspace Grid Manager for Hyprland

USAGE:
    hyprgrid <COMMAND> [ARGS...]

COMMANDS:
    grid [action] <axis> <target>    Navigate or move windows on the grid
    create <axis> <direction>        Create empty workspaces or shift grid
    status [--once]                  Output Waybar JSON status (stream or single)
    map                              Interactive 2D TUI grid map
    setup, install-keybinds          Generate & install Hyprland keybindings
    help, --help, -h                 Show this help message
    --version, -v                    Show version

GRID NAVIGATION:
    hyprgrid grid focus x <1..10|next|prev>   Focus column (or next/prev in row)
    hyprgrid grid focus y <1..10|next|prev>   Focus row (or next/prev row on monitor)
    hyprgrid grid move x <1..10|next|prev>    Move active window to column
    hyprgrid grid move y <1..10|next|prev>    Move active window to row
    hyprgrid grid move_all y <1..10>          Move all windows in current ws to row

    Shorthand syntax is also supported:
    hyprgrid grid x <target> [action]         e.g. hyprgrid grid x next
    hyprgrid grid y <target> [action]         e.g. hyprgrid grid y 3 move

WORKSPACE CREATION:
    hyprgrid create x empty          Jump to next empty column in current row
    hyprgrid create y empty          Jump to next completely empty row
    hyprgrid create x right|left     Insert workspace and shift columns right/left
    hyprgrid create y right|left     Insert workspace and shift rows down/up
"#
    );
}

fn parse_axis(s: &str) -> Result<Axis, String> {
    match s.to_lowercase().as_str() {
        "x" => Ok(Axis::X),
        "y" => Ok(Axis::Y),
        other => Err(format!("Invalid axis '{other}', expected 'x' or 'y'")),
    }
}

fn parse_action(s: &str) -> Result<Action, String> {
    match s.to_lowercase().as_str() {
        "focus" => Ok(Action::Focus),
        "move" => Ok(Action::Move),
        "move_all" | "moveall" => Ok(Action::MoveAll),
        other => Err(format!(
            "Invalid action '{other}', expected 'focus', 'move', or 'move_all'"
        )),
    }
}

fn parse_target(s: &str) -> Result<Target, String> {
    match s.to_lowercase().as_str() {
        "next" => Ok(Target::Next),
        "prev" | "previous" => Ok(Target::Prev),
        num => match num.parse::<u32>() {
            Ok(v) => Ok(Target::Absolute(v)),
            Err(_) => Err(format!(
                "Invalid target '{s}', expected a number or 'next'/'prev'"
            )),
        },
    }
}

fn parse_direction(s: &str) -> Result<Direction, String> {
    match s.to_lowercase().as_str() {
        "empty" => Ok(Direction::Empty),
        "right" => Ok(Direction::Right),
        "left" => Ok(Direction::Left),
        other => Err(format!(
            "Invalid direction '{other}', expected 'empty', 'right', or 'left'"
        )),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return ExitCode::FAILURE;
    }

    let cfg = Config::load();

    match args[1].as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "-v" | "--version" | "version" => {
            println!("hyprgrid {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "grid" => {
            if args.len() < 4 {
                eprintln!("Error: 'grid' requires axis and target arguments.");
                eprintln!("Usage: hyprgrid grid [action] <x|y> <target>");
                return ExitCode::FAILURE;
            }

            // Determine if arg2 is action or axis
            let (action, axis, target) = if args[2] == "focus"
                || args[2] == "move"
                || args[2] == "move_all"
                || args[2] == "moveall"
            {
                let act = match parse_action(&args[2]) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let ax = match parse_axis(&args[3]) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                if args.len() < 5 {
                    eprintln!("Error: missing target (number, 'next', or 'prev')");
                    return ExitCode::FAILURE;
                }
                let tg = match parse_target(&args[4]) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                (act, ax, tg)
            } else {
                // Syntax: hyprgrid grid <x|y> <target> [action]
                let ax = match parse_axis(&args[2]) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let tg = match parse_target(&args[3]) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                };
                let act = if args.len() >= 5 {
                    match parse_action(&args[4]) {
                        Ok(a) => a,
                        Err(e) => {
                            eprintln!("Error: {e}");
                            return ExitCode::FAILURE;
                        }
                    }
                } else {
                    Action::Focus
                };
                (act, ax, tg)
            };

            if let Err(e) = grid::run(&cfg, action, axis, target) {
                eprintln!("hyprgrid grid error: {e}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        "create" => {
            if args.len() < 4 {
                eprintln!("Error: 'create' requires axis and direction.");
                eprintln!("Usage: hyprgrid create <x|y> <empty|right|left>");
                return ExitCode::FAILURE;
            }
            let axis = match parse_axis(&args[2]) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let dir = match parse_direction(&args[3]) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(e) = create::run(&cfg, axis, dir) {
                eprintln!("hyprgrid create error: {e}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        "status" | "waybar" => {
            let once = args.iter().any(|a| a == "--once" || a == "-1");
            let res = if once {
                waybar::print_status(&cfg)
            } else {
                waybar::run_stream(&cfg)
            };

            if let Err(e) = res {
                eprintln!("hyprgrid status error: {e}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        "map" => {
            if let Err(e) = map::run(&cfg) {
                eprintln!("hyprgrid map error: {e}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        "setup" | "install-keybinds" | "init" => {
            if let Err(e) = setup::run(&args[2..]) {
                eprintln!("hyprgrid setup error: {e}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        other => {
            eprintln!("Unknown command '{other}'. Run 'hyprgrid --help' for usage.");
            ExitCode::FAILURE
        }
    }
}
