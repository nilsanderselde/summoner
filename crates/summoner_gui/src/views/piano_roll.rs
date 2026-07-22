use eframe::egui;
use summoner_project::schema::SequenceConfig;
use summoner_harmony::edo::EdoTuning;

#[derive(PartialEq)]
pub enum PianoRollMode {
    StepGrid,
    PianoRoll,
}

pub struct PianoRollState {
    pub mode: PianoRollMode,
    pub scroll_offset: egui::Vec2,
}

impl Default for PianoRollState {
    fn default() -> Self {
        Self {
            mode: PianoRollMode::StepGrid,
            scroll_offset: egui::Vec2::ZERO,
        }
    }
}

// Dummy Viewport for now, since it wasn't specified in the roadmap or previous phases
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

pub fn show_piano_roll(
    ui: &mut egui::Ui,
    sequence: &mut SequenceConfig,
    tuning: &EdoTuning,
    state: &mut PianoRollState,
    _viewport: &Viewport,
) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.mode, PianoRollMode::StepGrid, "Step Grid");
        ui.selectable_value(&mut state.mode, PianoRollMode::PianoRoll, "Piano Roll");
    });

    ui.separator();

    match state.mode {
        PianoRollMode::StepGrid => {
            let step_width = 40.0;
            let step_height = 120.0;
            
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut i = 0;
                    while i < sequence.steps.len() {
                        let step = &mut sequence.steps[i];
                        
                        ui.allocate_ui(egui::vec2(step_width, step_height), |ui| {
                            ui.vertical(|ui| {
                                // Step on/off
                                let mut active = step.gate > 0.0;
                                if ui.toggle_value(&mut active, format!("{}", i + 1)).changed() {
                                    step.gate = if active { 0.5 } else { 0.0 };
                                }
                                
                                // Velocity slider
                                ui.add(egui::Slider::new(&mut step.velocity, 0.0..=1.0).orientation(egui::SliderOrientation::Vertical));
                                
                                // Probability label
                                let prob = step.probability;
                                ui.label(format!("{:.0}%", prob * 100.0));
                            });
                        }).response.context_menu(|ui| {
                            ui.heading("Step Properties");
                            ui.add(egui::Slider::new(&mut step.ratchet, 1..=16).text("Ratchet"));
                            ui.add(egui::Slider::new(&mut step.micro_shift, -64..=64).text("Micro Shift"));
                        });
                        i += 1;
                    }
                });
            });
        }
        PianoRollMode::PianoRoll => {
            egui::ScrollArea::both().show(ui, |ui| {
                let keys_per_octave = tuning.divisions as usize;
                let num_octaves = 8;
                let key_height = 12.0;
                let beat_width = 100.0;
                let num_beats = sequence.steps.len() as f32 / 4.0; // Assuming 1/16th notes
                let num_beats = sequence.steps.len() as f32 / sequence.step_division as f32;
                
                let canvas_size = egui::vec2(num_beats * beat_width, keys_per_octave as f32 * num_octaves as f32 * key_height);
                let (mut response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
                
                // Draw keyboard background
                for i in 0..(keys_per_octave * num_octaves) {
                    let y = response.rect.bottom() - (i as f32 + 1.0) * key_height;
                    let is_black = i % keys_per_octave != 0; // Extremely naive for arbitrary EDO
                    let color = if is_black { egui::Color32::from_gray(20) } else { egui::Color32::from_gray(50) };
                    
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(response.rect.left(), y), egui::vec2(response.rect.width(), key_height)),
                        0.0,
                        color
                    );
                    painter.line_segment(
                        [egui::pos2(response.rect.left(), y), egui::pos2(response.rect.right(), y)],
                        egui::Stroke::new(1.0, egui::Color32::from_gray(10))
                    );
                }
                
                // Note entry interaction
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let y_offset = response.rect.bottom() - pos.y;
                        let pitch = (y_offset / key_height).floor() as u16;
                        // For a simple mock we don't insert arbitrary notes, just let it be.
                        // In reality we'd add to the sequence or piano roll data structure.
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piano_roll_renders_without_panic() {
        let mut sequence = SequenceConfig {
            step_division: 16.0,
            steps: vec![summoner_project::schema::StepConfig {
                velocity: 0.8,
                gate: 0.5,
                pitch_offset: 0,
                probability: 1.0,
                ratchet: 1,
                micro_shift: 0.0,
            }; 16],
        };
        let tuning = EdoTuning::new(19);
        let mut state = PianoRollState::default();
        let viewport = Viewport { width: 800.0, height: 600.0 };

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_piano_roll(ui, &mut sequence, &tuning, &mut state, &viewport);
            });
        });
    }
}
