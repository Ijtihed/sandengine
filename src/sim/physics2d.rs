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
        Cell::Sand => update_sand(grid, x, y, i, cell, rng),
        Cell::Water => update_water(grid, x, y, i, cell, rng),
        Cell::Fire => update_fire(grid, x, y, i, cell, rng),
        Cell::Steam => update_steam(grid, x, y, i, cell, rng),
        _ => {}
    }
}

fn update_sand(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        let bk = grid.next[below].kind;
        if bk == Cell::Air {
            grid.next[below] = cell;
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
        // Sand sinks through water
        if bk == Cell::Water {
            let water = grid.next[below];
            grid.next[below] = cell;
            grid.next[i] = water;
            grid.moved[below] = true;
            grid.moved[i] = true;
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
                if dk == Cell::Air {
                    grid.next[di] = cell;
                    grid.next[i] = CellData::AIR;
                    grid.moved[di] = true;
                    return;
                }
                if dk == Cell::Water {
                    let water = grid.next[di];
                    grid.next[di] = cell;
                    grid.next[i] = water;
                    grid.moved[di] = true;
                    grid.moved[i] = true;
                    return;
                }
            }
        }
    }
}

fn update_water(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    // Fall down
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = cell;
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
    }

    // Diagonal down
    if y + 1 < grid.h {
        let go_left: bool = rng.random();
        let offsets = if go_left { [-1i32, 1] } else { [1i32, -1] };
        for dx in offsets {
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

    // Spread sideways up to 5 cells
    let spread: i32 = rng.random_range(1..=5);
    let go_left: bool = rng.random();
    let dir: i32 = if go_left { -1 } else { 1 };
    for s in 1..=spread {
        let nx = x as i32 + dir * s;
        if nx >= 0 && nx < grid.w as i32 {
            let si = grid.idx(nx as usize, y);
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

fn update_fire(grid: &mut Grid2D, x: usize, y: usize, i: usize, cell: CellData, rng: &mut impl Rng) {
    let life = cell.extra;
    if life == 0 {
        grid.next[i] = CellData::steam(60);
        grid.moved[i] = true;
        return;
    }

    let new_cell = CellData::fire(life.saturating_sub(1));

    // Rise upward
    if y > 0 {
        let above = grid.idx(x, y - 1);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = new_cell;
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
    }

    // Drift sideways while rising
    let dx: i32 = rng.random_range(-1..=1);
    if y > 0 {
        let nx = x as i32 + dx;
        if nx >= 0 && nx < grid.w as i32 {
            let di = grid.idx(nx as usize, y - 1);
            if grid.next[di].kind == Cell::Air {
                grid.next[di] = new_cell;
                grid.next[i] = CellData::AIR;
                grid.moved[di] = true;
                return;
            }
        }
    }

    grid.next[i] = new_cell;
    grid.moved[i] = true;

    // Ignite nearby flammable cells (sand near fire can catch)
    if rng.random_bool(0.01) {
        let dirs: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (ddx, ddy) in dirs {
            let nx = x as i32 + ddx;
            let ny = y as i32 + ddy;
            if grid.in_bounds(nx, ny) {
                let ni = grid.idx(nx as usize, ny as usize);
                if grid.next[ni].kind == Cell::Sand {
                    grid.next[ni] = CellData::fire(rng.random_range(40..100));
                    break;
                }
            }
        }
    }

    // Fire evaporates water on contact
    let dirs: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (ddx, ddy) in dirs {
        let nx = x as i32 + ddx;
        let ny = y as i32 + ddy;
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
    if life == 0 {
        grid.next[i] = CellData::AIR;
        return;
    }

    let new_cell = CellData::steam(life.saturating_sub(1));

    // Rise
    if y > 0 {
        let dx: i32 = rng.random_range(-1..=1);
        let nx = (x as i32 + dx).clamp(0, grid.w as i32 - 1) as usize;
        let above = grid.idx(nx, y - 1);
        if grid.next[above].kind == Cell::Air {
            grid.next[above] = new_cell;
            grid.next[i] = CellData::AIR;
            grid.moved[above] = true;
            return;
        }
    }

    // Drift sideways
    let dx: i32 = rng.random_range(-1..=1);
    let nx = x as i32 + dx;
    if nx >= 0 && nx < grid.w as i32 {
        let si = grid.idx(nx as usize, y);
        if grid.next[si].kind == Cell::Air {
            grid.next[si] = new_cell;
            grid.next[i] = CellData::AIR;
            grid.moved[si] = true;
            return;
        }
    }

    grid.next[i] = new_cell;
    grid.moved[i] = true;
}
