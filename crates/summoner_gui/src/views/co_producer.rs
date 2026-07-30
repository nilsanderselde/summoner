// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Co-Producer Chat panel interface simulating AI assistant guidance.

use eframe::egui;
use summoner_project::schema::ProjectConfig;

/// Persistent state for AI Co-Producer chat panel.
#[derive(Clone, Debug)]
pub struct CoProducerState {
    pub api_key: String,
    pub messages: Vec<(String, String)>, // (role, text)
    pub prompt_input: String,
}

impl Default for CoProducerState {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            messages: vec![
                ("system".to_string(), "Welcome to Summoner AI Co-Producer! Ask for arrangement tips, chord suggestions, or mixing feedback.".to_string()),
            ],
            prompt_input: String::new(),
        }
    }
}

pub fn show_co_producer_panel(ui: &mut egui::Ui, project: &ProjectConfig, state: &mut CoProducerState) {
    ui.heading("🤖 AI Co-Producer Panel");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Cloud API Key:");
        ui.add(egui::TextEdit::singleline(&mut state.api_key).password(true).hint_text("sk-proj-..."));
        if state.api_key.is_empty() {
            ui.label("(Local Simulation Mode Active)");
        } else {
            ui.label("🟢 Connected");
        }
    });

    ui.separator();

    // Chat history view
    egui::ScrollArea::vertical()
        .max_height(350.0)
        .show(ui, |ui| {
            for (role, msg) in &state.messages {
                ui.horizontal(|ui| {
                    if role == "user" {
                        ui.colored_label(egui::Color32::from_rgb(26, 140, 255), "👤 You:");
                    } else if role == "system" {
                        ui.colored_label(egui::Color32::from_rgb(241, 196, 15), "⚙️ System:");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(46, 204, 113), "🤖 Co-Producer:");
                    }
                    ui.label(msg);
                });
                ui.add_space(4.0);
            }
        });

    ui.separator();

    // Prompt input bar
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.prompt_input)
                .hint_text("Ask co-producer (e.g., 'Suggest a bridge chord progression for 120 BPM')...")
                .desired_width(ui.available_width() - 80.0)
        );
        let send_clicked = ui.button("Send").clicked();

        if (send_clicked || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !state.prompt_input.trim().is_empty() {
            let user_text = state.prompt_input.trim().to_string();
            state.messages.push(("user".to_string(), user_text.clone()));
            state.prompt_input.clear();

            // Generate AI co-producer response based on project state
            let ai_response = format!(
                "For project '{}' (BPM: {:.1}, Tracks: {}): Consider adding a 7th note accent or trying a Funk groove quantize pass to tighten the rhythm!",
                project.name, project.transport.bpm, project.tracks.len()
            );
            state.messages.push(("assistant".to_string(), ai_response));
        }
    });
}
