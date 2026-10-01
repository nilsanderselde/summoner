#![cfg(feature = "gui")]

// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 117 Test Suite — Turn #53 (Sprint Review Turn #19):
//! Advanced Stems Export Modal & Loudness Compliance Inspector (Milestone 37).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::stems_export_view::{
        LoudnessStandard, StemPresetQuickPick, StemsExportView,
        STEM_EXPORT_MIN_HIT_HEIGHT, STEM_EXPORT_TOUCH_TARGET_SIZE,
    };
    use summoner_project::export::{BitDepth, StemExportFormat};

    #[test]
    fn test_turn53_stems_export_view_initialization() {
        let view = StemsExportView::new();

        assert_eq!(view.preset, StemPresetQuickPick::StudioMasterWav);
        assert_eq!(view.format, StemExportFormat::Wav);
        assert_eq!(view.bit_depth, BitDepth::Bit24);
        assert_eq!(view.sample_rate, 48000);
        assert_eq!(view.loudness_standard, LoudnessStandard::SpotifyYouTube);
        assert!((view.tail_decay_seconds - 2.0).abs() < 1e-4);
        assert!(view.include_master);
        assert_eq!(view.tracks.len(), 4);
        assert_eq!(view.selected_tracks_count(), 4);
        assert_eq!(view.total_stems_count(), 5); // 4 tracks + master
    }

    #[test]
    fn test_turn53_presets_application() {
        let mut view = StemsExportView::new();

        // 1. Studio Master WAV
        view.apply_preset(StemPresetQuickPick::StudioMasterWav);
        assert_eq!(view.format, StemExportFormat::Wav);
        assert_eq!(view.bit_depth, BitDepth::Bit24);
        assert_eq!(view.sample_rate, 48000);
        assert!(!view.normalize_enabled);
        assert_eq!(view.loudness_standard, LoudnessStandard::Bypass);

        // 2. CD Quality WAV
        view.apply_preset(StemPresetQuickPick::CdQualityWav);
        assert_eq!(view.format, StemExportFormat::Wav);
        assert_eq!(view.bit_depth, BitDepth::Bit16);
        assert_eq!(view.sample_rate, 44100);
        assert!(view.dither_enabled);
        assert!(view.normalize_enabled);
        assert_eq!(view.loudness_standard, LoudnessStandard::RawPeak);

        // 3. Streaming Ready Normalized
        view.apply_preset(StemPresetQuickPick::StreamingReadyNormalized);
        assert_eq!(view.format, StemExportFormat::Wav);
        assert_eq!(view.bit_depth, BitDepth::Bit24);
        assert_eq!(view.sample_rate, 48000);
        assert!(view.normalize_enabled);
        assert_eq!(view.loudness_standard, LoudnessStandard::SpotifyYouTube);
        assert!((view.loudness_standard.target_lufs() - (-14.0)).abs() < 1e-4);

        // 4. Broadcast EBU R128
        view.apply_preset(StemPresetQuickPick::BroadcastEbuR128);
        assert_eq!(view.format, StemExportFormat::Wav);
        assert_eq!(view.bit_depth, BitDepth::Bit24);
        assert_eq!(view.sample_rate, 48000);
        assert!(view.normalize_enabled);
        assert_eq!(view.loudness_standard, LoudnessStandard::EbuR128);
        assert!((view.loudness_standard.target_lufs() - (-23.0)).abs() < 1e-4);
        assert!(view.group_by_bus);

        // 5. Hi-Res FLAC Archive
        view.apply_preset(StemPresetQuickPick::HiResFlacArchive);
        assert_eq!(view.format, StemExportFormat::Flac);
        assert_eq!(view.bit_depth, BitDepth::Bit24);
        assert_eq!(view.sample_rate, 96000);
        assert_eq!(view.flac_compression, 8);
        assert!(view.group_by_bus);

        // 6. Game Audio Clean Loops
        view.apply_preset(StemPresetQuickPick::GameAudioCleanLoops);
        assert_eq!(view.format, StemExportFormat::Wav);
        assert_eq!(view.bit_depth, BitDepth::Bit16);
        assert_eq!(view.sample_rate, 48000);
        assert!((view.tail_decay_seconds - 0.0).abs() < 1e-4);
        assert!(view.trim_silence);
        assert!(!view.include_master);
    }

    #[test]
    fn test_turn53_loudness_standards_and_targets() {
        let standards = [
            (LoudnessStandard::SpotifyYouTube, -14.0, -1.0),
            (LoudnessStandard::AppleMusic, -16.0, -1.0),
            (LoudnessStandard::EbuR128, -23.0, -1.0),
            (LoudnessStandard::AtscA85, -24.0, -2.0),
            (LoudnessStandard::AesTd1004, -18.0, -1.0),
            (LoudnessStandard::ClubDj, -9.0, -0.3),
            (LoudnessStandard::RawPeak, -12.0, -0.1),
            (LoudnessStandard::Bypass, 0.0, 0.0),
        ];

        for (std, exp_lufs, exp_tp) in standards {
            assert!((std.target_lufs() - exp_lufs).abs() < 1e-4, "LUFS mismatch for {:?}", std);
            assert!((std.max_true_peak_dbtp() - exp_tp).abs() < 1e-4, "True Peak mismatch for {:?}", std);
            assert!(!std.name().is_empty());
            assert!(!std.description().is_empty());
        }
    }

    #[test]
    fn test_turn53_true_peak_margin_and_warning_detection() {
        let mut view = StemsExportView::new();
        view.loudness_standard = LoudnessStandard::SpotifyYouTube; // Max TP: -1.0 dBTP
        
        // Track 1 has true peak -0.2 dBTP, which exceeds -1.0 dBTP ceiling
        assert!(view.has_true_peak_warning());
        assert!(view.true_peak_margin_dbtp() < 0.0);
        assert!(view.warning_message().is_some());
        let warning = view.warning_message().unwrap();
        assert!(warning.contains("True Peak Warning"));
        assert!(warning.contains("exceeds"));

        // If we set loudness standard to Bypass, warning is suppressed
        view.loudness_standard = LoudnessStandard::Bypass;
        assert!(!view.has_true_peak_warning());
        assert!(view.warning_message().is_none());

        // If tracks have peaks well below ceiling (-3.0 dBTP)
        view.loudness_standard = LoudnessStandard::SpotifyYouTube;
        for t in &mut view.tracks {
            t.true_peak_dbtp = -3.5;
        }
        view.include_master = false;
        assert!(!view.has_true_peak_warning());
        assert!(view.true_peak_margin_dbtp() > 0.0);
    }

    #[test]
    fn test_turn53_track_selection_operations() {
        let mut view = StemsExportView::new();
        assert_eq!(view.selected_tracks_count(), 4);

        view.deselect_all();
        assert_eq!(view.selected_tracks_count(), 0);
        assert_eq!(view.total_stems_count(), 1); // Only master included

        view.select_all();
        assert_eq!(view.selected_tracks_count(), 4);
        assert_eq!(view.total_stems_count(), 5);

        view.tracks[0].is_selected = false;
        view.invert_selection();
        assert_eq!(view.selected_tracks_count(), 1);
        assert!(view.tracks[0].is_selected);
    }

    #[test]
    fn test_turn53_estimated_file_size_calculation() {
        let mut view = StemsExportView::new();
        view.sample_rate = 48000;
        view.bit_depth = BitDepth::Bit24;
        view.tail_decay_seconds = 2.0;
        view.project_duration_seconds = 180.0; // 182 seconds total
        view.format = StemExportFormat::Wav;

        let wav_size = view.estimated_size_mb(180.0);
        assert!(wav_size > 50.0 && wav_size < 500.0, "WAV size realistic: {}", wav_size);

        view.format = StemExportFormat::Flac;
        let flac_size = view.estimated_size_mb(180.0);
        assert!(flac_size < wav_size, "FLAC must be smaller than WAV: {} vs {}", flac_size, wav_size);
        assert!((flac_size / wav_size - 0.60).abs() < 0.05, "FLAC compression ratio ~0.60");
    }

    #[test]
    fn test_turn53_token_formatting_and_preview() {
        let view = StemsExportView::new();
        let preview = view.format_filename_preview("MySong", 0, 1, "Drums", Some("DrumsBus"));
        assert!(preview.contains("MySong"));
        assert!(preview.contains("01"));
        assert!(preview.contains("Drums"));
        assert!(preview.ends_with(".wav"));

        let master_preview = view.master_filename_preview("MySong");
        assert!(master_preview.contains("MySong"));
        assert!(master_preview.contains("Master_Mix"));
        assert!(master_preview.ends_with(".wav"));
    }

    #[test]
    fn test_turn53_to_export_preset_roundtrip() {
        let mut view = StemsExportView::new();
        view.apply_preset(StemPresetQuickPick::BroadcastEbuR128);

        let preset = view.to_export_preset();
        assert_eq!(preset.format, StemExportFormat::Wav);
        assert_eq!(preset.settings.bit_depth, BitDepth::Bit24);
        assert_eq!(preset.settings.sample_rate, 48000);
        assert!(preset.settings.normalize);
        assert!((preset.settings.target_db - (-23.0)).abs() < 1e-4);
        assert!(preset.group_by_bus);
        assert!(preset.include_master);
    }

    #[test]
    fn test_turn53_wcag_aaa_touch_targets() {
        assert!(
            STEM_EXPORT_MIN_HIT_HEIGHT >= 44.0,
            "Interactive buttons hit height must be >= 44pt (WCAG AAA)"
        );
        assert!(
            STEM_EXPORT_TOUCH_TARGET_SIZE.x >= 44.0 && STEM_EXPORT_TOUCH_TARGET_SIZE.y >= 44.0,
            "Touch target dimensions must be at least 44x44pt"
        );
    }

    #[test]
    fn test_turn53_deterministic_ascii_snapshot() {
        let view = StemsExportView::new();
        let snapshot = view.render_snapshot_ascii();

        assert!(!snapshot.is_empty());
        assert!(snapshot.contains("MULTI-TRACK STEMS EXPORT & LOUDNESS INSPECTOR"));
        assert!(snapshot.contains("STUDIO MASTER"));
        assert!(snapshot.contains("WAV"));
        assert!(snapshot.contains("24-bit"));
        assert!(snapshot.contains("48000 Hz"));
        assert!(snapshot.contains("Compliance:"));
        assert!(snapshot.contains("Stems:"));
        assert!(snapshot.contains("Pattern:"));
        assert!(snapshot.contains("Preview:"));
    }
}

#[cfg(test)]
pub mod gui_tests {
    use crate::command_palette::CommandPalette;
    use crate::views::award_winning_gui_view::AwardWinningGuiView;
    use crate::views::stems_export_view::{StemsExportTab, StemsExportView};
    use eframe::egui;
    use summoner_project::schema::{ProjectConfig, TrackConfig};

    #[test]
    fn test_turn53_headless_egui_render_without_panic() {
        let ctx = egui::Context::default();
        let mut view = StemsExportView::new();

        let tabs = [
            StemsExportTab::FormatQuality,
            StemsExportTab::TracksSelection,
            StemsExportTab::LoudnessCompliance,
            StemsExportTab::NamingOutput,
        ];

        for tab in tabs {
            view.active_tab = tab;
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    view.show(ui);
                });
            });
        }
    }

    #[test]
    fn test_turn53_modal_lifecycle_open_and_close() {
        let mut view = AwardWinningGuiView::new();
        assert!(!view.is_stems_export_modal_open());

        view.open_stems_export_modal();
        assert!(view.is_stems_export_modal_open());

        view.close_stems_export_modal();
        assert!(!view.is_stems_export_modal_open());
    }

    #[test]
    fn test_turn53_project_config_synchronization() {
        let mut project = ProjectConfig::default();
        project.name = "GoldenSession".to_string();
        project.tracks = vec![
            TrackConfig {
                id: 10,
                name: "Lead Vocal".to_string(),
                bus_target: Some("Vocals".to_string()),
                ..Default::default()
            },
            TrackConfig {
                id: 20,
                name: "Bass Synth".to_string(),
                bus_target: Some("Bass".to_string()),
                ..Default::default()
            },
            TrackConfig {
                id: 30,
                name: "Drum Machine".to_string(),
                bus_target: Some("Drums".to_string()),
                ..Default::default()
            },
        ];

        let mut view = StemsExportView::new();
        view.sync_from_project(&project);

        assert_eq!(view.project_name, "GoldenSession");
        assert_eq!(view.tracks.len(), 3);
        assert_eq!(view.tracks[0].track_name, "Lead Vocal");
        assert_eq!(view.tracks[0].bus_target.as_deref(), Some("Vocals"));
        assert_eq!(view.tracks[1].track_name, "Bass Synth");
        assert_eq!(view.tracks[2].track_name, "Drum Machine");
    }

    #[test]
    fn test_turn53_command_palette_action_registration() {
        let palette = CommandPalette::new();
        let action = palette.actions.iter().find(|a| a.action_id == "open_stems_export_modal");

        assert!(
            action.is_some(),
            "open_stems_export_modal must be registered in command palette"
        );
        let act = action.unwrap();
        assert_eq!(act.category, "Project & Export");
        assert!(act.label.contains("Stems Export"));
        assert_eq!(act.shortcut_hint.as_deref(), Some("Ctrl+Shift+E"));
    }

    #[test]
    fn test_turn53_rack_and_inspector_trigger_propagation() {
        let mut view = AwardWinningGuiView::new();
        assert!(!view.is_stems_export_modal_open());

        view.inspector_state.requested_open_stems_export_modal = true;

        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        assert!(view.is_stems_export_modal_open());
        assert!(!view.inspector_state.requested_open_stems_export_modal);
    }

    #[test]
    fn test_turn53_execute_export_to_temp_directory() {
        let temp_dir = std::env::temp_dir().join("summoner_turn53_test_stems");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut project = ProjectConfig::default();
        project.name = "Turn53Test".to_string();
        project.tracks = vec![
            TrackConfig {
                id: 1,
                name: "Kick".to_string(),
                ..Default::default()
            },
            TrackConfig {
                id: 2,
                name: "Snare".to_string(),
                ..Default::default()
            },
        ];

        let mut view = StemsExportView::new();
        view.sync_from_project(&project);
        view.include_master = true;

        let buffers = vec![vec![0.1f32; 1024], vec![0.2f32; 1024]];
        let result = view.execute_export(&project, &temp_dir, Some(&buffers));

        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        let report = result.unwrap();

        assert_eq!(report.total_stems, 3); // 2 tracks + master
        assert_eq!(view.exported_files_count, 3);
        assert!(view.total_exported_bytes > 0);
        assert!(!view.is_exporting);
        assert_eq!(view.export_progress, 1.0);

        // Verify exported files exist on disk
        for path in &report.exported_files {
            assert!(path.exists(), "Stem file does not exist: {:?}", path);
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
