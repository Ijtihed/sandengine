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

pub struct UiOutput {
    pub mode_switch: Option<Mode>,
    pub toggle_pause: bool,
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
) -> UiOutput {
    let mut output = UiOutput {
        mode_switch: None,
        toggle_pause: false,
    };

    egui::Area::new(egui::Id::new("stats_panel"))
        .fixed_pos(egui::pos2(10.0, 10.0))
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(210))
                .corner_radius(10.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    // --- Mode switch buttons ---
                    ui.horizontal(|ui| {
                        let modes = [
                            (Mode::D2, "2D"),
                            (Mode::D3, "3D"),
                        ];
                        for (m, label) in modes {
                            let active = mode == m;
                            let color = if active {
                                egui::Color32::from_rgb(70, 140, 220)
                            } else {
                                egui::Color32::from_black_alpha(140)
                            };
                            let text = egui::RichText::new(label)
                                .size(16.0)
                                .strong()
                                .color(if active { egui::Color32::WHITE } else { egui::Color32::LIGHT_GRAY });
                            let btn = egui::Button::new(text)
                                .fill(color)
                                .min_size(egui::vec2(48.0, 28.0));
                            if ui.add(btn).clicked() && !active {
                                output.mode_switch = Some(m);
                            }
                        }

                        ui.add_space(8.0);

                        // 4D (pause/time) button
                        let time_color = if paused {
                            egui::Color32::from_rgb(100, 200, 255)
                        } else {
                            egui::Color32::from_black_alpha(140)
                        };
                        let time_text = egui::RichText::new("4D")
                            .size(16.0)
                            .strong()
                            .color(if paused { egui::Color32::WHITE } else { egui::Color32::LIGHT_GRAY });
                        let time_btn = egui::Button::new(time_text)
                            .fill(time_color)
                            .min_size(egui::vec2(48.0, 28.0));
                        if ui.add(time_btn).clicked() {
                            output.toggle_pause = true;
                        }
                    });

                    ui.add_space(4.0);

                    let mode_str = match mode { Mode::D2 => "2D", Mode::D3 => "3D" };
                    ui.label(
                        egui::RichText::new(format!("{} | FPS {:.0} | {} particles | tick {}", mode_str, fps, particle_count, tick))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(170, 170, 180)),
                    );

                    ui.separator();

                    // Material palette
                    ui.label(egui::RichText::new("Material (1-8)").size(12.0).strong());
                    ui.horizontal_wrapped(|ui| {
                        let mats: [(Cell, &str, egui::Color32); 8] = [
                            (Cell::Sand, "Sand", egui::Color32::from_rgb(215, 192, 132)),
                            (Cell::Water, "Water", egui::Color32::from_rgb(25, 75, 195)),
                            (Cell::Stone, "Stone", egui::Color32::from_rgb(118, 120, 126)),
                            (Cell::Fire, "Fire", egui::Color32::from_rgb(255, 160, 30)),
                            (Cell::Gravel, "Gravel", egui::Color32::from_rgb(118, 108, 98)),
                            (Cell::Oil, "Oil", egui::Color32::from_rgb(58, 38, 18)),
                            (Cell::Acid, "Acid", egui::Color32::from_rgb(30, 200, 30)),
                            (Cell::Steam, "Steam", egui::Color32::from_rgb(210, 210, 220)),
                        ];
                        for (mat, name, color) in mats {
                            let active = *selected_material == mat;
                            let text = egui::RichText::new(name)
                                .size(11.0)
                                .color(if active { egui::Color32::WHITE } else { egui::Color32::from_rgb(160, 160, 160) });
                            let fill = if active { color } else { egui::Color32::from_black_alpha(100) };
                            let btn = egui::Button::new(text).fill(fill);
                            if ui.add(btn).clicked() {
                                *selected_material = mat;
                            }
                        }
                    });

                    ui.separator();

                    ui.add(egui::Slider::new(brush_size, 1..=40).text("Brush"));
                    ui.add(egui::Slider::new(sim_speed, 0.1..=10.0).text("Speed").logarithmic(true));

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

                    ui.horizontal(|ui| {
                        if ui.button("Reset (R)").clicked() {}
                        if ui.button("Help (H)").clicked() {
                            ui_state.show_help = !ui_state.show_help;
                        }
                    });
                });
        });

    // Time scrubber when paused
    if paused && timeline_len > 0 {
        egui::Area::new(egui::Id::new("time_panel"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_black_alpha(220))
                    .corner_radius(10.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("4D TIME TRAVEL")
                                    .color(egui::Color32::from_rgb(100, 200, 255))
                                    .size(14.0)
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
                    egui::RichText::new("4D MODE")
                        .size(52.0)
                        .color(egui::Color32::from_rgba_premultiplied(100, 200, 255, 50))
                        .strong(),
                );
            });
    }

    if ui_state.show_help {
        egui::Area::new(egui::Id::new("help_panel"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_black_alpha(235))
                    .corner_radius(10.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Controls").color(egui::Color32::WHITE).strong().size(16.0));
                        ui.separator();
                        for line in [
                            "F2/F3 -- Switch 2D/3D",
                            "1-8 -- Select material",
                            "Space -- Toggle 4D time mode",
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
                            ui.label(egui::RichText::new(line).color(egui::Color32::LIGHT_GRAY).size(11.0));
                        }
                    });
            });
    }

    output
}
