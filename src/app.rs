use crate::input::InputState;
use crate::render::camera::OrbitCamera;
use crate::render::context::GpuContext;
use crate::render::renderer2d::Renderer2D;
use crate::render::renderer3d::Renderer3D;
use crate::sim::block::{Block2D, Block3D};
use crate::sim::grid2d::Grid2D;
use crate::sim::grid3d::Grid3D;
use crate::sim::material::{Cell, CellData};
use crate::sim::scenarios;
use crate::sim::timeline::{Timeline2D, Timeline3D};
use crate::ui::{self, Mode, UiState};
use glam::{Vec2, Vec3};
use rand::RngExt;
use std::sync::Arc;
use std::time::Instant;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

const GRID2D_W: usize = 1280;
const GRID2D_H: usize = 960;
const GRID3D_SIZE: usize = 128;
const SIM_DT: f32 = 1.0 / 120.0;
const TIMELINE_CAP_2D: usize = 1500;
const TIMELINE_CAP_3D: usize = 300;

pub struct App {
    pub window: Arc<Window>,
    gpu: GpuContext,

    mode: Mode,
    paused: bool,
    sim_speed: f32,
    tick: u64,

    // 2D
    grid2d: Grid2D,
    block2d: Block2D,
    timeline2d: Timeline2D,
    scenario2d: usize,
    renderer2d: Renderer2D,
    rain_2d: bool,

    // 3D
    grid3d: Grid3D,
    block3d: Block3D,
    timeline3d: Timeline3D,
    scenario3d: usize,
    renderer3d: Renderer3D,
    camera: OrbitCamera,
    rain_3d: bool,
    drag_plane: Option<(Vec3, Vec3)>, // (point_on_plane, normal)

    // Input & UI
    input: InputState,
    ui_state: UiState,
    brush_size: u32,
    egui_consumed: bool,

    // egui
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    // Timing
    rng: rand::rngs::ThreadRng,
    last_frame: Instant,
    sim_accumulator: f32,
    frame_count: u32,
    fps_timer: f32,
    fps: f32,
}

impl App {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let attrs = Window::default_attributes()
            .with_title("Falling Sand Engine -- 2D / 3D / 4D")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 960));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let gpu = GpuContext::new(window.clone());

        let renderer2d = Renderer2D::new(
            &gpu.device,
            gpu.format(),
            GRID2D_W as u32,
            GRID2D_H as u32,
        );
        let renderer3d = Renderer3D::new(&gpu.device, &gpu.queue, gpu.format(), GRID3D_SIZE);

        let mut rng = rand::rng();
        let mut grid2d = Grid2D::new(GRID2D_W, GRID2D_H);
        let mut block2d = Block2D::new(0.0, 0.0, 30.0, 15.0);
        scenarios::apply_2d(0, &mut grid2d, &mut block2d, &mut rng);

        let mut grid3d = Grid3D::new(GRID3D_SIZE, GRID3D_SIZE, GRID3D_SIZE);
        let mut block3d = Block3D::new(64.0, 20.0, 64.0, 8.0, 8.0, 8.0);
        scenarios::apply_3d(0, &mut grid3d, &mut block3d, &mut rng);

        let camera = OrbitCamera::new(
            Vec3::new(GRID3D_SIZE as f32 / 2.0, GRID3D_SIZE as f32 / 3.0, GRID3D_SIZE as f32 / 2.0),
            GRID3D_SIZE as f32 * 1.5,
        );

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            Some(winit::window::Theme::Dark),
            None,
        );
        let egui_renderer =
            egui_wgpu::Renderer::new(&gpu.device, gpu.format(), egui_wgpu::RendererOptions::default());

        Self {
            window,
            gpu,
            mode: Mode::D2,
            paused: false,
            sim_speed: 3.0,
            tick: 0,
            grid2d,
            block2d,
            timeline2d: Timeline2D::new(TIMELINE_CAP_2D),
            scenario2d: 0,
            renderer2d,
            rain_2d: false,
            grid3d,
            block3d,
            timeline3d: Timeline3D::new(TIMELINE_CAP_3D),
            scenario3d: 0,
            renderer3d,
            camera,
            rain_3d: false,
            drag_plane: None,
            input: InputState::new(),
            ui_state: UiState::new(),
            brush_size: 8,
            egui_consumed: false,
            egui_ctx,
            egui_state,
            egui_renderer,
            rng,
            last_frame: Instant::now(),
            sim_accumulator: 0.0,
            frame_count: 0,
            fps_timer: 0.0,
            fps: 60.0,
        }
    }

    pub fn handle_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        let resp = self.egui_state.on_window_event(&self.window, &event);
        self.egui_consumed = resp.consumed;

        if !resp.consumed {
            self.input.handle_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.gpu.resize(size.width, size.height);
                self.camera.aspect = size.width as f32 / size.height as f32;
            }
            WindowEvent::RedrawRequested => self.frame(),
            _ => {}
        }
    }

    fn frame(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.fps_timer += dt;
        self.frame_count += 1;
        if self.fps_timer >= 0.5 {
            self.fps = self.frame_count as f32 / self.fps_timer;
            self.frame_count = 0;
            self.fps_timer = 0.0;
        }

        self.input.begin_frame();
        self.handle_keys();

        if !self.egui_consumed {
            match self.mode {
                Mode::D2 => self.handle_input_2d(),
                Mode::D3 => self.handle_input_3d(),
            }
        }

        // Simulation stepping
        if !self.paused {
            self.sim_accumulator += dt * self.sim_speed;
            let mut steps = 0;
            while self.sim_accumulator >= SIM_DT && steps < 8 {
                self.sim_accumulator -= SIM_DT;
                steps += 1;
                self.step_sim();
            }
            if self.sim_accumulator > SIM_DT * 3.0 {
                self.sim_accumulator = 0.0;
            }
        }

        self.render_frame();
    }

    fn handle_keys(&mut self) {
        if self.input.just_pressed(KeyCode::F2) {
            self.mode = Mode::D2;
        }
        if self.input.just_pressed(KeyCode::F3) {
            self.mode = Mode::D3;
        }
        if self.input.just_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }
        if self.input.just_pressed(KeyCode::KeyH) {
            self.ui_state.show_help = !self.ui_state.show_help;
        }
        if self.input.just_pressed(KeyCode::KeyR) {
            self.reset_scenario();
        }
        if self.input.just_pressed(KeyCode::Tab) {
            self.next_scenario();
        }
        if self.input.just_pressed(KeyCode::Equal) || self.input.just_pressed(KeyCode::NumpadAdd) {
            self.sim_speed = (self.sim_speed * 2.0).min(10.0);
        }
        if self.input.just_pressed(KeyCode::Minus) || self.input.just_pressed(KeyCode::NumpadSubtract) {
            self.sim_speed = (self.sim_speed * 0.5).max(0.1);
        }

        // Timeline scrubbing when paused
        if self.paused {
            let jump = if self.input.shift_held { 50 } else { 1 };
            if self.input.just_pressed(KeyCode::ArrowLeft) {
                match self.mode {
                    Mode::D2 => {
                        self.timeline2d.cursor = self.timeline2d.cursor.saturating_sub(jump);
                        self.restore_timeline_2d();
                    }
                    Mode::D3 => {
                        self.timeline3d.cursor = self.timeline3d.cursor.saturating_sub(jump);
                        self.restore_timeline_3d();
                    }
                }
            }
            if self.input.just_pressed(KeyCode::ArrowRight) {
                match self.mode {
                    Mode::D2 => {
                        let max = self.timeline2d.len().saturating_sub(1);
                        self.timeline2d.cursor = (self.timeline2d.cursor + jump).min(max);
                        self.restore_timeline_2d();
                    }
                    Mode::D3 => {
                        let max = self.timeline3d.len().saturating_sub(1);
                        self.timeline3d.cursor = (self.timeline3d.cursor + jump).min(max);
                        self.restore_timeline_3d();
                    }
                }
            }
        }
    }

    fn handle_input_2d(&mut self) {
        // Scroll changes brush size
        if self.input.scroll_delta != 0.0 {
            let new = self.brush_size as i32 + if self.input.scroll_delta > 0.0 { 2 } else { -2 };
            self.brush_size = new.clamp(1, 40) as u32;
        }

        let (ww, wh) = {
            let s = self.window.inner_size();
            (s.width as f32, s.height as f32)
        };
        let gx = (self.input.mouse_pos.x / ww * self.grid2d.w as f32) as f32;
        let gy = (self.input.mouse_pos.y / wh * self.grid2d.h as f32) as f32;

        // Left click: grab block or place sand
        if self.input.left_just_pressed() {
            if self.block2d.contains(gx, gy) {
                self.block2d.grabbed = true;
                self.block2d.grab_offset = self.block2d.pos - Vec2::new(gx, gy);
            }
        }

        if self.input.left_just_released() {
            self.block2d.grabbed = false;
        }

        if self.block2d.grabbed && self.input.left_down() {
            let new_pos = Vec2::new(gx, gy) + self.block2d.grab_offset;
            self.block2d.move_and_displace(new_pos, &mut self.grid2d);
        } else if self.input.left_down() && !self.block2d.grabbed {
            self.place_sand_2d(gx as i32, gy as i32);
        }

        // Right click: erase
        if self.input.right_down() {
            self.erase_2d(gx as i32, gy as i32);
        }
    }

    fn handle_input_3d(&mut self) {
        // Right drag: orbit camera
        if self.input.right_down() {
            self.camera.orbit(self.input.mouse_delta.x, self.input.mouse_delta.y);
        }

        // Middle drag: pan
        if self.input.middle_down() {
            self.camera.pan(self.input.mouse_delta.x, self.input.mouse_delta.y);
        }

        // Scroll: zoom
        if self.input.scroll_delta != 0.0 {
            self.camera.zoom(self.input.scroll_delta);
        }

        let (ww, wh) = {
            let s = self.window.inner_size();
            (s.width as f32, s.height as f32)
        };
        let ndc_x = self.input.mouse_pos.x / ww * 2.0 - 1.0;
        let ndc_y = 1.0 - self.input.mouse_pos.y / wh * 2.0;

        // Left click: grab block or place sand
        if self.input.left_just_pressed() {
            let (origin, dir) = self.camera.screen_ray(ndc_x, ndc_y);

            if let Some(_t) = self.block3d.ray_intersect(origin, dir) {
                self.block3d.grabbed = true;
                let hit_point = origin + dir * _t;
                self.block3d.grab_offset = self.block3d.pos - hit_point;
                let cam_fwd = (self.camera.target - self.camera.eye()).normalize();
                self.drag_plane = Some((hit_point, -cam_fwd));
            } else {
                // Try to place sand: find first grid cell the ray passes through
                self.place_sand_3d_raycast(origin, dir);
            }
        }

        if self.input.left_just_released() {
            self.block3d.grabbed = false;
            self.drag_plane = None;
        }

        if self.block3d.grabbed && self.input.left_down() {
            if let Some((plane_pt, plane_n)) = self.drag_plane {
                let (origin, dir) = self.camera.screen_ray(ndc_x, ndc_y);
                let denom = dir.dot(plane_n);
                if denom.abs() > 1e-6 {
                    let t = (plane_pt - origin).dot(plane_n) / denom;
                    let hit = origin + dir * t;
                    let new_pos = hit + self.block3d.grab_offset;
                    self.block3d.move_and_displace(new_pos, &mut self.grid3d);
                }
            }
        }
    }

    fn place_sand_2d(&mut self, cx: i32, cy: i32) {
        let r = self.brush_size as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let x = cx + dx;
                    let y = cy + dy;
                    if self.grid2d.in_bounds(x, y) {
                        let ux = x as usize;
                        let uy = y as usize;
                        if self.grid2d.get(ux, uy).kind == Cell::Air {
                            self.grid2d
                                .set(ux, uy, CellData::sand(self.rng.random()));
                        }
                    }
                }
            }
        }
    }

    fn erase_2d(&mut self, cx: i32, cy: i32) {
        let r = self.brush_size as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let x = cx + dx;
                    let y = cy + dy;
                    if self.grid2d.in_bounds(x, y) {
                        let ux = x as usize;
                        let uy = y as usize;
                        if self.grid2d.get(ux, uy).kind == Cell::Sand {
                            self.grid2d.set(ux, uy, CellData::AIR);
                        }
                    }
                }
            }
        }
    }

    fn place_sand_3d_raycast(&mut self, origin: Vec3, dir: Vec3) {
        // March the ray and find the first empty cell adjacent to a non-empty cell
        let step = 0.5;
        for i in 0..200 {
            let p = origin + dir * (i as f32 * step);
            let ix = p.x.round() as i32;
            let iy = p.y.round() as i32;
            let iz = p.z.round() as i32;
            if !self.grid3d.in_bounds(ix, iy, iz) {
                continue;
            }
            let (ux, uy, uz) = (ix as usize, iy as usize, iz as usize);
            if self.grid3d.get(ux, uy, uz).kind == Cell::Air {
                // Only place if we're inside the grid volume and there's some surface nearby
                if uy == 0 || self.has_neighbor_3d(ux, uy, uz) {
                    let r = self.brush_size as i32;
                    for dz in -r..=r {
                        for dy in -r..=r {
                            for dx in -r..=r {
                                if dx * dx + dy * dy + dz * dz <= r * r {
                                    let nx = ix + dx;
                                    let ny = iy + dy;
                                    let nz = iz + dz;
                                    if self.grid3d.in_bounds(nx, ny, nz) {
                                        let (unx, uny, unz) =
                                            (nx as usize, ny as usize, nz as usize);
                                        if self.grid3d.get(unx, uny, unz).kind == Cell::Air {
                                            self.grid3d.set(
                                                unx,
                                                uny,
                                                unz,
                                                CellData::sand(self.rng.random()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return;
                }
            }
        }
    }

    fn has_neighbor_3d(&self, x: usize, y: usize, z: usize) -> bool {
        let dirs: [(i32, i32, i32); 6] = [
            (1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1),
        ];
        for (dx, dy, dz) in dirs {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let nz = z as i32 + dz;
            if self.grid3d.in_bounds(nx, ny, nz) {
                let c = self.grid3d.get(nx as usize, ny as usize, nz as usize);
                if c.kind != Cell::Air {
                    return true;
                }
            }
        }
        false
    }

    fn step_sim(&mut self) {
        match self.mode {
            Mode::D2 => {
                self.block2d.clear_raster(&mut self.grid2d);
                if self.rain_2d {
                    scenarios::rain_tick_2d(&mut self.grid2d, &mut self.rng);
                }
                crate::sim::physics2d::step(&mut self.grid2d, &mut self.rng);
                self.block2d.rasterize(&mut self.grid2d);
                self.timeline2d
                    .record(&self.grid2d.cells, &self.block2d);
            }
            Mode::D3 => {
                self.block3d.clear_raster(&mut self.grid3d);
                if self.rain_3d {
                    scenarios::rain_tick_3d(&mut self.grid3d, &mut self.rng);
                }
                crate::sim::physics3d::step(&mut self.grid3d, &mut self.rng);
                self.block3d.rasterize(&mut self.grid3d);
                self.timeline3d
                    .record(&self.grid3d.cells, &self.block3d);
            }
        }
        self.tick += 1;
    }

    fn restore_timeline_2d(&mut self) {
        if let Some(snap) = self.timeline2d.scrub(self.timeline2d.cursor) {
            self.grid2d.cells.copy_from_slice(&snap.cells);
            self.block2d = snap.block.clone();
        }
    }

    fn restore_timeline_3d(&mut self) {
        if let Some(snap) = self.timeline3d.scrub(self.timeline3d.cursor) {
            self.grid3d.cells.copy_from_slice(&snap.cells);
            self.block3d = snap.block.clone();
        }
    }

    fn reset_scenario(&mut self) {
        match self.mode {
            Mode::D2 => {
                scenarios::apply_2d(self.scenario2d, &mut self.grid2d, &mut self.block2d, &mut self.rng);
                self.rain_2d = self.scenario2d == 3;
                self.timeline2d.clear();
            }
            Mode::D3 => {
                scenarios::apply_3d(self.scenario3d, &mut self.grid3d, &mut self.block3d, &mut self.rng);
                self.rain_3d = self.scenario3d == 3;
                self.timeline3d.clear();
            }
        }
        self.tick = 0;
    }

    fn next_scenario(&mut self) {
        match self.mode {
            Mode::D2 => {
                self.scenario2d = (self.scenario2d + 1) % scenarios::SCENARIO_NAMES_2D.len();
                self.reset_scenario();
            }
            Mode::D3 => {
                self.scenario3d = (self.scenario3d + 1) % scenarios::SCENARIO_NAMES_3D.len();
                self.reset_scenario();
            }
        }
    }

    fn render_frame(&mut self) {
        let surface_tex = match self.gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                let (w, h) = self.gpu.size();
                self.gpu.resize(w, h);
                return;
            }
            _ => {
                return;
            }
        };
        let surface_view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // --- egui input & UI ---
        let raw_input = self.egui_state.take_egui_input(&self.window);

        let sand_count = match self.mode {
            Mode::D2 => self.grid2d.sand_count(),
            Mode::D3 => self.grid3d.sand_count(),
        };
        let timeline_len = match self.mode {
            Mode::D2 => self.timeline2d.len(),
            Mode::D3 => self.timeline3d.len(),
        };
        let mut timeline_cursor_val = match self.mode {
            Mode::D2 => self.timeline2d.cursor,
            Mode::D3 => self.timeline3d.cursor,
        };
        let mut scenario_idx = match self.mode {
            Mode::D2 => self.scenario2d,
            Mode::D3 => self.scenario3d,
        };

        let mode = self.mode;
        let paused = self.paused;
        let fps = self.fps;
        let tick = self.tick;
        let brush_size = &mut self.brush_size;
        let sim_speed = &mut self.sim_speed;
        let ui_state = &mut self.ui_state;

        #[allow(deprecated)]
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            ui::draw(
                ctx,
                mode,
                paused,
                fps,
                sand_count,
                tick,
                &mut scenario_idx,
                brush_size,
                sim_speed,
                &mut timeline_cursor_val,
                timeline_len,
                ui_state,
            );
        });

        // Write back timeline cursor
        match self.mode {
            Mode::D2 => self.timeline2d.cursor = timeline_cursor_val,
            Mode::D3 => self.timeline3d.cursor = timeline_cursor_val,
        }

        let old_scenario = match self.mode {
            Mode::D2 => self.scenario2d,
            Mode::D3 => self.scenario3d,
        };
        match self.mode {
            Mode::D2 => self.scenario2d = scenario_idx,
            Mode::D3 => self.scenario3d = scenario_idx,
        }
        if scenario_idx != old_scenario {
            self.reset_scenario();
        }

        if self.paused {
            match self.mode {
                Mode::D2 => self.restore_timeline_2d(),
                Mode::D3 => self.restore_timeline_3d(),
            }
        }

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let paint_jobs = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let (sw, sh) = self.gpu.size();
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [sw, sh],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.gpu.device, &self.gpu.queue, *id, delta);
        }

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        self.egui_renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &paint_jobs,
            &screen_desc,
        );

        // Prepare data before render passes
        let mut instance_count = 0u32;
        match self.mode {
            Mode::D2 => {
                self.renderer2d.upload(&self.gpu.queue, &self.grid2d);
            }
            Mode::D3 => {
                let vp = self.camera.vp_matrix();
                let instances = self.renderer3d.build_instances(&self.grid3d);
                instance_count = instances.len() as u32;
                self.renderer3d.prepare(&self.gpu.queue, vp, &instances);
            }
        }

        match self.mode {
            Mode::D2 => {
                draw_2d_pass(
                    &mut encoder,
                    &self.renderer2d,
                    &self.egui_renderer,
                    &surface_view,
                    &paint_jobs,
                    &screen_desc,
                );
            }
            Mode::D3 => {
                draw_3d_pass(
                    &mut encoder,
                    &self.renderer3d,
                    &surface_view,
                    &self.gpu.depth_view,
                    instance_count,
                );
                draw_egui_pass(
                    &mut encoder,
                    &self.egui_renderer,
                    &surface_view,
                    &paint_jobs,
                    &screen_desc,
                );
            }
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_tex.present();

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
    }
}

fn draw_2d_pass(
    encoder: &mut wgpu::CommandEncoder,
    renderer2d: &Renderer2D,
    egui_renderer: &egui_wgpu::Renderer,
    surface_view: &wgpu::TextureView,
    paint_jobs: &[egui::ClippedPrimitive],
    screen_desc: &egui_wgpu::ScreenDescriptor,
) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("2d_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: surface_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.12, g: 0.12, b: 0.14, a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        ..Default::default()
    });
    renderer2d.render(&mut rpass);
    egui_renderer.render(&mut rpass.forget_lifetime(), paint_jobs, screen_desc);
}

fn draw_3d_pass(
    encoder: &mut wgpu::CommandEncoder,
    renderer3d: &Renderer3D,
    surface_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    instance_count: u32,
) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("3d_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: surface_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.08, g: 0.08, b: 0.12, a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        ..Default::default()
    });
    renderer3d.render(&mut rpass, instance_count);
}

fn draw_egui_pass(
    encoder: &mut wgpu::CommandEncoder,
    egui_renderer: &egui_wgpu::Renderer,
    surface_view: &wgpu::TextureView,
    paint_jobs: &[egui::ClippedPrimitive],
    screen_desc: &egui_wgpu::ScreenDescriptor,
) {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: surface_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        ..Default::default()
    });
    egui_renderer.render(&mut rpass.forget_lifetime(), paint_jobs, screen_desc);
}
