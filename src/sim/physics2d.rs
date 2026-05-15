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
        Cell::Sand | Cell::Gravel => update_granular(grid, x, y, i, cell, rng),
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

fn swap_cells(grid: &mut Grid2D, i: usize, j: usize) {
    let a = grid.next[i];
    grid.next[i] = grid.next[j];
    grid.next[j] = a;
    grid.moved[i] = true;
    grid.moved[j] = true;
}

// --- GRANULAR (Sand, Gravel) ---

fn update_granular(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    // Fall straight down, displacing lighter materials
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        if can_displace(cell.kind, grid.next[below].kind) {
            swap_cells(grid, i, below);
            return;
        }
    }

    // Diagonal fall
    if y + 1 < grid.h {
        let go_left: bool = rng.random();
        for dx in if go_left { [-1i32, 1] } else { [1i32, -1] } {
            let nx = x as i32 + dx;
            if nx >= 0 && nx < grid.w as i32 {
                let di = grid.idx(nx as usize, y + 1);
                if can_displace(cell.kind, grid.next[di].kind) {
                    swap_cells(grid, i, di);
                    return;
                }
            }
        }
    }
}

// --- LIQUID (Water) ---

fn update_liquid(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, spread: i32, rng: &mut impl Rng) {
    // Fall straight down
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        if can_displace(cell.kind, grid.next[below].kind) {
            swap_cells(grid, i, below);
            return;
        }
    }

    // Diagonal down through air or lighter liquids
    if y + 1 < grid.h {
        let go_left: bool = rng.random();
        for dx in if go_left { [-1i32, 1] } else { [1i32, -1] } {
            let nx = x as i32 + dx;
            if nx >= 0 && nx < grid.w as i32 {
                let di = grid.idx(nx as usize, y + 1);
                if can_displace(cell.kind, grid.next[di].kind) {
                    swap_cells(grid, i, di);
                    return;
                }
            }
        }
    }

    // Spread sideways through air
    let amt: i32 = rng.random_range(1..=spread);
    let dir: i32 = if rng.random() { -1 } else { 1 };
    for s in 1..=amt {
        let nx = x as i32 + dir * s;
        if nx >= 0 && nx < grid.w as i32 {
            let si = grid.idx(nx as usize, y);
            let sk = grid.next[si].kind;
            if sk == Cell::Air {
                grid.next[si] = cell;
                grid.next[i] = CellData::AIR;
                grid.moved[si] = true;
                return;
            }
            if sk != Cell::Water { break; }
        } else { break; }
    }
}

// --- OIL ---

fn update_oil(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    // Check ignition FIRST -- if fire is adjacent, catch fire immediately
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

    // Fall straight down into air
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        if grid.next[below].kind == Cell::Air {
            swap_cells(grid, i, below);
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
                    swap_cells(grid, i, di);
                    return;
                }
            }
        }
    }

    // Float up through water (oil is less dense)
    if y > 0 {
        let above = grid.idx(x, y - 1);
        if grid.next[above].kind == Cell::Water {
            swap_cells(grid, i, above);
            return;
        }
    }

    // Slide along water surface (spread as a slick)
    let dir: i32 = if rng.random() { -1 } else { 1 };
    for s in 1..=4i32 {
        let nx = x as i32 + dir * s;
        if nx >= 0 && nx < grid.w as i32 {
            let si = grid.idx(nx as usize, y);
            let sk = grid.next[si].kind;
            if sk == Cell::Air {
                grid.next[si] = cell;
                grid.next[i] = CellData::AIR;
                grid.moved[si] = true;
                return;
            }
            if sk != Cell::Oil { break; }
        } else { break; }
    }
}

// --- ACID ---

fn update_acid(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::AIR; return; }
    let new_acid = CellData::acid(life.saturating_sub(1));

    // React with neighbors FIRST
    let dirs: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
    for (dx, dy) in dirs {
        let nx = x as i32 + dx; let ny = y as i32 + dy;
        if !grid.in_bounds(nx, ny) { continue; }
        let ni = grid.idx(nx as usize, ny as usize);
        let nk = grid.next[ni].kind;

        // Dissolve solids
        if (nk == Cell::Sand || nk == Cell::Gravel || nk == Cell::Stone) && rng.random_bool(0.12) {
            grid.next[ni] = CellData::AIR;
            let remain = life.saturating_sub(15);
            grid.next[i] = if remain > 0 { CellData::acid(remain) } else { CellData::AIR };
            grid.moved[i] = true;
            return;
        }

        // Acid + water -> neutralize (acid dilutes, becomes water)
        if nk == Cell::Water && rng.random_bool(0.05) {
            grid.next[i] = CellData::water(rng.random());
            grid.moved[i] = true;
            return;
        }

        // Acid + oil -> volatile reaction (both become steam)
        if nk == Cell::Oil && rng.random_bool(0.08) {
            grid.next[ni] = CellData::steam(40);
            grid.next[i] = CellData::steam(30);
            grid.moved[i] = true;
            return;
        }
    }

    // Fall straight down
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        let bk = grid.next[below].kind;
        if bk == Cell::Air || bk == Cell::Steam || bk == Cell::Fire {
            grid.next[below] = new_acid;
            grid.next[i] = if bk == Cell::Air { CellData::AIR } else { grid.next[below] };
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
    }

    // Diagonal down
    if y + 1 < grid.h {
        let go_left: bool = rng.random();
        for dx in if go_left { [-1i32, 1] } else { [1i32, -1] } {
            let nx = x as i32 + dx;
            if nx >= 0 && nx < grid.w as i32 {
                let di = grid.idx(nx as usize, y + 1);
                if grid.next[di].kind == Cell::Air {
                    grid.next[di] = new_acid;
                    grid.next[i] = CellData::AIR;
                    grid.moved[di] = true;
                    return;
                }
            }
        }
    }

    // Spread sideways like water
    let dir: i32 = if rng.random() { -1 } else { 1 };
    for s in 1..=3i32 {
        let nx = x as i32 + dir * s;
        if nx >= 0 && nx < grid.w as i32 {
            let si = grid.idx(nx as usize, y);
            if grid.next[si].kind == Cell::Air {
                grid.next[si] = new_acid;
                grid.next[i] = CellData::AIR;
                grid.moved[si] = true;
                return;
            }
            if grid.next[si].kind != Cell::Acid { break; }
        } else { break; }
    }

    grid.next[i] = new_acid;
    grid.moved[i] = true;
}

// --- FIRE ---

fn update_fire(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 {
        // Dying fire: mostly steam, occasionally leaves ash (gravel)
        if rng.random_bool(0.15) {
            grid.next[i] = CellData::gravel(rng.random());
        } else {
            grid.next[i] = CellData::steam(rng.random_range(40..70));
        }
        grid.moved[i] = true;
        return;
    }
    let new_cell = CellData::fire(life.saturating_sub(1));

    // Evaporate adjacent water (high priority)
    let dirs: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
    for (dx, dy) in dirs {
        let nx = x as i32 + dx; let ny = y as i32 + dy;
        if grid.in_bounds(nx, ny) {
            let ni = grid.idx(nx as usize, ny as usize);
            if grid.next[ni].kind == Cell::Water {
                grid.next[ni] = CellData::steam(50);
                grid.next[i] = CellData::steam(20);
                grid.moved[i] = true;
                return;
            }
        }
    }

    // Ignite neighbors: oil is VERY flammable, sand is barely flammable, gravel doesn't burn
    for (dx, dy) in dirs {
        let nx = x as i32 + dx; let ny = y as i32 + dy;
        if grid.in_bounds(nx, ny) {
            let ni = grid.idx(nx as usize, ny as usize);
            let nk = grid.next[ni].kind;
            if nk == Cell::Oil && rng.random_bool(0.12) {
                grid.next[ni] = CellData::fire(rng.random_range(80..120));
            } else if nk == Cell::Sand && rng.random_bool(0.004) {
                grid.next[ni] = CellData::fire(rng.random_range(30..80));
            }
        }
    }

    // Rise upward with random drift
    if y > 0 {
        let dx: i32 = rng.random_range(-1..=1);
        let nx = (x as i32 + dx).clamp(0, grid.w as i32 - 1) as usize;
        let above = grid.idx(nx, y - 1);
        let ak = grid.next[above].kind;
        if ak == Cell::Air || ak == Cell::Steam {
            grid.next[above] = new_cell;
            grid.next[i] = if ak == Cell::Steam { grid.next[above] } else { CellData::AIR };
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
    }

    // Straight up
    if y > 0 {
        let above = grid.idx(x, y - 1);
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

// --- STEAM ---

fn update_steam(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 { grid.next[i] = CellData::AIR; return; }
    let nc = CellData::steam(life.saturating_sub(1));

    // Rise through air, fire, and even liquids (bubbles)
    if y > 0 {
        let dx: i32 = rng.random_range(-1..=1);
        let nx = (x as i32 + dx).clamp(0, grid.w as i32 - 1) as usize;
        let above = grid.idx(nx, y - 1);
        let ak = grid.next[above].kind;
        if ak == Cell::Air {
            grid.next[above] = nc;
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
        // Bubble through liquids and fire
        if ak == Cell::Water || ak == Cell::Oil || ak == Cell::Acid || ak == Cell::Fire {
            let displaced = grid.next[above];
            grid.next[above] = nc;
            grid.next[i] = displaced;
            grid.moved[above] = true;
            grid.moved[i] = true;
            return;
        }
    }

    // Drift sideways
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
