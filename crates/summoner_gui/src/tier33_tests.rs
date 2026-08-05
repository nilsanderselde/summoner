// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 33 unit tests for GUI state persistence, multi-monitor, panel docking, drag & drop, and sharing (Steps 821-840).

#[cfg(test)]
mod tests {
    use crate::app::{GuiState, SummonerApp};

    use std::sync::Arc;
    use summoner_core::param_bus::ParamBus;
    use summoner_project::schema::{ProjectConfig, ProjectMetadata, SequenceConfig, TrackConfig};

    #[test]
    fn test_tier33_keyboard_shortcuts_and_window_state() {
        let bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(ProjectConfig::default(), bus);

        app.window_pos = Some([150.0, 200.0]);
        app.window_size = Some([1280.0, 800.0]);
        app.multi_monitor_enabled = true;
        app.detached_panels = vec!["NodeGraph".to_string(), "Mixer".to_string()];
        app.panel_dock_edge = "Right".to_string();
        app.snap_to_edges = true;
        app.show_project_notes_panel = true;

        let test_path =
            std::env::temp_dir().join(format!("test_tier33_gui_state_{}.toml", std::process::id()));
        let state = GuiState {
            current_view: app.current_view.clone(),
            selected_track_id: app.selected_track_id,
            show_rack: app.show_rack,
            pixels_per_beat: app.pixels_per_beat,
            macro_rack_height: app.macro_rack_height,
            track_header_width: app.track_header_width,
            dark_theme: app.dark_theme,
            first_run: app.show_first_run_wizard,
            beginner_mode: app.beginner_mode,
            recent_projects: app.recent_projects.clone(),
            auto_save_interval_secs: app.auto_save_interval_secs,
            show_tutorial_tooltips: app.show_tutorial_tooltips,
            high_contrast_mode: app.high_contrast_mode,
            font_size: app.font_size,
            reduce_motion: app.reduce_motion,
            keyboard_navigation: app.keyboard_navigation,
            window_pos: app.window_pos,
            window_size: app.window_size,
            multi_monitor_enabled: app.multi_monitor_enabled,
            detached_panels: app.detached_panels.clone(),
            panel_dock_edge: app.panel_dock_edge.clone(),
            snap_to_edges: app.snap_to_edges,
            show_project_notes_panel: app.show_project_notes_panel,
        };
        state.save_to_path(&test_path);

        let loaded = GuiState::load_from_path(&test_path).expect("Failed to load saved GuiState");
        assert_eq!(loaded.window_pos, Some([150.0, 200.0]));
        assert_eq!(loaded.window_size, Some([1280.0, 800.0]));
        assert!(loaded.multi_monitor_enabled);
        assert_eq!(loaded.detached_panels, vec!["NodeGraph", "Mixer"]);
        assert_eq!(loaded.panel_dock_edge, "Right");
        assert!(loaded.snap_to_edges);
        assert!(loaded.show_project_notes_panel);

        let _ = std::fs::remove_file(&test_path);
    }

    #[test]
    fn test_tier33_file_drag_and_drop_handling() {
        let bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(ProjectConfig::default(), bus);

        let res_wav = app
            .handle_dropped_file(std::path::Path::new("samples/kick.wav"))
            .unwrap();
        assert!(res_wav.contains("Added audio asset"));
        assert_eq!(app.project.assets.len(), 1);
        assert_eq!(app.project.assets[0].path, "samples/kick.wav");

        let res_flac = app
            .handle_dropped_file(std::path::Path::new("samples/snare.flac"))
            .unwrap();
        assert!(res_flac.contains("Added audio asset"));
        assert_eq!(app.project.assets.len(), 2);

        let err_res = app.handle_dropped_file(std::path::Path::new("samples/kick.mp3"));
        assert!(err_res.is_err());
    }

    #[test]
    fn test_tier33_clip_and_preset_drag_and_drop() {
        let bus = Arc::new(ParamBus::new());
        let mut proj = ProjectConfig::default();
        proj.tracks.push(TrackConfig {
            id: 1,
            name: "Track 1".to_string(),
            clips: vec![SequenceConfig {
                clip_name: Some("Clip A".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        });
        proj.tracks.push(TrackConfig {
            id: 2,
            name: "Track 2".to_string(),
            clips: Vec::new(),
            ..Default::default()
        });

        let mut app = SummonerApp::new(proj, bus);

        // Move clip from track 1 to track 2
        app.move_clip_between_tracks(1, 0, 2).unwrap();
        assert_eq!(app.project.tracks[0].clips.len(), 0);
        assert_eq!(app.project.tracks[1].clips.len(), 1);
        assert_eq!(
            app.project.tracks[1].clips[0].clip_name.as_deref(),
            Some("Clip A")
        );

        // Drag preset into Arranger
        let new_track_id = app.create_track_from_preset("OscSaw");
        assert_eq!(new_track_id, 3);
        assert_eq!(app.project.tracks.len(), 3);
        assert_eq!(app.project.tracks[2].nodes[0].kind, "OscSaw");
    }

    #[test]
    fn test_tier33_go_to_definition_doc_url() {
        let bus = Arc::new(ParamBus::new());
        let app = SummonerApp::new(ProjectConfig::default(), bus);
        let url = app.get_node_doc_url("DistortionNode");
        assert_eq!(url, "https://summoner.audio/docs/nodes/distortionnode");
    }

    #[test]
    fn test_tier33_share_export() {
        let bus = Arc::new(ParamBus::new());
        let mut app = SummonerApp::new(ProjectConfig::default(), bus);

        let path = std::path::Path::new("export/master.wav");
        app.share_project_export(path);
        assert!(app
            .last_share_action_message
            .as_ref()
            .unwrap()
            .contains("master.wav"));
    }

    #[test]
    fn test_tier33_project_metadata() {
        let proj = ProjectConfig {
            meta: Some(ProjectMetadata {
                tags: vec!["ambient".to_string(), "synthwave".to_string()],
                genre: Some("Electronic".to_string()),
                bpm: Some(124.0),
                key: Some("F#m".to_string()),
                notes: Some("Demo project metadata".to_string()),
            }),
            ..Default::default()
        };

        let toml_str = toml::to_string(&proj).unwrap();
        assert!(toml_str.contains("tags = [\"ambient\", \"synthwave\"]"));
        assert!(toml_str.contains("genre = \"Electronic\""));

        let parsed: ProjectConfig = toml::from_str(&toml_str).unwrap();
        let meta = parsed.meta.unwrap();
        assert_eq!(meta.tags, vec!["ambient", "synthwave"]);
        assert_eq!(meta.genre.as_deref(), Some("Electronic"));
        assert_eq!(meta.key.as_deref(), Some("F#m"));
    }
}
