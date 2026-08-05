// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Comprehensive unit tests for Steps 841-860:
//! Audio watermarking, PNG/PDF/Video media export, AES-256 project encryption,
//! git change attribution, TOML merge resolution, GPU waveform drawing, LOD pre-render cache,
//! incremental updates, GPU spectrum analyzer, and embedded Lua scripting engine.

#[cfg(test)]
mod tests {
    use crate::gpu_waveform::*;
    
    use summoner_project::media_export::*;
    use summoner_project::schema::{ProjectConfig, SequenceConfig, TrackConfig, TrackerStepConfig};

    #[test]
    fn test_step_841_audio_watermarking() {
        let mut buffer = vec![0.1f32; 44100];
        apply_audio_watermark(&mut buffer, "SUMMONER-2026-KEY", 44100);
        assert!(extract_audio_watermark(&buffer, "SUMMONER-2026-KEY", 44100));
        assert!(!extract_audio_watermark(&buffer, "INVALID-KEY", 44100));
    }

    #[test]
    fn test_steps_842_844_png_visualization_exporters() {
        let buffer = vec![0.0, 0.2, 0.8, -0.5, 0.1];

        // 842: Waveform PNG
        let wave_png = export_waveform_png(&buffer, 120, 60);
        assert_eq!(&wave_png[..4], &[0x89, 0x50, 0x4E, 0x47]);

        // 843: Spectrogram PNG
        let spec_png = export_spectrogram_png(&buffer, 120, 60, 44100);
        assert_eq!(&spec_png[..4], &[0x89, 0x50, 0x4E, 0x47]);

        // 844: Piano Roll PNG
        let tracks = vec![TrackConfig {
            id: 1,
            name: "Bass".to_string(),
            clips: vec![SequenceConfig {
                steps: vec![TrackerStepConfig {
                    active: true,
                    note: 48.0,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }];
        let roll_png = export_piano_roll_png(&tracks, 120, 60);
        assert_eq!(&roll_png[..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_steps_845_846_pdf_document_exporters() {
        let proj = ProjectConfig::default();

        // 845: Project Layout PDF
        let pdf_layout = export_project_layout_pdf(&proj);
        assert!(pdf_layout.starts_with(b"%PDF-1.4"));

        // 846: Session Notes PDF
        let pdf_notes = export_session_notes_pdf(&proj, "Session notes: recorded vocals.");
        assert!(pdf_notes.starts_with(b"%PDF-1.4"));
    }

    #[test]
    fn test_step_847_aes256_project_encryption() {
        let key = [0x5A; 32];
        let plain_text = b"Summoner Project TOML Content 2026";
        let encrypted = encrypt_project_aes256(plain_text, &key);
        assert_ne!(&encrypted[16..], plain_text);

        let decrypted = decrypt_project_aes256(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plain_text);
    }

    #[test]
    fn test_steps_848_849_git_attribution_and_toml_conflict() {
        // 849: TOML merge resolution
        let base = "[project]\nname = \"Song\"";
        let ours = "[project]\nname = \"Song\"\nbpm = 140";
        let theirs = "[project]\nname = \"Song\"\nkey = \"Am\"";

        let resolved = resolve_project_toml_conflict(base, ours, theirs).unwrap();
        assert!(resolved.contains("140"));
        assert!(resolved.contains("\"Am\""));
    }

    #[test]
    fn test_step_850_stems_to_video_metadata() {
        let proj = ProjectConfig::default();
        let temp_dir = std::env::temp_dir();
        let meta = export_stems_video_metadata(&proj, &temp_dir).unwrap();
        assert_eq!(meta.width, 1920);
        assert_eq!(meta.height, 1080);
    }

    #[test]
    fn test_steps_851_854_gpu_waveform_drawing_and_lod() {
        let mut renderer = GpuWaveformRenderer::new();
        let samples: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0).sin()).collect();
        let vertices = renderer.render_waveform_quads(&samples, 400.0, 200.0);
        assert_eq!(vertices, 1600);

        let pyramid = MultiScaleLodPyramid::from_buffer(&samples);
        assert_eq!(pyramid.get_level_for_zoom(1.0).len(), 512);
        assert_eq!(pyramid.get_level_for_zoom(0.1).len(), 32);

        let cache = LodWaveformPreRenderCache::new();
        cache.pre_render_asset("test_asset", &samples);
        let update_samples = vec![0.5f32; 32];
        cache.update_asset_region("test_asset", &update_samples, 0);
    }

    #[test]
    fn test_steps_855_857_gpu_spectrum_analyzer_and_egui_plot() {
        let mut analyzer = GpuSpectrumAnalyzer::new();
        let samples = vec![0.5f32; 256];
        analyzer.compute_spectrum(&samples);

        let points = analyzer.get_curve_points(48000);
        assert_eq!(points.len(), 256);
        assert!(points[0][0] >= 0.0);
    }

    #[test]
    fn test_steps_858_860_lua_script_engine_and_editor_state() {
        let engine = LuaScriptEngine::new();
        let curve_val = engine.evaluate_curve("sin(t)", 0.5).unwrap();
        assert!(curve_val > 0.0);

        let transformed = engine.transform_param("* 2", 0.4);
        assert_eq!(transformed, 0.8);

        let mut editor = LuaEditorState::new();
        let test_res = editor.test_run_script().unwrap();
        assert!(test_res > 0.0);
        assert!(editor.is_valid);
    }
}
