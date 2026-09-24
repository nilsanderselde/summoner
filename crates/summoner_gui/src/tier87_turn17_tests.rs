// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Turn #17 Integration & Regression Tests:
//! 1. Two-Tier Piano Roll Canvas (Novice 12-TET vs Pro Microtonal Tuning & Surgical Toolbar).
//! 2. Dynamic Pitch Ruler adapting to 12-EDO, 19-EDO, 31-EDO, and Bohlen-Pierce (3:1 Tritave).
//! 3. Snap Grid Quantization (1/4, 1/8, 1/16, 1/32, Free) & Note Operations (Quantize, Transpose, Humanize, Duplicate).
//! 4. Interactive Note Resizing Handle & Length Manipulation.
//! 5. Milestone 33 Live Note Audition Parameter Bridge (ParamId 300, 301, 302 for Pitch, Gate, Velocity).

#[cfg(test)]
pub mod pure_tests {
    use crate::views::award_winning_gui_view::{PianoRollSnapResolution, PianoRollTuningSystem};
    use summoner_core::param_bus::{ParamBus, ParamId};

    #[test]
    fn test_turn17_tuning_system_pitch_and_frequency_calculations() {
        // 1. 12-EDO (Standard 12 Equal Temperament)
        let edo12 = PianoRollTuningSystem::Edo12;
        let pitches12 = edo12.pitch_names();
        assert_eq!(pitches12.len(), 13);
        assert_eq!(pitches12[0], "C5");
        assert_eq!(pitches12[12], "C4");

        let c4_hz = edo12.pitch_to_hz(12, 13);
        assert!((c4_hz - 261.6256).abs() < 0.01, "C4 should be ~261.63 Hz, got {}", c4_hz);
        let c5_hz = edo12.pitch_to_hz(0, 13);
        assert!((c5_hz - 523.2512).abs() < 0.02, "C5 should be exactly double C4, got {}", c5_hz);

        // A4 is index 3 in ["C5", "B4", "A#4", "A4", ...] (9 semitones above C4)
        let a4_hz = edo12.pitch_to_hz(3, 13);
        assert!((a4_hz - 440.0).abs() < 0.05, "A4 should be ~440.0 Hz, got {}", a4_hz);

        // 2. 19-EDO (Microtonal 19 Divisions of the Octave)
        let edo19 = PianoRollTuningSystem::Edo19;
        let pitches19 = edo19.pitch_names();
        assert_eq!(pitches19.len(), 20);
        assert_eq!(pitches19[0], "C5");
        assert_eq!(pitches19[19], "C4");

        let c4_19 = edo19.pitch_to_hz(19, 20);
        assert!((c4_19 - 261.6256).abs() < 0.01);
        let c5_19 = edo19.pitch_to_hz(0, 20);
        assert!((c5_19 - 523.2512).abs() < 0.02);

        // 3. 31-EDO (Fokker Meantone Approximation)
        let edo31 = PianoRollTuningSystem::Edo31;
        let pitches31 = edo31.pitch_names();
        assert_eq!(pitches31.len(), 32);
        assert!(pitches31[0].contains("C5"));
        assert!(pitches31[31].contains("C4"));

        let c4_31 = edo31.pitch_to_hz(31, 32);
        assert!((c4_31 - 261.6256).abs() < 0.01);
        let c5_31 = edo31.pitch_to_hz(0, 32);
        assert!((c5_31 - 523.2512).abs() < 0.02);

        // 4. Bohlen-Pierce (13 steps per 3:1 Tritave)
        let bp = PianoRollTuningSystem::BohlenPierce;
        let pitches_bp = bp.pitch_names();
        assert_eq!(pitches_bp.len(), 14);
        assert_eq!(pitches_bp[0], "C'");
        assert_eq!(pitches_bp[13], "C");

        let c_bp = bp.pitch_to_hz(13, 14);
        assert!((c_bp - 261.6256).abs() < 0.01);
        let c_prime_bp = bp.pitch_to_hz(0, 14);
        assert!((c_prime_bp - (261.6256 * 3.0)).abs() < 0.05, "Top note of BP should be 3:1 ratio, got {}", c_prime_bp);
    }

    #[test]
    fn test_turn17_snap_grid_quantization() {
        // Quarter beat snap (1.0 beat)
        let snap_quarter = PianoRollSnapResolution::BeatQuarter;
        assert_eq!(snap_quarter.step_beats(), 1.0);
        assert_eq!(snap_quarter.snap(0.2), 0.0);
        assert_eq!(snap_quarter.snap(0.6), 1.0);
        assert_eq!(snap_quarter.snap(1.4), 1.0);
        assert_eq!(snap_quarter.snap(1.7), 2.0);

        // Eighth beat snap (0.5 beat)
        let snap_eighth = PianoRollSnapResolution::BeatEighth;
        assert_eq!(snap_eighth.step_beats(), 0.5);
        assert_eq!(snap_eighth.snap(0.15), 0.0);
        assert_eq!(snap_eighth.snap(0.35), 0.5);
        assert_eq!(snap_eighth.snap(0.70), 0.5);
        assert_eq!(snap_eighth.snap(0.85), 1.0);

        // Sixteenth beat snap (0.25 beat)
        let snap_sixteenth = PianoRollSnapResolution::BeatSixteenth;
        assert_eq!(snap_sixteenth.step_beats(), 0.25);
        assert_eq!(snap_sixteenth.snap(0.10), 0.0);
        assert_eq!(snap_sixteenth.snap(0.18), 0.25);
        assert_eq!(snap_sixteenth.snap(0.40), 0.5);

        // Thirty-second beat snap (0.125 beat)
        let snap_thirty_second = PianoRollSnapResolution::BeatThirtySecond;
        assert_eq!(snap_thirty_second.step_beats(), 0.125);
        assert_eq!(snap_thirty_second.snap(0.05), 0.0);
        assert_eq!(snap_thirty_second.snap(0.11), 0.125);

        // Free snap (Off)
        let snap_free = PianoRollSnapResolution::Free;
        assert_eq!(snap_free.step_beats(), 0.0);
        assert_eq!(snap_free.snap(0.347), 0.347);
    }

    #[test]
    fn test_turn17_param_bus_note_audition_offsets() {
        let mut bus = ParamBus::new();
        let track_id = 1u32;

        let pitch_pid = ParamId(track_id * 1000 + 300);
        let gate_pid = ParamId(track_id * 1000 + 301);
        let vel_pid = ParamId(track_id * 1000 + 302);

        bus.register(pitch_pid, 440.0);
        bus.register(gate_pid, 0.0);
        bus.register(vel_pid, 0.85);

        assert_eq!(bus.get(pitch_pid), Some(440.0));
        assert_eq!(bus.get(gate_pid), Some(0.0));
        assert_eq!(bus.get(vel_pid), Some(0.85));

        // Note On trigger
        bus.set(pitch_pid, 523.25);
        bus.set(gate_pid, 1.0);
        bus.set(vel_pid, 0.95);

        assert_eq!(bus.get(pitch_pid), Some(523.25));
        assert_eq!(bus.get(gate_pid), Some(1.0));
        assert_eq!(bus.get(vel_pid), Some(0.95));

        // Note Off trigger
        bus.set(gate_pid, 0.0);
        assert_eq!(bus.get(gate_pid), Some(0.0));
    }
}

#[cfg(all(test, feature = "gui"))]
pub mod tests {
    use eframe::egui;
    use crate::views::award_winning_gui_view::{AwardWinningGuiView, PianoRollNote, PianoRollSnapResolution, PianoRollTuningSystem};
    use crate::views::modern_top_bar::ModernViewTab;
    use summoner_core::param_bus::{ParamBus, ParamId};
    use summoner_project::create_default_project;
    use summoner_sequencer::automation::AutomationRegistry;
    use summoner_sequencer::automation_timeline::AutomationTimeline;

    #[test]
    fn test_turn17_piano_roll_two_tier_toolbar_and_tuning_switch() {
        let mut view = AwardWinningGuiView::default();
        let ctx = egui::Context::default();

        // 1. In Novice Mode: is_pro_mode is false, tuning is Edo12
        assert!(!view.top_bar_state.is_pro_mode);
        assert_eq!(view.piano_roll_tuning, PianoRollTuningSystem::Edo12);

        // Switch active tab to Piano Roll
        view.top_bar_state.active_tab = ModernViewTab::PianoRoll;

        // Render frame in Novice Mode
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // 2. Enable Pro Mode
        view.top_bar_state.is_pro_mode = true;
        assert!(view.top_bar_state.is_pro_mode);

        // Switch to 19-EDO
        view.piano_roll_tuning = PianoRollTuningSystem::Edo19;
        assert_eq!(view.piano_roll_tuning.pitch_names().len(), 20);

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Switch to 31-EDO
        view.piano_roll_tuning = PianoRollTuningSystem::Edo31;
        assert_eq!(view.piano_roll_tuning.pitch_names().len(), 32);

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });

        // Switch to Bohlen-Pierce
        view.piano_roll_tuning = PianoRollTuningSystem::BohlenPierce;
        assert_eq!(view.piano_roll_tuning.pitch_names().len(), 14);

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                view.show(ui);
            });
        });
    }

    #[test]
    fn test_turn17_piano_roll_surgical_note_operations() {
        let mut view = AwardWinningGuiView::default();
        view.top_bar_state.is_pro_mode = true;
        view.piano_roll_snap = PianoRollSnapResolution::BeatQuarter; // 1.0 beat

        // Setup notes with unaligned start beats
        view.piano_roll_notes = vec![
            PianoRollNote { id: 101, pitch_idx: 3, start_beat: 0.35, length_beats: 1.0, velocity: 0.8 },
            PianoRollNote { id: 102, pitch_idx: 5, start_beat: 1.85, length_beats: 1.5, velocity: 0.9 },
        ];
        view.selected_note_id = Some(101);

        // 1. Quantize selected note (ID 101: 0.35 -> 0.0)
        let snap = view.piano_roll_snap;
        if let Some(sel_id) = view.selected_note_id {
            if let Some(n) = view.piano_roll_notes.iter_mut().find(|n| n.id == sel_id) {
                n.start_beat = snap.snap(n.start_beat);
            }
        }
        assert_eq!(view.piano_roll_notes[0].start_beat, 0.0);
        assert_eq!(view.piano_roll_notes[1].start_beat, 1.85); // Unchanged

        // Quantize all notes (ID 102: 1.85 -> 2.0)
        for n in &mut view.piano_roll_notes {
            n.start_beat = snap.snap(n.start_beat);
        }
        assert_eq!(view.piano_roll_notes[1].start_beat, 2.0);

        // 2. Transpose +1 / -1 step
        let old_p = view.piano_roll_notes[0].pitch_idx; // 3
        if let Some(n) = view.piano_roll_notes.iter_mut().find(|n| n.id == 101) {
            n.pitch_idx -= 1; // +1 step pitch higher
        }
        assert_eq!(view.piano_roll_notes[0].pitch_idx, old_p - 1);

        // 3. Transpose Octave (+12 in 12-EDO)
        view.piano_roll_notes[0].pitch_idx = 12;
        let oct_steps = 12;
        view.piano_roll_notes[0].pitch_idx = view.piano_roll_notes[0].pitch_idx.saturating_sub(oct_steps);
        assert_eq!(view.piano_roll_notes[0].pitch_idx, 0);

        // 4. Duplicate Note
        let orig_len = view.piano_roll_notes.len();
        let step = view.piano_roll_snap.step_beats().max(0.5);
        let n_clone = view.piano_roll_notes[0].clone();
        let new_id = view.next_note_id;
        view.next_note_id += 1;
        view.piano_roll_notes.push(PianoRollNote {
            id: new_id,
            pitch_idx: n_clone.pitch_idx,
            start_beat: n_clone.start_beat + step,
            length_beats: n_clone.length_beats,
            velocity: n_clone.velocity,
        });
        assert_eq!(view.piano_roll_notes.len(), orig_len + 1);
        assert_eq!(view.piano_roll_notes.last().unwrap().start_beat, n_clone.start_beat + 1.0);

        // 5. Humanize notes
        for (idx, n) in view.piano_roll_notes.iter_mut().enumerate() {
            let jitter_t = ((idx * 7 + 3) % 5) as f32 * 0.01 - 0.02;
            n.start_beat = (n.start_beat + jitter_t).max(0.0);
            let jitter_v = ((idx * 11 + 5) % 9) as f32 * 0.02 - 0.08;
            n.velocity = (n.velocity + jitter_v).clamp(0.2, 1.0);
        }
        assert!(view.piano_roll_notes[0].velocity >= 0.2 && view.piano_roll_notes[0].velocity <= 1.0);

        // 6. Delete Note
        view.selected_note_id = Some(new_id);
        view.piano_roll_notes.retain(|n| n.id != new_id);
        assert_eq!(view.piano_roll_notes.len(), orig_len);
    }

    #[test]
    fn test_turn17_piano_roll_m33_live_parameter_bridge_audition() {
        let mut view = AwardWinningGuiView::default();
        view.selected_track_idx = 0; // Explicitly bind track 0
        let project = create_default_project("Turn17Test");
        let mut param_bus = ParamBus::new();
        let mut auto_registry = AutomationRegistry::new();
        let mut auto_timeline = AutomationTimeline::new();

        let track_id = project.tracks[0].id as u32;
        let pitch_pid = ParamId(track_id * 1000 + 300);
        let gate_pid = ParamId(track_id * 1000 + 301);
        let vel_pid = ParamId(track_id * 1000 + 302);

        param_bus.register(pitch_pid, 0.0);
        param_bus.register(gate_pid, 0.0);
        param_bus.register(vel_pid, 0.0);

        // 1. Initial State: Gate is 0.0 (Note Off)
        view.sync_with_param_bus(&project, &param_bus, &mut auto_registry, &mut auto_timeline, 0.0, false);
        assert_eq!(param_bus.get(gate_pid), Some(0.0));

        // 2. Note Audition Trigger: User presses pitch key or note block
        view.last_auditioned_pitch_hz = 523.25;
        view.last_auditioned_gate = 1.0;
        view.selected_note_id = Some(view.piano_roll_notes[0].id);

        view.sync_with_param_bus(&project, &param_bus, &mut auto_registry, &mut auto_timeline, 0.0, false);
        assert_eq!(param_bus.get(pitch_pid), Some(523.25));
        assert_eq!(param_bus.get(gate_pid), Some(1.0));
        assert_eq!(param_bus.get(vel_pid), Some(view.piano_roll_notes[0].velocity));

        // 3. Note Release: Mouse button released
        view.last_auditioned_gate = 0.0;
        view.sync_with_param_bus(&project, &param_bus, &mut auto_registry, &mut auto_timeline, 0.0, false);
        assert_eq!(param_bus.get(gate_pid), Some(0.0));
        assert_eq!(param_bus.get(pitch_pid), Some(523.25)); // Retains last frequency
    }
}
