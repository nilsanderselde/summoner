#![allow(clippy::all)]

//! Test suite for Turn #119 (Sprint Turn #45) — Visual Spectrogram Art & Bitmap Sonification Engine HUD
//! and 10-20 BCI EEG Brainwave Decoder & Neuro-Feedback Modulation HUD Convergence (Milestones 33 & 34).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::spectrogram_art_view::{
        SpectroColorMode, SpectroFrequencyMapping, SpectrogramArtPreset, SpectrogramArtView,
        SPECTROGRAM_ART_PUCK_HIT_RADIUS,
    };
    use crate::views::bci_eeg_decoder_view::{
        BciEegDecoderView, BciEegPreset, BCI_EEG_PUCK_HIT_RADIUS,
        NUM_10_20_ELECTRODES, SCALP_ELECTRODES_10_20,
    };
    use crate::layout_math::Rect;

    #[test]
    fn test_turn119_pure_spectrogram_art_initialization_and_physics() {
        let mut view = SpectrogramArtView::new();
        assert_eq!(view.preset, SpectrogramArtPreset::AphexTwinFace);
        assert_eq!(SPECTROGRAM_ART_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target

        // Presets and expected parameters
        let presets = [
            (SpectrogramArtPreset::AphexTwinFace, "Aphex Twin Silhouette (50Hz - 15kHz)", 50.0, 15000.0, 128, SpectroFrequencyMapping::Logarithmic, SpectroColorMode::GrayscaleLuma, 4.0),
            (SpectrogramArtPreset::GeometricFractal, "Geometric Mandelbrot Fractal (80Hz - 12kHz)", 80.0, 12000.0, 96, SpectroFrequencyMapping::Logarithmic, SpectroColorMode::RGBHarmonicNote, 6.0),
            (SpectrogramArtPreset::CyberpunkMatrix, "Cyberpunk Matrix Waterfall (100Hz - 18kHz)", 100.0, 18000.0, 192, SpectroFrequencyMapping::Linear, SpectroColorMode::RGBHarmonicNote, 3.0),
            (SpectrogramArtPreset::CosmicWaterfall, "Cosmic Stellar Nebulae (30Hz - 20kHz)", 30.0, 20000.0, 64, SpectroFrequencyMapping::Logarithmic, SpectroColorMode::GrayscaleLuma, 8.0),
        ];

        for (p, exp_name, exp_min, exp_max, exp_density, exp_map, exp_color, exp_dur) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (f_min, f_max, density, map, color, dur) = p.nominal_parameters();
            assert_eq!(f_min, exp_min);
            assert_eq!(f_max, exp_max);
            assert_eq!(density, exp_density);
            assert_eq!(map, exp_map);
            assert_eq!(color, exp_color);
            assert_eq!(dur, exp_dur);

            assert_eq!(view.min_freq_hz, exp_min);
            assert_eq!(view.max_freq_hz, exp_max);
            assert_eq!(view.spectral_density, exp_density);
            assert_eq!(view.mapping, exp_map);
            assert_eq!(view.color_mode, exp_color);
            assert_eq!(view.loop_duration_sec, exp_dur);
            assert!(view.trigger_count > 0);
        }

        // Normalization round-trips for Playhead [0.0 ..= 1.0]
        let playhead_test = 0.635;
        let playhead_norm = SpectrogramArtView::playhead_to_normalized(playhead_test);
        let playhead_back = SpectrogramArtView::normalized_to_playhead(playhead_norm);
        assert!((playhead_back - playhead_test).abs() < 1e-4);

        // Normalization round-trips for Oscillator Density [16 ..= 256]
        let density_test = 144;
        let density_norm = SpectrogramArtView::density_to_normalized(density_test);
        let density_back = SpectrogramArtView::normalized_to_density(density_norm);
        assert_eq!(density_back, density_test);

        // Frequency mapping evaluation (Logarithmic & Linear)
        view.mapping = SpectroFrequencyMapping::Linear;
        view.min_freq_hz = 100.0;
        view.max_freq_hz = 10100.0;
        let mid_linear = view.map_normalized_to_freq(0.5);
        assert!((mid_linear - 5100.0).abs() < 1.0);

        view.mapping = SpectroFrequencyMapping::Logarithmic;
        view.min_freq_hz = 100.0;
        view.max_freq_hz = 10000.0;
        let mid_log = view.map_normalized_to_freq(0.5);
        assert!((mid_log - 1000.0).abs() < 10.0); // Geometric mean: sqrt(100 * 10000) = 1000

        // Advance playhead
        view.playhead_pos = 0.20;
        view.loop_duration_sec = 4.0;
        view.is_scanning = true;
        view.advance_playhead(1.0);
        assert!((view.playhead_pos - 0.45).abs() < 1e-3);

        // Synthetic pixel evaluations for all presets stay in [0.0 ..= 1.0]
        for p in [SpectrogramArtPreset::AphexTwinFace, SpectrogramArtPreset::GeometricFractal, SpectrogramArtPreset::CyberpunkMatrix, SpectrogramArtPreset::CosmicWaterfall] {
            view.preset = p;
            for u_step in 0..5 {
                for v_step in 0..5 {
                    let u = u_step as f32 / 4.0;
                    let v = v_step as f32 / 4.0;
                    let val = view.evaluate_synthetic_pixel(u, v);
                    assert!(val >= 0.0 && val <= 1.0);
                }
            }
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 320.0, 180.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_spectrogram_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_spectrogram_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_spectrogram_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_spectrogram_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("VISUAL SPECTROGRAM ART & BITMAP SONIFICATION HUD"));
        assert!(snap.contains("PASS") && snap.contains("44pt"));
    }

    #[test]
    fn test_turn119_pure_bci_eeg_decoder_initialization_and_physics() {
        let mut view = BciEegDecoderView::new();
        assert_eq!(view.preset, BciEegPreset::DeepAlphaFlow);
        assert_eq!(BCI_EEG_PUCK_HIT_RADIUS, 22.0); // >= 44x44pt touch bounding target
        assert_eq!(NUM_10_20_ELECTRODES, 10);
        assert_eq!(SCALP_ELECTRODES_10_20.len(), 10);

        // Presets and expected parameters
        let presets = [
            (BciEegPreset::DeepAlphaFlow, "Deep Alpha Flow (Creative Flow State)", 0.15, 0.60, 0.85, 0.20, 0.10, 0.70, 0.30),
            (BciEegPreset::HighGammaInsight, "High Gamma Insight (Cognitive Burst)", 0.05, 0.15, 0.30, 0.75, 0.90, 0.85, 0.80),
            (BciEegPreset::ThetaDreamTrance, "Theta Dream Trance (Ambient Hypnagogia)", 0.50, 0.88, 0.45, 0.10, 0.05, 0.50, 0.15),
            (BciEegPreset::ActiveBetaFocus, "Active Beta Focus (Surgical Precision)", 0.10, 0.25, 0.40, 0.80, 0.35, 0.20, 0.65),
        ];

        for (p, exp_name, exp_d, exp_th, exp_a, exp_b, exp_g, exp_val, exp_ar) in presets {
            view.set_preset(p);
            assert_eq!(p.name(), exp_name);
            let (d, th, a, b, g, val, ar) = p.nominal_parameters();
            assert_eq!(d, exp_d);
            assert_eq!(th, exp_th);
            assert_eq!(a, exp_a);
            assert_eq!(b, exp_b);
            assert_eq!(g, exp_g);
            assert_eq!(val, exp_val);
            assert_eq!(ar, exp_ar);

            assert_eq!(view.delta_power, exp_d);
            assert_eq!(view.theta_power, exp_th);
            assert_eq!(view.alpha_power, exp_a);
            assert_eq!(view.beta_power, exp_b);
            assert_eq!(view.gamma_power, exp_g);
            assert_eq!(view.emotional_valence, exp_val);
            assert_eq!(view.arousal_intensity, exp_ar);
            assert!(view.focus_index >= 0.0);
            assert!(view.cv_modulation_out >= 0.0 && view.cv_modulation_out <= 1.0);
        }

        // Normalization round-trips for Alpha power [0.0 ..= 1.0]
        let alpha_test = 0.785;
        let alpha_norm = BciEegDecoderView::alpha_to_normalized(alpha_test);
        let alpha_back = BciEegDecoderView::normalized_to_alpha(alpha_norm);
        assert!((alpha_back - alpha_test).abs() < 1e-4);

        // Normalization round-trips for Beta power [0.0 ..= 1.0]
        let beta_test = 0.342;
        let beta_norm = BciEegDecoderView::beta_to_normalized(beta_test);
        let beta_back = BciEegDecoderView::normalized_to_beta(beta_norm);
        assert!((beta_back - beta_test).abs() < 1e-4);

        // Focus index calculation: Beta / (Alpha + Theta)
        view.alpha_power = 0.50;
        view.theta_power = 0.50;
        view.beta_power = 0.40;
        view.recompute_metrics();
        assert!((view.focus_index - 0.40).abs() < 0.01);

        // Alpha stimulation
        let a_before = view.alpha_power;
        view.stimulate_alpha();
        assert!(view.alpha_power > a_before);

        // Scalp Electrodes Table verification
        for elec in SCALP_ELECTRODES_10_20 {
            assert!(!elec.name.is_empty());
            assert!(elec.norm_x >= 0.0 && elec.norm_x <= 1.0);
            assert!(elec.norm_y >= 0.0 && elec.norm_y <= 1.0);
        }

        // Touch puck hit testing (>= 22pt radius -> 44x44pt bounding box)
        let canvas = Rect::new(20.0, 20.0, 300.0, 180.0);
        let puck_x = canvas.x + 15.0 + view.puck_pos.0 * (canvas.width - 30.0);
        let puck_y = canvas.y + canvas.height - 15.0 - view.puck_pos.1 * (canvas.height - 30.0);

        assert!(view.hit_test_bci_puck((puck_x, puck_y), canvas));
        assert!(view.hit_test_bci_puck((puck_x + 14.0, puck_y), canvas));
        assert!(view.hit_test_bci_puck((puck_x, puck_y + 14.0), canvas));
        assert!(!view.hit_test_bci_puck((puck_x + 35.0, puck_y), canvas));

        // Deterministic ASCII snapshot string
        let snap = view.render_ascii_snapshot_str();
        assert!(snap.contains("10-20 BCI EEG BRAINWAVE DECODER & NEURO-FEEDBACK HUD"));
        assert!(snap.contains("PASS") && snap.contains("44pt"));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod gui_tests {
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::command_palette::CommandPalette;
    use eframe::egui;

    #[test]
    fn test_turn119_gui_spectrogram_and_bci_modal_lifecycle() {
        let mut view = AwardWinningGuiView::new();

        // 1. Spectrogram Art Modal Lifecycle
        assert!(!view.is_spectrogram_art_hud_open());
        view.open_spectrogram_art_hud();
        assert!(view.is_spectrogram_art_hud_open());
        view.close_spectrogram_art_hud();
        assert!(!view.is_spectrogram_art_hud_open());

        // 2. BCI EEG Decoder Modal Lifecycle
        assert!(!view.is_bci_eeg_decoder_hud_open());
        view.open_bci_eeg_decoder_hud();
        assert!(view.is_bci_eeg_decoder_hud_open());
        view.close_bci_eeg_decoder_hud();
        assert!(!view.is_bci_eeg_decoder_hud_open());

        // 3. Headless egui Context Rendering without panic
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.open_spectrogram_art_hud();
                view.show_spectrogram_art_modal_window(ui);

                view.open_bci_eeg_decoder_hud();
                view.show_bci_eeg_decoder_modal_window(ui);
            });
        });

        // 4. Lock-free Atomic ParamBus Synchronization without allocation
        let bus = std::sync::Arc::new(summoner_core::param_bus::ParamBus::new());
        view.live_param_bus = Some(bus.clone());

        view.spectrogram_art_view.playhead_pos = 0.55;
        view.spectrogram_art_view.spectral_density = 160;
        view.spectrogram_art_view.min_freq_hz = 60.0;
        view.spectrogram_art_view.max_freq_hz = 16000.0;
        view.sync_spectrogram_art_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("playhead_pos").copied(), Some(0.55));
        assert_eq!(view.device_rack_state.node_param_values.get("spectral_density").copied(), Some(160.0));
        assert_eq!(view.inspector_state.node_param_values.get("playhead_pos").copied(), Some(0.55));

        view.bci_eeg_decoder_view.alpha_power = 0.88;
        view.bci_eeg_decoder_view.beta_power = 0.42;
        view.bci_eeg_decoder_view.theta_power = 0.35;
        view.bci_eeg_decoder_view.cv_modulation_out = 0.75;
        view.sync_bci_eeg_decoder_hud_state();

        assert_eq!(view.device_rack_state.node_param_values.get("alpha_power").copied(), Some(0.88));
        assert_eq!(view.device_rack_state.node_param_values.get("beta_power").copied(), Some(0.42));
        assert_eq!(view.inspector_state.node_param_values.get("alpha_power").copied(), Some(0.88));

        // 5. Command Palette Action Verification
        let palette = CommandPalette::new();
        let actions = &palette.actions;
        assert!(actions.iter().any(|a| a.action_id == "open_spectrogram_art_hud"));
        assert!(actions.iter().any(|a| a.action_id == "open_bci_eeg_decoder_hud"));

        // 6. Trigger Requests from Inspector and Device Rack
        view.inspector_state.requested_open_spectrogram_art_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_spectrogram_art_hud_open());
        view.close_spectrogram_art_hud();

        view.device_rack_state.requested_open_bci_eeg_decoder_hud = true;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
        assert!(view.is_bci_eeg_decoder_hud_open());
    }
}
