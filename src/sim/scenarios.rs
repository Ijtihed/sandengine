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
    "Volcano",
    "Waterfall",
    "Dam Break",
    "Lava Flow",
    "Layer Cake",
    "Dual Sand Sort",
    "Oil Spill",
    "Acid Rain",
    "Erosion Cliff",
    "Geyser",
    "River Delta",
    "Sandstorm",
    "Explosion",
];

pub const SCENARIO_NAMES_3D: &[&str] = &[
    "Pyramid on Floor",
    "Falling Sphere",
    "Hourglass 3D",
    "Sand Rain 3D",
    "Block in Pile 3D",
    "Volcano 3D",
    "Waterfall 3D",
    "Dam Break 3D",
    "Lava Flow 3D",
    "Layer Cake 3D",
    "Dual Sand Sort 3D",
    "Oil Spill 3D",
    "Acid Rain 3D",
    "Erosion Cliff 3D",
    "Geyser 3D",
    "River Delta 3D",
    "Sandstorm 3D",
    "Explosion 3D",
];

pub fn apply_2d(index: usize, grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    grid.clear();
    match index % SCENARIO_NAMES_2D.len() {
        0 => triangle_on_floor_2d(grid, block, rng),
        1 => falling_circle_2d(grid, block, rng),
        2 => hourglass_2d(grid, block, rng),
        3 => sand_rain_2d(grid, block),
        4 => block_in_pile_2d(grid, block, rng),
        5 => volcano_2d(grid, block, rng),
        6 => waterfall_2d(grid, block, rng),
        7 => dam_break_2d(grid, block, rng),
        8 => lava_flow_2d(grid, block, rng),
        9 => layer_cake_2d(grid, block, rng),
        10 => dual_sand_sort_2d(grid, block, rng),
        11 => oil_spill_2d(grid, block, rng),
        12 => acid_rain_2d(grid, block, rng),
        13 => erosion_cliff_2d(grid, block, rng),
        14 => geyser_2d(grid, block, rng),
        15 => river_delta_2d(grid, block, rng),
        16 => sandstorm_2d(grid, block),
        17 => explosion_2d(grid, block, rng),
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
        5 => volcano_3d(grid, block, rng),
        6 => waterfall_3d(grid, block, rng),
        7 => dam_break_3d(grid, block, rng),
        8 => lava_flow_3d(grid, block, rng),
        9 => layer_cake_3d(grid, block, rng),
        10 => dual_sand_sort_3d(grid, block, rng),
        11 => oil_spill_3d(grid, block, rng),
        12 => acid_rain_3d(grid, block, rng),
        13 => erosion_cliff_3d(grid, block, rng),
        14 => geyser_3d(grid, block, rng),
        15 => river_delta_3d(grid, block, rng),
        16 => sandstorm_3d(grid, block),
        17 => explosion_3d(grid, block, rng),
        _ => unreachable!(),
    }
    block.rasterize(grid);
}

// ===================== 2D SCENARIOS =====================

fn triangle_on_floor_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let cx = grid.w / 2;
    let base = grid.w / 3;
    let peak_y = grid.h / 4;
    let floor_y = grid.h - 10;
    for y in peak_y..floor_y {
        let frac = (y - peak_y) as f32 / (floor_y - peak_y) as f32;
        let hw = (frac * base as f32 / 2.0) as usize;
        for x in cx.saturating_sub(hw)..=(cx + hw).min(grid.w - 1) {
            grid.set(x, y, CellData::sand(rng.random()));
        }
    }
    *block = Block2D::new(grid.w as f32 * 0.8, grid.h as f32 * 0.5, 30.0, 15.0);
}

fn falling_circle_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let cx = grid.w as f32 / 2.0;
    let cy = grid.h as f32 / 4.0;
    let r = (grid.w.min(grid.h) as f32 / 6.0).min(120.0);
    for y in 0..grid.h {
        for x in 0..grid.w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r { grid.set(x, y, CellData::sand(rng.random())); }
        }
    }
    *block = Block2D::new(cx, grid.h as f32 * 0.75, 40.0, 15.0);
}

fn hourglass_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let w = grid.w; let h = grid.h;
    let mid_y = h / 2; let gap = 8; let ww = 8;
    let inset = w / 5;
    for y in 0..h {
        for wx in 0..ww {
            grid.set(inset + wx, y, CellData::stone(rng.random()));
            grid.set(w - inset - ww + wx, y, CellData::stone(rng.random()));
        }
    }
    for x in (inset + ww)..(w - inset - ww) {
        let cx = w / 2;
        if (x as i32 - cx as i32).unsigned_abs() > gap {
            for wy in 0..ww { grid.set(x, mid_y + wy, CellData::stone(rng.random())); }
        }
    }
    for y in ww..mid_y {
        for x in (inset + ww + 1)..(w - inset - ww - 1) {
            grid.set(x, y, CellData::sand(rng.random()));
        }
    }
    *block = Block2D::new(w as f32 / 2.0, mid_y as f32 + 2.0, gap as f32, 3.0);
}

fn sand_rain_2d(grid: &mut Grid2D, block: &mut Block2D) {
    *block = Block2D::new(grid.w as f32 / 2.0, grid.h as f32 / 2.0, 50.0, 15.0);
}

fn block_in_pile_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let fill = grid.h / 2;
    for y in fill..grid.h { for x in 0..grid.w { grid.set(x, y, CellData::sand(rng.random())); } }
    *block = Block2D::new(grid.w as f32 / 2.0, (fill as f32 + grid.h as f32) / 2.0, 35.0, 20.0);
}

fn volcano_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let cx = grid.w / 2;
    let base_y = grid.h - 5;
    let peak_y = grid.h / 3;
    for y in peak_y..base_y {
        let frac = (y - peak_y) as f32 / (base_y - peak_y) as f32;
        let hw = (frac * grid.w as f32 / 4.0) as usize;
        for x in cx.saturating_sub(hw)..=(cx + hw).min(grid.w - 1) {
            grid.set(x, y, CellData::stone(rng.random()));
        }
    }
    // Crater at top
    let crater_w = 15;
    for y in peak_y..(peak_y + 20) {
        for x in cx.saturating_sub(crater_w)..=(cx + crater_w).min(grid.w - 1) {
            grid.set(x, y, CellData::AIR);
        }
    }
    *block = Block2D::new(grid.w as f32 * 0.2, grid.h as f32 * 0.5, 25.0, 12.0);
}

fn waterfall_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    // Stone ledges
    let ledge_y = grid.h / 3;
    for x in 0..(grid.w * 2 / 3) { grid.set(x, ledge_y, CellData::stone(rng.random())); }
    let ledge2_y = grid.h * 2 / 3;
    for x in (grid.w / 3)..grid.w { grid.set(x, ledge2_y, CellData::stone(rng.random())); }
    // Pre-fill some water on top ledge
    for y in (ledge_y - 40)..ledge_y {
        for x in 0..(grid.w / 3) { grid.set(x, y, CellData::water(rng.random())); }
    }
    *block = Block2D::new(grid.w as f32 / 2.0, ledge_y as f32 - 20.0, 30.0, 10.0);
}

fn dam_break_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let dam_x = grid.w / 3;
    let dam_w = 8;
    // Stone dam wall
    for y in (grid.h / 4)..grid.h {
        for wx in 0..dam_w { grid.set(dam_x + wx, y, CellData::stone(rng.random())); }
    }
    // Water behind the dam
    for y in (grid.h / 3)..grid.h {
        for x in 0..dam_x { grid.set(x, y, CellData::water(rng.random())); }
    }
    // Sand on the other side
    for y in (grid.h * 3 / 4)..grid.h {
        for x in (dam_x + dam_w + 20)..grid.w { grid.set(x, y, CellData::sand(rng.random())); }
    }
    *block = Block2D::new(dam_x as f32 + 4.0, grid.h as f32 / 2.0, dam_w as f32 / 2.0, 30.0);
}

fn lava_flow_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    // Stone slope
    for y in 0..grid.h {
        let slope_x = (y as f32 / grid.h as f32 * grid.w as f32 * 0.6) as usize;
        for x in slope_x..grid.w.min(slope_x + 6) { grid.set(x, y, CellData::stone(rng.random())); }
    }
    // Fire (lava) pool at top
    for y in 10..60 {
        for x in 10..80 { grid.set(x, y, CellData::fire(rng.random_range(60..120))); }
    }
    *block = Block2D::new(grid.w as f32 * 0.7, grid.h as f32 * 0.4, 25.0, 15.0);
}

fn layer_cake_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let layers = 8;
    let layer_h = grid.h / (layers + 2);
    for l in 0..layers {
        let y_start = grid.h - (l + 1) * layer_h;
        let y_end = y_start + layer_h;
        for y in y_start..y_end {
            for x in 0..grid.w {
                let cell = match l % 4 {
                    0 => CellData::sand(rng.random()),
                    1 => CellData::water(rng.random()),
                    2 => CellData::stone(rng.random()),
                    3 => CellData::sand(rng.random()),
                    _ => CellData::AIR,
                };
                grid.set(x, y, cell);
            }
        }
    }
    *block = Block2D::new(grid.w as f32 / 2.0, grid.h as f32 / 2.0, 40.0, 20.0);
}

fn dual_sand_sort_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let wall_x = grid.w / 2;
    let wall_w = 6;
    let fill_top = grid.h / 4;
    let gap_h = 80;
    let gap_top = grid.h - gap_h - 40;
    for y in 0..grid.h {
        if y >= gap_top && y < gap_top + gap_h { continue; }
        for wx in 0..wall_w {
            grid.set(wall_x + wx, y, CellData::stone(rng.random()));
        }
    }
    for y in fill_top..grid.h {
        for x in 0..wall_x { grid.set(x, y, CellData::gravel(rng.random())); }
    }
    for y in fill_top..grid.h {
        for x in (wall_x + wall_w)..grid.w { grid.set(x, y, CellData::sand(rng.random())); }
    }
    *block = Block2D::new(
        wall_x as f32 + 3.0, (gap_top + gap_h / 2) as f32,
        wall_w as f32 / 2.0 + 1.0, gap_h as f32 / 2.0 + 1.0,
    );
}

fn oil_spill_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let w = grid.w;
    let h = grid.h;
    let left = w / 4;
    let right = w * 3 / 4;
    let top = h / 2;
    let wall_t = 8;
    for x in left..right {
        for dy in 0..wall_t { grid.set(x, h - 1 - dy, CellData::stone(rng.random())); }
    }
    for y in top..h {
        for dx in 0..wall_t {
            grid.set(left + dx, y, CellData::stone(rng.random()));
            grid.set(right - 1 - dx, y, CellData::stone(rng.random()));
        }
    }
    for y in (top + 80)..(h - wall_t) {
        for x in (left + wall_t)..(right - wall_t) { grid.set(x, y, CellData::water(rng.random())); }
    }
    for y in (top + 50)..(top + 80) {
        for x in (left + wall_t)..(right - wall_t) { grid.set(x, y, CellData::oil(rng.random())); }
    }
    *block = Block2D::new(w as f32 / 2.0, top as f32 - 40.0, 30.0, 15.0);
}

fn acid_rain_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let w = grid.w;
    let h = grid.h;
    let pillar_w = 30;
    let spacing = w / 6;
    for i in 1..=5usize {
        let px = i * spacing;
        let top = h / 3 + (i % 3) * 60;
        for y in top..h {
            for dx in 0..pillar_w {
                let x = px.saturating_sub(pillar_w / 2) + dx;
                if x < w { grid.set(x, y, CellData::stone(rng.random())); }
            }
        }
    }
    let arch_y = h * 2 / 3;
    for x in (2 * spacing)..(4 * spacing).min(w) {
        for dy in 0..12 { grid.set(x, arch_y + dy, CellData::stone(rng.random())); }
    }
    *block = Block2D::new(w as f32 / 2.0, h as f32 / 5.0, 25.0, 12.0);
}

fn erosion_cliff_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let w = grid.w;
    let h = grid.h;
    let cliff_w = w / 3;
    let cliff_top = h / 6;
    for y in cliff_top..h {
        for x in 0..cliff_w { grid.set(x, y, CellData::stone(rng.random())); }
    }
    for y in (cliff_top - 30)..cliff_top {
        for x in 0..cliff_w { grid.set(x, y, CellData::water(rng.random())); }
    }
    *block = Block2D::new(w as f32 * 0.6, h as f32 / 2.0, 25.0, 15.0);
}

fn geyser_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let cx = grid.w / 2;
    let h = grid.h;
    let cone_top = h / 2;
    let cone_base_hw = grid.w / 4;
    for y in cone_top..h {
        let frac = (y - cone_top) as f32 / (h - cone_top) as f32;
        let hw = (frac * cone_base_hw as f32) as usize;
        for x in cx.saturating_sub(hw)..=(cx + hw).min(grid.w - 1) {
            grid.set(x, y, CellData::stone(rng.random()));
        }
    }
    let channel_hw = 4;
    for y in cone_top..(h - 20) {
        for x in cx.saturating_sub(channel_hw)..=(cx + channel_hw).min(grid.w - 1) {
            grid.set(x, y, CellData::AIR);
        }
    }
    *block = Block2D::new(grid.w as f32 * 0.8, h as f32 * 0.3, 20.0, 12.0);
}

fn river_delta_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let w = grid.w;
    let h = grid.h;
    for y in (h / 2)..h {
        for x in 0..w { grid.set(x, y, CellData::sand(rng.random())); }
    }
    for y in (h / 4)..h {
        let wobble = ((y as f32 * 0.03).sin() * 40.0) as i32;
        let cx = w as i32 / 4 + wobble;
        for dx in -30..30 {
            let x = (cx + dx).clamp(0, w as i32 - 1) as usize;
            grid.set(x, y, CellData::AIR);
        }
    }
    *block = Block2D::new(w as f32 * 0.7, h as f32 * 0.3, 25.0, 12.0);
}

fn sandstorm_2d(grid: &mut Grid2D, block: &mut Block2D) {
    *block = Block2D::new(grid.w as f32 / 3.0, grid.h as f32 / 2.0, 40.0, 20.0);
}

fn explosion_2d(grid: &mut Grid2D, block: &mut Block2D, rng: &mut impl Rng) {
    let cx = grid.w as f32 / 2.0;
    let cy = grid.h as f32 / 2.0;
    let fire_r = (grid.w.min(grid.h) as f32 / 5.0).min(150.0);
    let sand_r = fire_r + 80.0;
    for y in 0..grid.h {
        for x in 0..grid.w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= fire_r {
                grid.set(x, y, CellData::fire(rng.random_range(80..150)));
            } else if dist <= sand_r {
                grid.set(x, y, CellData::sand(rng.random()));
            }
        }
    }
    *block = Block2D::new(grid.w as f32 * 0.85, grid.h as f32 * 0.15, 25.0, 15.0);
}

// ===================== 3D SCENARIOS =====================

fn pyramid_on_floor_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx / 2; let cz = grid.sz / 2;
    let max_r = grid.sx.min(grid.sz) / 3;
    for y in 0..max_r {
        let r = (max_r - y) as f32;
        for z in 0..grid.sz { for x in 0..grid.sx {
            let dx = x as f32 - cx as f32;
            let dz = z as f32 - cz as f32;
            if dx.abs() <= r && dz.abs() <= r { grid.set(x, y, z, CellData::sand(rng.random())); }
        }}
    }
    *block = Block3D::new(cx as f32 + max_r as f32, max_r as f32 / 2.0, cz as f32, 8.0, 8.0, 8.0);
}

fn falling_sphere_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0;
    let cy = grid.sy as f32 * 0.75;
    let cz = grid.sz as f32 / 2.0;
    let r = (grid.sx.min(grid.sz) as f32 / 5.0).min(24.0);
    for y in 0..grid.sy { for z in 0..grid.sz { for x in 0..grid.sx {
        let d = (x as f32 - cx).powi(2) + (y as f32 - cy).powi(2) + (z as f32 - cz).powi(2);
        if d <= r * r { grid.set(x, y, z, CellData::sand(rng.random())); }
    }}}
    *block = Block3D::new(cx, 8.0, cz, 10.0, 6.0, 10.0);
}

fn hourglass_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0; let cz = grid.sz as f32 / 2.0;
    let mid_y = grid.sy / 2;
    let r = grid.sx.min(grid.sz) as f32 / 3.0;
    let gap_r = 4.0_f32;
    for y in 0..grid.sy { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx; let dz = z as f32 - cz;
        let dist = (dx * dx + dz * dz).sqrt();
        if dist >= r && dist <= r + 3.0 { grid.set(x, y, z, CellData::stone(rng.random())); }
    }}}
    for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx; let dz = z as f32 - cz;
        let dist = (dx * dx + dz * dz).sqrt();
        if dist < r && dist > gap_r { grid.set(x, mid_y, z, CellData::stone(rng.random())); }
    }}
    for y in (mid_y + 1)..grid.sy { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx; let dz = z as f32 - cz;
        if (dx * dx + dz * dz).sqrt() < r { grid.set(x, y, z, CellData::sand(rng.random())); }
    }}}
    *block = Block3D::new(cx, mid_y as f32, cz, gap_r, 1.0, gap_r);
}

fn sand_rain_3d(grid: &mut Grid3D, block: &mut Block3D) {
    let _ = grid;
    *block = Block3D::new(grid.sx as f32 / 2.0, grid.sy as f32 / 2.0, grid.sz as f32 / 2.0, 10.0, 6.0, 10.0);
}

fn block_in_pile_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let fill = grid.sy / 3;
    for y in 0..fill { for z in 0..grid.sz { for x in 0..grid.sx {
        grid.set(x, y, z, CellData::sand(rng.random()));
    }}}
    *block = Block3D::new(grid.sx as f32 / 2.0, fill as f32 / 2.0, grid.sz as f32 / 2.0, 8.0, 8.0, 8.0);
}

fn volcano_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0; let cz = grid.sz as f32 / 2.0;
    let peak = grid.sy * 2 / 3;
    for y in 0..peak { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx; let dz = z as f32 - cz;
        let dist = (dx * dx + dz * dz).sqrt();
        let r = (peak - y) as f32 * 0.6;
        if dist <= r && dist >= r - 3.0 { grid.set(x, y, z, CellData::stone(rng.random())); }
        if dist < r - 3.0 && y < peak / 3 { grid.set(x, y, z, CellData::stone(rng.random())); }
    }}}
    *block = Block3D::new(cx + 30.0, peak as f32 / 2.0, cz, 6.0, 6.0, 6.0);
}

fn waterfall_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let ledge_y = grid.sy / 3;
    for z in 0..grid.sz { for x in 0..(grid.sx * 2 / 3) {
        grid.set(x, ledge_y, z, CellData::stone(rng.random()));
    }}
    for y in (ledge_y + 1)..(ledge_y + 15) { for z in 5..(grid.sz - 5) { for x in 5..(grid.sx / 3) {
        grid.set(x, y, z, CellData::water(rng.random()));
    }}}
    *block = Block3D::new(grid.sx as f32 / 2.0, ledge_y as f32 + 10.0, grid.sz as f32 / 2.0, 8.0, 6.0, 8.0);
}

fn dam_break_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let dam_x = grid.sx / 3;
    for y in 0..grid.sy { for z in 0..grid.sz { for wx in 0..4 {
        grid.set(dam_x + wx, y, z, CellData::stone(rng.random()));
    }}}
    for y in 0..(grid.sy * 2 / 3) { for z in 4..(grid.sz - 4) { for x in 4..dam_x {
        grid.set(x, y, z, CellData::water(rng.random()));
    }}}
    *block = Block3D::new(dam_x as f32 + 2.0, grid.sy as f32 / 3.0, grid.sz as f32 / 2.0, 2.0, 20.0, grid.sz as f32 / 2.0 - 4.0);
}

fn lava_flow_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0; let cz = grid.sz as f32 / 2.0;
    // Stone bowl
    for y in 0..(grid.sy / 4) { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx; let dz = z as f32 - cz;
        let dist = (dx * dx + dz * dz).sqrt();
        let r = grid.sx as f32 / 2.5;
        if dist <= r && dist >= r - 3.0 { grid.set(x, y, z, CellData::stone(rng.random())); }
        if y == 0 && dist < r { grid.set(x, y, z, CellData::stone(rng.random())); }
    }}}
    // Fire pool
    for y in 1..(grid.sy / 8) { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx; let dz = z as f32 - cz;
        if (dx * dx + dz * dz).sqrt() < grid.sx as f32 / 3.0 {
            grid.set(x, y, z, CellData::fire(rng.random_range(60..120)));
        }
    }}}
    *block = Block3D::new(cx, grid.sy as f32 / 3.0, cz, 8.0, 6.0, 8.0);
}

fn layer_cake_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let layers = 6;
    let lh = grid.sy / (layers + 2);
    for l in 0..layers {
        let y0 = l * lh;
        for y in y0..(y0 + lh) { for z in 0..grid.sz { for x in 0..grid.sx {
            let cell = match l % 4 {
                0 => CellData::sand(rng.random()),
                1 => CellData::water(rng.random()),
                2 => CellData::stone(rng.random()),
                3 => CellData::sand(rng.random()),
                _ => CellData::AIR,
            };
            grid.set(x, y, z, cell);
        }}}
    }
    *block = Block3D::new(grid.sx as f32 / 2.0, grid.sy as f32 / 2.0, grid.sz as f32 / 2.0, 10.0, 10.0, 10.0);
}

fn dual_sand_sort_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let wall_x = grid.sx / 2;
    let wall_w = 3;
    let fill_h = grid.sy * 3 / 4;
    let gap_h = 12;
    let gap_bot = 2;
    for y in 0..grid.sy {
        if y >= gap_bot && y < gap_bot + gap_h { continue; }
        for z in 0..grid.sz { for wx in 0..wall_w {
            grid.set(wall_x + wx, y, z, CellData::stone(rng.random()));
        }}
    }
    for y in 0..fill_h { for z in 0..grid.sz { for x in 0..wall_x {
        grid.set(x, y, z, CellData::gravel(rng.random()));
    }}}
    for y in 0..fill_h { for z in 0..grid.sz { for x in (wall_x + wall_w)..grid.sx {
        grid.set(x, y, z, CellData::sand(rng.random()));
    }}}
    *block = Block3D::new(
        wall_x as f32 + 1.5, (gap_bot + gap_h / 2) as f32, grid.sz as f32 / 2.0,
        wall_w as f32 / 2.0 + 0.5, gap_h as f32 / 2.0 + 0.5, grid.sz as f32 / 2.0,
    );
}

fn oil_spill_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0;
    let cz = grid.sz as f32 / 2.0;
    let basin_r = grid.sx as f32 / 3.0;
    let wall_h = grid.sy / 3;
    for y in 0..wall_h { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx;
        let dz = z as f32 - cz;
        let dist = (dx * dx + dz * dz).sqrt();
        if y == 0 && dist < basin_r { grid.set(x, y, z, CellData::stone(rng.random())); }
        if dist >= basin_r - 2.0 && dist <= basin_r { grid.set(x, y, z, CellData::stone(rng.random())); }
    }}}
    for y in 1..(wall_h - 6) { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx;
        let dz = z as f32 - cz;
        if (dx * dx + dz * dz).sqrt() < basin_r - 3.0 {
            grid.set(x, y, z, CellData::water(rng.random()));
        }
    }}}
    for y in (wall_h - 6)..(wall_h - 3) { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx;
        let dz = z as f32 - cz;
        if (dx * dx + dz * dz).sqrt() < basin_r - 3.0 {
            grid.set(x, y, z, CellData::oil(rng.random()));
        }
    }}}
    *block = Block3D::new(cx, wall_h as f32 + 5.0, cz, 8.0, 6.0, 8.0);
}

fn acid_rain_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let n = 4usize;
    let sp_x = grid.sx / (n + 1);
    let sp_z = grid.sz / (n + 1);
    for i in 1..=n { for j in 1..=n {
        let px = i * sp_x;
        let pz = j * sp_z;
        let pillar_h = grid.sy / 3 + ((i + j) % 3) * (grid.sy / 10);
        let pr = 4;
        for y in 0..pillar_h {
            for z in pz.saturating_sub(pr)..(pz + pr).min(grid.sz) {
                for x in px.saturating_sub(pr)..(px + pr).min(grid.sx) {
                    grid.set(x, y, z, CellData::stone(rng.random()));
                }
            }
        }
    }}
    *block = Block3D::new(grid.sx as f32 / 2.0, grid.sy as f32 * 0.7, grid.sz as f32 / 2.0, 8.0, 6.0, 8.0);
}

fn erosion_cliff_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cliff_x = grid.sx / 3;
    let cliff_top = grid.sy * 3 / 4;
    for y in 0..cliff_top { for z in 0..grid.sz { for x in 0..cliff_x {
        grid.set(x, y, z, CellData::stone(rng.random()));
    }}}
    for y in cliff_top..(cliff_top + 5) { for z in 4..(grid.sz - 4) { for x in 4..cliff_x {
        grid.set(x, y, z, CellData::water(rng.random()));
    }}}
    *block = Block3D::new(grid.sx as f32 * 0.6, grid.sy as f32 / 2.0, grid.sz as f32 / 2.0, 8.0, 6.0, 8.0);
}

fn geyser_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0;
    let cz = grid.sz as f32 / 2.0;
    let peak = grid.sy / 2;
    let base_r = grid.sx as f32 / 3.0;
    let channel_r = 4.0_f32;
    for y in 0..peak { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx;
        let dz = z as f32 - cz;
        let dist = (dx * dx + dz * dz).sqrt();
        let r = base_r * (1.0 - y as f32 / peak as f32);
        if dist <= r { grid.set(x, y, z, CellData::stone(rng.random())); }
    }}}
    for y in 0..(peak - 3) { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx;
        let dz = z as f32 - cz;
        if (dx * dx + dz * dz).sqrt() < channel_r { grid.set(x, y, z, CellData::AIR); }
    }}}
    *block = Block3D::new(cx + 30.0, peak as f32, cz, 6.0, 6.0, 6.0);
}

fn river_delta_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    for y in 0..(grid.sy / 3) { for z in 0..grid.sz { for x in 0..grid.sx {
        if rng.random_bool(0.7) { grid.set(x, y, z, CellData::sand(rng.random())); }
    }}}
    let channel_r = 8;
    for y in 0..(grid.sy / 3) { for z in 0..grid.sz {
        let cx = grid.sx / 4 + ((z as f32 / 10.0).sin() * 8.0) as usize;
        for x in cx.saturating_sub(channel_r)..(cx + channel_r).min(grid.sx) {
            grid.set(x, y, z, CellData::AIR);
        }
    }}
    *block = Block3D::new(grid.sx as f32 * 0.7, grid.sy as f32 * 0.5, grid.sz as f32 / 2.0, 8.0, 6.0, 8.0);
}

fn sandstorm_3d(grid: &mut Grid3D, block: &mut Block3D) {
    let _ = grid;
    *block = Block3D::new(grid.sx as f32 / 2.0, grid.sy as f32 / 2.0, grid.sz as f32 / 2.0, 8.0, 8.0, 8.0);
}

fn explosion_3d(grid: &mut Grid3D, block: &mut Block3D, rng: &mut impl Rng) {
    let cx = grid.sx as f32 / 2.0;
    let cy = grid.sy as f32 / 2.0;
    let cz = grid.sz as f32 / 2.0;
    let fire_r = grid.sx as f32 / 5.0;
    let sand_r = fire_r + 12.0;
    for y in 0..grid.sy { for z in 0..grid.sz { for x in 0..grid.sx {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let dz = z as f32 - cz;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist <= fire_r {
            grid.set(x, y, z, CellData::fire(rng.random_range(80..150)));
        } else if dist <= sand_r {
            grid.set(x, y, z, CellData::sand(rng.random()));
        }
    }}}
    *block = Block3D::new(cx + sand_r + 5.0, cy, cz, 6.0, 6.0, 6.0);
}

// ===================== LIVE SPAWNERS =====================

pub fn spawner_tick_2d(scenario: usize, grid: &mut Grid2D, rng: &mut impl Rng) {
    match scenario % SCENARIO_NAMES_2D.len() {
        3 => rain_sand_2d(grid, rng),
        5 => volcano_spawn_2d(grid, rng),
        6 => waterfall_spawn_2d(grid, rng),
        11 => oil_spill_spawn_2d(grid, rng),
        12 => acid_rain_spawn_2d(grid, rng),
        13 => erosion_cliff_spawn_2d(grid, rng),
        14 => geyser_spawn_2d(grid, rng),
        15 => river_delta_spawn_2d(grid, rng),
        16 => sandstorm_spawn_2d(grid, rng),
        _ => {}
    }
}

pub fn spawner_tick_3d(scenario: usize, grid: &mut Grid3D, rng: &mut impl Rng) {
    match scenario % SCENARIO_NAMES_3D.len() {
        3 => rain_sand_3d(grid, rng),
        5 => volcano_spawn_3d(grid, rng),
        6 => waterfall_spawn_3d(grid, rng),
        11 => oil_spill_spawn_3d(grid, rng),
        12 => acid_rain_spawn_3d(grid, rng),
        13 => erosion_cliff_spawn_3d(grid, rng),
        14 => geyser_spawn_3d(grid, rng),
        15 => river_delta_spawn_3d(grid, rng),
        16 => sandstorm_spawn_3d(grid, rng),
        _ => {}
    }
}

fn rain_sand_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    for x in (0..grid.w).step_by(2) {
        if rng.random_bool(0.4) && grid.get(x, 0).kind == super::material::Cell::Air {
            grid.set(x, 0, CellData::sand(rng.random()));
        }
    }
}

fn volcano_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    let cx = grid.w / 2;
    for dx in -6..=6i32 {
        let x = (cx as i32 + dx).clamp(0, grid.w as i32 - 1) as usize;
        if rng.random_bool(0.3) && grid.get(x, 0).kind == super::material::Cell::Air {
            if rng.random_bool(0.5) {
                grid.set(x, 0, CellData::fire(rng.random_range(80..120)));
            } else {
                grid.set(x, 0, CellData::sand(rng.random()));
            }
        }
    }
}

fn waterfall_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    for x in 5..30 {
        if rng.random_bool(0.5) && grid.get(x, 0).kind == super::material::Cell::Air {
            grid.set(x, 0, CellData::water(rng.random()));
        }
    }
}

fn oil_spill_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    let left = grid.w / 4 + 10;
    let right = grid.w * 3 / 4 - 10;
    for x in (left..right).step_by(4) {
        if rng.random_bool(0.15) && grid.get(x, 0).kind == super::material::Cell::Air {
            grid.set(x, 0, CellData::oil(rng.random()));
        }
    }
}

fn acid_rain_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    for x in (0..grid.w).step_by(3) {
        if rng.random_bool(0.15) && grid.get(x, 0).kind == super::material::Cell::Air {
            grid.set(x, 0, CellData::acid(rng.random_range(60..100)));
        }
    }
}

fn erosion_cliff_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    for x in 0..(grid.w / 3) {
        if rng.random_bool(0.4) && grid.get(x, 0).kind == super::material::Cell::Air {
            grid.set(x, 0, CellData::water(rng.random()));
        }
    }
    for x in 10..(grid.w / 4) {
        if rng.random_bool(0.05) && grid.get(x, 0).kind == super::material::Cell::Air {
            grid.set(x, 0, CellData::acid(rng.random_range(60..100)));
        }
    }
}

fn geyser_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    let cx = grid.w / 2;
    let bottom = grid.h - 25;
    for dx in -4..=4i32 {
        let x = (cx as i32 + dx).clamp(0, grid.w as i32 - 1) as usize;
        if rng.random_bool(0.5) && grid.get(x, bottom).kind == super::material::Cell::Air {
            if rng.random_bool(0.6) {
                grid.set(x, bottom, CellData::water(rng.random()));
            } else {
                grid.set(x, bottom, CellData::steam(rng.random_range(60..120)));
            }
        }
    }
}

fn river_delta_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    for x in 0..40 {
        if rng.random_bool(0.5) && grid.get(x, 0).kind == super::material::Cell::Air {
            grid.set(x, 0, CellData::water(rng.random()));
        }
    }
}

fn sandstorm_spawn_2d(grid: &mut Grid2D, rng: &mut impl Rng) {
    let x = grid.w - 1;
    for y in (0..grid.h).step_by(2) {
        if rng.random_bool(0.6) && grid.get(x, y).kind == super::material::Cell::Air {
            grid.set(x, y, CellData::sand(rng.random()));
        }
    }
}

fn rain_sand_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let top = grid.sy - 1;
    for z in (0..grid.sz).step_by(2) {
        for x in (0..grid.sx).step_by(2) {
            if rng.random_bool(0.08) && grid.get(x, top, z).kind == super::material::Cell::Air {
                grid.set(x, top, z, CellData::sand(rng.random()));
            }
        }
    }
}

fn volcano_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let cx = grid.sx / 2; let cz = grid.sz / 2;
    let peak = grid.sy * 2 / 3;
    for dz in -4..=4i32 { for dx in -4..=4i32 {
        let x = (cx as i32 + dx).clamp(0, grid.sx as i32 - 1) as usize;
        let z = (cz as i32 + dz).clamp(0, grid.sz as i32 - 1) as usize;
        if rng.random_bool(0.15) && grid.get(x, peak, z).kind == super::material::Cell::Air {
            grid.set(x, peak, z, CellData::fire(rng.random_range(80..120)));
        }
    }}
}

fn waterfall_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let ledge_y = grid.sy / 3;
    for z in 10..(grid.sz - 10) { for x in 2..8 {
        if rng.random_bool(0.3) && grid.get(x, ledge_y + 1, z).kind == super::material::Cell::Air {
            grid.set(x, ledge_y + 1, z, CellData::water(rng.random()));
        }
    }}
}

fn oil_spill_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let cx = grid.sx / 2;
    let cz = grid.sz / 2;
    let top = grid.sy - 1;
    for dz in -5..=5i32 { for dx in -5..=5i32 {
        let x = (cx as i32 + dx).clamp(0, grid.sx as i32 - 1) as usize;
        let z = (cz as i32 + dz).clamp(0, grid.sz as i32 - 1) as usize;
        if rng.random_bool(0.15) && grid.get(x, top, z).kind == super::material::Cell::Air {
            grid.set(x, top, z, CellData::oil(rng.random()));
        }
    }}
}

fn acid_rain_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let top = grid.sy - 1;
    for z in (0..grid.sz).step_by(3) { for x in (0..grid.sx).step_by(3) {
        if rng.random_bool(0.05) && grid.get(x, top, z).kind == super::material::Cell::Air {
            grid.set(x, top, z, CellData::acid(rng.random_range(60..100)));
        }
    }}
}

fn erosion_cliff_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let top = grid.sy - 1;
    let cliff_x = grid.sx / 3;
    for z in 4..(grid.sz - 4) { for x in 4..cliff_x {
        if rng.random_bool(0.1) && grid.get(x, top, z).kind == super::material::Cell::Air {
            grid.set(x, top, z, CellData::water(rng.random()));
        }
    }}
    for z in 10..(grid.sz - 10) { for x in 2..(cliff_x / 2) {
        if rng.random_bool(0.02) && grid.get(x, top, z).kind == super::material::Cell::Air {
            grid.set(x, top, z, CellData::acid(rng.random_range(60..100)));
        }
    }}
}

fn geyser_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let cx = grid.sx / 2;
    let cz = grid.sz / 2;
    let bottom = 3;
    for dz in -3..=3i32 { for dx in -3..=3i32 {
        let x = (cx as i32 + dx).clamp(0, grid.sx as i32 - 1) as usize;
        let z = (cz as i32 + dz).clamp(0, grid.sz as i32 - 1) as usize;
        if rng.random_bool(0.3) && grid.get(x, bottom, z).kind == super::material::Cell::Air {
            if rng.random_bool(0.6) {
                grid.set(x, bottom, z, CellData::water(rng.random()));
            } else {
                grid.set(x, bottom, z, CellData::steam(rng.random_range(60..120)));
            }
        }
    }}
}

fn river_delta_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let top = grid.sy - 1;
    for z in 4..(grid.sz - 4) { for x in 0..8 {
        if rng.random_bool(0.2) && grid.get(x, top, z).kind == super::material::Cell::Air {
            grid.set(x, top, z, CellData::water(rng.random()));
        }
    }}
}

fn sandstorm_spawn_3d(grid: &mut Grid3D, rng: &mut impl Rng) {
    let x = grid.sx - 1;
    for z in (0..grid.sz).step_by(2) { for y in (0..grid.sy).step_by(2) {
        if rng.random_bool(0.15) && grid.get(x, y, z).kind == super::material::Cell::Air {
            grid.set(x, y, z, CellData::sand(rng.random()));
        }
    }}
}
