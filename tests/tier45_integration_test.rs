// Summoner DAW - Tier 45 End-to-End Integration Tests
// Steps 1231 & 1232: Live session recording pipeline, Spectrogram Art synthesis, scratch folder audio cache transformations

use summoner_dsp::spectrogram_art::{SpectrogramArtConfig, SpectrogramArtEngine, SpectrogramImage, FrequencyMapping, ColorMappingMode};
use summoner_dsp::live_session_recorder::{LiveSessionRecorder, RecordingFormat};
use summoner_dsp::visualizer_engine::{VisualizerIntegrationEngine, VisualizerPreset};
use summoner_project::scratch_audio_cache::ScratchAudioCache;
use std::env;
use std::fs;
use std::path::Path;

#[test]
fn test_tier45_spectrogram_art_pipeline_integration() {
    let config = SpectrogramArtConfig {
        min_freq_hz: 60.0,
        max_freq_hz: 12000.0,
        mapping: FrequencyMapping::Logarithmic,
        color_mode: ColorMappingMode::Grayscale,
        num_oscillators: 48,
    };
    let engine = SpectrogramArtEngine::new(config);
    let mut image = SpectrogramImage::new(32, 32);

    for x in 0..32 {
        image.set_pixel(x, x, 200, 200, 200);
    }

    let audio = engine.generate_audio_buffer(&image, 44100, 0.2);
    assert!(!audio.is_empty());
    assert!(audio.iter().any(|&s| s.abs() > 0.05));

    let triggers = engine.generate_sequencer_triggers(&image, 0.5, 2000);
    assert!(!triggers.is_empty());
}

#[test]
fn test_tier45_live_session_recording_pipeline_integration() {
    let temp_dir = env::temp_dir().join("summoner_tier45_live_rec");
    let rec_path = temp_dir.join("live_master_output.wav");

    let mut recorder = LiveSessionRecorder::new();
    recorder
        .start_recording(&rec_path, 44100, 2, RecordingFormat::Wav)
        .expect("Start live session recording");

    assert!(recorder.is_recording());

    let l_stream = vec![0.25f32; 1024];
    let r_stream = vec![-0.25f32; 1024];

    for _ in 0..10 {
        recorder.process_block(&l_stream, &r_stream);
    }

    let stats = recorder.stop_recording().expect("Stop recording");
    assert_eq!(stats.total_samples, 10240);
    assert!(stats.file_size_bytes > 44);
    assert!(rec_path.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_tier45_scratch_folder_audio_cache_integration() {
    let scratch_dir = env::temp_dir().join("summoner_tier45_scratch_cache");
    let cache = ScratchAudioCache::new(&scratch_dir);

    let source = Path::new("tests/fixtures/sample_vocal.wav");
    let key1 = cache.compute_cache_key(source, 1.0, 0.0);
    let key2 = cache.compute_cache_key(source, 1.25, 3.0);
    assert_ne!(key1, key2);

    let audio_data = vec![0.0f32, 0.3f32, 0.6f32, 0.9f32, 0.6f32, 0.3f32, 0.0f32];
    cache.store_cached_audio(&key1, &audio_data, 44100, 1).expect("store");

    let (retrieved, sr, ch) = cache.get_cached_audio(&key1).expect("retrieve");
    assert_eq!(sr, 44100);
    assert_eq!(ch, 1);
    assert_eq!(retrieved.len(), audio_data.len());

    let count = cache.clear_cache().expect("clear");
    assert!(count >= 1);

    let _ = fs::remove_dir_all(&scratch_dir);
}

#[test]
fn test_tier45_visualizer_engine_routing_integration() {
    let mut vis = VisualizerIntegrationEngine::new();
    vis.load_preset(VisualizerPreset {
        name: "Cream of the Crop OpenGL".to_string(),
        preset_path: "presets/cream.milk".to_string(),
        blend_video: true,
    });

    let l = vec![0.8f32; 256];
    let r = vec![0.8f32; 256];
    let frame = vis.dispatch_frame(&l, &r, 44100, &[48, 52, 55], 140.0);

    assert!(frame.bass_energy > 0.0);
    let frame_desc = vis.render_preset_frame(&frame);
    assert!(frame_desc.contains("Cream of the Crop OpenGL"));
}
