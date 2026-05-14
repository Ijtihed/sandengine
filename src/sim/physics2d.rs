use super::grid2d::Grid2D;
use super::material::{Cell, CellData};
use rand::Rng;
use rand::RngExt;

pub fn step(grid: &mut Grid2D, rng: &mut impl Rng) {
    grid.next.copy_from_slice(&grid.cells);
    grid.moved.fill(false);

    for y in (0..grid.h).rev() {
        let left_to_right: bool = rng.random();
        if left_to_right {
            for x in 0..grid.w { update_cell(grid, x, y, rng); }
        } else {
            for x in (0..grid.w).rev() { update_cell(grid, x, y, rng); }
        }
    }

    grid.swap_buffers();
}

fn update_cell(grid: &mut Grid2D, x: usize, y: usize, rng: &mut impl Rng) {
    let i = grid.idx(x, y);
    if grid.moved[i] { return; }
    let cell = grid.next[i];
    match cell.kind {
        Cell::Sand => update_granular(grid, x, y, i, cell, rng),
        Cell::Gravel => update_granular(grid, x, y, i, cell, rng),
        Cell::Water => update_liquid(grid, x, y, i, cell, 5, rng),
        Cell::Oil => update_oil(grid, x, y, i, cell, rng),
        Cell::Acid => update_acid(grid, x, y, i, cell, rng),
        Cell::Fire => update_fire(grid, x, y, i, cell, rng),
        Cell::Steam => update_steam(grid, x, y, i, cell, rng),
        _ => {}
    }
}

fn can_displace(mover: Cell, target: Cell) -> bool {
    if target == Cell::Air { return true; }
    if target == Cell::Block || target == Cell::Stone { return false; }
    mover.density() > target.density()
}

fn update_granular(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        let bk = grid.next[below].kind;
        if can_displace(cell.kind, bk) {
            let displaced = grid.next[below];
            grid.next[below] = cell;
            grid.next[i] = displaced;
            grid.moved[below] = true;
            if displaced.kind != Cell::Air { grid.moved[i] = true; }
            return;
        }
    }

    if y + 1 < grid.h {
        let go_left: bool = rng.random();
        let offsets = if go_left { [-1i32, 1] } else { [1i32, -1] };
        for dx in offsets {
            let nx = x as i32 + dx;
            if nx >= 0 && nx < grid.w as i32 {
                let di = grid.idx(nx as usize, y + 1);
                let dk = grid.next[di].kind;
                if can_displace(cell.kind, dk) {
                    let displaced = grid.next[di];
                    grid.next[di] = cell;
                    grid.next[i] = displaced;
                    grid.moved[di] = true;
                    if displaced.kind != Cell::Air { grid.moved[i] = true; }
                    return;
                }
            }
        }
    }
}

fn update_liquid(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, spread: i32, rng: &mut impl Rng) {
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        let bk = grid.next[below].kind;
        if bk == Cell::Air {
            grid.next[below] = cell;
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
        if can_displace(cell.kind, bk) {
            let d = grid.next[below];
            grid.next[below] = cell;
            grid.next[i] = d;
            grid.moved[below] = true;
            grid.moved[i] = true;
            return;
        }
    }

    if y + 1 < grid.h {
        let go_left: bool = rng.random();
        for dx in if go_left { [-1i32, 1] } else { [1i32, -1] } {
            let nx = x as i32 + dx;
            if nx >= 0 && nx < grid.w as i32 {
                let di = grid.idx(nx as usize, y + 1);
                if grid.next[di].kind == Cell::Air {
                    grid.next[di] = cell;
                    grid.next[i] = CellData::AIR;
                    grid.moved[di] = true;
                    return;
                }
            }
        }
    }

    let amt: i32 = rng.random_range(1..=spread);
    let dir: i32 = if rng.random() { -1 } else { 1 };
    for s in 1..=amt {
        let nx = x as i32 + dir * s;
        if nx >= 0 && nx < grid.w as i32 {
            let si = grid.idx(nx as usize, y);
            if grid.next[si].kind == Cell::Air {
                grid.next[si] = cell;
                grid.next[i] = CellData::AIR;
                grid.moved[si] = true;
                return;
            }
            if grid.next[si].kind != cell.kind { break; }
        } else { break; }
    }
}

fn update_oil(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    // Oil is lighter than water but heavier than air -- it floats on water
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        let bk = grid.next[below].kind;
        if bk == Cell::Air {
            grid.next[below] = cell;
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
    }

    // Diagonal down into air
    if y + 1 < grid.h {
        let go_left: bool = rng.random();
        for dx in if go_left { [-1i32, 1] } else { [1i32, -1] } {
            let nx = x as i32 + dx;
            if nx >= 0 && nx < grid.w as i32 {
                let di = grid.idx(nx as usize, y + 1);
                if grid.next[di].kind == Cell::Air {
                    grid.next[di] = cell;
                    grid.next[i] = CellData::AIR;
                    grid.moved[di] = true;
                    return;
                }
            }
        }
    }

    // Float up through water (water is denser)
    if y > 0 {
        let above = grid.idx(x, y - 1);
        if grid.next[above].kind == Cell::Water {
            let w = grid.next[above];
            grid.next[above] = cell;
            grid.next[i] = w;
            grid.moved[above] = true;
            grid.moved[i] = true;
            return;
        }
    }

    // Spread sideways
    let dir: i32 = if rng.random() { -1 } else { 1 };
    for s in 1..=3i32 {
        let nx = x as i32 + dir * s;
        if nx >= 0 && nx < grid.w as i32 {
            let si = grid.idx(nx as usize, y);
            if grid.next[si].kind == Cell::Air {
                grid.next[si] = cell;
                grid.next[i] = CellData::AIR;
                grid.moved[si] = true;
                return;
            }
            if grid.next[si].kind != Cell::Oil { break; }
        } else { break; }
    }

    // Fire ignites oil
    let dirs: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
    for (dx, dy) in dirs {
        let nx = x as i32 + dx; let ny = y as i32 + dy;
        if grid.in_bounds(nx, ny) {
            let ni = grid.idx(nx as usize, ny as usize);
            if grid.next[ni].kind == Cell::Fire {
                grid.next[i] = CellData::fire(rng.random_range(80..120));
                grid.moved[i] = true;
                return;
            }
        }
    }
}

fn update_acid(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 {
        grid.next[i] = CellData::AIR;
        return;
    }
    let new_acid = CellData::acid(life.saturating_sub(1));

    // Acid erodes sand, gravel, and stone on contact
    let dirs: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
    for (dx, dy) in dirs {
        let nx = x as i32 + dx; let ny = y as i32 + dy;
        if grid.in_bounds(nx, ny) {
            let ni = grid.idx(nx as usize, ny as usize);
            let nk = grid.next[ni].kind;
            if (nk == Cell::Sand || nk == Cell::Gravel || nk == Cell::Stone) && rng.random_bool(0.08) {
                grid.next[ni] = CellData::AIR;
                grid.next[i] = CellData::acid(life.saturating_sub(20));
                grid.moved[i] = true;
                return;
            }
        }
    }

    // Falls like a liquid
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        let bk = grid.next[below].kind;
        if bk == Cell::Air {
            grid.next[below] = new_acid;
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
    }

    // Spread
    let dir: i32 = if rng.random() { -1 } else { 1 };
    let nx = x as i32 + dir;
    if nx >= 0 && nx < grid.w as i32 {
        let si = grid.idx(nx as usize, y);
        if grid.next[si].kind == Cell::Air {
            grid.next[si] = new_acid;
            grid.next[i] = CellData::AIR;
            grid.moved[si] = true;
            return;
        }
    }

    grid.next[i] = new_acid;
    grid.moved[i] = true;
}

fn update_fire(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 {
        grid.next[i] = CellData::steam(60);
        grid.moved[i] = true;
        return;
    }
    let new_cell = CellData::fire(life.saturating_sub(1));

    if y > 0 {
        let above = grid.idx(x, y - 1);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = new_cell;
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
    }

    let dx: i32 = rng.random_range(-1..=1);
    if y > 0 {
        let nx = (x as i32 + dx).clamp(0, grid.w as i32 - 1) as usize;
        let di = grid.idx(nx, y - 1);
        if grid.next[di].kind == Cell::Air {
            grid.next[di] = new_cell;
            grid.next[i] = CellData::AIR;
            grid.moved[di] = true;
            return;
        }
    }

    grid.next[i] = new_cell;
    grid.moved[i] = true;

    // Ignite neighbors
    if rng.random_bool(0.015) {
        let dirs: [(i32,i32);4] = [(-1,0),(1,0),(0,-1),(0,1)];
        for (ddx, ddy) in dirs {
            let nx = x as i32+ddx; let ny = y as i32+ddy;
            if grid.in_bounds(nx, ny) {
                let ni = grid.idx(nx as usize, ny as usize);
                let nk = grid.next[ni].kind;
                if nk == Cell::Sand || nk == Cell::Oil {
                    grid.next[ni] = CellData::fire(rng.random_range(50..110));
                    break;
                }
            }
        }
    }

    // Evaporate water
    let dirs: [(i32,i32);4] = [(-1,0),(1,0),(0,-1),(0,1)];
    for (ddx, ddy) in dirs {
        let nx = x as i32+ddx; let ny = y as i32+ddy;
        if grid.in_bounds(nx, ny) {
            let ni = grid.idx(nx as usize, ny as usize);
            if grid.next[ni].kind == Cell::Water {
                grid.next[ni] = CellData::steam(50);
                grid.next[i] = CellData::AIR;
                grid.moved[i] = true;
                return;
            }
        }
    }
}

fn update_steam(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::AIR; return; }
    let nc = CellData::steam(life.saturating_sub(1));

    if y > 0 {
        let dx: i32 = rng.random_range(-1..=1);
        let nx = (x as i32 + dx).clamp(0, grid.w as i32 - 1) as usize;
        let above = grid.idx(nx, y - 1);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = nc;
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
    }
    let dx: i32 = rng.random_range(-1..=1);
    let nx = x as i32 + dx;
    if nx >= 0 && nx < grid.w as i32 {
        let si = grid.idx(nx as usize, y);
        if grid.next[si].kind == Cell::Air {
            grid.next[si] = nc;
            grid.next[i] = CellData::AIR;
            grid.moved[si] = true;
            return;
        }
    }
    grid.next[i] = nc;
    grid.moved[i] = true;
}
