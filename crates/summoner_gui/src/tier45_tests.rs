// Summoner DAW - Tier 45 GUI & DSP Unit Tests (Steps 1221-1234)

#[cfg(test)]
mod tests {
    use summoner_dsp::spectrogram_art::{
        SpectrogramArtConfig, SpectrogramArtEngine, SpectrogramImage, FrequencyMapping, ColorMappingMode, SpectrogramArtNode,
    };
    use summoner_dsp::live_session_recorder::{LiveSessionRecorder, RecordingFormat};
    use summoner_dsp::visualizer_engine::{VisualizerIntegrationEngine, VisualizerPreset};
    use summoner_project::scratch_audio_cache::ScratchAudioCache;
    #[cfg(feature = "gui")]
    use crate::app::{SummonerApp, GuiDisplayMode};
    use summoner_project::schema::ProjectConfig;
    use summoner_core::param_bus::ParamBus;
    use summoner_core::node::AudioNode;
    use summoner_core::transport::Transport;
    use summoner_core::node::ProcessContext;
    use std::sync::Arc;
    use std::path::Path;
    use std::env;

    #[test]
    fn test_step_1221_spectrogram_art_frequency_mapping() {
        let config = SpectrogramArtConfig {
            min_freq_hz: 100.0,
            max_freq_hz: 10000.0,
            mapping: FrequencyMapping::Logarithmic,
            color_mode: ColorMappingMode::Grayscale,
            num_oscillators: 32,
        };
        let engine = SpectrogramArtEngine::new(config);
        
        let freq_bottom = engine.map_y_to_freq(31, 32);
        let freq_top = engine.map_y_to_freq(0, 32);
        assert!((freq_bottom - 100.0).abs() < 1.0);
        assert!((freq_top - 10000.0).abs() < 1.0);
    }

    #[test]
    fn test_step_1222_spectrogram_image_to_audio_buffer() {
        let config = SpectrogramArtConfig::default();
        let engine = SpectrogramArtEngine::new(config.clone());
        let mut img = SpectrogramImage::new(16, 16);
        
        // Draw diagonal line
        for i in 0..16 {
            img.set_pixel(i, i, 255, 255, 255);
        }

        let audio = engine.generate_audio_buffer(&img, 44100, 0.1);
        assert_eq!(audio.len(), 4410);
        assert!(audio.iter().any(|&s| s.abs() > 0.01));

        let node_image = img.clone();
        let mut node = SpectrogramArtNode::new(config, node_image);
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);
        let mut buf_l = vec![0.0f32; 64];
        let mut buf_r = vec![0.0f32; 64];

        node.process(&[], &mut [&mut buf_l, &mut buf_r], &ctx);
        assert!(buf_l.iter().any(|&s| s.abs() > 0.0));
    }

    #[test]
    fn test_step_1223_live_session_recorder() {
        let temp_wav = env::temp_dir().join("summoner_live_session_test.wav");
        let mut recorder = LiveSessionRecorder::new();

        assert!(!recorder.is_recording());
        recorder
            .start_recording(&temp_wav, 44100, 2, RecordingFormat::Wav)
            .expect("start recording");

        assert!(recorder.is_recording());

        let l_block = vec![0.1f32; 256];
        let r_block = vec![-0.1f32; 256];
        recorder.process_block(&l_block, &r_block);

        let stats = recorder.stop_recording().expect("stop recording");
        assert_eq!(stats.total_samples, 256);
        assert!(stats.file_size_bytes > 44);

        let _ = std::fs::remove_file(&temp_wav);
    }

    #[test]
    fn test_step_1224_visualizer_engine_dispatch() {
        let mut engine = VisualizerIntegrationEngine::new();
        engine.open_visualizer_window();
        assert!(engine.window_open);

        let preset = VisualizerPreset {
            name: "Milkdrop Test Preset".to_string(),
            preset_path: "presets/test.milk".to_string(),
            blend_video: false,
        };
        engine.load_preset(preset);

        let l_audio = vec![0.5f32; 128];
        let r_audio = vec![0.3f32; 128];
        let frame = engine.dispatch_frame(&l_audio, &r_audio, 44100, &[60, 64, 67], 128.0);

        assert!(frame.peak_amplitude > 0.0);
        assert_eq!(frame.active_notes, vec![60, 64, 67]);

        let state_str = engine.render_preset_frame(&frame);
        assert!(state_str.contains("Milkdrop Test Preset"));
    }

    #[test]
    fn test_step_1225_scratch_folder_audio_cache() {
        let temp_dir = env::temp_dir().join("summoner_scratch_folder_test");
        let cache = ScratchAudioCache::new(&temp_dir);

        let key = cache.compute_cache_key(Path::new("sample.wav"), 1.5, -3.0);
        let data = vec![0.1f32, 0.2f32, 0.3f32];
        cache.store_cached_audio(&key, &data, 48000, 1).expect("store cache");

        let (retrieved, sr, ch) = cache.get_cached_audio(&key).expect("get cached audio");
        assert_eq!(sr, 48000);
        assert_eq!(ch, 1);
        assert_eq!(retrieved.len(), 3);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(feature = "gui")]
    fn test_step_1226_gui_toggle_simple_advanced_mode() {
        let bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(ProjectConfig::default(), bus);

        assert_eq!(app.display_mode, GuiDisplayMode::SimpleMode);
        app.toggle_display_mode();
        assert_eq!(app.display_mode, GuiDisplayMode::AdvancedMode);
        app.toggle_display_mode();
        assert_eq!(app.display_mode, GuiDisplayMode::SimpleMode);
    }
}
