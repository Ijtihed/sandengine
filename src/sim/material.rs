#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Cell {
    #[default]
    Air = 0,
    Sand = 1,
    Block = 2,
    Water = 3,
    Stone = 4,
    Fire = 5,
    Steam = 6,
    Oil = 7,
    Gravel = 8,
    Acid = 9,
}

impl Cell {
    pub fn density(self) -> u8 {
        match self {
            Cell::Air => 0,
            Cell::Steam => 1,
            Cell::Fire => 2,
            Cell::Oil => 20,
            Cell::Water => 30,
            Cell::Sand => 50,
            Cell::Gravel => 70,
            Cell::Acid => 28,
            Cell::Stone => 100,
            Cell::Block => 100,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CellData {
    pub kind: Cell,
    pub extra: u8,
}

impl CellData {
    pub const AIR: Self = Self { kind: Cell::Air, extra: 0 };

    pub fn sand(seed: u8) -> Self { Self { kind: Cell::Sand, extra: seed } }
    pub fn gravel(seed: u8) -> Self { Self { kind: Cell::Gravel, extra: seed } }
    pub fn water(seed: u8) -> Self { Self { kind: Cell::Water, extra: seed } }
    pub fn oil(seed: u8) -> Self { Self { kind: Cell::Oil, extra: seed } }
    pub fn stone(seed: u8) -> Self { Self { kind: Cell::Stone, extra: seed } }
    pub fn fire(life: u8) -> Self { Self { kind: Cell::Fire, extra: life } }
    pub fn steam(life: u8) -> Self { Self { kind: Cell::Steam, extra: life } }
    pub fn acid(life: u8) -> Self { Self { kind: Cell::Acid, extra: life } }
    pub fn block() -> Self { Self { kind: Cell::Block, extra: 0 } }

    pub fn to_rgba(self) -> [u8; 4] {
        let e = self.extra;
        match self.kind {
            Cell::Air => [8, 8, 14, 255],

            Cell::Sand => {
                // Realistic desert sand: warm golden with per-grain variation
                // Uses 3 overlapping hue waves for natural randomness
                let h1 = ((e as u16).wrapping_mul(197)) as u8;
                let h2 = ((e as u16).wrapping_mul(53)) as u8;
                let r = (215i16 + (h1 as i16 - 128) / 6 + (h2 as i16 - 128) / 14).clamp(178, 242) as u8;
                let g = (192i16 + (h1 as i16 - 128) / 8 + (h2 as i16 - 128) / 16).clamp(155, 220) as u8;
                let b = (132i16 + (h1 as i16 - 128) / 12 + (h2 as i16 - 128) / 20).clamp(95, 168) as u8;
                [r, g, b, 255]
            }

            Cell::Gravel => {
                // Dark rocky gravel: cool grey-brown with speckle
                let h1 = ((e as u16).wrapping_mul(173)) as u8;
                let h2 = ((e as u16).wrapping_mul(89)) as u8;
                let speck = if h2 > 220 { 18i16 } else { 0 }; // occasional bright speckle
                let r = (118i16 + (h1 as i16 - 128) / 9 + speck).clamp(92, 148) as u8;
                let g = (108i16 + (h1 as i16 - 128) / 11 + speck).clamp(82, 138) as u8;
                let b = (98i16 + (h1 as i16 - 128) / 13 + speck).clamp(72, 128) as u8;
                [r, g, b, 255]
            }

            Cell::Water => {
                // Deep translucent blue with subtle variation
                let h = ((e as u16).wrapping_mul(131)) as u8;
                let r = (25i16 + (h as i16 - 128) / 20).clamp(15, 45) as u8;
                let g = (75i16 + (h as i16 - 128) / 10).clamp(55, 105) as u8;
                let b = (195i16 + (h as i16 - 128) / 8).clamp(160, 230) as u8;
                [r, g, b, 170]
            }

            Cell::Oil => {
                // Dark viscous amber-brown, slightly iridescent
                let h = ((e as u16).wrapping_mul(67)) as u8;
                let irid = (h as i16 - 128).abs() / 40;
                let r = (58i16 + (h as i16 - 128) / 14 + irid).clamp(38, 78) as u8;
                let g = (38i16 + (h as i16 - 128) / 18).clamp(22, 54) as u8;
                let b = (18i16 + (h as i16 - 128) / 24 + irid * 2).clamp(8, 38) as u8;
                [r, g, b, 220]
            }

            Cell::Stone => {
                // Layered grey granite with mineral flecks
                let h1 = ((e as u16).wrapping_mul(211)) as u8;
                let h2 = ((e as u16).wrapping_mul(97)) as u8;
                let fleck = if h2 > 235 { 22i16 } else if h2 < 20 { -12 } else { 0 };
                let base = (118i16 + (h1 as i16 - 128) / 8 + fleck).clamp(85, 148) as u8;
                [base, (base as i16 + 3).clamp(0, 255) as u8, (base as i16 + 8).clamp(0, 255) as u8, 255]
            }

            Cell::Fire => {
                // Hot ember to bright flame gradient based on lifetime
                let f = e as f32 / 120.0;
                let f2 = f * f; // quadratic falloff
                let r = (255.0 * f.min(1.0).max(0.4)) as u8;
                let g = (60.0 + 180.0 * f2.min(1.0)) as u8;
                let b = (15.0 + 45.0 * (f2 * f).min(1.0)) as u8;
                [r, g, b, 255]
            }

            Cell::Steam => {
                // Wispy white with fade
                let f = e as f32 / 80.0;
                let h = ((e as u16).wrapping_mul(151)) as u8;
                let tint = (h as i16 - 128) / 30;
                let v = (210i16 + tint).clamp(195, 225) as u8;
                let a = (160.0 * f.clamp(0.0, 1.0)) as u8;
                [v, v, (v as i16 + 8).min(255) as u8, a.max(15)]
            }

            Cell::Acid => {
                // Toxic neon green with pulsing glow based on life
                let f = e as f32 / 100.0;
                let pulse = ((e as f32 * 0.5).sin() * 0.5 + 0.5) * 0.15;
                let r = (30.0 + 30.0 * (1.0 - f) + pulse * 60.0) as u8;
                let g = (180.0 * f.clamp(0.25, 1.0) + pulse * 40.0).min(255.0) as u8;
                let b = (20.0 + 15.0 * pulse) as u8;
                [r, g, b, 205]
            }

            Cell::Block => {
                // Brushed dark metal
                [48, 52, 68, 255]
            }
        }
    }

    pub fn to_rgba_f32(self) -> [f32; 4] {
        let c = self.to_rgba();
        [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, c[3] as f32 / 255.0]
    }
}
