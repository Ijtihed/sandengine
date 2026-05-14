use super::grid3d::Grid3D;
use super::material::{Cell, CellData};
use rand::Rng;
use rand::RngExt;

pub fn step(grid: &mut Grid3D, rng: &mut impl Rng) {
    grid.next.copy_from_slice(&grid.cells);
    grid.moved.fill(false);

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
    if grid.moved[i] { return; }
    let cell = grid.next[i];

    match cell.kind {
        Cell::Sand => update_sand(grid, x, y, z, i, cell, rng),
        Cell::Water => update_water(grid, x, y, z, i, cell, rng),
        Cell::Fire => update_fire(grid, x, y, z, i, cell, rng),
        Cell::Steam => update_steam(grid, x, y, z, i, cell, rng),
        _ => {}
    }
}

fn update_sand(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    if y > 0 {
        let below = grid.idx(x, y - 1, z);
        let bk = grid.next[below].kind;
        if bk == Cell::Air {
            grid.next[below] = cell;
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
        if bk == Cell::Water {
            let water = grid.next[below];
            grid.next[below] = cell;
            grid.next[i] = water;
            grid.moved[below] = true;
            grid.moved[i] = true;
            return;
        }
    }

    if y > 0 {
        let mut diags: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        shuffle4(&mut diags, rng);
        for (dx, dz) in diags {
            let nx = x as i32 + dx;
            let nz = z as i32 + dz;
            if grid.in_bounds(nx, y as i32 - 1, nz) {
                let di = grid.idx(nx as usize, y - 1, nz as usize);
                let dk = grid.next[di].kind;
                if dk == Cell::Air || dk == Cell::Water {
                    if dk == Cell::Water {
                        let water = grid.next[di];
                        grid.next[di] = cell;
                        grid.next[i] = water;
                        grid.moved[i] = true;
                    } else {
                        grid.next[di] = cell;
                        grid.next[i] = CellData::AIR;
                    }
                    grid.moved[di] = true;
                    return;
                }
            }
        }
    }
}

fn update_water(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    if y > 0 {
        let below = grid.idx(x, y - 1, z);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = cell;
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
    }

    if y > 0 {
        let mut diags: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        shuffle4(&mut diags, rng);
        for (dx, dz) in diags {
            let nx = x as i32 + dx;
            let nz = z as i32 + dz;
            if grid.in_bounds(nx, y as i32 - 1, nz) {
                let di = grid.idx(nx as usize, y - 1, nz as usize);
                if grid.next[di].kind == Cell::Air {
                    grid.next[di] = cell;
                    grid.next[i] = CellData::AIR;
                    grid.moved[di] = true;
                    return;
                }
            }
        }
    }

    // Spread horizontally
    let mut spread_dirs: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    shuffle4(&mut spread_dirs, rng);
    for (dx, dz) in spread_dirs {
        let spread = rng.random_range(1..=3);
        for s in 1..=spread {
            let nx = x as i32 + dx * s;
            let nz = z as i32 + dz * s;
            if grid.in_bounds(nx, y as i32, nz) {
                let si = grid.idx(nx as usize, y, nz as usize);
                if grid.next[si].kind == Cell::Air {
                    grid.next[si] = cell;
                    grid.next[i] = CellData::AIR;
                    grid.moved[si] = true;
                    return;
                }
                if grid.next[si].kind != Cell::Water { break; }
            } else { break; }
        }
    }
}

fn update_fire(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 {
        grid.next[i] = CellData::steam(50);
        grid.moved[i] = true;
        return;
    }

    let new_cell = CellData::fire(life.saturating_sub(1));

    if y + 1 < grid.sy {
        let dx: i32 = rng.random_range(-1..=1);
        let dz: i32 = rng.random_range(-1..=1);
        let nx = (x as i32 + dx).clamp(0, grid.sx as i32 - 1) as usize;
        let nz = (z as i32 + dz).clamp(0, grid.sz as i32 - 1) as usize;
        let above = grid.idx(nx, y + 1, nz);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = new_cell;
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
    }

    grid.next[i] = new_cell;
    grid.moved[i] = true;

    // Evaporate adjacent water
    let dirs: [(i32, i32, i32); 6] = [(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1)];
    for (ddx, ddy, ddz) in dirs {
        let nx = x as i32 + ddx;
        let ny = y as i32 + ddy;
        let nz = z as i32 + ddz;
        if grid.in_bounds(nx, ny, nz) {
            let ni = grid.idx(nx as usize, ny as usize, nz as usize);
            if grid.next[ni].kind == Cell::Water {
                grid.next[ni] = CellData::steam(40);
                grid.next[i] = CellData::AIR;
                return;
            }
        }
    }
}

fn update_steam(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 {
        grid.next[i] = CellData::AIR;
        return;
    }
    let new_cell = CellData::steam(life.saturating_sub(1));

    if y + 1 < grid.sy {
        let dx: i32 = rng.random_range(-1..=1);
        let dz: i32 = rng.random_range(-1..=1);
        let nx = (x as i32 + dx).clamp(0, grid.sx as i32 - 1) as usize;
        let nz = (z as i32 + dz).clamp(0, grid.sz as i32 - 1) as usize;
        let above = grid.idx(nx, y + 1, nz);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = new_cell;
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
    }

    grid.next[i] = new_cell;
    grid.moved[i] = true;
}

fn shuffle4<T>(arr: &mut [T; 4], rng: &mut impl Rng) {
    for i in (1..4).rev() {
        let j = rng.random_range(0..=i);
        arr.swap(i, j);
    }
}
