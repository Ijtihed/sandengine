use super::grid2d::Grid2D;
use super::grid3d::Grid3D;
use super::material::{Cell, CellData};
use glam::{Vec2, Vec3};
use std::collections::VecDeque;

// --- 2D Block ---

#[derive(Clone)]
pub struct Block2D {
    pub pos: Vec2,
    pub half_size: Vec2,
    pub grabbed: bool,
    pub grab_offset: Vec2,
}

impl Block2D {
    pub fn new(cx: f32, cy: f32, hw: f32, hh: f32) -> Self {
        Self {
            pos: Vec2::new(cx, cy),
            half_size: Vec2::new(hw, hh),
            grabbed: false,
            grab_offset: Vec2::ZERO,
        }
    }

    pub fn min(&self) -> (i32, i32) {
        (
            (self.pos.x - self.half_size.x).floor() as i32,
            (self.pos.y - self.half_size.y).floor() as i32,
        )
    }

    pub fn max(&self) -> (i32, i32) {
        (
            (self.pos.x + self.half_size.x).ceil() as i32,
            (self.pos.y + self.half_size.y).ceil() as i32,
        )
    }

    fn covered_cells(&self, w: usize, h: usize) -> Vec<(usize, usize)> {
        let (minx, miny) = self.min();
        let (maxx, maxy) = self.max();
        let mut out = Vec::new();
        for y in miny.max(0)..maxy.min(h as i32) {
            for x in minx.max(0)..maxx.min(w as i32) {
                out.push((x as usize, y as usize));
            }
        }
        out
    }

    pub fn contains(&self, gx: f32, gy: f32) -> bool {
        let dx = (gx - self.pos.x).abs();
        let dy = (gy - self.pos.y).abs();
        dx <= self.half_size.x && dy <= self.half_size.y
    }

    pub fn rasterize(&self, grid: &mut Grid2D) {
        let cells = self.covered_cells(grid.w, grid.h);
        for (x, y) in cells {
            let i = grid.idx(x, y);
            grid.cells[i] = CellData::block();
        }
    }

    pub fn clear_raster(&self, grid: &mut Grid2D) {
        let cells = self.covered_cells(grid.w, grid.h);
        for (x, y) in cells {
            let i = grid.idx(x, y);
            if grid.cells[i].kind == Cell::Block {
                grid.cells[i] = CellData::AIR;
            }
        }
    }

    pub fn move_and_displace(&mut self, new_pos: Vec2, grid: &mut Grid2D) {
        let old_cells = self.covered_cells(grid.w, grid.h);
        let old_pos = self.pos;
        self.pos = new_pos;
        let new_cells = self.covered_cells(grid.w, grid.h);

        // Clear old block cells
        for &(x, y) in &old_cells {
            let i = grid.idx(x, y);
            if grid.cells[i].kind == Cell::Block {
                grid.cells[i] = CellData::AIR;
            }
        }

        // Find invaded cells that contain sand
        let new_set: std::collections::HashSet<(usize, usize)> =
            new_cells.iter().copied().collect();

        let mut failed = false;
        for &(x, y) in &new_cells {
            let i = grid.idx(x, y);
            if grid.cells[i].kind == Cell::Sand {
                let cell_data = grid.cells[i];
                grid.cells[i] = CellData::AIR;
                if !bfs_push_2d(grid, x, y, &new_set, cell_data) {
                    grid.cells[i] = cell_data;
                    failed = true;
                    break;
                }
            }
        }

        if failed {
            // Revert: clear any block cells we set, restore old position
            self.pos = old_pos;
            let reverted = self.covered_cells(grid.w, grid.h);
            for &(x, y) in &reverted {
                let i = grid.idx(x, y);
                if grid.cells[i].kind == Cell::Block {
                    grid.cells[i] = CellData::AIR;
                }
            }
        }

        self.rasterize(grid);
    }
}

fn bfs_push_2d(
    grid: &mut Grid2D,
    sx: usize,
    sy: usize,
    blocked: &std::collections::HashSet<(usize, usize)>,
    cell_data: CellData,
) -> bool {
    let mut queue = VecDeque::new();
    let mut visited = std::collections::HashSet::new();
    queue.push_back((sx, sy));
    visited.insert((sx, sy));

    let dirs: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];
    let max_search = 400;
    let mut steps = 0;

    while let Some((cx, cy)) = queue.pop_front() {
        steps += 1;
        if steps > max_search {
            return false;
        }
        for (dx, dy) in dirs {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if !grid.in_bounds(nx, ny) {
                continue;
            }
            let (ux, uy) = (nx as usize, ny as usize);
            if visited.contains(&(ux, uy)) || blocked.contains(&(ux, uy)) {
                continue;
            }
            visited.insert((ux, uy));
            let ni = grid.idx(ux, uy);
            if grid.cells[ni].kind == Cell::Air {
                grid.cells[ni] = cell_data;
                return true;
            }
            if grid.cells[ni].kind == Cell::Sand {
                queue.push_back((ux, uy));
            }
        }
    }
    false
}

// --- 3D Block ---

#[derive(Clone)]
pub struct Block3D {
    pub pos: Vec3,
    pub half_size: Vec3,
    pub grabbed: bool,
    pub grab_offset: Vec3,
}

impl Block3D {
    pub fn new(cx: f32, cy: f32, cz: f32, hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            pos: Vec3::new(cx, cy, cz),
            half_size: Vec3::new(hx, hy, hz),
            grabbed: false,
            grab_offset: Vec3::ZERO,
        }
    }

    pub fn min(&self) -> (i32, i32, i32) {
        (
            (self.pos.x - self.half_size.x).floor() as i32,
            (self.pos.y - self.half_size.y).floor() as i32,
            (self.pos.z - self.half_size.z).floor() as i32,
        )
    }

    pub fn max(&self) -> (i32, i32, i32) {
        (
            (self.pos.x + self.half_size.x).ceil() as i32,
            (self.pos.y + self.half_size.y).ceil() as i32,
            (self.pos.z + self.half_size.z).ceil() as i32,
        )
    }

    fn covered_cells(&self, sx: usize, sy: usize, sz: usize) -> Vec<(usize, usize, usize)> {
        let (minx, miny, minz) = self.min();
        let (maxx, maxy, maxz) = self.max();
        let mut out = Vec::new();
        for y in miny.max(0)..maxy.min(sy as i32) {
            for z in minz.max(0)..maxz.min(sz as i32) {
                for x in minx.max(0)..maxx.min(sx as i32) {
                    out.push((x as usize, y as usize, z as usize));
                }
            }
        }
        out
    }

    pub fn contains_point(&self, p: Vec3) -> bool {
        let d = (p - self.pos).abs();
        d.x <= self.half_size.x && d.y <= self.half_size.y && d.z <= self.half_size.z
    }

    pub fn ray_intersect(&self, origin: Vec3, dir: Vec3) -> Option<f32> {
        let bmin = self.pos - self.half_size;
        let bmax = self.pos + self.half_size;
        let inv = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);

        let t1 = (bmin - origin) * inv;
        let t2 = (bmax - origin) * inv;

        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        let t_enter = tmin.x.max(tmin.y).max(tmin.z);
        let t_exit = tmax.x.min(tmax.y).min(tmax.z);

        if t_enter <= t_exit && t_exit >= 0.0 {
            Some(t_enter.max(0.0))
        } else {
            None
        }
    }

    pub fn rasterize(&self, grid: &mut Grid3D) {
        let cells = self.covered_cells(grid.sx, grid.sy, grid.sz);
        for (x, y, z) in cells {
            let i = grid.idx(x, y, z);
            grid.cells[i] = CellData::block();
        }
    }

    pub fn clear_raster(&self, grid: &mut Grid3D) {
        let cells = self.covered_cells(grid.sx, grid.sy, grid.sz);
        for (x, y, z) in cells {
            let i = grid.idx(x, y, z);
            if grid.cells[i].kind == Cell::Block {
                grid.cells[i] = CellData::AIR;
            }
        }
    }

    pub fn move_and_displace(&mut self, new_pos: Vec3, grid: &mut Grid3D) {
        let old_cells = self.covered_cells(grid.sx, grid.sy, grid.sz);
        let old_pos = self.pos;
        self.pos = new_pos;
        let new_cells = self.covered_cells(grid.sx, grid.sy, grid.sz);

        for &(x, y, z) in &old_cells {
            let i = grid.idx(x, y, z);
            if grid.cells[i].kind == Cell::Block {
                grid.cells[i] = CellData::AIR;
            }
        }

        let new_set: std::collections::HashSet<(usize, usize, usize)> =
            new_cells.iter().copied().collect();

        let mut failed = false;
        for &(x, y, z) in &new_cells {
            let i = grid.idx(x, y, z);
            if grid.cells[i].kind == Cell::Sand {
                let cell_data = grid.cells[i];
                grid.cells[i] = CellData::AIR;
                if !bfs_push_3d(grid, x, y, z, &new_set, cell_data) {
                    grid.cells[i] = cell_data;
                    failed = true;
                    break;
                }
            }
        }

        if failed {
            self.pos = old_pos;
            let reverted = self.covered_cells(grid.sx, grid.sy, grid.sz);
            for &(x, y, z) in &reverted {
                let i = grid.idx(x, y, z);
                if grid.cells[i].kind == Cell::Block {
                    grid.cells[i] = CellData::AIR;
                }
            }
        }

        self.rasterize(grid);
    }
}

fn bfs_push_3d(
    grid: &mut Grid3D,
    sx: usize,
    sy: usize,
    sz: usize,
    blocked: &std::collections::HashSet<(usize, usize, usize)>,
    cell_data: CellData,
) -> bool {
    let mut queue = VecDeque::new();
    let mut visited = std::collections::HashSet::new();
    queue.push_back((sx, sy, sz));
    visited.insert((sx, sy, sz));

    let dirs: [(i32, i32, i32); 6] = [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];
    let max_search = 1000;
    let mut steps = 0;

    while let Some((cx, cy, cz)) = queue.pop_front() {
        steps += 1;
        if steps > max_search {
            return false;
        }
        for (dx, dy, dz) in dirs {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            let nz = cz as i32 + dz;
            if !grid.in_bounds(nx, ny, nz) {
                continue;
            }
            let (ux, uy, uz) = (nx as usize, ny as usize, nz as usize);
            if visited.contains(&(ux, uy, uz)) || blocked.contains(&(ux, uy, uz)) {
                continue;
            }
            visited.insert((ux, uy, uz));
            let ni = grid.idx(ux, uy, uz);
            if grid.cells[ni].kind == Cell::Air {
                grid.cells[ni] = cell_data;
                return true;
            }
            if grid.cells[ni].kind == Cell::Sand {
                queue.push_back((ux, uy, uz));
            }
        }
    }
    false
}
