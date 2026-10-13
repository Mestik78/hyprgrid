use crate::config::Config;

/// 3D workspace coordinate in the hyprgrid model.
///
/// - `m`: Monitor index (Z dimension), 0-based
/// - `y`: Row index (Y dimension), 0-based (displayed as 1-based)
/// - `x`: Column index (X dimension), 1-based
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord3D {
    pub m: u32,
    pub y: u32,
    pub x: u32,
}

impl Coord3D {
    /// Encode 3D coordinates to a Hyprland workspace ID.
    ///
    /// Formula: `M × capacity + Y × cols + X`
    /// where `capacity = rows × cols`.
    pub fn to_ws_id(&self, cfg: &Config) -> u32 {
        let cap = cfg.grid.capacity().max(1);
        let cols = cfg.grid.cols.max(1);
        let m = if cfg.monitors.isolate { self.m } else { 0 };
        m * cap + self.y * cols + self.x.max(1)
    }

    /// Decode a Hyprland workspace ID to 3D coordinates.
    pub fn from_ws_id(id: u32, cfg: &Config) -> Self {
        let id = id.max(1);
        let cap = cfg.grid.capacity().max(1);
        let cols = cfg.grid.cols.max(1);
        let m = if cfg.monitors.isolate { (id - 1) / cap } else { 0 };
        let local = (id - 1) % cap + 1;
        let y = (local - 1) / cols;
        let x = (local - 1) % cols + 1;
        Self { m, y, x }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, GridConfig, MonitorConfig};

    fn cfg(rows: u32, cols: u32) -> Config {
        Config {
            grid: GridConfig { rows, cols },
            monitors: MonitorConfig { isolate: true, order: vec![] },
            waybar: Default::default(),
        }
    }

    #[test]
    fn default_10x10_roundtrip() {
        let c = cfg(10, 10);
        // Monitor 0, Row 0, Col 1 → ws 1
        assert_eq!(Coord3D { m: 0, y: 0, x: 1 }.to_ws_id(&c), 1);
        assert_eq!(Coord3D::from_ws_id(1, &c), Coord3D { m: 0, y: 0, x: 1 });

        // Monitor 0, Row 0, Col 10 → ws 10
        assert_eq!(Coord3D { m: 0, y: 0, x: 10 }.to_ws_id(&c), 10);
        assert_eq!(Coord3D::from_ws_id(10, &c), Coord3D { m: 0, y: 0, x: 10 });

        // Monitor 0, Row 1, Col 1 → ws 11
        assert_eq!(Coord3D { m: 0, y: 1, x: 1 }.to_ws_id(&c), 11);
        assert_eq!(Coord3D::from_ws_id(11, &c), Coord3D { m: 0, y: 1, x: 1 });

        // Monitor 0, Row 9, Col 10 → ws 100
        assert_eq!(Coord3D { m: 0, y: 9, x: 10 }.to_ws_id(&c), 100);
        assert_eq!(Coord3D::from_ws_id(100, &c), Coord3D { m: 0, y: 9, x: 10 });

        // Monitor 1, Row 0, Col 1 → ws 101
        assert_eq!(Coord3D { m: 1, y: 0, x: 1 }.to_ws_id(&c), 101);
        assert_eq!(Coord3D::from_ws_id(101, &c), Coord3D { m: 1, y: 0, x: 1 });

        // Monitor 1, Row 9, Col 10 → ws 200
        assert_eq!(Coord3D { m: 1, y: 9, x: 10 }.to_ws_id(&c), 200);
        assert_eq!(Coord3D::from_ws_id(200, &c), Coord3D { m: 1, y: 9, x: 10 });
    }

    #[test]
    fn custom_5x8_roundtrip() {
        let c = cfg(5, 8); // capacity = 40

        assert_eq!(Coord3D { m: 0, y: 0, x: 1 }.to_ws_id(&c), 1);
        assert_eq!(Coord3D { m: 0, y: 0, x: 8 }.to_ws_id(&c), 8);
        assert_eq!(Coord3D { m: 0, y: 1, x: 1 }.to_ws_id(&c), 9);
        assert_eq!(Coord3D { m: 0, y: 4, x: 8 }.to_ws_id(&c), 40);
        assert_eq!(Coord3D { m: 1, y: 0, x: 1 }.to_ws_id(&c), 41);

        assert_eq!(Coord3D::from_ws_id(9, &c), Coord3D { m: 0, y: 1, x: 1 });
        assert_eq!(Coord3D::from_ws_id(40, &c), Coord3D { m: 0, y: 4, x: 8 });
        assert_eq!(Coord3D::from_ws_id(41, &c), Coord3D { m: 1, y: 0, x: 1 });
    }
}
