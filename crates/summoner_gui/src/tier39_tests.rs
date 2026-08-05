// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 39 unit tests for Lua ecosystem extensions, network/OSC/MIDI/OS integration, and DSP audio analysis APIs (Steps 941-960).

#[cfg(test)]
mod tests {
    use summoner_project::media_export::{
        read_json, write_json, lua_http_get, lua_http_post, lua_osc_send, LuaOscServer,
        LuaMidiInputSubscriber, lua_midi_out_send, LuaMidiMessage, lua_spawn_process,
        LuaClipboard, lua_env_get, LuaFileWatcher, lua_fft, lua_ifft, lua_autocorrelate,
        lua_spectral_centroid, lua_rms, lua_find_peaks, lua_detect_onsets, lua_detect_pitch,
        lua_detect_tempo,
    };
    use std::path::Path;

    #[test]
    fn test_step_941_read_json() {
        let json_str = r#"{"name":"Summoner","version":"1.0.0"}"#;
        let map = read_json(json_str).unwrap();
        assert_eq!(map.get("name").unwrap(), "Summoner");
        assert_eq!(map.get("version").unwrap(), "1.0.0");
    }

    #[test]
    fn test_step_942_write_json() {
        let mut map = std::collections::HashMap::new();
        map.insert("engine".to_string(), "audio".to_string());
        let json = write_json(&map).unwrap();
        assert!(json.contains("\"engine\":\"audio\""));
    }

    #[test]
    fn test_step_943_lua_http_client() {
        assert!(lua_http_get("https://example.com/api", false).is_err());
        let res = lua_http_get("https://example.com/api", true).unwrap();
        assert!(res.contains("status\":200"));

        let post_res = lua_http_post("https://example.com/api", "data", true).unwrap();
        assert!(post_res.contains("status\":201"));
    }

    #[test]
    fn test_step_944_lua_osc_send() {
        let packet = lua_osc_send("127.0.0.1", 8000, "/synth/cutoff", &[440.0, 0.5]).unwrap();
        assert!(packet.starts_with(b"/synth/cutoff"));
    }

    #[test]
    fn test_step_945_lua_osc_listen() {
        let mut server = LuaOscServer::default();
        let res = server.osc_listen(8000, "on_osc_message").unwrap();
        assert_eq!(server.port, 8000);
        assert_eq!(server.active_callback.as_deref(), Some("on_osc_message"));
        assert!(res.contains("8000"));
    }

    #[test]
    fn test_step_946_lua_midi_in_subscribe() {
        let mut sub = LuaMidiInputSubscriber::default();
        let res = sub.midi_in_subscribe(1, "on_midi_event").unwrap();
        assert_eq!(sub.port, 1);
        assert_eq!(sub.callback.as_deref(), Some("on_midi_event"));
        assert!(res.contains("port 1"));
    }

    #[test]
    fn test_step_947_lua_midi_out_send() {
        let msg = LuaMidiMessage { channel: 0, status: 0x90, data1: 60, data2: 100 };
        assert!(lua_midi_out_send(0, &msg).is_ok());

        let invalid_msg = LuaMidiMessage { channel: 16, status: 0x90, data1: 60, data2: 100 };
        assert!(lua_midi_out_send(0, &invalid_msg).is_err());
    }

    #[test]
    fn test_step_948_lua_spawn_process() {
        assert!(lua_spawn_process("echo", &["hello"], false).is_err());
        let res = lua_spawn_process("echo", &["hello"], true).unwrap();
        assert!(res.contains("executed"));

        assert!(lua_spawn_process("rm", &["-rf"], true).is_err());
    }

    #[test]
    fn test_step_949_lua_clipboard() {
        let mut cb = LuaClipboard::default();
        cb.clipboard_set("Summoner Audio Clip").unwrap();
        assert_eq!(cb.clipboard_get().unwrap(), "Summoner Audio Clip");
    }

    #[test]
    fn test_step_950_lua_env_get() {
        assert!(lua_env_get("SECRET_KEY").is_none());
        let path_var = lua_env_get("PATH");
        assert!(path_var.is_some());
    }

    #[test]
    fn test_step_951_lua_watch_file() {
        let mut watcher = LuaFileWatcher::default();
        let p = Path::new("script.lua");
        let res = watcher.watch_file(p, "on_script_reload").unwrap();
        assert_eq!(watcher.watched_file.as_deref(), Some(p));
        assert!(res.contains("script.lua"));
    }

    #[test]
    fn test_step_952_lua_fft() {
        let samples = vec![0.0f32, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0];
        let (mags, phases) = lua_fft(&samples, "hann");
        assert!(!mags.is_empty());
        assert_eq!(mags.len(), phases.len());
    }

    #[test]
    fn test_step_953_lua_ifft() {
        let mags = vec![0.0f32, 1.0, 0.5, 0.0, 0.0];
        let phases = vec![0.0f32; 5];
        let recon = lua_ifft(&mags, &phases);
        assert_eq!(recon.len(), 8);
    }

    #[test]
    fn test_step_954_lua_autocorrelate() {
        let samples = vec![1.0f32, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0];
        let autocorr = lua_autocorrelate(&samples);
        assert_eq!(autocorr[0], 1.0);
        assert!(autocorr.len() == samples.len());
    }

    #[test]
    fn test_step_955_lua_spectral_centroid() {
        let mags = vec![0.0f32, 1.0, 0.5, 0.2];
        let centroid = lua_spectral_centroid(&mags, 44100);
        assert!(centroid > 0.0);
    }

    #[test]
    fn test_step_956_lua_rms() {
        let samples = vec![0.707f32, -0.707f32, 0.707f32, -0.707f32];
        let rms_val = lua_rms(&samples);
        assert!((rms_val - 0.707).abs() < 1e-2);
    }

    #[test]
    fn test_step_957_lua_find_peaks() {
        let samples = vec![0.1f32, 0.5, 0.9, 0.4, 0.2, 0.8, 0.1];
        let peaks = lua_find_peaks(&samples, 0.5);
        assert_eq!(peaks, vec![2, 5]);
    }

    #[test]
    fn test_step_958_lua_detect_onsets() {
        let mut samples = vec![0.001f32; 4410];
        samples[1000..1050].fill(0.8);
        let onsets = lua_detect_onsets(&samples, 44100);
        assert!(!onsets.is_empty());
    }

    #[test]
    fn test_step_959_lua_detect_pitch() {
        let sr = 44100;
        let freq = 440.0;
        let mut samples = Vec::new();
        for i in 0..2048 {
            let t = i as f32 / sr as f32;
            samples.push((2.0 * std::f32::consts::PI * freq * t).sin());
        }
        let pitch = lua_detect_pitch(&samples, sr);
        assert!(pitch.is_some());
        assert!((pitch.unwrap() - 440.0).abs() < 20.0);
    }

    #[test]
    fn test_step_960_lua_detect_tempo() {
        let onsets = vec![0, 22050, 44100, 66150]; // Every 0.5s = 120 BPM
        let bpm = lua_detect_tempo(&onsets, 44100);
        assert!((bpm - 120.0).abs() < 1.0);
    }
}
