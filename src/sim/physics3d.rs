use super::grid3d::Grid3D;
use super::material::{Cell, CellData};
use rand::Rng;
use rand::RngExt;

pub fn step(grid: &mut Grid3D, rng: &mut impl Rng) {
    grid.next.copy_from_slice(&grid.cells);
    grid.moved.fill(false);

    // Process bottom to top (y=0 is bottom)
    for y in 0..grid.sy {
        for z in 0..grid.sz {
            for x in 0..grid.sx {
                update_cell(grid, x, y, z, rng);
            }
        }
    }

    grid.swap_buffers();
}

fn update_cell(grid: &mut Grid3D, x: usize, y: usize, z: usize, rng: &mut impl Rng) {
    let i = grid.idx(x, y, z);
    if grid.next[i].kind != Cell::Sand || grid.moved[i] {
        return;
    }

    // Fall straight down
    if y > 0 {
        let below = grid.idx(x, y - 1, z);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = grid.next[i];
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
    }

    // Diagonal fall: 4 cardinal-diagonal-down neighbors, shuffled
    if y > 0 {
        let mut diags: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        shuffle4(&mut diags, rng);

        for (dx, dz) in diags {
            let nx = x as i32 + dx;
            let nz = z as i32 + dz;
            if grid.in_bounds(nx, y as i32 - 1, nz) {
                let di = grid.idx(nx as usize, y - 1, nz as usize);
                if grid.next[di].kind == Cell::Air {
                    grid.next[di] = grid.next[i];
                    grid.next[i] = CellData::AIR;
                    grid.moved[di] = true;
                    return;
                }
            }
        }
    }

    // Corner fall: 4 corner-diagonal-down neighbors, shuffled
    if y > 0 {
        let mut corners: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        shuffle4(&mut corners, rng);

        for (dx, dz) in corners {
            let nx = x as i32 + dx;
            let nz = z as i32 + dz;
            if grid.in_bounds(nx, y as i32 - 1, nz) {
                let ci = grid.idx(nx as usize, y - 1, nz as usize);
                if grid.next[ci].kind == Cell::Air {
                    grid.next[ci] = grid.next[i];
                    grid.next[i] = CellData::AIR;
                    grid.moved[ci] = true;
                    return;
                }
            }
        }
    }
}

fn shuffle4<T>(arr: &mut [T; 4], rng: &mut impl Rng) {
    for i in (1..4).rev() {
        let j = rng.random_range(0..=i);
        arr.swap(i, j);
    }
}
