use crate::sim::material::Cell;
use crate::sim::scenarios::{SCENARIO_NAMES_2D, SCENARIO_NAMES_3D};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    D2,
    D3,
}

pub struct UiState {
    pub show_help: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self { show_help: false }
    }
}

pub fn draw(
    ctx: &egui::Context,
    mode: Mode,
    paused: bool,
    fps: f32,
    particle_count: usize,
    tick: u64,
    scenario_idx: &mut usize,
    brush_size: &mut u32,
    sim_speed: &mut f32,
    timeline_cursor: &mut usize,
    timeline_len: usize,
    ui_state: &mut UiState,
    selected_material: &mut Cell,
) {
    egui::Area::new(egui::Id::new("stats_panel"))
        .fixed_pos(egui::pos2(10.0, 10.0))
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(200))
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    let mode_str = match mode { Mode::D2 => "2D", Mode::D3 => "3D" };
                    ui.label(format!("Mode: {} | FPS: {:.0}", mode_str, fps));
                    ui.label(format!("Particles: {} | Tick: {}", particle_count, tick));

                    ui.separator();

                    // Material palette
                    ui.label(egui::RichText::new("Material (1-8)").strong());
                    ui.horizontal_wrapped(|ui| {
                        let mats = [
                            (Cell::Sand, "Sand", egui::Color32::from_rgb(210, 190, 128)),
                            (Cell::Water, "Water", egui::Color32::from_rgb(40, 90, 200)),
                            (Cell::Stone, "Stone", egui::Color32::from_rgb(112, 114, 118)),
                            (Cell::Fire, "Fire", egui::Color32::from_rgb(245, 165, 30)),
                            (Cell::Gravel, "Gravel", egui::Color32::from_rgb(130, 115, 95)),
                            (Cell::Oil, "Oil", egui::Color32::from_rgb(55, 40, 20)),
                            (Cell::Acid, "Acid", egui::Color32::from_rgb(50, 200, 40)),
                            (Cell::Steam, "Steam", egui::Color32::from_rgb(200, 200, 215)),
                        ];
                        for (mat, name, color) in mats {
                            let selected = *selected_material == mat;
                            let text = egui::RichText::new(name).color(if selected {
                                egui::Color32::WHITE
                            } else {
                                egui::Color32::GRAY
                            });
                            let btn = egui::Button::new(text).fill(if selected {
                                color
                            } else {
                                egui::Color32::from_black_alpha(120)
                            });
                            if ui.add(btn).clicked() {
                                *selected_material = mat;
                            }
                        }
                    });

                    ui.separator();

                    ui.add(egui::Slider::new(brush_size, 1..=40).text("Brush"));
                    ui.add(
                        egui::Slider::new(sim_speed, 0.1..=10.0)
                            .text("Speed")
                            .logarithmic(true),
                    );

                    let names = match mode {
                        Mode::D2 => SCENARIO_NAMES_2D,
                        Mode::D3 => SCENARIO_NAMES_3D,
                    };
                    let current = names[*scenario_idx % names.len()];
                    egui::ComboBox::from_label("Scenario")
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for (i, name) in names.iter().enumerate() {
                                if ui.selectable_label(i == *scenario_idx, *name).clicked() {
                                    *scenario_idx = i;
                                }
                            }
                        });

                    if ui.button("Reset (R)").clicked() {
                        // handled by key detection
                    }
                });
        });

    if paused && timeline_len > 0 {
        egui::Area::new(egui::Id::new("time_panel"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_black_alpha(210))
                    .corner_radius(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("4D TIME")
                                    .color(egui::Color32::from_rgb(100, 200, 255))
                                    .strong(),
                            );
                            let max = timeline_len.saturating_sub(1);
                            let mut c = *timeline_cursor as f64;
                            ui.add(
                                egui::Slider::new(&mut c, 0.0..=(max as f64))
                                    .text("Frame")
                                    .integer(),
                            );
                            *timeline_cursor = c as usize;
                        });
                    });
            });

        egui::Area::new(egui::Id::new("paused_overlay"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .interactable(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("PAUSED")
                        .size(48.0)
                        .color(egui::Color32::from_rgba_premultiplied(255, 255, 255, 60))
                        .strong(),
                );
            });
    }

    if ui_state.show_help {
        egui::Area::new(egui::Id::new("help_panel"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_black_alpha(230))
                    .corner_radius(8.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Controls").color(egui::Color32::WHITE).strong().size(16.0));
                        ui.separator();
                        for line in [
                            "F2/F3 -- Switch 2D/3D",
                            "1-8 -- Select material",
                            "Space -- Pause/Resume",
                            "Left/Right -- Scrub time",
                            "Shift+L/R -- Jump 50 frames",
                            "Tab -- Next scenario",
                            "R -- Reset scenario",
                            "H -- Toggle this help",
                            "+/- -- Sim speed",
                            "",
                            "2D: L-click place/grab, R-click erase",
                            "    Scroll = brush size",
                            "3D: L-click place/grab",
                            "    R-drag = orbit, Scroll = zoom",
                            "    Middle-drag = pan",
                        ] {
                            ui.label(egui::RichText::new(line).color(egui::Color32::LIGHT_GRAY));
                        }
                    });
            });
    }
}
