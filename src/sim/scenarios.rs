use super::block::{Block2D, Block3D};
use super::grid2d::Grid2D;
use super::grid3d::Grid3D;
use super::material::CellData;
use rand::Rng;
use rand::RngExt;

pub const SCENARIO_NAMES_2D: &[&str] = &[
    "Triangle on Floor",
    "Falling Circle",
    "Hourglass",
    "Sand Rain",
    "Block in Pile",
];

pub const SCENARIO_NAMES_3D: &[&str] = &[
    "Pyramid on Floor",
    "Falling Sphere",
    "Hourglass 3D",
    "Sand Rain 3D",
    "Block in Pile 3D",
];

pub fn apply_2d(index: usize, grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    grid.clear();
    match index % SCENARIO_NAMES_2D.len() {
        0 => triangle_on_floor_2d(grid, block, rng),
        1 => falling_circle_2d(grid, block, rng),
        2 => hourglass_2d(grid, block, rng),
        3 => sand_rain_2d(grid, block),
        4 => block_in_pile_2d(grid, block, rng),
        _ => unreachable!(),
    }
    block.rasterize(grid);
}

pub fn apply_3d(index: usize, grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    grid.clear();
    match index % SCENARIO_NAMES_3D.len() {
        0 => pyramid_on_floor_3d(grid, block, rng),
        1 => falling_sphere_3d(grid, block, rng),
        2 => hourglass_3d(grid, block, rng),
        3 => sand_rain_3d(grid, block),
        4 => block_in_pile_3d(grid, block, rng),
        _ => unreachable!(),
    }
    block.rasterize(grid);
}

// --- 2D scenarios (scaled for 1280x960) ---

fn triangle_on_floor_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let cx = grid.w / 2;
    let base = grid.w / 3;
    let peak_y = grid.h / 4;
    let floor_y = grid.h - 10;

    for y in peak_y..floor_y {
        let frac = (y - peak_y) as f32 / (floor_y - peak_y) as f32;
        let half_w = (frac * base as f32 / 2.0) as usize;
        for x in (cx.saturating_sub(half_w))..=(cx + half_w).min(grid.w - 1) {
            grid.set(x, y, CellData::sand(rng.random()));
        }
    }

    *block = Block2D::new(grid.w as f32 * 0.8, grid.h as f32 * 0.5, 30.0, 15.0);
}

fn falling_circle_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let cx = grid.w as f32 / 2.0;
    let cy = grid.h as f32 / 4.0;
    let radius = (grid.w.min(grid.h) as f32 / 6.0).min(120.0);

    for y in 0..grid.h {
        for x in 0..grid.w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= radius * radius {
                grid.set(x, y, CellData::sand(rng.random()));
            }
        }
    }

    *block = Block2D::new(cx, grid.h as f32 * 0.75, 40.0, 15.0);
}

fn hourglass_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let w = grid.w;
    let h = grid.h;
    let mid_y = h / 2;
    let gap = 8;
    let wall_w = 8;
    let chamber_inset = w / 5;

    for y in 0..h {
        for wx in 0..wall_w {
            grid.set(chamber_inset + wx, y, CellData::block());
            grid.set(w - chamber_inset - wall_w + wx, y, CellData::block());
        }
    }

    for x in (chamber_inset + wall_w)..(w - chamber_inset - wall_w) {
        let cx = w / 2;
        if (x as i32 - cx as i32).unsigned_abs() > gap {
            for wy in 0..wall_w {
                grid.set(x, mid_y + wy, CellData::block());
            }
        }
    }

    for y in wall_w..(mid_y) {
        for x in (chamber_inset + wall_w + 1)..(w - chamber_inset - wall_w - 1) {
            grid.set(x, y, CellData::sand(rng.random()));
        }
    }

    *block = Block2D::new(w as f32 / 2.0, mid_y as f32 + 2.0, gap as f32, 3.0);
}

fn sand_rain_2d(grid: &mut Grid2D, block: &mut Block2D) {
    *block = Block2D::new(grid.w as f32 / 2.0, grid.h as f32 / 2.0, 50.0, 15.0);
}

fn block_in_pile_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let fill_from = grid.h / 2;
    for y in fill_from..grid.h {
        for x in 0..grid.w {
            grid.set(x, y, CellData::sand(rng.random()));
        }
    }

    *block = Block2D::new(
        grid.w as f32 / 2.0,
        (fill_from as f32 + grid.h as f32) / 2.0,
        35.0,
        20.0,
    );
}

// --- 3D scenarios (scaled for 128^3) ---

fn pyramid_on_floor_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx / 2;
    let cz = grid.sz / 2;
    let max_r = grid.sx.min(grid.sz) / 3;

    for y in 0..max_r {
        let r = (max_r - y) as f32;
        for z in 0..grid.sz {
            for x in 0..grid.sx {
                let dx = x as f32 - cx as f32;
                let dz = z as f32 - cz as f32;
                if dx.abs() <= r && dz.abs() <= r {
                    grid.set(x, y, z, CellData::sand(rng.random()));
                }
            }
        }
    }

    *block = Block3D::new(cx as f32 + max_r as f32, max_r as f32 / 2.0, cz as f32, 8.0, 8.0, 8.0);
}

fn falling_sphere_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0;
    let cy = grid.sy as f32 * 0.75;
    let cz = grid.sz as f32 / 2.0;
    let r = (grid.sx.min(grid.sz) as f32 / 5.0).min(24.0);

    for y in 0..grid.sy {
        for z in 0..grid.sz {
            for x in 0..grid.sx {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dz = z as f32 - cz;
                if dx * dx + dy * dy + dz * dz <= r * r {
                    grid.set(x, y, z, CellData::sand(rng.random()));
                }
            }
        }
    }

    *block = Block3D::new(cx, 8.0, cz, 10.0, 6.0, 10.0);
}

fn hourglass_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0;
    let cz = grid.sz as f32 / 2.0;
    let mid_y = grid.sy / 2;
    let r = grid.sx.min(grid.sz) as f32 / 3.0;
    let gap_r = 4.0_f32;

    for y in 0..grid.sy {
        for z in 0..grid.sz {
            for x in 0..grid.sx {
                let dx = x as f32 - cx;
                let dz = z as f32 - cz;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist >= r && dist <= r + 3.0 {
                    grid.set(x, y, z, CellData::block());
                }
            }
        }
    }

    for z in 0..grid.sz {
        for x in 0..grid.sx {
            let dx = x as f32 - cx;
            let dz = z as f32 - cz;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist < r && dist > gap_r {
                grid.set(x, mid_y, z, CellData::block());
            }
        }
    }

    for y in (mid_y + 1)..grid.sy {
        for z in 0..grid.sz {
            for x in 0..grid.sx {
                let dx = x as f32 - cx;
                let dz = z as f32 - cz;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist < r {
                    grid.set(x, y, z, CellData::sand(rng.random()));
                }
            }
        }
    }

    *block = Block3D::new(cx, mid_y as f32, cz, gap_r, 1.0, gap_r);
}

fn sand_rain_3d(grid: &mut Grid3D, block: &mut Block3D) {
    let _ = grid;
    *block = Block3D::new(
        grid.sx as f32 / 2.0,
        grid.sy as f32 / 2.0,
        grid.sz as f32 / 2.0,
        10.0,
        6.0,
        10.0,
    );
}

fn block_in_pile_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let fill_from = grid.sy / 3;
    for y in 0..fill_from {
        for z in 0..grid.sz {
            for x in 0..grid.sx {
                grid.set(x, y, z, CellData::sand(rng.random()));
            }
        }
    }

    *block = Block3D::new(
        grid.sx as f32 / 2.0,
        fill_from as f32 / 2.0,
        grid.sz as f32 / 2.0,
        8.0,
        8.0,
        8.0,
    );
}

pub fn rain_tick_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    for x in (0..grid.w).step_by(2) {
        if rng.random_bool(0.4) {
            if grid.get(x, 0).kind == super::material::Cell::Air {
                grid.set(x, 0, CellData::sand(rng.random()));
            }
        }
    }
}

pub fn rain_tick_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let top = grid.sy - 1;
    for z in (0..grid.sz).step_by(2) {
        for x in (0..grid.sx).step_by(2) {
            if rng.random_bool(0.08) {
                if grid.get(x, top, z).kind == super::material::Cell::Air {
                    grid.set(x, top, z, CellData::sand(rng.random()));
                }
            }
        }
    }
}
