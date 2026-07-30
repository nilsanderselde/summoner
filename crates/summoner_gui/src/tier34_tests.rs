// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 34 unit tests for media export, visualization generation (PNG/PDF/Video), AES-256 project encryption,
//! audio watermarking, change attribution, TOML merge resolution, GPU waveform rendering, and Lua scripting (Steps 841-860).

#[cfg(test)]
mod tests {
    use summoner_project::media_export::{
        apply_audio_watermark, extract_audio_watermark, export_waveform_png, export_spectrogram_png,
        export_piano_roll_png, export_project_layout_pdf, export_session_notes_pdf,
        encrypt_project_aes256, decrypt_project_aes256, resolve_project_toml_conflict,
        export_stems_video_metadata, LuaScriptEngine,
    };
    use crate::gpu_waveform::{
        GpuWaveformRenderer, MultiScaleLodPyramid, LodWaveformPreRenderCache,
        GpuSpectrumAnalyzer, LuaEditorState,
    };
    use summoner_project::schema::{ProjectConfig, TrackConfig, SequenceConfig, TrackerStepConfig};

    #[test]
    fn test_tier34_audio_watermarking() {
        let mut buffer = vec![0.5f32; 44100];
        apply_audio_watermark(&mut buffer, "SUMMONER-WATERMARK-2026", 44100);
        assert!(extract_audio_watermark(&buffer, "SUMMONER-WATERMARK-2026", 44100));
        assert!(!extract_audio_watermark(&buffer, "WRONG-KEY", 44100));
    }

    #[test]
    fn test_tier34_png_visualization_exports() {
        let samples = vec![0.0, 0.5, 1.0, 0.2, -0.8, 0.0];
        let png_wave = export_waveform_png(&samples, 100, 50);
        assert!(png_wave.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

        let png_spec = export_spectrogram_png(&samples, 100, 50, 44100);
        assert!(png_spec.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

        let tracks = vec![TrackConfig {
            id: 1,
            name: "Lead".to_string(),
            clips: vec![SequenceConfig {
                steps: vec![TrackerStepConfig { active: true, note: 60.0, ..Default::default() }],
                ..Default::default()
            }],
            ..Default::default()
        }];
        let png_roll = export_piano_roll_png(&tracks, 100, 50);
        assert!(png_roll.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
    }

    #[test]
    fn test_tier34_pdf_exports() {
        let proj = ProjectConfig::default();
        let pdf_layout = export_project_layout_pdf(&proj);
        assert!(pdf_layout.starts_with(b"%PDF-1.4"));

        let pdf_notes = export_session_notes_pdf(&proj, "Session notes for project.");
        assert!(pdf_notes.starts_with(b"%PDF-1.4"));
    }

    #[test]
    fn test_tier34_aes256_encryption() {
        let data = b"Confidential Project TOML Content";
        let key = [99u8; 32];
        let encrypted = encrypt_project_aes256(data, &key);
        assert_ne!(encrypted[16..], *data);

        let decrypted = decrypt_project_aes256(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_tier34_toml_conflict_resolution() {
        let base = "[project]\nname = \"Base\"";
        let ours = "[project]\nname = \"Base\"\nbpm = 120";
        let theirs = "[project]\nname = \"Base\"\nkey = \"C\"";

        let resolved = resolve_project_toml_conflict(base, ours, theirs).unwrap();
        assert!(resolved.contains("120"));
        assert!(resolved.contains("\"C\""));
    }

    #[test]
    fn test_tier34_video_export_spec() {
        let mut proj = ProjectConfig::default();
        proj.tracks.push(TrackConfig::default());
        let temp_dir = std::env::temp_dir().join(format!("test_stems_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let video_spec = export_stems_video_metadata(&proj, &temp_dir).unwrap();
        assert_eq!(video_spec.width, 1920);
        assert_eq!(video_spec.height, 1080);
        assert_eq!(video_spec.fps, 60);
        assert_eq!(video_spec.stems_count, 1);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tier34_gpu_waveform_and_lod_caching() {
        let mut renderer = GpuWaveformRenderer::new();
        let buffer: Vec<f32> = (0..512).map(|i| (i as f32 / 512.0).sin()).collect();
        let quads = renderer.render_waveform_quads(&buffer, 256.0, 100.0);
        assert!(quads > 0);

        let mut pyramid = MultiScaleLodPyramid::from_buffer(&buffer);
        assert_eq!(pyramid.level_1x.len(), 512);

        let update_samples = vec![0.8f32; 32];
        pyramid.update_slice(&update_samples, 0);

        let cache = LodWaveformPreRenderCache::new();
        cache.pre_render_asset("asset_a", &buffer);
        cache.update_asset_region("asset_a", &update_samples, 0);
    }

    #[test]
    fn test_tier34_spectrum_analyzer_and_lua_editor() {
        let mut analyzer = GpuSpectrumAnalyzer::new();
        let samples = vec![0.5f32; 256];
        analyzer.compute_spectrum(&samples);
        let pts = analyzer.get_curve_points(44100);
        assert_eq!(pts.len(), 256);

        let mut lua_editor = LuaEditorState::new();
        let val = lua_editor.test_run_script().unwrap();
        assert!(val > 0.0);
        assert!(lua_editor.is_valid);
    }
}
