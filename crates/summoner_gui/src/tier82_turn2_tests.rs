// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #2 Integration & Regression Tests:
//! 1. Pro Parameter Drawer inside Device Rack.
//! 2. Robust bidirectional synchronization between Device Rack and Inspector without data clobbering.
//! 3. General MIDI & Chiptune factory soundbank preset integration (WISHLIST.md alignment).
//! 4. Context-aware Inspector parameter reflection with ParamBus routing.

#[cfg(test)]
mod pure_tests {
    use crate::dsp_node_ui::DspNodeRegistry;

    #[test]
    fn test_turn2_node_registry_target_nodes() {
        let registry = DspNodeRegistry::new();
        let target_nodes = [
            "GrandPianoModel",
            "ElectricPianoModel",
            "PipeOrganModel",
            "PluckSynth",
            "DistortionNode",
            "BowedStringModel",
            "WaveguideBrassModel",
            "ShakuhachiModel",
            "DrumMachineDevice",
            "OscPulse",
            "OscTriangle",
            "TrackerStepConfig",
            "NoiseGen",
        ];
        for node in target_nodes {
            assert!(registry.get(node).is_some(), "Node {} must exist in DspNodeRegistry", node);
        }
    }
}

#[cfg(all(test, feature = "gui"))]
mod tests {
    use eframe::egui;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::modern_device_rack::{show_modern_device_rack, ModernDeviceRackState};
    use crate::views::modern_inspector::{show_modern_inspector_with_context, ModernInspectorState};
    use crate::views::modern_top_bar::ModernTopBarState;

    #[test]
    fn test_turn2_device_rack_pro_params_drawer_mode() {
        let mut state = ModernDeviceRackState::default();
        let ctx = egui::Context::default();

        // Initially in standard chassis mode
        assert!(!state.is_expanded_params);

        // Toggle into Pro Params Drawer mode
        state.is_expanded_params = true;
        state.selected_node_kind = Some("AetherSynth".to_string());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_device_rack(ui, &mut state, None);
            });
        });

        assert!(state.is_expanded_params);
        assert!(!state.node_param_values.is_empty());
        assert!(state.node_param_values.contains_key("cutoff"));
        assert!(state.node_param_values.contains_key("resonance"));
    }

    #[test]
    fn test_turn2_inspector_with_context_and_param_bus() {
        let mut state = ModernInspectorState::default();
        let ctx = egui::Context::default();
        let mut bus = ParamBus::new();

        // Pre-register track 2 parameter
        bus.register(ParamId(2000), 0.0);

        state.selected_node_kind = Some("AetherSynth".to_string());

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_modern_inspector_with_context(ui, &mut state, Some(&bus), 2);
            });
        });

        assert!(!state.node_param_values.is_empty());
        assert_eq!(state.target_name, "Synth 1");
    }

    #[test]
    fn test_turn2_bidirectional_sync_inspector_to_rack_without_clobbering() {
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        // Initialize view by running one frame
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // User edits cutoff in the Inspector to 0.92
        view.inspector_state.node_param_values.insert("cutoff".to_string(), 0.92);

        // Run next frame: sync should propagate 0.92 to device_rack_state without overwriting
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.device_rack_state.node_param_values.get("cutoff"), Some(&0.92));
        assert_eq!(view.inspector_state.node_param_values.get("cutoff"), Some(&0.92));

        // Now user edits resonance in the Device Rack to 0.84
        view.device_rack_state.node_param_values.insert("resonance".to_string(), 0.84);

        // Run next frame: sync should propagate 0.84 to inspector_state without overwriting
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.device_rack_state.node_param_values.get("resonance"), Some(&0.84));
        assert_eq!(view.inspector_state.node_param_values.get("resonance"), Some(&0.84));
    }

    #[test]
    fn test_turn2_bidirectional_module_selection_sync() {
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 1. User changes module in Inspector dropdown
        view.inspector_state.selected_node_kind = Some("TubeSaturationNode".to_string());
        view.inspector_state.target_name = "Analog Vacuum Tube Saturation Stage".to_string();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.device_rack_state.selected_node_kind.as_deref(), Some("TubeSaturationNode"));
        assert_eq!(view.device_rack_state.device_name, "Analog Vacuum Tube Saturation Stage");

        // 2. User changes module in Device Rack dropdown
        view.device_rack_state.selected_node_kind = Some("PluckSynth".to_string());
        view.device_rack_state.device_name = "Karplus-Strong Plucked String Synthesis Core".to_string();

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert_eq!(view.inspector_state.selected_node_kind.as_deref(), Some("PluckSynth"));
        assert_eq!(view.inspector_state.target_name, "Karplus-Strong Plucked String Synthesis Core");
    }

    #[test]
    fn test_turn2_general_midi_and_chiptune_presets() {
        let top_bar = ModernTopBarState::default();

        // Verify GM and Chiptune presets exist in available_presets (WISHLIST.md alignment)
        let gm_presets = [
            "General MIDI Acoustic Grand Piano",
            "General MIDI Electric Piano 1 (Rhodes)",
            "General MIDI Church Pipe Organ",
            "General MIDI Nylon String Guitar",
            "General MIDI Overdriven Rock Guitar",
            "General MIDI Acoustic Bass",
            "General MIDI Synth Brass 1",
            "General MIDI Shakuhachi Flute",
            "General MIDI Standard Drum Kit 1",
            "Chiptune 8-Bit NES Pulse Lead",
            "Chiptune 8-Bit GameBoy Triangle Bass",
            "Chiptune FastTracker II Arp Arpeggio",
            "Chiptune Noise Snare & Hi-Hat",
        ];

        for preset in gm_presets {
            assert!(
                top_bar.available_presets.contains(&preset.to_string()),
                "Preset '{}' must be available in top bar",
                preset
            );
        }

        // Test applying each preset in AwardWinningGuiView
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        for preset in gm_presets {
            view.top_bar_state.selected_preset = preset.to_string();
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });

            assert_eq!(view.last_applied_preset, preset);
            assert!(view.device_rack_state.selected_node_kind.is_some());
            assert_eq!(
                view.inspector_state.selected_node_kind,
                view.device_rack_state.selected_node_kind
            );
        }
    }
}
