// Summoner DAW - Tier 46 GUI & Ecosystem Integration Unit Tests (Steps 1241-1260)

#[cfg(test)]
mod tests {
    use summoner_project::session_markers::{
        SessionMarkerNavigationManager, ChapterType, NavigationCommand,
    };
    use summoner_project::create_default_project;

    #[test]
    fn test_step_1246_session_marker_hotkey_bindings_and_chapter_navigation() {
        let mut nav = SessionMarkerNavigationManager::new();

        // 1. Add chapters for song structure
        nav.add_chapter("Intro", 0.0, 16.0, ChapterType::Intro);
        nav.add_chapter("Verse 1", 16.0, 48.0, ChapterType::Verse);
        nav.add_chapter("Pre-Chorus", 48.0, 64.0, ChapterType::Bridge);
        nav.add_chapter("Chorus 1", 64.0, 96.0, ChapterType::Chorus);
        nav.add_chapter("Outro", 96.0, 128.0, ChapterType::Outro);

        assert_eq!(nav.len(), 5);

        // 2. Test active chapter lookup across timeline
        assert_eq!(nav.find_chapter_at(8.0).unwrap().name, "Intro");
        assert_eq!(nav.find_chapter_at(32.0).unwrap().name, "Verse 1");
        assert_eq!(nav.find_chapter_at(50.0).unwrap().name, "Pre-Chorus");
        assert_eq!(nav.find_chapter_at(80.0).unwrap().name, "Chorus 1");
        assert_eq!(nav.find_chapter_at(110.0).unwrap().name, "Outro");

        // 3. Test hotkey navigation (Next / Prev)
        let cmd_next = nav.handle_key_input("Ctrl+Right", 0.0);
        assert_eq!(cmd_next, Some(NavigationCommand::JumpToBeat(16.0)));

        let cmd_prev = nav.handle_key_input("Ctrl+Left", 64.0);
        assert_eq!(cmd_prev, Some(NavigationCommand::JumpToBeat(48.0)));

        // 4. Test numbered hotkey jump
        let cmd_jump_3 = nav.handle_key_input("3", 0.0);
        assert_eq!(cmd_jump_3, Some(NavigationCommand::JumpToBeat(48.0)));

        // 5. Test loop active chapter command
        let cmd_loop = nav.handle_key_input("L", 48.0);
        assert_eq!(cmd_loop, Some(NavigationCommand::LoopChapter { start_beat: 48.0, end_beat: 64.0 }));

        // 6. Test project synchronization
        let mut proj = create_default_project("Session Nav Test");
        nav.sync_to_project(&mut proj);
        assert_eq!(proj.markers.len(), 5);

        let restored = SessionMarkerNavigationManager::from_project(&proj);
        assert_eq!(restored.len(), 5);
        assert_eq!(restored.get_marker(3).unwrap().chapter_type, ChapterType::Chorus);

        // 7. Test CUE sheet and YouTube timestamp export
        let timestamps = restored.export_chapter_timestamps_text(120.0);
        assert!(timestamps.contains("00:00 Intro"));
        assert!(timestamps.contains("00:08 Verse 1"));
        assert!(timestamps.contains("00:32 Chorus 1"));

        let cue = restored.export_cue_sheet("Chapter Master", "Summoner Producer", "master.wav", 120.0);
        assert!(cue.contains("TITLE \"Chapter Master\""));
        assert!(cue.contains("PERFORMER \"Summoner Producer\""));
        assert!(cue.contains("TRACK 04 AUDIO"));
        assert!(cue.contains("TITLE \"Chorus 1\""));
    }

    #[test]
    fn test_step_1247_offline_crash_dump_analyzer() {
        use summoner_project::crash_analyzer::{
            CrashDump, CrashDumpAnalyzer, CrashSeverity,
        };

        let dump1 = CrashDump::new(
            "dump-20260805-001",
            CrashSeverity::Fatal,
            "summoner_dsp",
            "vst3_host",
            "Access violation writing location 0x00000000",
            vec![
                "vst3_host::process_audio_block (line 120)".to_string(),
                "audio_engine::render_loop (line 45)".to_string(),
            ],
        )
        .with_metadata("sample_rate", "48000")
        .with_log("INFO: Native host started");

        let dump2 = CrashDump::new(
            "dump-20260805-002",
            CrashSeverity::Error,
            "summoner_core",
            "audio_driver",
            "Real-time audio buffer starvation underflow",
            vec!["audio_driver::wasapi::callback (line 89)".to_string()],
        )
        .with_metadata("buffer_frames", "64");

        let res1 = CrashDumpAnalyzer::analyze_dump(&dump1);
        assert_eq!(res1.dump_id, "dump-20260805-001");
        assert!(res1.is_offline_safe);
        assert!(res1.probable_root_cause.contains("Memory Access Violation"));
        assert!(res1.formatted_report.contains("OFFLINE CRASH REPORT DUMP ANALYZER"));

        let res2 = CrashDumpAnalyzer::analyze_dump(&dump2);
        assert!(res2.probable_root_cause.contains("Buffer Processing Underflow"));

        let temp_dir = std::env::temp_dir().join("summoner_crash_dumps_tier46");
        dump1.save_to_file(&temp_dir.join("dump1.json")).unwrap();
        dump2.save_to_file(&temp_dir.join("dump2.json")).unwrap();

        let summary = CrashDumpAnalyzer::analyze_dumps_directory(&temp_dir).unwrap();
        assert_eq!(summary.total_dumps_analyzed, 2);
        assert!(summary.formatted_summary.contains("LOCAL DISK CRASH REPORT DUMP SUMMARY"));
        assert!(!summary.recommendations.is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_step_1248_macro_parameter_mapping_matrix() {
        use summoner_dsp::modulators::{
            LfoShape, MacroModulationMatrix, ModulationCurve, ModulationSourceId,
        };

        let mut matrix = MacroModulationMatrix::new("Synth Main Matrix");

        // 1. Register modulation sources
        let macro_cutoff = matrix.add_macro(0.7);
        let macro_res = matrix.add_macro(0.4);
        let lfo_vibrato = matrix.add_lfo(6.0, LfoShape::Sine);
        let env_filter = matrix.add_envelope(0.005, 0.2, 0.3, 0.4);

        // 2. Register modulation targets
        let target_cutoff = matrix.add_target("Filter Cutoff", 2000.0, 20.0, 20000.0);
        let target_res = matrix.add_target("Filter Resonance", 1.0, 0.1, 10.0);
        let target_pitch = matrix.add_target("Oscillator Pitch", 440.0, 110.0, 1760.0);

        // 3. Add modulation assignments
        let assign_1 = matrix.add_assignment(
            macro_cutoff,
            target_cutoff,
            0.6,
            true,
            ModulationCurve::Exponential,
        );
        let assign_2 = matrix.add_assignment(
            lfo_vibrato,
            target_pitch,
            0.05,
            true,
            ModulationCurve::Linear,
        );
        let _assign_3 = matrix.add_assignment(
            env_filter,
            target_cutoff,
            0.5,
            false,
            ModulationCurve::SmoothStep,
        );
        let _assign_4 = matrix.add_assignment(
            macro_res,
            target_res,
            0.8,
            true,
            ModulationCurve::Logarithmic,
        );
        let _assign_vel = matrix.add_assignment(
            ModulationSourceId::Velocity,
            target_cutoff,
            0.2,
            false,
            ModulationCurve::Linear,
        );

        // 4. Trigger envelope gate and process audio samples
        matrix.trigger_envelope(0, true);
        matrix.set_velocity(0.85);

        for _ in 0..200 {
            matrix.process_sample(44100);
        }

        let cutoff_val = matrix.get_modulated_value(target_cutoff).unwrap();
        let res_val = matrix.get_modulated_value(target_res).unwrap();
        let pitch_val = matrix.get_modulated_value(target_pitch).unwrap();

        assert!((20.0..=20000.0).contains(&cutoff_val));
        assert!((0.1..=10.0).contains(&res_val));
        assert!((110.0..=1760.0).contains(&pitch_val));

        // 5. Test assignment modification and disable toggle
        matrix.set_assignment_enabled(assign_1, false);
        matrix.set_assignment_amount(assign_2, 0.1);

        matrix.process_sample(44100);
        assert!(!matrix.assignments[assign_1].enabled);
        assert_eq!(matrix.assignments[assign_2].amount, 0.1);
        assert_eq!(matrix.assignments.len(), 5);
        assert_eq!(matrix.targets.len(), 3);
    }

    #[test]
    fn test_step_1249_multi_track_stems_auto_naming_and_export_preset_manager() {
        use summoner_project::export::{
            format_stem_filename, BitDepth, ExportPreset, ExportPresetManager, StemExportFormat, ExportSettings,
        };
        use summoner_project::schema::{ProjectConfig, TrackConfig, TransportConfig};

        // 1. Test auto-naming template string token substitution
        let filename_wav = format_stem_filename(
            "{project}_{index}_{name}_{bus}_{sr}_{bit_depth}",
            "Solar Odyssey",
            0,
            1,
            "Synth Lead",
            Some("Melodic Bus"),
            48000,
            BitDepth::Bit24,
            "wav",
        );
        assert_eq!(filename_wav, "Solar_Odyssey_01_Synth_Lead_Melodic_Bus_48000Hz_24bit.wav");

        let filename_flac = format_stem_filename(
            "{index}_{name}",
            "Solar Odyssey",
            1,
            2,
            "Bass Synth",
            None,
            96000,
            BitDepth::Bit24,
            "flac",
        );
        assert_eq!(filename_flac, "02_Bass_Synth.flac");

        // 2. Test ExportPresetManager built-in presets
        let mut manager = ExportPresetManager::new();
        assert!(manager.list_presets().len() >= 5);

        let preset_cd = manager.get_preset("CD Quality WAV Stems").unwrap();
        assert_eq!(preset_cd.format, StemExportFormat::Wav);
        assert_eq!(preset_cd.settings.sample_rate, 44100);

        let preset_flac = manager.get_preset("Hi-Res FLAC Archive").unwrap();
        assert_eq!(preset_flac.format, StemExportFormat::Flac);
        assert_eq!(preset_flac.settings.sample_rate, 96000);

        // 3. Test custom preset creation and manager management
        let custom_preset = ExportPreset {
            name: "Custom Techno OGG Stems".to_string(),
            description: "Custom OGG stem preset with bus subfolders".to_string(),
            format: StemExportFormat::Ogg,
            settings: ExportSettings {
                bit_depth: BitDepth::Bit16,
                sample_rate: 48000,
                flac_compression_level: 5,
                ogg_quality: 0.9,
                normalize: true,
                target_db: -0.5,
                trim_silence: true,
                silence_threshold_db: -50.0,
            },
            naming_pattern: "{project}_{idx}_{name}".to_string(),
            include_master: true,
            group_by_bus: true,
        };

        manager.add_preset(custom_preset.clone());
        assert!(manager.get_preset("Custom Techno OGG Stems").is_some());

        // 4. Test saving and loading preset manager configuration JSON
        let temp_dir = std::env::temp_dir().join("summoner_preset_test_tier46");
        let json_path = temp_dir.join("export_presets.json");
        manager.save_to_json_file(&json_path).unwrap();

        let restored_manager = ExportPresetManager::load_from_json_file(&json_path).unwrap();
        assert!(restored_manager.get_preset("Custom Techno OGG Stems").is_some());

        // 5. Build mock project and test stem export execution
        let proj = ProjectConfig {
            version: "1.0".to_string(),
            name: "Hyperdrive".to_string(),
            transport: TransportConfig {
                bpm: 128.0,
                sample_rate: 48000,
                ..Default::default()
            },
            tracks: vec![
                TrackConfig {
                    id: 1,
                    name: "Kick Drum".to_string(),
                    bus_target: Some("Drums Bus".to_string()),
                    ..Default::default()
                },
                TrackConfig {
                    id: 2,
                    name: "Acid Line".to_string(),
                    bus_target: Some("Synth Bus".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let export_dir = temp_dir.join("stems_out");
        let report = restored_manager.export_stems(&custom_preset, &proj, &export_dir, None).unwrap();

        assert_eq!(report.total_stems, 3); // 2 tracks + 1 master
        assert_eq!(report.format, "ogg");
        assert_eq!(report.preset_used, "Custom Techno OGG Stems");

        let drums_folder = export_dir.join("Drums_Bus");
        let synth_folder = export_dir.join("Synth_Bus");
        assert!(drums_folder.join("Hyperdrive_01_Kick_Drum.ogg").exists());
        assert!(synth_folder.join("Hyperdrive_02_Acid_Line.ogg").exists());
        assert!(export_dir.join("Hyperdrive_03_Master_Mix.ogg").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_step_1250_audio_graph_benchmark_suite() {
        use summoner_project::benchmark::{
            AudioGraphBenchmarkConfig, AudioGraphBenchmarkSuite,
        };
        use summoner_project::create_default_project;

        let proj = create_default_project("Benchmark Test Session");
        let config = AudioGraphBenchmarkConfig {
            frames_to_process: 44100 * 2, // 2 seconds
            block_sizes: vec![64, 256, 1024],
            runs_per_block_size: 3,
            warmup_runs: 1,
            sample_rate: 44100,
            channels: 2,
        };

        // 1. Run benchmark directly on project config
        let report = AudioGraphBenchmarkSuite::run_benchmark_on_project(&proj, &config);

        assert_eq!(report.project_name, "Benchmark Test Session");
        assert_eq!(report.block_results.len(), 3);
        assert_eq!(report.sample_rate, 44100);
        assert_eq!(report.channels, 2);
        assert!(report.peak_realtime_factor > 0.0);
        assert!(report.peak_throughput_mb_s > 0.0);
        assert!(report.best_block_size > 0);

        // 2. Verify summary text table and JSON reporting output
        assert!(report.formatted_summary.contains("SUMMONER AUDIO GRAPH BUFFER PROCESSING THROUGHPUT BENCHMARK"));
        assert!(report.formatted_summary.contains("Block Size"));
        assert!(report.formatted_summary.contains("Speed Factor"));
        assert!(report.formatted_json.contains("best_block_size"));
        assert!(report.formatted_json.contains("peak_realtime_factor"));

        // 3. Test benchmark execution with custom closure runner
        let mut closure_calls = 0;
        let custom_report = AudioGraphBenchmarkSuite::run_benchmark_with_runner(
            "Custom Closure Graph",
            2,
            4,
            &config,
            |block_size, outputs| {
                closure_calls += 1;
                for ch in outputs {
                    for sample in ch.iter_mut().take(block_size) {
                        *sample = 0.42;
                    }
                }
            },
        );

        assert_eq!(custom_report.project_name, "Custom Closure Graph");
        assert_eq!(custom_report.block_results.len(), 3);
        assert!(closure_calls > 0);
        assert!(custom_report.block_results.iter().all(|r| r.checksum > 0.0));
    }

    #[test]
    fn test_step_1253_workspace_dependency_security_audit() {
        use summoner_project::dependency_audit::WorkspaceDependencyAuditor;

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = WorkspaceDependencyAuditor::audit_workspace_manifests(&root).expect("Workspace audit failed");

        assert_eq!(report.total_workspace_crates, 7);
        assert!(report.total_dependencies_audited > 0);
        assert_eq!(report.vulnerabilities_found, 0);
        assert_eq!(report.wildcard_dependencies_found, 0);
        assert_eq!(report.telemetry_dependencies_found, 0);
        assert_eq!(report.non_foss_licenses_found, 0);
        assert!(report.is_security_compliant);
        assert!(report.formatted_summary.contains("SUMMONER DAW - WORKSPACE DEPENDENCY SECURITY AUDIT"));
        assert!(report.formatted_summary.contains("Compliance Status          : VERIFIED PASS"));
    }
}



