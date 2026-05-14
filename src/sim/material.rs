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
    pub fn is_pushable(self) -> bool {
        !matches!(self, Cell::Air | Cell::Block | Cell::Stone)
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
        match self.kind {
            Cell::Air => [12, 12, 18, 255],
            Cell::Sand => {
                let w = (self.extra as u16 * 3) % 256;
                let r = (210 + (w as i16 - 128) / 5).clamp(170, 240) as u8;
                let g = (190 + (w as i16 - 128) / 7).clamp(148, 218) as u8;
                let b = (128 + (w as i16 - 128) / 11).clamp(88, 162) as u8;
                [r, g, b, 255]
            }
            Cell::Gravel => {
                let w = (self.extra as u16 * 5) % 256;
                let r = (130 + (w as i16 - 128) / 8).clamp(105, 155) as u8;
                let g = (115 + (w as i16 - 128) / 10).clamp(90, 140) as u8;
                let b = (95 + (w as i16 - 128) / 12).clamp(72, 118) as u8;
                [r, g, b, 255]
            }
            Cell::Water => {
                let d = (self.extra as u16 * 7) % 64;
                [30 + (d / 4) as u8, 80 + (d / 2) as u8, (185 + d / 3).min(240) as u8, 190]
            }
            Cell::Oil => {
                let d = (self.extra as u16 * 3) % 32;
                [45 + d as u8, 30 + (d / 2) as u8, 15 + (d / 3) as u8, 210]
            }
            Cell::Stone => {
                let v = (self.extra as i16 - 128) / 10;
                let b = (112 + v).clamp(92, 132) as u8;
                [b, (b as u16 + 2).min(255) as u8, (b as u16 + 6).min(255) as u8, 255]
            }
            Cell::Fire => {
                let f = self.extra as f32 / 120.0;
                let r = (255.0 * f.min(1.0)) as u8;
                let g = (210.0 * (f * 0.7).min(1.0)) as u8;
                let b = (60.0 * (f * 0.3)) as u8;
                [r.max(80), g, b, 255]
            }
            Cell::Steam => {
                let f = self.extra as f32 / 80.0;
                let a = (180.0 * f.clamp(0.0, 1.0)) as u8;
                [205, 205, 215, a.max(25)]
            }
            Cell::Acid => {
                let f = self.extra as f32 / 100.0;
                let g = (220.0 * f.clamp(0.3, 1.0)) as u8;
                [40, g, 30, 200]
            }
            Cell::Block => [55, 60, 78, 255],
        }
    }

    pub fn to_rgba_f32(self) -> [f32; 4] {
        let c = self.to_rgba();
        [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0, c[3] as f32 / 255.0]
    }
}
