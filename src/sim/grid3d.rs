use super::material::{Cell, CellData};

pub struct Grid3D {
    pub sx: usize,
    pub sy: usize,
    pub sz: usize,
    pub cells: Vec<CellData>,
    pub next: Vec<CellData>,
    pub moved: Vec<bool>,
}

impl Grid3D {
    pub fn new(sx: usize, sy: usize, sz: usize) -> Self {
        let n = sx * sy * sz;
        Self {
            sx,
            sy,
            sz,
            cells: vec![CellData::AIR; n],
            next: vec![CellData::AIR; n],
            moved: vec![false; n],
        }
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + z * self.sx + y * self.sx * self.sz
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && x < self.sx as i32
            && y >= 0
            && y < self.sy as i32
            && z >= 0
            && z < self.sz as i32
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> CellData {
        self.cells[self.idx(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, cell: CellData) {
        let i = self.idx(x, y, z);
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

    pub fn is_surface(&self, x: usize, y: usize, z: usize) -> bool {
        let dirs: [(i32, i32, i32); 6] = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        for (dx, dy, dz) in dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;
            if !self.in_bounds(nx, ny, nz) {
                return true;
            }
            if self.cells[self.idx(nx as usize, ny as usize, nz as usize)].kind == Cell::Air {
                return true;
            }
        }
        false
    }
}
