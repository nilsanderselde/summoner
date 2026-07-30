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
        assert!(summary.recommendations.len() > 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

