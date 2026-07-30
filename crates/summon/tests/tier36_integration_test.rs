// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 36 Integration Tests: Hardware Controller Message Encoding, Plugin Hosting, Wasm Sandboxing,
//! CV/Gate, DIN Sync, OSC Mapping, and BLE-MIDI Parsing (Steps 1041-1060).

use summoner_dsp::ecosystem_hardware::*;
use summoner_dsp::plugin_host::PluginFormat;

#[test]
fn test_tier36_vst3_custom_gui_embedding() {
    let mut embedder = Vst3WindowEmbedder::new("Vst3PluginWindow");
    assert!(!embedder.is_embedded);

    let res = embedder.embed_window(0xDEADBEEF, 1920, 1080);
    assert!(res.is_ok());
    assert!(embedder.is_embedded);
    assert_eq!(embedder.width, 1920);
    assert_eq!(embedder.height, 1080);

    embedder.resize_window(1280, 720);
    assert_eq!(embedder.width, 1280);

    embedder.detach_window();
    assert!(!embedder.is_embedded);
    assert_eq!(embedder.window_handle, 0);
}

#[test]
fn test_tier36_clap_host_mpe_and_sample_accurate_automation() {
    let mut host = ClapHostEngine::new("AdvancedClapSynth");
    let mut proc_buf = ClapProcessBuffer::new(128, 2);

    proc_buf.add_mpe_event(ClapMpeEvent {
        note_id: 42,
        channel: 1,
        key: 64,
        pitch_bend: 3.5,
        pressure: 0.9,
        timbre: 0.7,
    });

    proc_buf.add_automation(ClapSampleAccurateAutomation {
        param_id: 0,
        sample_offset: 32,
        value: 0.75,
    });

    host.process_block(&mut proc_buf);
    assert_eq!(host.parameters.get(&0), Some(&0.75));
    assert_eq!(host.mpe_handler.len(), 1);
    assert_eq!(host.mpe_handler[0].pitch_bend, 3.5);
}

#[test]
fn test_tier36_push_controller_encoding() {
    let mut push = PushControllerDriver::new();
    push.set_pad_rgb(3, 4, 128, 255, 64);
    assert_eq!(push.pad_rgb[3][4], (128, 255, 64));

    let val = push.handle_encoder_turn(2, 10).unwrap();
    assert!((val - 0.10).abs() < 1e-5);

    let header = push.render_display_header();
    assert_eq!(header.len(), 12);
    assert_eq!(header[0], 0xFF);
}

#[test]
fn test_tier36_launchpad_pro_sysex_encoding() {
    let mut lp = LaunchpadProDriver::new();
    let programmer_sysex = lp.set_mode(LaunchpadMode::Programmer);
    assert_eq!(programmer_sysex, vec![0xF0, 0x00, 0x20, 0x29, 0x02, 0x10, 0x0E, 0x03, 0xF7]);

    let led_cmd = lp.set_grid_led(7, 7, 45);
    assert_eq!(led_cmd, vec![0x90, 88, 45]);

    let pad_event = lp.parse_pad_press(88, 100).unwrap();
    assert_eq!(pad_event.row, 7);
    assert_eq!(pad_event.col, 7);
    assert_eq!(pad_event.velocity, 100);
}

#[test]
fn test_tier36_komplete_kontrol_nks_driver() {
    let mut nks = NksIntegrationDriver::new();
    let sysex = nks.set_light_guide(48, 0, 255, 128);
    assert_eq!(sysex[0], 0xF0);
    assert_eq!(sysex[5], 48);

    nks.set_parameter_page(0, &[("Filter Cutoff".to_string(), 0.85)]);
    let (name, val_str) = nks.get_knob_display(0).unwrap();
    assert_eq!(name, "Filter Cutoff");
    assert_eq!(val_str, "0.85");
}

#[test]
fn test_tier36_mcu_motorized_fader_and_lcd_encoding() {
    let mut mcu = McuControllerDriver::new();
    let pitch_bend_msg = mcu.set_fader_position(0, 1.0); // Max fader height
    assert_eq!(pitch_bend_msg, vec![0xE0, 0x7F, 0x7F]);

    let vpot_msg = mcu.set_vpot_led_ring(3, 2, 10);
    assert_eq!(vpot_msg[0], 0xB0);
    assert_eq!(vpot_msg[1], 0x33);

    let lcd_sysex = mcu.set_lcd_text(0, 0, "SUMMONER");
    assert!(lcd_sysex.ends_with(&[0xF7]));
}

#[test]
fn test_tier36_osc_bidirectional_mapping() {
    let mut osc = OscMappingEngine::new();
    osc.add_rule(OscMappingRule {
        osc_path: "/master/volume".to_string(),
        param_id: "MasterGain".to_string(),
        min_val: -60.0,
        max_val: 6.0,
        bidirectional: true,
    });

    let (id, val) = osc.process_incoming_osc("/master/volume", 0.5).unwrap();
    assert_eq!(id, "MasterGain");
    assert_eq!(val, -27.0);

    let (path, outgoing_bytes) = osc.format_outgoing_osc("MasterGain", -27.0).unwrap();
    assert_eq!(path, "/master/volume");
    assert!(!outgoing_bytes.is_empty());
}

#[test]
fn test_tier36_hardware_control_editor_gui_state() {
    let mut editor = HardwareControlEditorState::new();
    assert_eq!(editor.surface_name, "Generic Control Surface");
    assert!(!editor.is_learning);

    editor.toggle_midi_learn();
    assert!(editor.is_learning);

    editor.bind_cc("Master Fader", 7);
    assert_eq!(editor.bound_cc_map.get("Master Fader"), Some(&7));

    let preview = editor.render_layout_preview();
    assert!(preview.contains("Mapped Elements"));
}

#[test]
fn test_tier36_multitrack_routing_matrix() {
    let mut matrix = AudioChannelRoutingMatrix::new(4, 4);
    matrix.set_route(0, 2, 0.5); // Route In 0 -> Out 2 with 0.5 gain

    let in0 = vec![1.0f32; 32];
    let in1 = vec![0.0f32; 32];
    let in2 = vec![0.0f32; 32];
    let in3 = vec![0.0f32; 32];

    let mut out0 = vec![0.0f32; 32];
    let mut out1 = vec![0.0f32; 32];
    let mut out2 = vec![0.0f32; 32];
    let mut out3 = vec![0.0f32; 32];

    matrix.process_matrix(
        &[&in0[..], &in1[..], &in2[..], &in3[..]],
        &mut [&mut out0[..], &mut out1[..], &mut out2[..], &mut out3[..]],
    );

    assert_eq!(out0[0], 1.0); // Direct 1:1 default route
    assert_eq!(out2[0], 0.5); // Additional routed signal
}

#[test]
fn test_tier36_wasm_dsp_runtime() {
    let mut wasm = WasmDspRuntime::new("WasmDelayPlugin", 8);
    assert!(wasm.load_wasm_bytes(&[0x00, 0x61, 0x73, 0x6d, 0x01]).is_ok());
    assert!(wasm.is_loaded);

    let input = vec![0.8f32; 128];
    let mut output = vec![0.0f32; 128];
    assert!(wasm.process_samples(&input, &mut output, 0.25).is_ok());
    assert_eq!(output[0], 0.20);
}

#[test]
fn test_tier36_midi_clock_calibrator() {
    let mut cal = MidiClockCalibrator::new();
    for i in 0..10 {
        cal.record_clock_pulse(i * 20833); // ~120 BPM clock pulse intervals in microseconds
    }
    assert!(cal.calculated_jitter_ms >= 0.0);

    cal.set_latency_compensation(12.5);
    assert_eq!(cal.get_compensated_timestamp(100000), 112500);
}

#[test]
fn test_tier36_isolated_plugin_scanner() {
    let temp_vst = std::env::temp_dir().join("isolated_test_synth.vst3");
    std::fs::create_dir_all(&temp_vst).unwrap();

    let desc = IsolatedPluginScanner::scan_isolated(&temp_vst, 5000).unwrap();
    assert_eq!(desc.format, PluginFormat::Vst3);
    assert_eq!(desc.name, "isolated_test_synth");

    let crash_msg = IsolatedPluginScanner::detect_crash_signature(0xC0000005u32 as i32);
    assert!(crash_msg.contains("Access Violation"));

    let _ = std::fs::remove_dir_all(&temp_vst);
}

#[test]
fn test_tier36_cv_gate_generator() {
    let cv_gen = CvGateGenerator::new(44100);
    assert_eq!(cv_gen.note_to_cv_voltage(60.0), 0.0); // C3 = 0V
    assert_eq!(cv_gen.note_to_cv_voltage(72.0), 1.0); // C4 = +1V

    let mut buf = vec![0.0f32; 64];
    cv_gen.generate_cv_audio(72.0, &mut buf);
    assert_eq!(buf[0], 0.2); // 1.0V / 5.0V max = 0.2 audio signal level

    cv_gen.generate_gate_pulse(true, &mut buf);
    assert_eq!(buf[0], 1.0);
}

#[test]
fn test_tier36_din_sync_generator() {
    let mut din = DinSyncGenerator::new(44100, 120.0);
    let mut clock_buf = vec![0.0f32; 2048];
    let mut run_buf = vec![0.0f32; 2048];

    din.process_block(&mut clock_buf, &mut run_buf, true);
    assert_eq!(run_buf[0], 1.0);
    assert!(clock_buf.iter().any(|&s| s == 1.0));
    assert!(clock_buf.iter().any(|&s| s == 0.0));
}

#[test]
fn test_tier36_ble_midi_controller() {
    let mut ble = BleMidiController::new("BLE-Pedal");
    let packet = vec![0x80, 0x10, 0xB0, 64, 127]; // Sustain pedal CC64 = 127
    let events = ble.parse_ble_packet(&packet);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].1, vec![0xB0, 64, 127]);
}
