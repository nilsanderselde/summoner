// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 38 unit tests for advanced Lua scripting features, standard libraries, UI/Graphics APIs, & file I/O (Steps 921-940).

#[cfg(test)]
mod tests {
    use summoner_project::media_export::{
        lua_animate, lua_bit_ops, lua_color_api, lua_math_lib, lua_string_lib, lua_table_lib,
        read_midi_file, read_toml, read_wav, write_toml, write_wav, LuaCoroutinePattern,
        LuaDrawCommand, LuaDspObjectMetatable, LuaEventSystem, LuaPainterBuffer, LuaScriptEngine,
        LuaScriptErrorRecovery, LuaScriptSafeMode, LuaTimer, LuaUiLayout, LuaUiWidget,
    };

    #[test]
    fn test_step_921_script_error_recovery() {
        let mut recovery = LuaScriptErrorRecovery::default();
        let engine = LuaScriptEngine::new();

        let ok = recovery.update_script("sin(t)", &engine);
        assert!(ok);
        assert_eq!(recovery.last_valid_script, "sin(t)");
        assert!(!recovery.has_error);

        let err = recovery.update_script("invalid_script_error()", &engine);
        assert!(!err);
        assert!(recovery.has_error);
        assert_eq!(recovery.current_script, "sin(t)");
    }

    #[test]
    fn test_step_922_script_safe_mode() {
        let safe = LuaScriptSafeMode::BuiltinOnly;
        assert!(safe.validate_script("return sin(t)").is_ok());
        assert!(safe.validate_script("os.execute('rm -rf')").is_err());
        assert!(safe.validate_script("io.open('file.txt')").is_err());
    }

    #[test]
    fn test_step_923_lua_string_functions() {
        assert_eq!(lua_string_lib::format("Hello %s", "World"), "Hello World");
        assert_eq!(lua_string_lib::upper("summoner"), "SUMMONER");
        assert_eq!(lua_string_lib::lower("SUMMONER"), "summoner");
        assert_eq!(lua_string_lib::sub("Summoner", 1, 3), "Sum");
        assert_eq!(lua_string_lib::find("Summoner", "mon"), Some(4));
    }

    #[test]
    fn test_step_924_lua_table_functions() {
        let mut tbl = vec!["a".to_string(), "c".to_string()];
        lua_table_lib::insert(&mut tbl, 2, "b".to_string());
        assert_eq!(tbl, vec!["a", "b", "c"]);

        let removed = lua_table_lib::remove(&mut tbl, 2);
        assert_eq!(removed, Some("b".to_string()));

        let mut unsorted = vec!["z".to_string(), "a".to_string(), "m".to_string()];
        lua_table_lib::sort(&mut unsorted);
        assert_eq!(unsorted, vec!["a", "m", "z"]);

        assert_eq!(lua_table_lib::concat(&unsorted, "-"), "a-m-z");
    }

    #[test]
    fn test_step_925_lua_math_functions() {
        assert_eq!(lua_math_lib::min(3.0, 5.0), 3.0);
        assert_eq!(lua_math_lib::max(3.0, 5.0), 5.0);
        assert_eq!(lua_math_lib::abs(-4.2), 4.2);
        assert_eq!(lua_math_lib::floor(4.9), 4.0);
        assert_eq!(lua_math_lib::ceil(4.1), 5.0);
        assert_eq!(lua_math_lib::fmod(5.5, 2.0), 1.5);
    }

    #[test]
    fn test_step_926_lua_bit_ops() {
        assert_eq!(lua_bit_ops::band(0b1100, 0b1010), 0b1000);
        assert_eq!(lua_bit_ops::bor(0b1100, 0b1010), 0b1110);
        assert_eq!(lua_bit_ops::bxor(0b1100, 0b1010), 0b0110);
        assert_eq!(lua_bit_ops::lshift(1, 4), 16);
        assert_eq!(lua_bit_ops::rshift(16, 4), 1);
    }

    #[test]
    fn test_step_927_lua_coroutine_pattern() {
        let mut coro = LuaCoroutinePattern::new(vec![60, 64, 67]);
        assert_eq!(coro.resume(), Some(60));
        assert_eq!(coro.resume(), Some(64));
        assert_eq!(coro.resume(), Some(67));
        assert_eq!(coro.resume(), None);
    }

    #[test]
    fn test_step_928_lua_metatable_dsp_access() {
        let mut dsp = LuaDspObjectMetatable::new("Oscillator");
        dsp.set_property("freq", 440.0);
        assert_eq!(dsp.get_property("freq"), Some(440.0));
        assert_eq!(dsp.get_property("unknown"), None);
    }

    #[test]
    fn test_step_929_lua_event_system() {
        let mut events = LuaEventSystem::default();
        events.subscribe("note_on", "on_note_on");
        let dispatches = events.dispatch("note_on", "60");
        assert_eq!(dispatches, vec!["on_note_on(60)"]);
    }

    #[test]
    fn test_step_930_lua_timer() {
        let mut timer = LuaTimer::default();
        timer.schedule(2.0, "tick_callback");
        assert!(timer.tick(1.0).is_empty());
        let triggered = timer.tick(1.5);
        assert_eq!(triggered, vec!["tick_callback"]);
    }

    #[test]
    fn test_step_931_lua_ui_widget_creation() {
        let slider = LuaUiWidget::create_slider("Cutoff", 20.0, 20000.0);
        assert_eq!(slider.kind, "slider");
        assert_eq!(slider.label, "Cutoff");

        let btn = LuaUiWidget::create_button("Trigger");
        assert_eq!(btn.kind, "button");
    }

    #[test]
    fn test_step_932_lua_ui_layout_helpers() {
        let w1 = LuaUiWidget::create_button("B1");
        let w2 = LuaUiWidget::create_button("B2");
        let horiz = LuaUiLayout::horizontal(vec![w1, w2]);
        assert_eq!(horiz.direction, "horizontal");
        assert_eq!(horiz.children.len(), 2);
    }

    #[test]
    fn test_step_933_lua_color_api() {
        let color_rgb = lua_color_api::rgb(255, 128, 0);
        assert_eq!(color_rgb, (255, 128, 0, 255));

        let color_hsv = lua_color_api::hsv(0.0, 1.0, 1.0);
        assert_eq!(color_hsv, (255, 0, 0, 255));
    }

    #[test]
    fn test_step_934_lua_painter_api() {
        let mut painter = LuaPainterBuffer::default();
        painter.draw_line(0.0, 0.0, 10.0, 10.0);
        painter.draw_circle(5.0, 5.0, 2.0);
        painter.draw_rect(1.0, 1.0, 8.0, 8.0);
        painter.draw_text(2.0, 2.0, "Wave");

        assert_eq!(painter.commands.len(), 4);
        match &painter.commands[0] {
            LuaDrawCommand::Line { x1, .. } => assert_eq!(*x1, 0.0),
            _ => panic!("Expected Line command"),
        }
    }

    #[test]
    fn test_step_935_lua_animation_api() {
        let val_lin = lua_animate(0.0, 100.0, 0.5, "linear");
        assert_eq!(val_lin, 50.0);

        let val_ease_in = lua_animate(0.0, 100.0, 0.5, "ease_in");
        assert_eq!(val_ease_in, 25.0);
    }

    #[test]
    fn test_steps_936_938_lua_file_io_midi_wav() {
        let temp_dir = std::env::temp_dir();
        let midi_file = temp_dir.join("test_lua_track.mid");
        std::fs::write(&midi_file, b"MThd dummy content").unwrap();

        let midi_events = read_midi_file(&midi_file).unwrap();
        assert!(!midi_events.is_empty());

        let wav_file = temp_dir.join("test_lua_render.wav");
        let samples = vec![0.0f32, 0.5, -0.5, 0.0];
        write_wav(&wav_file, &samples, 44100).unwrap();
        let read_back = read_wav(&wav_file).unwrap();
        assert_eq!(read_back.len(), 4);
        assert!((read_back[1] - 0.5).abs() < 1e-3);

        let _ = std::fs::remove_file(midi_file);
        let _ = std::fs::remove_file(wav_file);
    }

    #[test]
    fn test_steps_939_940_lua_toml_reading_writing() {
        let temp_dir = std::env::temp_dir();
        let toml_file = temp_dir.join("test_lua_config.toml");

        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), "\"Summoner\"".to_string());
        map.insert("bpm".to_string(), "120".to_string());

        write_toml(&toml_file, &map).unwrap();
        let read_map = read_toml(&toml_file).unwrap();
        assert_eq!(read_map.get("name").unwrap(), "\"Summoner\"");
        assert_eq!(read_map.get("bpm").unwrap(), "120");

        let _ = std::fs::remove_file(toml_file);
    }
}
