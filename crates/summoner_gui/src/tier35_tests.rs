// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 35 unit tests for Advanced Lua Scripting, Sandboxing, Debugger, Profiler, REPL,
//! CLI Integration, and Scripted Automation Tools (Steps 861-880).

#[cfg(test)]
mod tests {
    use summoner_project::media_export::{
        LuaScriptEngine, LuaDebugger, LuaProfiler,
    };
    use summoner_project::schema::{ProjectConfig, LuaScriptConfig};
    use crate::gpu_waveform::LuaEditorState;
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn test_step_861_status_bar_error_display() {
        let mut editor = LuaEditorState::new();
        editor.script_code = "invalid error code".to_string();
        // Trigger error
        let _ = editor.test_run_script();
        assert!(editor.status_bar_error.is_some());
    }

    #[test]
    fn test_step_862_community_scripts_list() {
        let scripts = LuaScriptEngine::list_community_scripts();
        assert!(!scripts.is_empty());
        assert_eq!(scripts[0].author, "Community");
    }

    #[test]
    fn test_step_863_test_script_button_execution() {
        let mut editor = LuaEditorState::new();
        let res = editor.test_run_script().unwrap();
        assert!(res > 0.0);
        assert!(editor.status_msg.contains("successful"));
    }

    #[test]
    fn test_steps_864_865_bind_script_to_cc_and_lane() {
        let mut editor = LuaEditorState::new();
        editor.bind_to_cc(74);
        assert_eq!(editor.bound_cc, Some(74));

        editor.bind_to_lane("filter_cutoff");
        assert_eq!(editor.bound_lane, Some("filter_cutoff".to_string()));
    }

    #[test]
    fn test_step_866_api_docs_access() {
        let docs = LuaEditorState::get_api_documentation();
        assert!(docs.contains("Summoner DAW Lua API"));
        assert!(docs.contains("curve"));
    }

    #[test]
    fn test_steps_867_868_eval_and_list_scripts() {
        let proj = ProjectConfig::default();
        let engine = LuaScriptEngine::new();
        let res = engine.eval_script("return 42", &proj).unwrap();
        assert!(res.contains("successfully"));
    }

    #[test]
    fn test_step_869_persistent_lua_state() {
        let mut proj = ProjectConfig::default();
        proj.scripts.push(LuaScriptConfig {
            name: "MyScript".to_string(),
            script: "return 1".to_string(),
            bound_cc: Some(10),
            bound_lane: Some("pan".to_string()),
            sandbox_fs: false,
        });
        proj.lua_state = Some("{\"var\": 100}".to_string());

        let serialized = summoner_project::serialize_project_toml(&proj).unwrap();
        let deserialized = summoner_project::parse_project_toml(&serialized).unwrap();

        assert_eq!(deserialized.scripts.len(), 1);
        assert_eq!(deserialized.scripts[0].name, "MyScript");
        assert_eq!(deserialized.lua_state.as_deref(), Some("{\"var\": 100}"));
    }

    #[test]
    fn test_step_870_lua_repl_panel() {
        let mut editor = LuaEditorState::new();
        let output = editor.run_repl_input("sin(0.5)");
        assert!(output.contains("Result"));
        assert!(editor.repl_history.len() >= 2);
    }

    #[test]
    fn test_step_871_scripted_clip_generation() {
        let engine = LuaScriptEngine::new();
        let steps = engine.generate_clip_script("generate_steps()").unwrap();
        assert_eq!(steps.len(), 4);
        assert!(steps[0].active);
    }

    #[test]
    fn test_step_872_scripted_node_param_mutation() {
        let engine = LuaScriptEngine::new();
        let mut params = HashMap::new();
        params.insert("cutoff".to_string(), 0.5f32);
        engine.mutate_params_script("* 2", &mut params).unwrap();
        assert_eq!(params["cutoff"], 1.0f32);
    }

    #[test]
    fn test_step_873_scripted_automation_generation() {
        let engine = LuaScriptEngine::new();
        let points = engine.generate_automation_script("sine_wave()", 4.0).unwrap();
        assert_eq!(points.len(), 9);
        assert_eq!(points[0].0, 0.0);
    }

    #[test]
    fn test_step_874_scripted_rendering_pipeline() {
        let engine = LuaScriptEngine::new();
        let mut proj = ProjectConfig::default();
        let msg = engine.control_render_pipeline("set_bpm", &mut proj).unwrap();
        assert_eq!(proj.transport.bpm, 140.0);
        assert!(msg.contains("controlled"));
    }

    #[test]
    fn test_step_875_scripted_export_pipeline_postprocess() {
        let engine = LuaScriptEngine::new();
        let mut samples = vec![0.0, 0.5, 2.0, -1.0];
        engine.post_process_render("normalize", &mut samples).unwrap();
        assert_eq!(samples[2], 1.0);
    }

    #[test]
    fn test_step_876_scripted_ui_panels() {
        let editor = LuaEditorState::new();
        let widgets = editor.render_scripted_panel_widgets();
        assert!(!widgets.is_empty());
        assert!(widgets[0].contains("Slider"));
    }

    #[test]
    fn test_steps_877_878_secure_sandboxing() {
        let engine = LuaScriptEngine::new();
        let unsafe_script = "io.open('/etc/passwd')";
        assert!(engine.check_sandboxing(unsafe_script, false, None).is_err());
        assert!(engine.check_sandboxing(unsafe_script, true, Some(Path::new("/tmp/project"))).is_err());
        assert!(engine.check_sandboxing("print('hello')", false, None).is_ok());
    }

    #[test]
    fn test_step_879_lua_debugger() {
        let mut dbg = LuaDebugger::new();
        dbg.add_breakpoint(10);
        dbg.step_next();
        dbg.set_var("x", "42");

        assert_eq!(dbg.breakpoints, vec![10]);
        assert_eq!(dbg.current_step, 1);
        assert_eq!(dbg.variables["x"], "42");
    }

    #[test]
    fn test_step_880_lua_profiler() {
        let mut profiler = LuaProfiler::new();
        let script = "local a = 1\nlocal b = 2\nreturn a + b";
        let line_times = profiler.profile_script(script);

        assert_eq!(line_times.len(), 3);
        assert!(line_times[&1] > 0.0);
    }
}
