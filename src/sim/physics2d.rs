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
            for x in 0..grid.w {
                update_cell(grid, x, y, rng);
            }
        } else {
            for x in (0..grid.w).rev() {
                update_cell(grid, x, y, rng);
            }
        }
    }

    grid.swap_buffers();
}

fn update_cell(grid: &mut Grid2D, x: usize, y: usize, rng: &mut impl Rng) {
    let i = grid.idx(x, y);
    if grid.next[i].kind != Cell::Sand || grid.moved[i] {
        return;
    }

    // Fall straight down
    if y + 1 < grid.h {
        let below = grid.idx(x, y + 1);
        if grid.next[below].kind == Cell::Air {
            grid.next[below] = grid.next[i];
            grid.next[i] = CellData::AIR;
            grid.moved[below] = true;
            return;
        }
    }

    // Diagonal fall with random L/R bias
    if y + 1 < grid.h {
        let go_left_first: bool = rng.random();
        let offsets = if go_left_first {
            [-1i32, 1i32]
        } else {
            [1i32, -1i32]
        };

        for dx in offsets {
            let nx = x as i32 + dx;
            if nx >= 0 && nx < grid.w as i32 {
                let diag = grid.idx(nx as usize, y + 1);
                if grid.next[diag].kind == Cell::Air {
                    grid.next[diag] = grid.next[i];
                    grid.next[i] = CellData::AIR;
                    grid.moved[diag] = true;
                    return;
                }
            }
        }
    }
}
