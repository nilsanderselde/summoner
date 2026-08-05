// Summoner DAW - Tier 47 GUI & DSP Integration Unit Tests (Steps 1261-1280)

#[cfg(test)]
mod tests {
    use summoner_core::node::ProcessContext;
    use summoner_dsp::oscillators::{OscWavetable, SimdPolyWavetableOscillator};
    use summoner_dsp::SignalProcessor;

    #[test]
    fn test_step_1261_simd_polyphonic_wavetable_oscillator_gui_integration() {
        let mut synth = SimdPolyWavetableOscillator::new(48000);

        // Polyphonic note trigger
        synth.note_on(48, 0.9); // C3
        synth.note_on(52, 0.8); // E3
        synth.note_on(55, 0.8); // G3
        synth.note_on(59, 0.7); // B3
        synth.note_on(62, 0.7); // D4
        assert_eq!(synth.active_voice_count(), 5);

        // Wavetable morphing setup
        let sine_table = OscWavetable::default_sine();
        let triangle_table = OscWavetable::default_triangle();
        synth = synth
            .with_table(sine_table)
            .with_table2(triangle_table, 0.75);

        let mut out_buffer = vec![vec![0.0f32; 512]; 2];
        let mut slices: Vec<&mut [f32]> = out_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();
        let ctx = ProcessContext::new(48000, 120.0, 0);

        synth.process_block(&[], &mut slices, &ctx);

        for s in slices[0].iter() {
            assert!(s.is_finite());
            assert!(s.abs() <= 5.0, "Stereo output sample should remain bounded");
        }

        // Steal voice test (max voices exceed)
        for note in 60..80 {
            synth.note_on(note, 0.5);
        }
        assert!(synth.active_voice_count() <= synth.max_voices);

        // Turn all notes off
        synth.all_notes_off();
        for _ in 0..10000 {
            synth.process_sample();
        }
        assert_eq!(synth.active_voice_count(), 0);
    }

    #[test]
    fn test_step_1262_multi_channel_spectral_equalizer_node() {
        use summoner_core::node::AudioNode;
        use summoner_dsp::MultiChannelSpectralEqualizerNode;

        let mut eq = MultiChannelSpectralEqualizerNode::new(48000, 2, 8);
        eq.set_band_gain(0, 6.0);
        eq.set_band_gain(1, -3.0);

        let input_ch0 = vec![0.5f32; 256];
        let input_ch1 = vec![-0.5f32; 256];
        let inputs: Vec<&[f32]> = vec![&input_ch0, &input_ch1];

        let mut out_ch0 = vec![0.0f32; 256];
        let mut out_ch1 = vec![0.0f32; 256];
        let mut outputs: Vec<&mut [f32]> = vec![&mut out_ch0, &mut out_ch1];

        let ctx = ProcessContext::new(48000, 120.0, 0);
        eq.process(&inputs, &mut outputs, &ctx);

        let spec0 = eq.get_live_spectrum(0);
        let spec1 = eq.get_live_spectrum(1);
        assert!(!spec0.is_empty());
        assert!(!spec1.is_empty());
        assert!(out_ch0.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_step_1263_adaptive_buffer_size_auto_scaling() {
        use std::time::Duration;
        use summoner_core::adaptive_buffer::AdaptiveBufferScaler;

        let mut scaler = AdaptiveBufferScaler::new(48000, 256);
        assert_eq!(scaler.current_buffer_size, 256);
        assert!(scaler.current_latency_ms() > 0.0);

        // Record underrun -> should scale up buffer size
        scaler.record_block_processing(Duration::from_micros(500), true);
        assert_eq!(scaler.current_buffer_size, 512);

        // Reset to small buffer
        let sub_scaler = AdaptiveBufferScaler::new(48000, 32);
        assert!(sub_scaler.is_sub_millisecond());
    }

    #[test]
    fn test_step_1264_stem_metadata_and_multi_track_router() {
        use std::collections::HashMap;
        use summoner_dsp::sampler::SampleBuffer;
        use summoner_dsp::stem_separator::{
            MultiTrackAudioRouter, StemMetadata, StemMetadataParser,
        };

        let parser = StemMetadataParser::new();
        let metadata = vec![
            StemMetadata {
                stem_name: "drums".to_string(),
                gain_db: 0.0,
                target_track_index: 0,
                pan: 0.0,
                is_muted: false,
            },
            StemMetadata {
                stem_name: "bass".to_string(),
                gain_db: -2.0,
                target_track_index: 1,
                pan: 0.0,
                is_muted: false,
            },
        ];

        let json = parser.export_json(&metadata);
        let parsed = parser.parse_json(&json).expect("JSON parse should succeed");
        assert_eq!(parsed.len(), 2);

        let router = MultiTrackAudioRouter::new(2);
        let mut stems = HashMap::new();
        stems.insert(
            "drums".to_string(),
            SampleBuffer::new(vec![0.8f32; 100], 48000, 1),
        );
        stems.insert(
            "bass".to_string(),
            SampleBuffer::new(vec![0.4f32; 100], 48000, 1),
        );

        let routed = router.route_stems(&stems, &metadata);
        assert_eq!(routed.len(), 2);
    }

    #[test]
    fn test_step_1265_mpe_expression_curve_editor() {
        use summoner_core::mpe::{ExpressionCurveType, MpeExpressionCurveEditor};

        let editor = MpeExpressionCurveEditor::new(48.0);
        let val_lin = editor.map_expression_value(0.5, ExpressionCurveType::Linear);
        let val_exp = editor.map_expression_value(0.5, ExpressionCurveType::Exponential);
        let val_log = editor.map_expression_value(0.5, ExpressionCurveType::Logarithmic);

        assert!((val_lin - 0.5).abs() < 1e-5);
        assert!((0.0..=1.0).contains(&val_exp));
        assert!((0.0..=1.0).contains(&val_log));
    }

    #[test]
    fn test_step_1267_neural_audio_style_transfer_renderer() {
        use summoner_dsp::neural_dsp::{AudioStylePreset, NeuralAudioStyleTransferPreviewRenderer};
        use summoner_dsp::sampler::SampleBuffer;

        let renderer = NeuralAudioStyleTransferPreviewRenderer::new();
        let input = SampleBuffer::new(vec![0.3f32; 256], 48000, 1);
        let output = renderer.render_preview(&input, AudioStylePreset::VintageTape, 0.5);

        assert_eq!(output.data.len(), 256);
        assert!(output.data.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_step_1271_simd_wavetable_oscillator_unit_tests() {
        let mut synth = SimdPolyWavetableOscillator::new(44100);
        synth.note_on(60, 0.8);
        assert_eq!(synth.active_voice_count(), 1);

        let sample = synth.process_sample();
        assert!(sample.is_finite());
        assert!(sample.abs() <= 2.0);
    }

    #[test]
    fn test_step_1272_ebu_r128_loudness_and_peak_headroom_unit_tests() {
        use summoner_dsp::meter::{EbuR128LoudnessMeter, PeakHeadroomAnalyzer};

        let mut meter = EbuR128LoudnessMeter::new(-23.0);
        let mut analyzer = PeakHeadroomAnalyzer::new(1.0);

        let test_buffer = vec![0.5f32, -0.4f32, 0.8f32, -0.7f32];
        meter.process_block(&test_buffer);
        analyzer.analyze(&test_buffer);

        assert!(meter.momentary_lufs.is_finite());
        assert!(analyzer.peak_sample_db.is_finite());
        assert!(analyzer.true_peak_db.is_finite());
    }

    #[test]
    fn test_step_1273_project_auto_save_snapshot_manager_integration() {
        use summoner_project::backup::ProjectAutoSaveManager;
        use summoner_project::create_default_project;

        let dir = std::env::temp_dir().join("summoner_test_autosave_1273");
        let _ = std::fs::create_dir_all(&dir);
        let project = create_default_project("AutoSave Test Session");

        let mut manager = ProjectAutoSaveManager::new(&dir, 1, 3);
        assert!(manager.should_auto_save());

        let snapshot_path = manager
            .create_backup_snapshot(&project)
            .expect("Snapshot creation should succeed");
        assert!(snapshot_path.exists());

        let backups = manager
            .list_backups()
            .expect("Listing backups should succeed");
        assert!(!backups.is_empty());

        let restored = manager
            .restore_snapshot(&snapshot_path)
            .expect("Restore should succeed");
        assert_eq!(restored.name, "AutoSave Test Session");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_step_1266_backup_snapshot_creator_configurable_interval() {
        use summoner_project::backup::ProjectAutoSaveManager;
        use summoner_project::create_default_project;

        let dir = std::env::temp_dir().join("summoner_test_backup_1266");
        let _ = std::fs::create_dir_all(&dir);
        let project = create_default_project("Backup Config Test");

        let mut manager = ProjectAutoSaveManager::new(&dir, 60, 5);
        assert_eq!(manager.auto_save_interval.as_secs(), 60);
        assert_eq!(manager.max_backups, 5);

        let snapshot = manager
            .create_backup_snapshot(&project)
            .expect("Backup creation should succeed");
        assert!(snapshot.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_step_1268_cli_project_wizard_template_creation() {
        use summoner_project::{
            create_project_from_template, parse_project_toml, serialize_project_toml,
        };

        let template_names = vec!["Default", "Electronic", "Orchestral", "Minimal"];
        for t in template_names {
            let proj = create_project_from_template("Wizard Session", t);
            let serialized = serialize_project_toml(&proj).expect("Serialization failed");
            let parsed = parse_project_toml(&serialized).expect("Parsing failed");
            assert_eq!(parsed.name, "Wizard Session");
        }
    }

    #[test]
    fn test_step_1269_audio_driver_selector_panel_defaults() {
        use crate::visualizer::AudioDriverSelectorPanel;

        let mut panel = AudioDriverSelectorPanel::default();
        assert_eq!(panel.selected_driver_name, "WASAPI");
        assert_eq!(panel.sample_rate, 48000);
        assert_eq!(panel.buffer_size, 256);
        assert!(!panel.exclusive_mode);
        assert!(panel.device_list.len() >= 4);

        panel.selected_driver_name = "ALSA".to_string();
        panel.exclusive_mode = true;
        assert_eq!(panel.selected_driver_name, "ALSA");
        assert!(panel.exclusive_mode);
    }
}
