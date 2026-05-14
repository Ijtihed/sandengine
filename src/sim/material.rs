#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Cell {
    #[default]
    Air = 0,
    Sand = 1,
    Block = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CellData {
    pub kind: Cell,
    pub color_seed: u8,
}

impl CellData {
    pub const AIR: Self = Self {
        kind: Cell::Air,
        color_seed: 0,
    };

    pub fn sand(seed: u8) -> Self {
        Self {
            kind: Cell::Sand,
            color_seed: seed,
        }
    }

    pub fn block() -> Self {
        Self {
            kind: Cell::Block,
            color_seed: 0,
        }
    }

    pub fn to_rgba(self) -> [u8; 4] {
        match self.kind {
            Cell::Air => [20, 20, 25, 255],
            Cell::Sand => {
                let s = self.color_seed;
                let warm = (s as u16 * 3) % 256;
                let r = (200 + (warm as i16 - 128) / 6).clamp(160, 230) as u8;
                let g = (180 + (warm as i16 - 128) / 8).clamp(140, 210) as u8;
                let b = (120 + (warm as i16 - 128) / 12).clamp(80, 160) as u8;
                [r, g, b, 255]
            }
            Cell::Block => [70, 75, 85, 255],
        }
    }

    pub fn to_rgba_f32(self) -> [f32; 4] {
        let c = self.to_rgba();
        [
            c[0] as f32 / 255.0,
            c[1] as f32 / 255.0,
            c[2] as f32 / 255.0,
            c[3] as f32 / 255.0,
        ]
    }
}
