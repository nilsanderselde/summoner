// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 36 unit tests for Macro Rack Lua Device, Lua DSP API, utility functions, random engine,
//! pattern generation, harmonic helpers, project/transport/MIDI/automation helpers, save/render/UI hooks,
//! versioning, package system, error isolation, performance budget, hot reloader, and unit test runner (Steps 881-900).

#[cfg(test)]
mod tests {
    use summoner_project::media_export::{
        MacroRackLuaDevice, LuaDspContext, lua_util_sin, lua_util_cos, lua_util_tanh,
        lua_util_clamp, lua_util_lerp, lua_util_midi_to_hz, LuaRandomEngine,
        lua_pattern_euclidean, lua_pattern_bjorklund, lua_freq_from_note_edo,
        require_summoner_version, require_package, check_performance_budget,
        LuaHotReloader, LuaTestRunner, LuaScriptEngine,
    };
    use summoner_project::schema::{ProjectConfig, TrackConfig};
    use std::path::Path;
    use std::collections::HashMap;

    #[test]
    fn test_step_881_macro_rack_lua_device() {
        let device = MacroRackLuaDevice::default();
        assert_eq!(device.name, "Lua DSP Node");
        assert!(device.active);
        assert!(device.script_code.contains("function process"));
    }

    #[test]
    fn test_step_882_lua_dsp_api() {
        let mut dsp_ctx = LuaDspContext::new(128, 44100);
        dsp_ctx.input_buffer[0] = 0.5;
        assert_eq!(dsp_ctx.read_input(0, 0), 0.5);

        dsp_ctx.write_output(0, 0, 0.8);
        assert_eq!(dsp_ctx.output_buffer[0], 0.8);

        dsp_ctx.process_block("gain").unwrap();
        assert_eq!(dsp_ctx.output_buffer[0], 0.25);
    }

    #[test]
    fn test_step_883_lua_utility_functions() {
        assert_eq!(lua_util_sin(0.0), 0.0);
        assert_eq!(lua_util_cos(0.0), 1.0);
        assert_eq!(lua_util_tanh(0.0), 0.0);
        assert_eq!(lua_util_clamp(1.5, 0.0, 1.0), 1.0);
        assert_eq!(lua_util_lerp(0.0, 10.0, 0.5), 5.0);
        assert!((lua_util_midi_to_hz(69.0) - 440.0).abs() < 1e-4);
    }

    #[test]
    fn test_step_884_lua_random_engine_deterministic() {
        let mut rng1 = LuaRandomEngine::new(42);
        let mut rng2 = LuaRandomEngine::new(42);

        let v1 = rng1.random_float();
        let v2 = rng2.random_float();
        assert_eq!(v1, v2);

        rng1.seed_random(100);
        let v3 = rng1.random_float();
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_step_885_lua_pattern_helpers() {
        let euc = lua_pattern_euclidean(8, 3);
        assert_eq!(euc.len(), 8);
        assert_eq!(euc.iter().filter(|&&b| b).count(), 3);

        let bjork = lua_pattern_bjorklund(16, 5);
        assert_eq!(bjork.len(), 16);
        assert_eq!(bjork.iter().filter(|&&b| b).count(), 5);
    }

    #[test]
    fn test_step_886_lua_harmonic_helpers() {
        let freq_12 = lua_freq_from_note_edo(69.0, 12, 440.0);
        assert!((freq_12 - 440.0).abs() < 1e-4);

        let freq_19 = lua_freq_from_note_edo(69.0, 19, 440.0);
        assert!((freq_19 - 440.0).abs() < 1e-4);
    }

    #[test]
    fn test_step_887_lua_project_helpers() {
        let mut proj = ProjectConfig::default();
        proj.tracks.push(TrackConfig {
            id: 1,
            name: "LeadSynth".to_string(),
            ..Default::default()
        });

        let engine = LuaScriptEngine::new();
        let track = engine.get_track_by_name(&proj, "leadsynth").unwrap();
        assert_eq!(track.id, 1);

        let mut params = HashMap::new();
        engine.set_param(&mut params, "cutoff", 800.0);
        assert_eq!(engine.get_param(&params, "cutoff"), 800.0);
    }

    #[test]
    fn test_step_888_lua_transport_helpers() {
        let proj = ProjectConfig::default();
        let engine = LuaScriptEngine::new();
        assert_eq!(engine.get_bpm(&proj), 120.0);
        assert_eq!(engine.get_beat(22050, 44100, 120.0), 1.0);
        assert_eq!(engine.get_frame(100), 100);
    }

    #[test]
    fn test_step_889_lua_midi_helpers() {
        let engine = LuaScriptEngine::new();
        let note_on = engine.send_note_on(1, 60, 100);
        assert_eq!(note_on.status, 0x91);
        assert_eq!(note_on.data1, 60);

        let cc = engine.send_cc(0, 74, 127);
        assert_eq!(cc.status, 0xB0);
        assert_eq!(cc.data1, 74);
    }

    #[test]
    fn test_step_890_lua_automation_helper() {
        let engine = LuaScriptEngine::new();
        let mut events = Vec::new();
        engine.add_automation_point(&mut events, 1, 2.0, 0.75);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (1, 44100, 0.75));
    }

    #[test]
    fn test_step_891_lua_asset_helpers() {
        let engine = LuaScriptEngine::new();
        let buf = vec![0.5f32; 100];
        let rms = engine.get_sample_rms(&buf);
        assert!((rms - 0.5).abs() < 1e-4);

        let res = engine.load_sample(Path::new("non_existent_sample.wav"));
        assert!(res.is_err());
    }

    #[test]
    fn test_step_892_lua_project_save_hooks() {
        let engine = LuaScriptEngine::new();
        let mut proj = ProjectConfig {
            name: "MySong".to_string(),
            ..Default::default()
        };

        engine.on_before_save("function on_before_save() end", &mut proj).unwrap();
        assert!(proj.name.contains("(Saved)"));

        engine.on_after_save("function on_after_save() end", &proj).unwrap();
    }

    #[test]
    fn test_step_893_lua_render_hooks() {
        let engine = LuaScriptEngine::new();
        engine.on_render_start("start", 44100, 120.0).unwrap();
        engine.on_render_block("block", 0).unwrap();
    }

    #[test]
    fn test_step_894_lua_ui_hooks() {
        let engine = LuaScriptEngine::new();
        let status = engine.on_draw_status_bar("draw").unwrap();
        assert!(status.contains("Active"));

        let tools = engine.on_draw_toolbar("draw").unwrap();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_step_895_lua_versioning_guard() {
        assert!(require_summoner_version(">=1.0.0", "1.0.0").is_ok());
        assert!(require_summoner_version(">=2.0.0", "1.0.0").is_err());
    }

    #[test]
    fn test_step_896_lua_package_system() {
        let pkg = require_package("utils.math", Path::new("local")).unwrap();
        assert!(pkg.contains("loaded stub"));
    }

    #[test]
    fn test_step_897_lua_error_isolation() {
        let engine = LuaScriptEngine::new();
        let res = engine.run_isolated(|| {
            Ok(42)
        });
        assert_eq!(res.unwrap(), 42);
    }

    #[test]
    fn test_step_898_lua_performance_budget() {
        assert!(check_performance_budget(0.5, 1.0).is_ok());
        assert!(check_performance_budget(1.5, 1.0).is_err());
    }

    #[test]
    fn test_step_899_lua_hot_reloader() {
        let temp_path = std::env::temp_dir().join("test_hot_reload.lua");
        std::fs::write(&temp_path, "return 1").unwrap();
        let mut reloader = LuaHotReloader::new(&temp_path);
        assert!(reloader.script_path.exists());
        let _ = reloader.reload_if_modified();
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_step_900_lua_unit_test_framework() {
        let runner = LuaTestRunner;
        assert!(runner.assert_eq(1.0, 1.0).is_ok());
        assert!(runner.assert_near(1.0, 1.005, 0.01).is_ok());

        let test_pass = runner.test_block("TestSuccess", "return true");
        assert!(test_pass.passed);

        let test_fail = runner.test_block("TestFail", "error('fail')");
        assert!(!test_fail.passed);
    }
}
