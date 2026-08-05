// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 37 unit tests for Lua ecosystem, documentation generator, LSP integration, API docs,
//! marketplace, script sandboxing, analytics, import/export, ZIP backup, git script tracker,
//! script blame, conflict detection, script inspector, and default script resets (Steps 901-920).

#[cfg(test)]
mod tests {
    use summoner_project::media_export::{
        backup_lua_scripts_to_zip, detect_script_merge_conflicts,
        export_lua_api_reference_markdown, export_lua_script_file, generate_lua_docs,
        get_script_line_blame, import_lua_script_file, reset_device_default_script,
        LuaAutomationOnlyGuard, LuaGitScriptTracker, LuaLspServer, LuaScriptAnalytics,
        LuaScriptInspectorState, LuaScriptMarketplace, LuaScriptSandboxMode, LuaTestRunner,
        MarketplaceScriptEntry, ScriptExecutionLog,
    };

    #[test]
    fn test_step_901_lua_test_runner_panel_display() {
        let runner = LuaTestRunner;
        let res = runner.test_block("unit_test_1", "function process() return 1 end");
        assert!(res.passed);
        assert_eq!(res.test_name, "unit_test_1");
    }

    #[test]
    fn test_step_903_generate_lua_docs() {
        let script = "---@param freq number Frequency in Hz\n---@return number Processed sample\nfunction process(freq)\n  return 0.5\nend";
        let docs = generate_lua_docs(script);
        assert!(docs.contains("## `process(freq)`"));
        assert!(docs.contains("- **freq** (number): Frequency in Hz"));
        assert!(docs.contains("- Returns (number): Processed sample"));
    }

    #[test]
    fn test_step_904_lua_lsp_server_integration() {
        let lsp = LuaLspServer;
        let init_resp = lsp.handle_lsp_request(r#"{"method":"initialize"}"#);
        assert!(init_resp.contains("hoverProvider"));

        let comp_resp = lsp.handle_lsp_request(r#"{"method":"textDocument/completion"}"#);
        assert!(comp_resp.contains("read_input"));
    }

    #[test]
    fn test_step_905_export_lua_api_reference_markdown() {
        let api_ref = export_lua_api_reference_markdown();
        assert!(api_ref.contains("# Summoner Lua API Reference"));
        assert!(api_ref.contains("midi_to_hz"));
        assert!(api_ref.contains("euclidean"));
    }

    #[test]
    fn test_steps_906_909_lua_script_marketplace() {
        let mut mp = LuaScriptMarketplace::new_with_defaults();
        assert_eq!(mp.entries.len(), 1);

        let forked = mp
            .fork_script("community-euclidean-1", "user_alice")
            .unwrap();
        assert_eq!(forked.author, "user_alice");
        assert!(forked.name.contains("(Fork)"));

        let new_entry = MarketplaceScriptEntry {
            id: "custom-script-1".to_string(),
            name: "Custom LFO".to_string(),
            author: "user_bob".to_string(),
            category: "DSP".to_string(),
            description: "Sine wave LFO".to_string(),
            version: "1.0".to_string(),
            script_code: "function process() return 0 end".to_string(),
            rating: 5.0,
            downloads: 10,
            comments: vec!["Awesome".to_string()],
        };
        let pub_id = mp.publish_script(new_entry).unwrap();
        assert_eq!(pub_id, "custom-script-1");
        assert_eq!(mp.entries.len(), 2);
    }

    #[test]
    fn test_step_910_script_sandbox_modes() {
        let full = LuaScriptSandboxMode::FullAccess;
        assert!(full.allows_project_access());
        assert!(full.allows_dsp());

        let strict = LuaScriptSandboxMode::StrictSandbox;
        assert!(!strict.allows_project_access());
        assert!(!strict.allows_dsp());
    }

    #[test]
    fn test_step_911_script_analytics() {
        let mut analytics = LuaScriptAnalytics::new(true);
        analytics.record_execution("lfo.lua", 1.25);
        analytics.record_execution("lfo.lua", 0.75);
        assert_eq!(*analytics.execution_counts.get("lfo.lua").unwrap(), 2);
        assert_eq!(*analytics.total_exec_time_ms.get("lfo.lua").unwrap(), 2.0);
    }

    #[test]
    fn test_step_912_automation_only_guard() {
        let guard = LuaAutomationOnlyGuard {
            automation_only: true,
        };
        assert!(guard.allow_execution("automation"));
        assert!(!guard.allow_execution("dsp"));
        assert!(!guard.allow_execution("ui"));
    }

    #[test]
    fn test_steps_913_915_script_import_export_zip() {
        let temp_dir = std::env::temp_dir();
        let script_file = temp_dir.join("test_export_script.lua");
        let script_content = "function process() return 0.5 end";

        export_lua_script_file(script_content, &script_file).unwrap();
        let imported = import_lua_script_file(&script_file).unwrap();
        assert_eq!(imported, script_content);

        let zip_file = temp_dir.join("test_scripts_backup.zip");
        let count = backup_lua_scripts_to_zip(&[("test.lua", script_content)], &zip_file).unwrap();
        assert_eq!(count, 1);
        assert!(zip_file.exists());

        let _ = std::fs::remove_file(script_file);
        let _ = std::fs::remove_file(zip_file);
    }

    #[test]
    fn test_step_916_git_script_tracker() {
        let mut tracker = LuaGitScriptTracker::default();
        tracker.track_script_commit("lfo.lua", "content v1", "commit123");
        let history = tracker.script_commits.get("lfo.lua").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].0, "commit123");
    }

    #[test]
    fn test_step_917_script_line_blame() {
        let logs = vec![ScriptExecutionLog {
            script_name: "macro.lua".to_string(),
            line_number: 14,
            param_id: "cutoff".to_string(),
            timestamp_ms: 1000,
            previous_value: 400.0,
            new_value: 800.0,
        }];
        let blame = get_script_line_blame("cutoff", &logs).unwrap();
        assert_eq!(blame.script_name, "macro.lua");
        assert_eq!(blame.line_number, 14);
        assert_eq!(blame.new_value, 800.0);
    }

    #[test]
    fn test_step_918_script_merge_conflicts() {
        let base = "line1\nline2\nline3";
        let ours = "line1\nline2_ours\nline3";
        let theirs = "line1\nline2_theirs\nline3";
        let conflicts = detect_script_merge_conflicts(base, ours, theirs);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].line_number, 2);
    }

    #[test]
    fn test_step_919_reset_device_default_script() {
        let default_synth = reset_device_default_script("AetherSynth");
        assert!(default_synth.contains("AetherSynth Default Lua Controller"));

        let default_macro = reset_device_default_script("MacroRackLuaDevice");
        assert!(default_macro.contains("Macro Rack Default Lua Script"));
    }

    #[test]
    fn test_step_920_lua_script_inspector_state() {
        let mut inspector = LuaScriptInspectorState::default();
        inspector.update_variable("cutoff_freq", "800.0 Hz", 128);
        assert_eq!(
            inspector.variable_values.get("cutoff_freq").unwrap(),
            "800.0 Hz"
        );
        assert_eq!(inspector.last_updated_frame, 128);
    }
}
