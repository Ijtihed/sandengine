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

fn swap3(grid: &mut Grid3D, i: usize, j: usize) {
    let a = grid.next[i];
    grid.next[i] = grid.next[j];
    grid.next[j] = a;
    grid.moved[i] = true;
    grid.moved[j] = true;
}

const DIRS6: [(i32,i32,i32); 6] = [(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1)];

fn update_granular(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    if y > 0 {
        let below = grid.idx(x, y - 1, z);
        if can_displace(cell.kind, grid.next[below].kind) {
            swap3(grid, i, below);
            return;
        }
    }
    if y > 0 {
        let mut diags: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
        shuffle4(&mut diags, rng);
        for (dx, dz) in diags {
            let nx = x as i32+dx; let nz = z as i32+dz;
            if grid.in_bounds(nx, y as i32-1, nz) {
                let di = grid.idx(nx as usize, y-1, nz as usize);
                if can_displace(cell.kind, grid.next[di].kind) {
                    swap3(grid, i, di);
                    return;
                }
            }
        }
    }
}

fn update_liquid(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, spread: i32, rng: &mut impl Rng) {
    // Fall straight down, displacing lighter materials
    if y > 0 {
        let below = grid.idx(x, y-1, z);
        if can_displace(cell.kind, grid.next[below].kind) {
            swap3(grid, i, below);
            return;
        }
    }
    // Diagonal down through air or lighter liquids
    if y > 0 {
        let mut d4: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
        shuffle4(&mut d4, rng);
        for (dx,dz) in d4 {
            let nx = x as i32+dx; let nz = z as i32+dz;
            if grid.in_bounds(nx, y as i32-1, nz) {
                let di = grid.idx(nx as usize, y-1, nz as usize);
                if can_displace(cell.kind, grid.next[di].kind) {
                    swap3(grid, i, di);
                    return;
                }
            }
        }
    }
    // Spread sideways
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
    // Check ignition FIRST
    for (dx,dy,dz) in DIRS6 {
        let nx = x as i32+dx; let ny = y as i32+dy; let nz = z as i32+dz;
        if grid.in_bounds(nx, ny, nz) {
            let ni = grid.idx(nx as usize, ny as usize, nz as usize);
            if grid.next[ni].kind == Cell::Fire {
                grid.next[i] = CellData::fire(rng.random_range(80..120));
                grid.moved[i] = true; return;
            }
        }
    }

    // Fall into air
    if y > 0 {
        let below = grid.idx(x, y-1, z);
        if grid.next[below].kind == Cell::Air {
            swap3(grid, i, below); return;
        }
    }

    // Diagonal down into air
    if y > 0 {
        let mut d4: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
        shuffle4(&mut d4, rng);
        for (dx,dz) in d4 {
            let nx = x as i32+dx; let nz = z as i32+dz;
            if grid.in_bounds(nx, y as i32-1, nz) {
                let di = grid.idx(nx as usize, y-1, nz as usize);
                if grid.next[di].kind == Cell::Air {
                    swap3(grid, i, di); return;
                }
            }
        }
    }

    // Float up through water
    if y+1 < grid.sy {
        let above = grid.idx(x, y+1, z);
        if grid.next[above].kind == Cell::Water {
            swap3(grid, i, above); return;
        }
    }

    // Spread sideways / form slick on water
    let mut sd: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
    shuffle4(&mut sd, rng);
    for (dx,dz) in sd {
        for s in 1..=3i32 {
            let nx = x as i32+dx*s; let nz = z as i32+dz*s;
            if grid.in_bounds(nx, y as i32, nz) {
                let si = grid.idx(nx as usize, y, nz as usize);
                if grid.next[si].kind == Cell::Air {
                    grid.next[si] = cell; grid.next[i] = CellData::AIR;
                    grid.moved[si] = true; return;
                }
                if grid.next[si].kind != Cell::Oil { break; }
            } else { break; }
        }
    }
}

fn update_acid(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::AIR; return; }
    let na = CellData::acid(life.saturating_sub(1));

    // React with neighbors
    for (dx,dy,dz) in DIRS6 {
        let nx = x as i32+dx; let ny = y as i32+dy; let nz = z as i32+dz;
        if !grid.in_bounds(nx, ny, nz) { continue; }
        let ni = grid.idx(nx as usize, ny as usize, nz as usize);
        let nk = grid.next[ni].kind;

        if (nk == Cell::Sand || nk == Cell::Gravel || nk == Cell::Stone) && rng.random_bool(0.10) {
            grid.next[ni] = CellData::AIR;
            let remain = life.saturating_sub(15);
            grid.next[i] = if remain > 0 { CellData::acid(remain) } else { CellData::AIR };
            grid.moved[i] = true; return;
        }
        if nk == Cell::Water && rng.random_bool(0.05) {
            grid.next[i] = CellData::water(rng.random());
            grid.moved[i] = true; return;
        }
        if nk == Cell::Oil && rng.random_bool(0.08) {
            grid.next[ni] = CellData::steam(40);
            grid.next[i] = CellData::steam(30);
            grid.moved[i] = true; return;
        }
    }

    // Fall
    if y > 0 {
        let below = grid.idx(x, y-1, z);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = na; grid.next[i] = CellData::AIR;
            grid.moved[below] = true; return;
        }
    }
    // Diagonal down
    if y > 0 {
        let mut d4: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
        shuffle4(&mut d4, rng);
        for (dx,dz) in d4 {
            let nx = x as i32+dx; let nz = z as i32+dz;
            if grid.in_bounds(nx, y as i32-1, nz) {
                let di = grid.idx(nx as usize, y-1, nz as usize);
                if grid.next[di].kind == Cell::Air {
                    grid.next[di] = na; grid.next[i] = CellData::AIR;
                    grid.moved[di] = true; return;
                }
            }
        }
    }
    // Spread sideways
    let mut sd: [(i32,i32);4] = [(1,0),(-1,0),(0,1),(0,-1)];
    shuffle4(&mut sd, rng);
    for (dx,dz) in sd {
        let nx = x as i32+dx; let nz = z as i32+dz;
        if grid.in_bounds(nx, y as i32, nz) {
            let si = grid.idx(nx as usize, y, nz as usize);
            if grid.next[si].kind == Cell::Air {
                grid.next[si] = na; grid.next[i] = CellData::AIR;
                grid.moved[si] = true; return;
            }
        }
    }
    grid.next[i] = na; grid.moved[i] = true;
}

fn update_fire(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 {
        if rng.random_bool(0.15) {
            grid.next[i] = CellData::gravel(rng.random());
        } else {
            grid.next[i] = CellData::steam(rng.random_range(40..70));
        }
        grid.moved[i] = true; return;
    }
    let nc = CellData::fire(life.saturating_sub(1));

    // Evaporate water (high priority)
    for (dx,dy,dz) in DIRS6 {
        let nx = x as i32+dx; let ny = y as i32+dy; let nz = z as i32+dz;
        if grid.in_bounds(nx, ny, nz) {
            let ni = grid.idx(nx as usize, ny as usize, nz as usize);
            if grid.next[ni].kind == Cell::Water {
                grid.next[ni] = CellData::steam(50);
                grid.next[i] = CellData::steam(20);
                grid.moved[i] = true; return;
            }
        }
    }

    // Ignite neighbors: oil very flammable, sand barely, gravel immune
    for (dx,dy,dz) in DIRS6 {
        let nx = x as i32+dx; let ny = y as i32+dy; let nz = z as i32+dz;
        if grid.in_bounds(nx, ny, nz) {
            let ni = grid.idx(nx as usize, ny as usize, nz as usize);
            let nk = grid.next[ni].kind;
            if nk == Cell::Oil && rng.random_bool(0.12) {
                grid.next[ni] = CellData::fire(rng.random_range(80..120));
            } else if nk == Cell::Sand && rng.random_bool(0.004) {
                grid.next[ni] = CellData::fire(rng.random_range(30..80));
            }
        }
    }

    // Rise with drift
    if y+1 < grid.sy {
        let dx: i32 = rng.random_range(-1..=1);
        let dz: i32 = rng.random_range(-1..=1);
        let nx = (x as i32+dx).clamp(0, grid.sx as i32-1) as usize;
        let nz = (z as i32+dz).clamp(0, grid.sz as i32-1) as usize;
        let above = grid.idx(nx, y+1, nz);
        let ak = grid.next[above].kind;
        if ak == Cell::Air || ak == Cell::Steam {
            grid.next[above] = nc; grid.next[i] = CellData::AIR;
            grid.moved[above] = true; return;
        }
    }
    if y+1 < grid.sy {
        let above = grid.idx(x, y+1, z);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = nc; grid.next[i] = CellData::AIR;
            grid.moved[above] = true; return;
        }
    }

    grid.next[i] = nc; grid.moved[i] = true;
}

fn update_steam(grid: &mut Grid3D, x: usize, y: usize, z: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::AIR; return; }
    let nc = CellData::steam(life.saturating_sub(1));

    // Rise through air, liquids (bubbles), and fire
    if y+1 < grid.sy {
        let dx: i32 = rng.random_range(-1..=1);
        let dz: i32 = rng.random_range(-1..=1);
        let nx = (x as i32+dx).clamp(0, grid.sx as i32-1) as usize;
        let nz = (z as i32+dz).clamp(0, grid.sz as i32-1) as usize;
        let above = grid.idx(nx, y+1, nz);
        let ak = grid.next[above].kind;
        if ak == Cell::Air {
            grid.next[above] = nc; grid.next[i] = CellData::AIR;
            grid.moved[above] = true; return;
        }
        if ak == Cell::Water || ak == Cell::Oil || ak == Cell::Acid || ak == Cell::Fire {
            let displaced = grid.next[above];
            grid.next[above] = nc; grid.next[i] = displaced;
            grid.moved[above] = true; grid.moved[i] = true; return;
        }
    }

    grid.next[i] = nc; grid.moved[i] = true;
}

fn shuffle4<T>(arr: &mut [T; 4], rng: &mut impl Rng) {
    for i in (1..4).rev() { let j = rng.random_range(0..=i); arr.swap(i, j); }
}
