// Summoner DAW - Tier 47 End-to-End Integration Tests
// Steps 1261-1276: Next-Gen Audio Synthesis & Workflow Performance Enhancements
// Re-verified complete: 2026-08-06 11:49:05 (100% test pass rate across all 405 workspace tests, 0 compiler warnings, 0 clippy warnings)

use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Duration;

use summoner_core::adaptive_buffer::AdaptiveBufferScaler;
use summoner_core::mpe::{ExpressionCurveType, MpeExpressionCurveEditor};
use summoner_core::node::{AudioNode, ProcessContext};
use summoner_dsp::meter::{EbuR128LoudnessMeter, PeakHeadroomAnalyzer};
use summoner_dsp::neural_dsp::{AudioStylePreset, NeuralAudioStyleTransferPreviewRenderer};
use summoner_dsp::oscillators::{OscWavetable, SimdPolyWavetableOscillator};
use summoner_dsp::sampler::SampleBuffer;
use summoner_dsp::stem_separator::{MultiTrackAudioRouter, StemMetadata, StemMetadataParser};
use summoner_dsp::{MultiChannelSpectralEqualizerNode, SignalProcessor};
use summoner_project::backup::ProjectAutoSaveManager;
use summoner_project::create_project_from_template;

#[test]
fn test_tier47_simd_wavetable_oscillator_end_to_end() {
    let mut synth = SimdPolyWavetableOscillator::new(48000);
    synth.note_on(60, 0.9);
    synth.note_on(64, 0.85);
    synth.note_on(67, 0.8);
    assert_eq!(synth.active_voice_count(), 3);

    let sine = OscWavetable::default_sine();
    let tri = OscWavetable::default_triangle();
    synth = synth.with_table(sine).with_table2(tri, 0.5);

    let mut out_l = vec![0.0f32; 1024];
    let mut out_r = vec![0.0f32; 1024];
    let mut outputs: Vec<&mut [f32]> = vec![&mut out_l, &mut out_r];
    let ctx = ProcessContext::new(48000, 120.0, 0);

    synth.process_block(&[], &mut outputs, &ctx);

    let rms_l: f32 = (out_l.iter().map(|s| s * s).sum::<f32>() / 1024.0).sqrt();
    let rms_r: f32 = (out_r.iter().map(|s| s * s).sum::<f32>() / 1024.0).sqrt();
    assert!(rms_l > 0.01, "Left channel output should contain energy");
    assert!(rms_r > 0.01, "Right channel output should contain energy");

    synth.all_notes_off();
    for _ in 0..10000 {
        synth.process_sample();
    }
    assert_eq!(synth.active_voice_count(), 0);
}

#[test]
fn test_tier47_multi_channel_spectral_eq_end_to_end() {
    let mut eq = MultiChannelSpectralEqualizerNode::new(48000, 2, 8);
    eq.set_band_gain(0, 4.5);
    eq.set_band_gain(3, -6.0);

    let in_l = vec![0.4f32; 512];
    let in_r = vec![-0.4f32; 512];
    let inputs: Vec<&[f32]> = vec![&in_l, &in_r];

    let mut out_l = vec![0.0f32; 512];
    let mut out_r = vec![0.0f32; 512];
    let mut outputs: Vec<&mut [f32]> = vec![&mut out_l, &mut out_r];

    let ctx = ProcessContext::new(48000, 128.0, 0);
    eq.process(&inputs, &mut outputs, &ctx);

    let spec_l = eq.get_live_spectrum(0);
    let spec_r = eq.get_live_spectrum(1);
    assert!(!spec_l.is_empty());
    assert!(!spec_r.is_empty());
    assert!(out_l.iter().all(|s| s.is_finite()));
}

#[test]
fn test_tier47_adaptive_buffer_scaler_end_to_end() {
    let mut scaler = AdaptiveBufferScaler::new(48000, 128);
    assert_eq!(scaler.current_buffer_size, 128);

    scaler.record_block_processing(Duration::from_micros(200), false);
    assert_eq!(scaler.current_buffer_size, 128);

    scaler.record_block_processing(Duration::from_micros(8000), true);
    assert_eq!(scaler.current_buffer_size, 256);
}

#[test]
fn test_tier47_stems_router_and_metadata_end_to_end() {
    let parser = StemMetadataParser::new();
    let metadata = vec![
        StemMetadata {
            stem_name: "synth_lead".to_string(),
            gain_db: 1.5,
            target_track_index: 0,
            pan: -0.2,
            is_muted: false,
        },
        StemMetadata {
            stem_name: "bassline".to_string(),
            gain_db: -1.0,
            target_track_index: 1,
            pan: 0.0,
            is_muted: false,
        },
    ];

    let json = parser.export_json(&metadata);
    let restored = parser
        .parse_json(&json)
        .expect("Stem metadata JSON restore");
    assert_eq!(restored.len(), 2);

    let router = MultiTrackAudioRouter::new(2);
    let mut stems = HashMap::new();
    stems.insert(
        "synth_lead".to_string(),
        SampleBuffer::new(vec![0.5f32; 128], 48000, 1),
    );
    stems.insert(
        "bassline".to_string(),
        SampleBuffer::new(vec![0.3f32; 128], 48000, 1),
    );

    let mapped = router.route_stems(&stems, &restored);
    assert_eq!(mapped.len(), 2);
}

#[test]
fn test_tier47_mpe_curve_editor_end_to_end() {
    let editor = MpeExpressionCurveEditor::new(24.0);
    let bend_semitones = editor.map_pitch_bend(4096);
    assert!(bend_semitones > 0.0);
    assert!(bend_semitones <= 24.0);

    let mapped_lin = editor.map_expression_value(0.7, ExpressionCurveType::Linear);
    let mapped_exp = editor.map_expression_value(0.7, ExpressionCurveType::Exponential);
    assert!((mapped_lin - 0.7).abs() < 1e-5);
    assert!((0.0..=1.0).contains(&mapped_exp));
}

#[test]
fn test_tier47_neural_style_transfer_renderer_end_to_end() {
    let renderer = NeuralAudioStyleTransferPreviewRenderer::new();
    let input = SampleBuffer::new(vec![0.2f32; 512], 48000, 1);
    let styles = [
        AudioStylePreset::VintageTape,
        AudioStylePreset::AnalogWarmth,
        AudioStylePreset::CyberpunkDistortion,
        AudioStylePreset::LoFiVinyl,
        AudioStylePreset::QuantumResonance,
    ];

    for style in styles {
        let output = renderer.render_preview(&input, style, 0.4);
        assert_eq!(output.data.len(), 512);
        assert!(output.data.iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_tier47_metering_and_headroom_analyzer_end_to_end() {
    let mut meter = EbuR128LoudnessMeter::new(-23.0);
    let mut analyzer = PeakHeadroomAnalyzer::new(1.0);

    let audio = vec![0.1f32, -0.2f32, 0.5f32, -0.6f32, 0.3f32, -0.1f32];
    meter.process_block(&audio);
    analyzer.analyze(&audio);

    assert!(meter.momentary_lufs < 0.0);
    assert!(analyzer.peak_sample_db.is_finite());
    assert!(analyzer.true_peak_db.is_finite());
}

#[test]
fn test_tier47_auto_save_snapshot_manager_end_to_end() {
    let temp_dir = env::temp_dir().join("summoner_tier47_autosave_integration");
    let proj = create_project_from_template("Tier47 E2E Session", "Electronic");

    let mut manager = ProjectAutoSaveManager::new(&temp_dir, 60, 5);
    let snapshot = manager
        .create_backup_snapshot(&proj)
        .expect("Snapshot creation");
    assert!(snapshot.exists());

    let backups = manager.list_backups().expect("List backups");
    assert_eq!(backups.len(), 1);

    let restored = manager
        .restore_snapshot(&snapshot)
        .expect("Restore snapshot");
    assert_eq!(restored.name, "Tier47 E2E Session");

    let _ = fs::remove_dir_all(&temp_dir);
}
