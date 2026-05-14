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
}

impl Cell {
    pub fn is_solid(self) -> bool {
        matches!(self, Cell::Sand | Cell::Stone | Cell::Block)
    }
    pub fn is_liquid(self) -> bool {
        matches!(self, Cell::Water)
    }
    pub fn is_gas(self) -> bool {
        matches!(self, Cell::Fire | Cell::Steam | Cell::Air)
    }
    pub fn density(self) -> u8 {
        match self {
            Cell::Air => 0,
            Cell::Steam => 1,
            Cell::Fire => 2,
            Cell::Water => 30,
            Cell::Sand => 50,
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
    pub fn water(seed: u8) -> Self { Self { kind: Cell::Water, extra: seed } }
    pub fn stone(seed: u8) -> Self { Self { kind: Cell::Stone, extra: seed } }
    pub fn fire(life: u8) -> Self { Self { kind: Cell::Fire, extra: life } }
    pub fn steam(life: u8) -> Self { Self { kind: Cell::Steam, extra: life } }
    pub fn block() -> Self { Self { kind: Cell::Block, extra: 0 } }

    pub fn to_rgba(self) -> [u8; 4] {
        match self.kind {
            Cell::Air => [15, 15, 20, 255],
            Cell::Sand => {
                let s = self.extra;
                let w = (s as u16 * 3) % 256;
                let r = (205 + (w as i16 - 128) / 5).clamp(165, 235) as u8;
                let g = (185 + (w as i16 - 128) / 7).clamp(145, 215) as u8;
                let b = (125 + (w as i16 - 128) / 11).clamp(85, 160) as u8;
                [r, g, b, 255]
            }
            Cell::Water => {
                let d = self.extra;
                let depth = (d as u16 * 7) % 64;
                let r = (30 + depth / 4) as u8;
                let g = (80 + depth / 2) as u8;
                let b = (180 + depth / 3).min(240) as u8;
                [r, g, b, 200]
            }
            Cell::Stone => {
                let v = (self.extra as i16 - 128) / 10;
                let base = (110 + v).clamp(90, 130) as u8;
                [base, (base as i16 + 2).min(255) as u8, (base as i16 + 5).min(255) as u8, 255]
            }
            Cell::Fire => {
                let life_frac = self.extra as f32 / 120.0;
                let r = (255.0 * life_frac.min(1.0)) as u8;
                let g = (200.0 * (life_frac * 0.6).min(1.0)) as u8;
                let b = (40.0 * (life_frac * 0.2)) as u8;
                [r.max(60), g, b, 255]
            }
            Cell::Steam => {
                let life_frac = self.extra as f32 / 80.0;
                let a = (180.0 * life_frac.clamp(0.0, 1.0)) as u8;
                [200, 200, 210, a.max(30)]
            }
            Cell::Block => [60, 65, 80, 255],
        }
    }

    pub fn to_rgba_f32(self) -> [f32; 4] {
        let c = self.to_rgba();
        [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, c[3] as f32 / 255.0]
    }
}
