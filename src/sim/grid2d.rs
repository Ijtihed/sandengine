use super::material::{Cell, CellData};

pub struct Grid2D {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<CellData>,
    pub next: Vec<CellData>,
    pub moved: Vec<bool>,
}

impl Grid2D {
    pub fn new(w: usize, h: usize) -> Self {
        let n = w * h;
        Self {
            w,
            h,
            cells: vec![CellData::AIR; n],
            next: vec![CellData::AIR; n],
            moved: vec![false; n],
        }
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.w as i32 && y >= 0 && y < self.h as i32
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> CellData {
        self.cells[self.idx(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: CellData) {
        let i = self.idx(x, y);
        self.cells[i] = cell;
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.cells, &mut self.next);
        self.moved.fill(false);
    }

    pub fn clear(&mut self) {
        self.cells.fill(CellData::AIR);
        self.next.fill(CellData::AIR);
        self.moved.fill(false);
    }

    pub fn particle_count(&self) -> usize {
        self.cells.iter().filter(|c| c.kind != Cell::Air && c.kind != Cell::Block).count()
    }
}
