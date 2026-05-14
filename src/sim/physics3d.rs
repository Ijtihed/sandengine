use super::grid3d::Grid3D;
use super::material::{Cell, CellData};
use rand::Rng;
use rand::RngExt;

pub fn step(grid: &mut Grid3D, rng: &mut impl Rng) {
    grid.next.copy_from_slice(&grid.cells);
    grid.moved.fill(false);
    for y in 0..grid.sy {
        for z in 0..grid.sz {
            for x in 0..grid.sx { update_cell(grid, x, y, z, rng); }
        }
    }
    grid.swap_buffers();
}

fn update_cell(grid: &mut Grid3D, x: usize, y: usize, z: usize, rng: &mut impl Rng) {
    let i = grid.idx(x, y, z);
    if grid.moved[i] { return; }
    let cell = grid.next[i];
    match cell.kind {
        Cell::Sand | Cell::Gravel => update_granular(grid, x, y, z, i, cell, rng),
        Cell::Water => update_liquid(grid, x, y, z, i, cell, 3, rng),
        Cell::Oil => update_oil(grid, x, y, z, i, cell, rng),
        Cell::Acid => update_acid(grid, x, y, z, i, cell, rng),
        Cell::Fire => update_fire(grid, x, y, z, i, cell, rng),
        Cell::Steam => update_steam(grid, x, y, z, i, cell, rng),
        _ => {}
    }
}

fn can_displace(mover: Cell, target: Cell) -> bool {
    if target == Cell::Air { return true; }
    if target == Cell::Block || target == Cell::Stone { return false; }
    mover.density() > target.density()
}

fn update_granular(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    if y > 0 {
        let below = grid.idx(x, y - 1, z);
        let bk = grid.next[below].kind;
        if can_displace(cell.kind, bk) {
            let d = grid.next[below];
            grid.next[below] = cell; grid.next[i] = d;
            grid.moved[below] = true;
            if d.kind != Cell::Air { grid.moved[i] = true; }
            return;
        }
    }
    if y > 0 {
        let mut diags: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
        shuffle4(&mut diags, rng);
        for (dx, dz) in diags {
            let nx = x as i32+dx; let nz = z as i32+dz;
            if grid.in_bounds(nx, y as i32 - 1, nz) {
                let di = grid.idx(nx as usize, y-1, nz as usize);
                let dk = grid.next[di].kind;
                if can_displace(cell.kind, dk) {
                    let d = grid.next[di];
                    grid.next[di] = cell; grid.next[i] = d;
                    grid.moved[di] = true;
                    if d.kind != Cell::Air { grid.moved[i] = true; }
                    return;
                }
            }
        }
    }
}

fn update_liquid(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, spread: i32, rng: &mut impl Rng) {
    if y > 0 {
        let below = grid.idx(x, y-1, z);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = cell; grid.next[i] = CellData::AIR;
            grid.moved[below] = true; return;
        }
    }
    if y > 0 {
        let mut d4: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
        shuffle4(&mut d4, rng);
        for (dx,dz) in d4 {
            let nx = x as i32+dx; let nz = z as i32+dz;
            if grid.in_bounds(nx, y as i32-1, nz) {
                let di = grid.idx(nx as usize, y-1, nz as usize);
                if grid.next[di].kind == Cell::Air {
                    grid.next[di] = cell; grid.next[i] = CellData::AIR;
                    grid.moved[di] = true; return;
                }
            }
        }
    }
    let mut sd: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
    shuffle4(&mut sd, rng);
    for (dx,dz) in sd {
        let amt = rng.random_range(1..=spread);
        for s in 1..=amt {
            let nx = x as i32+dx*s; let nz = z as i32+dz*s;
            if grid.in_bounds(nx, y as i32, nz) {
                let si = grid.idx(nx as usize, y, nz as usize);
                if grid.next[si].kind == Cell::Air {
                    grid.next[si] = cell; grid.next[i] = CellData::AIR;
                    grid.moved[si] = true; return;
                }
                if grid.next[si].kind != cell.kind { break; }
            } else { break; }
        }
    }
}

fn update_oil(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    if y > 0 {
        let below = grid.idx(x, y-1, z);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = cell; grid.next[i] = CellData::AIR;
            grid.moved[below] = true; return;
        }
    }
    // Float up through water
    if y+1 < grid.sy {
        let above = grid.idx(x, y+1, z);
        if grid.next[above].kind == Cell::Water {
            let w = grid.next[above];
            grid.next[above] = cell; grid.next[i] = w;
            grid.moved[above] = true; grid.moved[i] = true; return;
        }
    }
    // Spread
    let mut sd: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
    shuffle4(&mut sd, rng);
    for (dx,dz) in sd {
        let nx = x as i32+dx; let nz = z as i32+dz;
        if grid.in_bounds(nx, y as i32, nz) {
            let si = grid.idx(nx as usize, y, nz as usize);
            if grid.next[si].kind == Cell::Air {
                grid.next[si] = cell; grid.next[i] = CellData::AIR;
                grid.moved[si] = true; return;
            }
        }
    }
    // Fire ignites oil
    let dirs: [(i32,i32,i32);6] = [(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1)];
    for (dx,dy,dz) in dirs {
        let nx = x as i32+dx; let ny = y as i32+dy; let nz = z as i32+dz;
        if grid.in_bounds(nx, ny, nz) {
            let ni = grid.idx(nx as usize, ny as usize, nz as usize);
            if grid.next[ni].kind == Cell::Fire {
                grid.next[i] = CellData::fire(rng.random_range(80..120));
                grid.moved[i] = true; return;
            }
        }
    }
}

fn update_acid(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::AIR; return; }
    let na = CellData::acid(life.saturating_sub(1));

    let dirs: [(i32,i32,i32);6] = [(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1)];
    for (dx,dy,dz) in dirs {
        let nx = x as i32+dx; let ny = y as i32+dy; let nz = z as i32+dz;
        if grid.in_bounds(nx, ny, nz) {
            let ni = grid.idx(nx as usize, ny as usize, nz as usize);
            let nk = grid.next[ni].kind;
            if (nk == Cell::Sand || nk == Cell::Gravel || nk == Cell::Stone) && rng.random_bool(0.06) {
                grid.next[ni] = CellData::AIR;
                grid.next[i] = CellData::acid(life.saturating_sub(25));
                grid.moved[i] = true; return;
            }
        }
    }

    if y > 0 {
        let below = grid.idx(x, y-1, z);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = na; grid.next[i] = CellData::AIR;
            grid.moved[below] = true; return;
        }
    }
    grid.next[i] = na; grid.moved[i] = true;
}

fn update_fire(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::steam(50); grid.moved[i] = true; return; }
    let nc = CellData::fire(life.saturating_sub(1));

    if y+1 < grid.sy {
        let dx: i32 = rng.random_range(-1..=1);
        let dz: i32 = rng.random_range(-1..=1);
        let nx = (x as i32+dx).clamp(0, grid.sx as i32-1) as usize;
        let nz = (z as i32+dz).clamp(0, grid.sz as i32-1) as usize;
        let above = grid.idx(nx, y+1, nz);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = nc; grid.next[i] = CellData::AIR;
            grid.moved[above] = true; return;
        }
    }
    grid.next[i] = nc; grid.moved[i] = true;

    let dirs: [(i32,i32,i32);6] = [(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1)];
    for (dx,dy,dz) in dirs {
        let nx = x as i32+dx; let ny = y as i32+dy; let nz = z as i32+dz;
        if grid.in_bounds(nx, ny, nz) {
            let ni = grid.idx(nx as usize, ny as usize, nz as usize);
            if grid.next[ni].kind == Cell::Water {
                grid.next[ni] = CellData::steam(40); grid.next[i] = CellData::AIR; return;
            }
            if (grid.next[ni].kind == Cell::Oil || grid.next[ni].kind == Cell::Sand) && rng.random_bool(0.01) {
                grid.next[ni] = CellData::fire(rng.random_range(50..100)); break;
            }
        }
    }
}

fn update_steam(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::AIR; return; }
    let nc = CellData::steam(life.saturating_sub(1));
    if y+1 < grid.sy {
        let dx: i32 = rng.random_range(-1..=1);
        let dz: i32 = rng.random_range(-1..=1);
        let nx = (x as i32+dx).clamp(0, grid.sx as i32-1) as usize;
        let nz = (z as i32+dz).clamp(0, grid.sz as i32-1) as usize;
        let above = grid.idx(nx, y+1, nz);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = nc; grid.next[i] = CellData::AIR;
            grid.moved[above] = true; return;
        }
    }
    grid.next[i] = nc; grid.moved[i] = true;
}

fn shuffle4<T>(arr: &mut [T; 4], rng: &mut impl Rng) {
    for i in (1..4).rev() { let j = rng.random_range(0..=i); arr.swap(i, j); }
}
