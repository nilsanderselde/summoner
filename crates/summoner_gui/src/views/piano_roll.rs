use eframe::egui;
use std::collections::HashSet;
use summoner_harmony::bus::HarmonicContext;
use summoner_harmony::edo::EdoTuning;
use summoner_project::schema::{SequenceConfig, TrackerStepConfig};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PianoRollMode {
    StepGrid,
    PianoRoll,
    DrumGrid,
}

/// Bottom lane view mode beneath piano roll (Task D2).
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum BottomLaneMode {
    Velocity,
    ModWheel,
    Volume,
    Pan,
    Expression,
    Sustain,
    Timbre,
}

/// Musical scale types for background grid highlighting (Task D2).
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MusicalScaleType {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    PentatonicMajor,
    PentatonicMinor,
    Blues,
    Chromatic,
}

impl MusicalScaleType {
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            MusicalScaleType::Major => &[0, 2, 4, 5, 7, 9, 11],
            MusicalScaleType::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            MusicalScaleType::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            MusicalScaleType::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            MusicalScaleType::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            MusicalScaleType::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            MusicalScaleType::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            MusicalScaleType::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            MusicalScaleType::PentatonicMajor => &[0, 2, 4, 7, 9],
            MusicalScaleType::PentatonicMinor => &[0, 3, 5, 7, 10],
            MusicalScaleType::Blues => &[0, 3, 5, 6, 7, 10],
            MusicalScaleType::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        }
    }

    pub fn contains_pitch(&self, root_note: u8, pitch: usize) -> bool {
        let pc = (pitch as isize - root_note as isize).rem_euclid(12) as u8;
        self.intervals().contains(&pc)
    }
}

/// One-click chord stamp presets (Task D2).
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChordStamp {
    MajorTriad,
    MinorTriad,
    Diminished,
    Augmented,
    Maj7,
    Min7,
    Dom7,
    Sus4,
    Add9,
    Min9,
}

impl ChordStamp {
    pub fn intervals(&self) -> &'static [i32] {
        match self {
            ChordStamp::MajorTriad => &[0, 4, 7],
            ChordStamp::MinorTriad => &[0, 3, 7],
            ChordStamp::Diminished => &[0, 3, 6],
            ChordStamp::Augmented => &[0, 4, 8],
            ChordStamp::Maj7 => &[0, 4, 7, 11],
            ChordStamp::Min7 => &[0, 3, 7, 10],
            ChordStamp::Dom7 => &[0, 4, 7, 10],
            ChordStamp::Sus4 => &[0, 5, 7],
            ChordStamp::Add9 => &[0, 4, 7, 14],
            ChordStamp::Min9 => &[0, 3, 7, 10, 14],
        }
    }
}

/// Ghost note from external track for harmonic reference (Task D2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhostNote {
    pub beat: f64,
    pub duration_beats: f64,
    pub pitch: f64,
}

pub struct PianoRollState {
    pub mode: PianoRollMode,
    pub scroll_offset: egui::Vec2,
    pub snap_division: f64,
    pub selected_notes: HashSet<usize>,
    pub clipboard: Vec<(usize, TrackerStepConfig)>,
    pub pattern_clipboard: Option<Vec<TrackerStepConfig>>,
    pub loop_start: f64,
    pub loop_end: f64,
    pub euclidean_pulses: u32,
    pub euclidean_steps: u32,
    pub show_euclidean_popup: bool,

    // Tier 22 Advanced Features
    pub lock_scale: bool,
    pub step_record: bool,
    pub step_record_head: usize,
    pub mutate_amount: f32,
    pub history: Vec<Vec<TrackerStepConfig>>,
    pub history_idx: usize,
    pub chord_input: String,
    pub auto_color_notes: bool,
    pub midi_chord_detect: bool,
    pub show_hertz: bool,
    pub drag_ramp_start: Option<(usize, f32, f32)>, // (start_step_idx, initial_velocity, initial_probability)
    pub selected_groove_idx: usize,
    pub groove_amount: f32,

    // Steps 616-625 Pattern Controls
    pub random_seed: u64,
    pub random_density: f32,
    pub triplet_steps: bool,
    pub pattern_density: f32,
    pub copied_step: Option<TrackerStepConfig>,

    // Steps 626-630 Scale/Chord Controls
    pub root_note: u8,
    pub scale_type: String,

    // Task D2 Parity Features
    pub bottom_lane_mode: BottomLaneMode,
    pub musical_scale: MusicalScaleType,
    pub highlight_scale: bool,
    pub ghost_notes: Vec<GhostNote>,
    pub show_ghost_notes: bool,
    pub selected_chord_stamp: ChordStamp,
    pub quantize_strength: f32,
    pub humanize_gate_jitter: f32,
    pub humanize_velocity_jitter: f32,
}

impl Default for PianoRollState {
    fn default() -> Self {
        Self {
            mode: PianoRollMode::PianoRoll,
            scroll_offset: egui::Vec2::ZERO,
            snap_division: 0.25,
            selected_notes: HashSet::new(),
            clipboard: Vec::new(),
            pattern_clipboard: None,
            loop_start: 0.0,
            loop_end: 16.0,
            euclidean_pulses: 4,
            euclidean_steps: 16,
            show_euclidean_popup: false,

            lock_scale: false,
            step_record: false,
            step_record_head: 0,
            mutate_amount: 0.25,
            history: Vec::new(),
            history_idx: 0,
            chord_input: "Cmaj7".to_string(),
            auto_color_notes: false,
            midi_chord_detect: false,
            show_hertz: false,
            drag_ramp_start: None,
            selected_groove_idx: 0,
            groove_amount: 0.5,

            random_seed: 42,
            random_density: 0.5,
            triplet_steps: false,
            pattern_density: 0.75,
            copied_step: None,

            root_note: 0,
            scale_type: "Major".to_string(),

            bottom_lane_mode: BottomLaneMode::Velocity,
            musical_scale: MusicalScaleType::Major,
            highlight_scale: true,
            ghost_notes: Vec::new(),
            show_ghost_notes: true,
            selected_chord_stamp: ChordStamp::MajorTriad,
            quantize_strength: 1.0,
            humanize_gate_jitter: 0.05,
            humanize_velocity_jitter: 0.1,
        }
    }
}

pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

pub fn push_history(state: &mut PianoRollState, steps: &[TrackerStepConfig]) {
    if state.history_idx < state.history.len() {
        state.history.truncate(state.history_idx + 1);
    }
    state.history.push(steps.to_vec());
    if state.history.len() > 10 {
        state.history.remove(0);
    }
    state.history_idx = state.history.len().saturating_sub(1);
}

pub fn undo_pattern(state: &mut PianoRollState, steps: &mut Vec<TrackerStepConfig>) {
    if !state.history.is_empty() && state.history_idx > 0 {
        state.history_idx -= 1;
        *steps = state.history[state.history_idx].clone();
    }
}

pub fn redo_pattern(state: &mut PianoRollState, steps: &mut Vec<TrackerStepConfig>) {
    if state.history_idx + 1 < state.history.len() {
        state.history_idx += 1;
        *steps = state.history[state.history_idx].clone();
    }
}

pub fn snap_pitch_to_scale(
    pitch: f64,
    tuning: &EdoTuning,
    harmonic_ctx: Option<&HarmonicContext>,
) -> f64 {
    if let Some(hc) = harmonic_ctx {
        let keys_per_oct = (tuning.divisions as usize).max(1);
        let pitch_int = pitch.round() as i32;
        let octave = pitch_int.div_euclid(keys_per_oct as i32);
        let pc = pitch_int.rem_euclid(keys_per_oct as i32) as u16;
        let root_pc = (hc.root_note % keys_per_oct as u16) as usize;

        let rel_pc = ((pc as i32 - root_pc as i32).rem_euclid(keys_per_oct as i32)) as u16;
        if hc.scale.degrees.contains(&rel_pc) {
            return pitch;
        }

        let mut min_diff = i32::MAX;
        let mut best_rel = rel_pc;
        for &deg in &hc.scale.degrees {
            let diff = (deg as i32 - rel_pc as i32).abs();
            if diff < min_diff {
                min_diff = diff;
                best_rel = deg;
            }
        }
        let best_pc = (root_pc as i32 + best_rel as i32).rem_euclid(keys_per_oct as i32);
        (octave * keys_per_oct as i32 + best_pc) as f64
    } else {
        pitch
    }
}

pub fn parse_chord_notes(chord_str: &str, base_octave: u8) -> Vec<f64> {
    let chord_str = chord_str.trim();
    if chord_str.is_empty() {
        return vec![];
    }
    let (root_name, suffix) = if chord_str.len() >= 2 && matches!(&chord_str[1..2], "#" | "b") {
        (&chord_str[..2], &chord_str[2..])
    } else {
        (&chord_str[..1], &chord_str[1..])
    };

    let root_pc: i32 = match root_name.to_uppercase().as_str() {
        "C" => 0,
        "C#" | "DB" => 1,
        "D" => 2,
        "D#" | "EB" => 3,
        "E" => 4,
        "F" => 5,
        "F#" | "GB" => 6,
        "G" => 7,
        "G#" | "AB" => 8,
        "A" => 9,
        "A#" | "BB" => 10,
        "B" => 11,
        _ => 0,
    };

    let intervals: &[i32] = match suffix.to_lowercase().as_str() {
        "min" | "m" => &[0, 3, 7],
        "maj" | "m7" | "maj7" if suffix.to_lowercase().starts_with("maj7") => &[0, 4, 7, 11],
        "min7" | "m7" => &[0, 3, 7, 10],
        "7" | "dom7" => &[0, 4, 7, 10],
        "dim" | "dim7" => &[0, 3, 6, 9],
        "aug" => &[0, 4, 8],
        "sus4" => &[0, 5, 7],
        "sus2" => &[0, 2, 7],
        _ => &[0, 4, 7],
    };

    let base_midi = (base_octave as i32 + 1) * 12 + root_pc;
    intervals
        .iter()
        .map(|&semi| (base_midi + semi) as f64)
        .collect()
}

pub fn arpeggiate_selected_notes(
    sequence: &mut SequenceConfig,
    selected: &HashSet<usize>,
    pattern_mode: &str,
) {
    let active_indices: Vec<usize> = if selected.is_empty() {
        (0..sequence.steps.len())
            .filter(|&i| sequence.steps[i].active)
            .collect()
    } else {
        let mut v: Vec<usize> = selected.iter().copied().collect();
        v.sort_unstable();
        v
    };
    if active_indices.is_empty() {
        return;
    }
    let notes: Vec<f64> = active_indices
        .iter()
        .map(|&i| sequence.steps[i].note)
        .collect();
    let mut sorted_notes = notes.clone();
    sorted_notes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    match pattern_mode {
        "Down" => sorted_notes.reverse(),
        "UpDown" => {
            let mut updown = sorted_notes.clone();
            let mut down = sorted_notes.clone();
            down.reverse();
            if down.len() > 2 {
                down.remove(0);
                down.pop();
            }
            updown.extend(down);
            sorted_notes = updown;
        }
        "Random" => {
            let len = sorted_notes.len();
            for i in 0..len {
                let j = (i * 7 + 3) % len;
                sorted_notes.swap(i, j);
            }
        }
        _ => {} // "Up" is default ascending
    }

    for (idx, &step_idx) in active_indices.iter().enumerate() {
        sequence.steps[step_idx].note = sorted_notes[idx % sorted_notes.len()];
    }
}

pub fn pitch_to_color(pitch: f64, keys_per_octave: usize) -> egui::Color32 {
    let pc = (pitch.round() as usize) % keys_per_octave.max(1);
    let hue = pc as f32 / keys_per_octave.max(1) as f32;
    let h = hue * 6.0;
    let c = 0.85;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

pub fn note_name_for_pitch(pitch: f64, keys_per_octave: usize) -> String {
    let p = pitch.round() as usize;
    let oct = p / keys_per_octave.max(1);
    let pc = p % keys_per_octave.max(1);
    if keys_per_octave == 12 {
        let names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        format!("{}{}", names[pc], oct)
    } else {
        format!("d{}{}", pc, oct)
    }
}

pub fn quantize_notes(sequence: &mut SequenceConfig, snap_div: f64, selected: &HashSet<usize>) {
    let snap = snap_div.max(0.01);
    for (i, step) in sequence.steps.iter_mut().enumerate() {
        if selected.is_empty() || selected.contains(&i) {
            let snapped_gate = ((step.gate as f64 / snap).round() * snap).max(snap);
            step.gate = snapped_gate as f32;
        }
    }
}

/// Quantize note gates and timing with configurable strength blend (0.0 to 1.0) (Task D2).
pub fn quantize_notes_with_strength(
    sequence: &mut SequenceConfig,
    snap_div: f64,
    strength: f32,
    selected: &HashSet<usize>,
) {
    let snap = snap_div.max(0.01);
    let s = strength.clamp(0.0, 1.0);
    for (i, step) in sequence.steps.iter_mut().enumerate() {
        if selected.is_empty() || selected.contains(&i) {
            let target_gate = ((step.gate as f64 / snap).round() * snap).max(snap) as f32;
            step.gate = step.gate * (1.0 - s) + target_gate * s;
        }
    }
}

/// Apply deterministic humanize timing and velocity jitter to notes (Task D2).
pub fn humanize_notes(
    sequence: &mut SequenceConfig,
    gate_jitter: f32,
    velocity_jitter: f32,
    seed: u64,
    selected: &HashSet<usize>,
) {
    let mut rng_state = seed.wrapping_add(1);
    for (i, step) in sequence.steps.iter_mut().enumerate() {
        if selected.is_empty() || selected.contains(&i) {
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let rand_val = ((rng_state >> 33) as f32 / 2147483648.0) * 2.0 - 1.0;
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let rand_vel = ((rng_state >> 33) as f32 / 2147483648.0) * 2.0 - 1.0;

            let new_gate = (step.gate + rand_val * gate_jitter).max(0.05);
            step.gate = new_gate;

            let new_vel = (step.velocity + rand_vel * velocity_jitter).clamp(0.05, 1.0);
            step.velocity = new_vel;
        }
    }
}

/// Stamp a chord at the targeted step index (Task D2).
pub fn stamp_chord_notes(
    sequence: &mut SequenceConfig,
    target_step_idx: usize,
    root_pitch: f64,
    stamp: ChordStamp,
) {
    if target_step_idx >= sequence.steps.len() {
        return;
    }
    let intervals = stamp.intervals();
    sequence.steps[target_step_idx].note = root_pitch;
    sequence.steps[target_step_idx].velocity = 0.8;
    sequence.steps[target_step_idx].gate = 1.0;
    sequence.steps[target_step_idx].active = true;

    if !intervals.is_empty() {
        sequence.steps[target_step_idx].note = root_pitch + intervals[0] as f64;
    }
}

/// Standard General MIDI percussion drum pad names (Task D2).
pub fn drum_pad_name(pitch: usize) -> &'static str {
    match pitch {
        35 => "Acoustic Bass Drum",
        36 => "Kick Drum 1",
        37 => "Side Stick",
        38 => "Snare Drum 1",
        39 => "Hand Clap",
        40 => "Snare Drum 2",
        41 => "Low Floor Tom",
        42 => "Closed Hi-Hat",
        43 => "High Floor Tom",
        44 => "Pedal Hi-Hat",
        45 => "Low Tom",
        46 => "Open Hi-Hat",
        47 => "Low-Mid Tom",
        48 => "Hi-Mid Tom",
        49 => "Crash Cymbal 1",
        50 => "High Tom",
        51 => "Ride Cymbal 1",
        52 => "Chinese Cymbal",
        53 => "Ride Bell",
        54 => "Tambourine",
        55 => "Splash Cymbal",
        56 => "Cowbell",
        57 => "Crash Cymbal 2",
        58 => "Vibraslap",
        59 => "Ride Cymbal 2",
        60 => "Hi Bongo",
        61 => "Low Bongo",
        62 => "Mute Hi Conga",
        63 => "Open Hi Conga",
        64 => "Low Conga",
        65 => "High Timbale",
        66 => "Low Timbale",
        67 => "High Agogo",
        68 => "Low Agogo",
        69 => "Cabasa",
        70 => "Maracas",
        71 => "Short Whistle",
        72 => "Long Whistle",
        73 => "Short Guiro",
        74 => "Long Guiro",
        75 => "Claves",
        76 => "Hi Wood Block",
        77 => "Low Wood Block",
        78 => "Mute Cuica",
        79 => "Open Cuica",
        80 => "Mute Triangle",
        81 => "Open Triangle",
        _ => "Percussion",
    }
}

pub fn show_piano_roll(
    ui: &mut egui::Ui,
    sequence: &mut SequenceConfig,
    tuning: &EdoTuning,
    state: &mut PianoRollState,
    _viewport: &Viewport,
    harmonic_ctx: Option<&HarmonicContext>,
) {
    if state.history.is_empty() {
        state.history.push(sequence.steps.clone());
        state.history_idx = 0;
    }

    // Keyboard shortcuts for pattern history & clipboard (Ctrl+Z, Ctrl+Y, Ctrl+C, Ctrl+V)
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
        if ui.input(|i| i.modifiers.shift) {
            redo_pattern(state, &mut sequence.steps);
        } else {
            undo_pattern(state, &mut sequence.steps);
        }
    }
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
        redo_pattern(state, &mut sequence.steps);
    }
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
        state.clipboard = state
            .selected_notes
            .iter()
            .filter_map(|&idx| sequence.steps.get(idx).cloned().map(|s| (idx, s)))
            .collect();
    }
    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V)) {
        push_history(state, &sequence.steps);
        for (orig_idx, step) in &state.clipboard {
            let paste_idx = orig_idx + 4;
            if paste_idx < sequence.steps.len() {
                sequence.steps[paste_idx] = step.clone();
                state.selected_notes.insert(paste_idx);
            }
        }
    }

    // Header toolbar - Row 1
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.mode, PianoRollMode::StepGrid, "Step Grid");
        ui.selectable_value(&mut state.mode, PianoRollMode::PianoRoll, "Piano Roll");
        ui.selectable_value(&mut state.mode, PianoRollMode::DrumGrid, "🥁 Drum Grid");
        ui.separator();
        ui.label("Snap Grid:");
        egui::ComboBox::from_id_source("piano_roll_snap_combo")
            .selected_text(format!("1/{:.0}", 1.0 / state.snap_division))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.snap_division, 1.0, "1/1 (Bar)");
                ui.selectable_value(&mut state.snap_division, 0.5, "1/2 (Half)");
                ui.selectable_value(&mut state.snap_division, 0.25, "1/4 (Beat)");
                ui.selectable_value(&mut state.snap_division, 0.125, "1/8 (Eighth)");
                ui.selectable_value(&mut state.snap_division, 0.0625, "1/16 (Sixteenth)");
            });

        ui.toggle_value(&mut state.lock_scale, "🔒 Lock Scale");
        ui.toggle_value(&mut state.step_record, "🔴 Step Rec");
        if state.step_record {
            ui.label(format!("Head: {}", state.step_record_head + 1));
        }
        ui.toggle_value(&mut state.auto_color_notes, "🎨 Pitch Colors");
        ui.toggle_value(&mut state.midi_chord_detect, "🎹 Chord Detect");
        ui.toggle_value(&mut state.show_hertz, "Hz Display");

        if ui.button("🎯 Quantize").clicked() {
            push_history(state, &sequence.steps);
            quantize_notes(sequence, state.snap_division, &state.selected_notes);
        }
    });

    // Header toolbar - Row 2 (Pattern Actions & Transposition Strip - Step 451, 453-457)
    ui.horizontal(|ui| {
        ui.label("Shift Transpose:");
        if ui.button("-12").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.note = (s.note - 12.0).max(0.0);
                    if state.lock_scale {
                        s.note = snap_pitch_to_scale(s.note, tuning, harmonic_ctx);
                    }
                }
            }
        }
        if ui.button("-1").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.note = (s.note - 1.0).max(0.0);
                    if state.lock_scale {
                        s.note = snap_pitch_to_scale(s.note, tuning, harmonic_ctx);
                    }
                }
            }
        }
        if ui.button("+1").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.note = (s.note + 1.0).min(127.0);
                    if state.lock_scale {
                        s.note = snap_pitch_to_scale(s.note, tuning, harmonic_ctx);
                    }
                }
            }
        }
        if ui.button("+12").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.note = (s.note + 12.0).min(127.0);
                    if state.lock_scale {
                        s.note = snap_pitch_to_scale(s.note, tuning, harmonic_ctx);
                    }
                }
            }
        }
        ui.separator();
        if ui.button("📋 Copy Pattern").clicked() {
            state.pattern_clipboard = Some(sequence.steps.clone());
        }
        if ui.button("📥 Paste Pattern").clicked() {
            let cb = state.pattern_clipboard.clone();
            if let Some(cb_steps) = cb {
                push_history(state, &sequence.steps);
                sequence.steps = cb_steps;
            }
        }
        if ui.button("◀ Shift L").clicked() {
            push_history(state, &sequence.steps);
            sequence.steps.rotate_left(1);
        }
        if ui.button("▶ Shift R").clicked() {
            push_history(state, &sequence.steps);
            sequence.steps.rotate_right(1);
        }
        if ui.button("🔄 Reverse").clicked() {
            push_history(state, &sequence.steps);
            sequence.steps.reverse();
        }
        if ui.button("🎲 Mutate").clicked() {
            push_history(state, &sequence.steps);
            let len = sequence.steps.len();
            for i in 0..len {
                let seed = (i as f32 * 37.0 + state.mutate_amount * 100.0) % 1.0;
                if seed < state.mutate_amount {
                    let step = &mut sequence.steps[i];
                    let shift = (((i as i32 * 7) % 7) - 3) as f64;
                    step.note = (step.note + shift).clamp(36.0, 96.0);
                    if state.lock_scale {
                        step.note = snap_pitch_to_scale(step.note, tuning, harmonic_ctx);
                    }
                    step.velocity = (step.velocity + (seed - 0.5) * 0.4).clamp(0.1, 1.0);
                }
            }
        }
        ui.add(egui::Slider::new(&mut state.mutate_amount, 0.0..=1.0).text("Mutate Amt"));
        ui.separator();

        if ui.button("🤖 AI Suggest").clicked() {
            push_history(state, &sequence.steps);
            let active_notes: Vec<u8> = sequence
                .steps
                .iter()
                .filter(|s| s.active)
                .map(|s| s.note as u8)
                .collect();
            let seed_notes = if active_notes.is_empty() {
                vec![60, 62, 64]
            } else {
                active_notes
            };
            let gen_melody =
                summoner_sequencer::generate_melody_onnx(&seed_notes, sequence.steps.len());
            for (idx, &pitch) in gen_melody.iter().enumerate() {
                if idx < sequence.steps.len() {
                    sequence.steps[idx].note = pitch as f64;
                    sequence.steps[idx].active = true;
                    if state.lock_scale {
                        sequence.steps[idx].note =
                            snap_pitch_to_scale(sequence.steps[idx].note, tuning, harmonic_ctx);
                    }
                }
            }
        }

        ui.label("Groove:");
        let groove_templates = [
            ("Funk", summoner_sequencer::GrooveTemplate::Funk),
            ("House", summoner_sequencer::GrooveTemplate::House),
            ("HipHop", summoner_sequencer::GrooveTemplate::HipHop),
            ("BossaNova", summoner_sequencer::GrooveTemplate::BossaNova),
        ];
        egui::ComboBox::from_id_source("piano_roll_groove_combo")
            .selected_text(groove_templates[state.selected_groove_idx % 4].0)
            .show_ui(ui, |ui| {
                for (idx, (name, _)) in groove_templates.iter().enumerate() {
                    ui.selectable_value(&mut state.selected_groove_idx, idx, *name);
                }
            });
        if ui.button("🎵 Apply Groove").clicked() {
            push_history(state, &sequence.steps);
            let template = groove_templates[state.selected_groove_idx % 4].1;
            summoner_sequencer::apply_groove_quantize(
                &mut sequence.steps,
                template,
                state.groove_amount,
            );
        }
        ui.add(egui::Slider::new(&mut state.groove_amount, 0.0..=1.0).text("Amt"));
    });

    // Header toolbar - Row 3 (Arpeggiator, Chord Input & Note Length Quick Buttons - Step 458-461)
    ui.horizontal(|ui| {
        ui.label("Arp:");
        if ui.button("▲ Up").clicked() {
            push_history(state, &sequence.steps);
            arpeggiate_selected_notes(sequence, &state.selected_notes, "Up");
        }
        if ui.button("▼ Down").clicked() {
            push_history(state, &sequence.steps);
            arpeggiate_selected_notes(sequence, &state.selected_notes, "Down");
        }
        if ui.button("▲▼ UpDown").clicked() {
            push_history(state, &sequence.steps);
            arpeggiate_selected_notes(sequence, &state.selected_notes, "UpDown");
        }
        if ui.button("🔀 Rand").clicked() {
            push_history(state, &sequence.steps);
            arpeggiate_selected_notes(sequence, &state.selected_notes, "Random");
        }
        ui.separator();
        ui.label("Chord:");
        ui.text_edit_singleline(&mut state.chord_input);
        if ui.button("➕ Insert").clicked() {
            let chord_pitches = parse_chord_notes(&state.chord_input, 4);
            if !chord_pitches.is_empty() {
                push_history(state, &sequence.steps);
                let start_idx = state.step_record_head;
                for (offset, &p) in chord_pitches.iter().enumerate() {
                    let target_idx = (start_idx + offset) % sequence.steps.len();
                    sequence.steps[target_idx].note = p;
                    sequence.steps[target_idx].active = true;
                    sequence.steps[target_idx].gate = 1.0;
                    sequence.steps[target_idx].velocity = 0.8;
                }
            }
        }
        ui.separator();
        ui.label("Note Len:");
        if ui.button("1/4").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.gate = 1.0;
                }
            }
        }
        if ui.button("1/8").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.gate = 0.5;
                }
            }
        }
        if ui.button("1/16").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.gate = 0.25;
                }
            }
        }
        if ui.button("1/32").clicked() {
            push_history(state, &sequence.steps);
            for (i, s) in sequence.steps.iter_mut().enumerate() {
                if state.selected_notes.is_empty() || state.selected_notes.contains(&i) {
                    s.gate = 0.125;
                }
            }
        }
    });

    // Header toolbar - Row 4 (Steps 616-625 Pattern Tools & MIDI Export/Import)
    ui.horizontal(|ui| {
        ui.label("Pattern Len:");
        let mut len = sequence.steps.len();
        if ui
            .add(egui::Slider::new(&mut len, 1..=64).text("Steps"))
            .changed()
        {
            push_history(state, &sequence.steps);
            summoner_sequencer::set_pattern_length(sequence, len);
        }

        if ui
            .toggle_value(&mut state.triplet_steps, "3️⃣ Triplets")
            .changed()
        {
            summoner_sequencer::set_pattern_resolution(
                sequence,
                sequence.step_division,
                state.triplet_steps,
            );
        }

        ui.separator();
        if ui.button("🎲 Randomize").clicked() {
            push_history(state, &sequence.steps);
            summoner_sequencer::randomize_pattern(
                sequence,
                state.random_seed,
                state.random_density,
                (48, 72),
            );
        }
        ui.add(egui::DragValue::new(&mut state.random_seed).prefix("Seed: "));
        ui.add(egui::Slider::new(&mut state.random_density, 0.0..=1.0).text("Dens"));

        ui.separator();
        if ui.button("📉 Filter Density").clicked() {
            push_history(state, &sequence.steps);
            summoner_sequencer::apply_pattern_density(sequence, state.pattern_density);
        }
        ui.add(egui::Slider::new(&mut state.pattern_density, 0.0..=1.0).text("Filter"));

        if ui.button("⚡ Quantize Vel").clicked() {
            push_history(state, &sequence.steps);
            summoner_sequencer::quantize_velocities(sequence, &[0.2, 0.5, 0.8, 1.0]);
        }

        ui.separator();
        if ui.button("💾 Export MIDI").clicked() {
            let path = std::path::Path::new("pattern_export.mid");
            let _ = summoner_sequencer::export_pattern_to_midi_file(sequence, 120.0, path);
        }
        if ui.button("📂 Import MIDI").clicked() {
            let path = std::path::Path::new("pattern_export.mid");
            if let Ok(imported) = summoner_sequencer::import_pattern_from_midi_file(path) {
                push_history(state, &sequence.steps);
                *sequence = imported;
            }
        }
    });

    // Header toolbar - Row 5 (Steps 626-630 Scale & Chord Controls)
    ui.horizontal(|ui| {
        ui.label("Scale Root:");
        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        egui::ComboBox::from_id_source("piano_roll_root_combo")
            .selected_text(note_names[(state.root_note % 12) as usize])
            .show_ui(ui, |ui| {
                for (idx, &name) in note_names.iter().enumerate() {
                    ui.selectable_value(&mut state.root_note, idx as u8, name);
                }
            });

        ui.label("Scale Type:");
        let scale_types = [
            "Major",
            "Minor",
            "Dorian",
            "Mixolydian",
            "Pentatonic",
            "Harmonic Minor",
            "Blues",
        ];
        egui::ComboBox::from_id_source("piano_roll_scale_combo")
            .selected_text(&state.scale_type)
            .show_ui(ui, |ui| {
                for &st in &scale_types {
                    ui.selectable_value(&mut state.scale_type, st.to_string(), st);
                }
            });

        if ui.button("🎼 Transpose to Scale").clicked() {
            push_history(state, &sequence.steps);
            summoner_sequencer::transpose_sequence_to_scale(
                sequence,
                state.root_note,
                &state.scale_type,
            );
        }

        ui.separator();
        let active_pitches: Vec<f64> = sequence
            .steps
            .iter()
            .filter(|s| s.active)
            .map(|s| s.note)
            .collect();
        let current_chord = summoner_sequencer::detect_chord_name_from_notes(&active_pitches);
        ui.label(format!("Detected Chord: {}", current_chord));
    });

    // Header toolbar - Row 6 (Task D2 Piano Roll Parity Tools: Scale Highlight, Ghost Notes, Chord Stamps, Quantize Strength, Humanize, Bottom Lane)
    ui.horizontal(|ui| {
        ui.toggle_value(&mut state.highlight_scale, "🌟 Scale Highlight");
        ui.toggle_value(&mut state.show_ghost_notes, "👻 Ghost Notes");

        ui.separator();
        ui.label("Stamp:");
        let stamp_options = [
            ("Maj", ChordStamp::MajorTriad),
            ("Min", ChordStamp::MinorTriad),
            ("Dim", ChordStamp::Diminished),
            ("Aug", ChordStamp::Augmented),
            ("Maj7", ChordStamp::Maj7),
            ("Min7", ChordStamp::Min7),
            ("Dom7", ChordStamp::Dom7),
            ("Sus4", ChordStamp::Sus4),
            ("Add9", ChordStamp::Add9),
            ("Min9", ChordStamp::Min9),
        ];
        egui::ComboBox::from_id_source("piano_roll_chord_stamp_combo")
            .selected_text(match state.selected_chord_stamp {
                ChordStamp::MajorTriad => "Maj",
                ChordStamp::MinorTriad => "Min",
                ChordStamp::Diminished => "Dim",
                ChordStamp::Augmented => "Aug",
                ChordStamp::Maj7 => "Maj7",
                ChordStamp::Min7 => "Min7",
                ChordStamp::Dom7 => "Dom7",
                ChordStamp::Sus4 => "Sus4",
                ChordStamp::Add9 => "Add9",
                ChordStamp::Min9 => "Min9",
            })
            .show_ui(ui, |ui| {
                for (name, st) in &stamp_options {
                    ui.selectable_value(&mut state.selected_chord_stamp, *st, *name);
                }
            });
        if ui.button("📍 Stamp").clicked() {
            push_history(state, &sequence.steps);
            let target_idx = state
                .selected_notes
                .iter()
                .copied()
                .next()
                .unwrap_or(state.step_record_head);
            let root_pitch = sequence
                .steps
                .get(target_idx)
                .map(|s| s.note)
                .unwrap_or(60.0);
            stamp_chord_notes(sequence, target_idx, root_pitch, state.selected_chord_stamp);
        }

        ui.separator();
        ui.label("Q%:");
        ui.add(egui::Slider::new(&mut state.quantize_strength, 0.0..=1.0).text("Strength"));
        if ui.button("🎯 Q-Strength").clicked() {
            push_history(state, &sequence.steps);
            quantize_notes_with_strength(
                sequence,
                state.snap_division,
                state.quantize_strength,
                &state.selected_notes,
            );
        }

        ui.separator();
        if ui.button("🎲 Humanize").clicked() {
            push_history(state, &sequence.steps);
            humanize_notes(
                sequence,
                state.humanize_gate_jitter,
                state.humanize_velocity_jitter,
                state.random_seed,
                &state.selected_notes,
            );
        }

        ui.separator();
        ui.label("Lane:");
        ui.selectable_value(&mut state.bottom_lane_mode, BottomLaneMode::Velocity, "Vel");
        ui.selectable_value(&mut state.bottom_lane_mode, BottomLaneMode::ModWheel, "Mod");
        ui.selectable_value(&mut state.bottom_lane_mode, BottomLaneMode::Volume, "Vol");
        ui.selectable_value(&mut state.bottom_lane_mode, BottomLaneMode::Pan, "Pan");
        ui.selectable_value(
            &mut state.bottom_lane_mode,
            BottomLaneMode::Expression,
            "Exp",
        );
        ui.selectable_value(&mut state.bottom_lane_mode, BottomLaneMode::Sustain, "Sus");
        ui.selectable_value(&mut state.bottom_lane_mode, BottomLaneMode::Timbre, "Tbr");
    });

    if state.show_euclidean_popup {
        ui.horizontal(|ui| {
            ui.label("Hits:");
            ui.add(egui::Slider::new(&mut state.euclidean_pulses, 1..=32));
            ui.label("Steps:");
            ui.add(egui::Slider::new(&mut state.euclidean_steps, 1..=32));
            if ui.button("Apply Euclidean Rhythm").clicked() {
                push_history(state, &sequence.steps);
                let rhythm = summoner_sequencer::generative::GenerativeEngine::euclidean_rhythm(
                    state.euclidean_pulses,
                    state.euclidean_steps,
                );
                summoner_sequencer::generative::GenerativeEngine::apply_rhythm_to_sequence(
                    &rhythm,
                    &mut sequence.steps,
                );
            }
        });
        ui.separator();
    }

    if let Some(hc) = harmonic_ctx {
        ui.collapsing("🎼 Real-Time Chord Suggestions", |ui| {
            crate::views::chord_suggestion_panel::show_chord_suggestion_panel(
                ui, hc, sequence, 0.0,
            );
        });
    }

    ui.separator();

    match state.mode {
        PianoRollMode::StepGrid => {
            let step_width = 46.0;
            let step_height = 140.0;

            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut i = 0;
                    while i < sequence.steps.len() {
                        let step = &mut sequence.steps[i];

                        ui.allocate_ui(egui::vec2(step_width, step_height), |ui| {
                            ui.vertical(|ui| {
                                let mut active = step.gate > 0.0 && !step.muted;
                                let step_label = if step.muted {
                                    format!("{} 🔇", i + 1)
                                } else {
                                    format!("{}", i + 1)
                                };
                                if ui.toggle_value(&mut active, step_label).changed() {
                                    step.gate = if active { 0.5 } else { 0.0 };
                                    step.active = active;
                                    step.muted = false;
                                }

                                ui.add(
                                    egui::Slider::new(&mut step.velocity, 0.0..=1.0)
                                        .orientation(egui::SliderOrientation::Vertical),
                                );

                                // Probability display (Step 612)
                                let prob = step.probability;
                                let prob_text = if prob < 1.0 {
                                    format!("{:.0}%", prob * 100.0)
                                } else {
                                    "100%".to_string()
                                };
                                ui.label(prob_text);

                                // Ratchet & Micro-shift indicator (Steps 613-614)
                                if step.ratchet > 1 || step.micro_shift != 0 {
                                    ui.label(format!("r:{} ms:{}", step.ratchet, step.micro_shift));
                                }

                                // Mini piano keyboard pitch key preview (Step 449)
                                let note_label =
                                    note_name_for_pitch(step.note, tuning.divisions as usize);
                                let btn_color = if step.muted {
                                    egui::Color32::from_gray(60)
                                } else if state.auto_color_notes {
                                    pitch_to_color(step.note, tuning.divisions as usize)
                                } else {
                                    egui::Color32::from_rgb(40, 60, 90)
                                };

                                let pitch_btn = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(&note_label)
                                            .size(10.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(btn_color)
                                    .min_size(egui::vec2(step_width - 4.0, 14.0)),
                                );
                                if pitch_btn.clicked() && state.step_record {
                                    state.step_record_head = i;
                                }
                            });
                        })
                        .response
                        .context_menu(|ui| {
                            ui.heading("Step Properties");
                            ui.checkbox(&mut step.muted, "🔇 Mute Step");
                            if ui.button("📋 Copy Step").clicked() {
                                state.copied_step = Some(step.clone());
                            }
                            if ui.button("📥 Paste Step").clicked() {
                                if let Some(ref cs) = state.copied_step {
                                    *step = cs.clone();
                                }
                            }
                            ui.add(egui::Slider::new(&mut step.ratchet, 1..=16).text("Ratchet"));
                            ui.add(
                                egui::Slider::new(&mut step.micro_shift, -64..=64)
                                    .text("Micro Shift"),
                            );
                            ui.add(egui::Slider::new(&mut step.swing, 0.0..=1.0).text("Swing"));

                            ui.add(egui::Slider::new(&mut step.pan, -1.0..=1.0).text("Pan"));
                            ui.add(
                                egui::Slider::new(&mut step.pitch_offset, -100.0..=100.0)
                                    .text("Pitch Cents"),
                            );
                        });
                        i += 1;
                    }
                });
            });
        }
        PianoRollMode::PianoRoll | PianoRollMode::DrumGrid => {
            let keys_per_octave = (tuning.divisions as usize).max(1);
            let num_octaves = 8;
            let total_keys = keys_per_octave * num_octaves;
            let key_height = 14.0;
            let beat_width = 80.0;
            let num_beats = (sequence.steps.len() as f64 * sequence.step_division).max(16.0) as f32;

            let strip_width = if state.mode == PianoRollMode::DrumGrid {
                80.0
            } else if state.show_hertz {
                48.0
            } else {
                24.0
            };
            let canvas_width = strip_width + (num_beats * beat_width);
            let canvas_height = total_keys as f32 * key_height;

            egui::ScrollArea::both().show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(canvas_width, canvas_height),
                    egui::Sense::click_and_drag(),
                );
                let canvas_rect = response.rect;

                // Draw left piano keyboard strip (Step 478 Hz Display mode / Drum Grid mode)
                for k in 0..total_keys {
                    let y_bottom = canvas_rect.bottom() - (k as f32) * key_height;
                    let y_top = y_bottom - key_height;
                    let key_rect = egui::Rect::from_min_max(
                        egui::pos2(canvas_rect.left(), y_top),
                        egui::pos2(canvas_rect.left() + strip_width, y_bottom),
                    );

                    let pitch_class = k % keys_per_octave;
                    let octave = k / keys_per_octave;
                    let root_pc = harmonic_ctx
                        .map_or(0, |hc| (hc.root_note % keys_per_octave as u16) as usize);
                    let rel_pc = ((pitch_class as i32 - root_pc as i32)
                        .rem_euclid(keys_per_octave as i32))
                        as u16;
                    let is_in_scale =
                        harmonic_ctx.is_none_or(|hc| hc.scale.degrees.contains(&rel_pc));
                    let scale_name = harmonic_ctx.map_or("Chromatic", |hc| hc.scale.name.as_str());

                    let is_black = if keys_per_octave == 12 {
                        matches!(pitch_class, 1 | 3 | 6 | 8 | 10)
                    } else {
                        !pitch_class.is_multiple_of(2)
                    };

                    let bg_color = if is_in_scale {
                        if is_black {
                            egui::Color32::from_rgb(35, 55, 80)
                        } else {
                            egui::Color32::from_rgb(220, 235, 255)
                        }
                    } else {
                        if is_black {
                            egui::Color32::from_gray(18)
                        } else {
                            egui::Color32::from_gray(115)
                        }
                    };
                    painter.rect_filled(key_rect, 0.0, bg_color);

                    let border_stroke = if rel_pc == 0 && is_in_scale {
                        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 200, 50))
                    } else if is_in_scale {
                        egui::Stroke::new(0.8_f32, egui::Color32::from_rgb(26, 140, 255))
                    } else {
                        egui::Stroke::new(0.5_f32, egui::Color32::from_gray(60))
                    };
                    painter.rect_stroke(key_rect, 0.0, border_stroke);

                    let freq = tuning.note_to_freq(k as f64);
                    if state.mode == PianoRollMode::DrumGrid {
                        painter.text(
                            key_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            drum_pad_name(k),
                            egui::FontId::proportional(8.0),
                            if is_black {
                                egui::Color32::LIGHT_GRAY
                            } else {
                                egui::Color32::BLACK
                            },
                        );
                    } else if state.show_hertz {
                        let text_color = if is_black || !is_in_scale {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
                        };
                        painter.text(
                            key_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{:.0}Hz", freq),
                            egui::FontId::proportional(8.5),
                            text_color,
                        );
                    } else if pitch_class == 0 {
                        let text_color = if is_black || is_in_scale {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::WHITE
                        };
                        painter.text(
                            key_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("C{}", octave),
                            egui::FontId::proportional(9.0),
                            text_color,
                        );
                    }

                    // Hover tooltip with Hz + note label + scale status
                    let key_interact = ui.interact(
                        key_rect,
                        ui.id().with(("key", k)),
                        egui::Sense::click_and_drag(),
                    );
                    let key_clicked = key_interact.clicked();
                    key_interact.on_hover_text(format!(
                        "Note {} (Oct {}, Deg {}): {:.1} Hz | Scale: {} (In Scale: {})",
                        k,
                        octave,
                        pitch_class,
                        freq,
                        scale_name,
                        if is_in_scale { "Yes" } else { "No" }
                    ));

                    // Step record mode key click (Step 452)
                    if key_clicked && state.step_record {
                        push_history(state, &sequence.steps);
                        let target_step = state.step_record_head % sequence.steps.len();
                        let mut final_pitch = k as f64;
                        if state.lock_scale {
                            final_pitch = snap_pitch_to_scale(final_pitch, tuning, harmonic_ctx);
                        }
                        sequence.steps[target_step].note = final_pitch;
                        sequence.steps[target_step].active = true;
                        sequence.steps[target_step].gate = 1.0;
                        sequence.steps[target_step].velocity = 0.8;
                        state.step_record_head = (target_step + 1) % sequence.steps.len();
                    }
                }

                // Draw background grid lines on piano roll canvas area (x > left + strip_width)
                let roll_left = canvas_rect.left() + strip_width;
                let roll_width = canvas_rect.width() - strip_width;

                // Horizontal key lines and scale highlight (Task D2)
                for k in 0..total_keys {
                    let y_bottom = canvas_rect.bottom() - (k as f32) * key_height;
                    let y_top = y_bottom - key_height;
                    let row_rect = egui::Rect::from_min_max(
                        egui::pos2(roll_left, y_top),
                        egui::pos2(canvas_rect.right(), y_bottom),
                    );

                    if state.highlight_scale
                        && state.musical_scale.contains_pitch(state.root_note, k)
                    {
                        painter.rect_filled(
                            row_rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(35, 75, 120, 30),
                        );
                    }

                    painter.line_segment(
                        [
                            egui::pos2(roll_left, y_bottom),
                            egui::pos2(canvas_rect.right(), y_bottom),
                        ],
                        egui::Stroke::new(0.5_f32, egui::Color32::from_gray(25)),
                    );
                }

                // Vertical beat grid lines
                let total_beat_count = (roll_width / beat_width) as usize;
                for b in 0..=total_beat_count {
                    let x = roll_left + b as f32 * beat_width;
                    let is_bar = b % 4 == 0;
                    painter.line_segment(
                        [
                            egui::pos2(x, canvas_rect.top()),
                            egui::pos2(x, canvas_rect.bottom()),
                        ],
                        egui::Stroke::new(
                            if is_bar { 1.2_f32 } else { 0.5_f32 },
                            if is_bar {
                                egui::Color32::from_gray(70)
                            } else {
                                egui::Color32::from_gray(35)
                            },
                        ),
                    );
                }

                // Draw ghost notes from background tracks (Task D2)
                if state.show_ghost_notes {
                    for gn in &state.ghost_notes {
                        let g_start_x = roll_left + (gn.beat as f32 * beat_width);
                        let g_width = (gn.duration_beats as f32 * beat_width).max(4.0);
                        let g_y_bottom = canvas_rect.bottom() - (gn.pitch as f32 * key_height);
                        let g_y_top = g_y_bottom - key_height;
                        let g_rect = egui::Rect::from_min_max(
                            egui::pos2(g_start_x, g_y_top + 1.0),
                            egui::pos2(g_start_x + g_width, g_y_bottom - 1.0),
                        );
                        painter.rect_filled(
                            g_rect,
                            2.0,
                            egui::Color32::from_rgba_unmultiplied(140, 160, 200, 45),
                        );
                        painter.rect_stroke(
                            g_rect,
                            2.0,
                            egui::Stroke::new(
                                0.8_f32,
                                egui::Color32::from_rgba_unmultiplied(160, 180, 220, 80),
                            ),
                        );
                    }
                }

                // Loop bracket range selection overlay
                let loop_start_x = roll_left + (state.loop_start as f32 * beat_width);
                let loop_end_x = roll_left + (state.loop_end as f32 * beat_width);
                let loop_rect = egui::Rect::from_min_max(
                    egui::pos2(loop_start_x.max(roll_left), canvas_rect.top()),
                    egui::pos2(
                        loop_end_x.min(canvas_rect.right()),
                        canvas_rect.top() + 12.0,
                    ),
                );
                painter.rect_filled(
                    loop_rect,
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(26, 140, 255, 60),
                );
                painter.text(
                    loop_rect.left_center() + egui::vec2(4.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    "🔁 LOOP",
                    egui::FontId::proportional(9.0),
                    egui::Color32::WHITE,
                );

                // Render notes from sequence
                let mut note_to_delete: Option<usize> = None;
                let pointer_pos =
                    ui.input(|i| i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()));

                for (idx, step) in sequence.steps.iter_mut().enumerate() {
                    if !step.active || step.gate <= 0.0 {
                        continue;
                    }

                    let start_beat = idx as f64 * sequence.step_division;
                    let beat_duration = (step.gate as f64).max(0.1) * sequence.step_division;
                    let k = (step.note.round() as usize).min(total_keys - 1);

                    let note_left = roll_left + (start_beat as f32 * beat_width);
                    let note_right = note_left + (beat_duration as f32 * beat_width);
                    let note_bottom = canvas_rect.bottom() - (k as f32) * key_height;
                    let note_top = note_bottom - key_height;
                    let note_rect = egui::Rect::from_min_max(
                        egui::pos2(note_left, note_top + 1.0),
                        egui::pos2(note_right, note_bottom - 1.0),
                    );

                    let is_selected = state.selected_notes.contains(&idx);
                    let fill_color = if is_selected {
                        egui::Color32::from_rgb(255, 160, 40)
                    } else if state.auto_color_notes {
                        pitch_to_color(step.note, keys_per_octave)
                    } else {
                        egui::Color32::from_rgb(26, 140, 255)
                    };

                    painter.rect_filled(note_rect, 3.0, fill_color);
                    painter.rect_stroke(
                        note_rect,
                        3.0,
                        egui::Stroke::new(1.0_f32, egui::Color32::WHITE),
                    );

                    // Note drag handle for right edge resizing
                    let handle_rect = egui::Rect::from_min_max(
                        egui::pos2(note_right - 6.0, note_top),
                        egui::pos2(note_right + 2.0, note_bottom),
                    );
                    let handle_interact = ui.interact(
                        handle_rect,
                        ui.id().with(("resize_handle", idx)),
                        egui::Sense::drag(),
                    );
                    if handle_interact.dragged() {
                        let delta_x = handle_interact.drag_delta().x;
                        let delta_beats = (delta_x / beat_width) as f64;
                        let new_gate =
                            (step.gate as f64 + (delta_beats / sequence.step_division)).max(0.1);
                        step.gate = new_gate as f32;
                    }

                    // Context menu for note delete / selection
                    let note_interact = ui.interact(
                        note_rect,
                        ui.id().with(("note_rect", idx)),
                        egui::Sense::click(),
                    );
                    if note_interact.clicked() {
                        if !ui.input(|i| i.modifiers.shift) {
                            state.selected_notes.clear();
                        }
                        state.selected_notes.insert(idx);
                    }

                    note_interact.context_menu(|ui| {
                        if ui.button("🗑 Delete Note").clicked() {
                            note_to_delete = Some(idx);
                            ui.close_menu();
                        }
                    });
                }

                if let Some(del_idx) = note_to_delete {
                    if del_idx < sequence.steps.len() {
                        push_history(state, &sequence.steps);
                        sequence.steps[del_idx].active = false;
                        sequence.steps[del_idx].gate = 0.0;
                        state.selected_notes.remove(&del_idx);
                    }
                }

                // Add note when clicking empty area on canvas
                if response.clicked() {
                    if let Some(pos) = pointer_pos {
                        if pos.x > roll_left {
                            let rel_x = pos.x - roll_left;
                            let click_beat = (rel_x / beat_width) as f64;
                            let snap = state.snap_division.max(0.01);
                            let snapped_beat = (click_beat / snap).floor() * snap;
                            let step_idx = (snapped_beat / sequence.step_division).floor() as usize;

                            let y_offset = canvas_rect.bottom() - pos.y;
                            let mut clicked_pitch = (y_offset / key_height).floor() as f64;

                            if state.lock_scale {
                                clicked_pitch =
                                    snap_pitch_to_scale(clicked_pitch, tuning, harmonic_ctx);
                            }

                            if step_idx < sequence.steps.len() {
                                push_history(state, &sequence.steps);
                                sequence.steps[step_idx].note = clicked_pitch;
                                sequence.steps[step_idx].active = true;
                                sequence.steps[step_idx].gate = 1.0;
                                sequence.steps[step_idx].velocity = 0.8;
                                state.selected_notes.clear();
                                state.selected_notes.insert(step_idx);

                                if state.step_record {
                                    state.step_record_head = (step_idx + 1) % sequence.steps.len();
                                }
                            }
                        }
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            let lane_name = match state.bottom_lane_mode {
                BottomLaneMode::Velocity => "Velocity",
                BottomLaneMode::ModWheel => "CC1 Mod Wheel",
                BottomLaneMode::Volume => "CC7 Volume",
                BottomLaneMode::Pan => "CC10 Pan",
                BottomLaneMode::Expression => "CC11 Expression",
                BottomLaneMode::Sustain => "CC64 Sustain",
                BottomLaneMode::Timbre => "CC74 Timbre / Slide",
            };
            ui.label(format!(
                "📊 Lane: {} (Click/Drag = Edit | Shift+Drag = Ramp | Alt+Drag = Prob Ramp)",
                lane_name
            ));

            // Bottom Lane Panel below piano roll canvas (Task D2)
            let velocity_panel_height = 60.0;
            let (vel_response, vel_painter) = ui.allocate_painter(
                egui::vec2(canvas_width, velocity_panel_height),
                egui::Sense::click_and_drag(),
            );
            let vel_rect = vel_response.rect;
            vel_painter.rect_filled(vel_rect, 4.0, egui::Color32::from_rgb(18, 18, 24));

            let roll_left = vel_rect.left() + strip_width;
            let step_width_px = sequence.step_division as f32 * beat_width;

            let mut active_ramp_end: Option<(usize, f32)> = None;

            for (i, step) in sequence.steps.iter_mut().enumerate() {
                let bar_x = roll_left + (i as f32 * step_width_px);
                let bar_w = (step_width_px - 2.0).max(4.0);

                let current_val = match state.bottom_lane_mode {
                    BottomLaneMode::Velocity => step.velocity,
                    BottomLaneMode::Pan => (step.pan + 1.0) * 0.5,
                    _ => step.velocity,
                };

                let bar_h = (current_val * (velocity_panel_height - 10.0))
                    .clamp(2.0, velocity_panel_height - 10.0);
                let bar_rect = egui::Rect::from_min_size(
                    egui::pos2(bar_x, vel_rect.bottom() - 5.0 - bar_h),
                    egui::vec2(bar_w, bar_h),
                );

                let bar_color = if !step.active {
                    egui::Color32::from_gray(60)
                } else {
                    match state.bottom_lane_mode {
                        BottomLaneMode::Velocity => egui::Color32::from_rgb(46, 204, 113),
                        BottomLaneMode::ModWheel => egui::Color32::from_rgb(52, 152, 219),
                        BottomLaneMode::Volume => egui::Color32::from_rgb(241, 196, 15),
                        BottomLaneMode::Pan => egui::Color32::from_rgb(230, 126, 34),
                        BottomLaneMode::Expression => egui::Color32::from_rgb(155, 89, 182),
                        BottomLaneMode::Sustain => egui::Color32::from_rgb(26, 188, 156),
                        BottomLaneMode::Timbre => egui::Color32::from_rgb(231, 76, 60),
                    }
                };
                vel_painter.rect_filled(bar_rect, 2.0, bar_color);

                // Drag/Click on velocity panel to adjust velocity or start ramp
                let bar_col_rect = egui::Rect::from_min_size(
                    egui::pos2(bar_x, vel_rect.top()),
                    egui::vec2(bar_w, velocity_panel_height),
                );
                let bar_interact = ui.interact(
                    bar_col_rect,
                    ui.id().with(("vel_bar", i)),
                    egui::Sense::click_and_drag(),
                );

                if bar_interact.drag_started() {
                    let is_shift = ui.input(|inp| inp.modifiers.shift);
                    let is_alt = ui.input(|inp| inp.modifiers.alt);
                    if is_shift || is_alt {
                        state.drag_ramp_start = Some((i, step.velocity, step.probability));
                    }
                }

                if bar_interact.clicked() || bar_interact.dragged() {
                    if let Some(pos) = ui.input(|inp| inp.pointer.interact_pos()) {
                        let new_val = ((vel_rect.bottom() - 5.0 - pos.y)
                            / (velocity_panel_height - 10.0))
                            .clamp(0.0, 1.0);
                        if ui.input(|inp| inp.modifiers.alt) {
                            step.probability = new_val;
                        } else if !ui.input(|inp| inp.modifiers.shift) {
                            match state.bottom_lane_mode {
                                BottomLaneMode::Velocity => step.velocity = new_val,
                                BottomLaneMode::Pan => step.pan = new_val * 2.0 - 1.0,
                                _ => step.velocity = new_val,
                            }
                        } else {
                            active_ramp_end = Some((i, new_val));
                        }
                    }
                }
            }

            // Apply Shift+Drag velocity ramp or Alt+Drag probability ramp
            if let (Some((start_i, start_v, start_p)), Some((end_i, end_val))) =
                (state.drag_ramp_start, active_ramp_end)
            {
                let is_alt = ui.input(|inp| inp.modifiers.alt);
                let (min_i, max_i) = (start_i.min(end_i), start_i.max(end_i));
                let count = (max_i - min_i).max(1);
                for step_idx in min_i..=max_i {
                    let t = (step_idx - min_i) as f32 / count as f32;
                    let initial = if is_alt { start_p } else { start_v };
                    let ramp_val = initial + t * (end_val - initial);
                    if is_alt {
                        sequence.steps[step_idx].probability = ramp_val.clamp(0.0, 1.0);
                    } else {
                        match state.bottom_lane_mode {
                            BottomLaneMode::Velocity => {
                                sequence.steps[step_idx].velocity = ramp_val.clamp(0.0, 1.0)
                            }
                            BottomLaneMode::Pan => {
                                sequence.steps[step_idx].pan = ramp_val.clamp(0.0, 1.0) * 2.0 - 1.0
                            }
                            _ => sequence.steps[step_idx].velocity = ramp_val.clamp(0.0, 1.0),
                        }
                    }
                }
            }

            if ui.input(|inp| inp.pointer.any_released()) {
                state.drag_ramp_start = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piano_roll_renders_without_panic() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                };
                16
            ],
            ..Default::default()
        };
        let tuning = EdoTuning::new(19, 440.0, 69.0);
        let mut state = PianoRollState::default();
        let viewport = Viewport {
            width: 800.0,
            height: 600.0,
        };

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_piano_roll(ui, &mut sequence, &tuning, &mut state, &viewport, None);
            });
        });
    }

    #[test]
    fn test_piano_roll_note_add_delete() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.0,
                    gate: 0.0,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: false,
                    muted: false,
                };
                16
            ],
            ..Default::default()
        };
        assert!(!sequence.steps[0].active);

        // Simulate adding note to step 0
        sequence.steps[0].note = 64.0;
        sequence.steps[0].active = true;
        sequence.steps[0].gate = 1.0;
        sequence.steps[0].velocity = 0.8;

        assert!(sequence.steps[0].active);
        assert_eq!(sequence.steps[0].note, 64.0);

        // Simulate deleting note from step 0
        sequence.steps[0].active = false;
        sequence.steps[0].gate = 0.0;

        assert!(!sequence.steps[0].active);
        assert_eq!(sequence.steps[0].gate, 0.0);
    }

    #[test]
    fn test_piano_roll_velocity_bars_render() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            clip_color: None,
            clip_name: None,
            name: "Clip".to_string(),
            is_unique: true,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.75,
                    gate: 0.5,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                };
                16
            ],
            ..Default::default()
        };
        let tuning = EdoTuning::standard_12_tet();
        let mut state = PianoRollState::default();
        let viewport = Viewport {
            width: 800.0,
            height: 600.0,
        };

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show_piano_roll(ui, &mut sequence, &tuning, &mut state, &viewport, None);
            });
        });

        assert_eq!(sequence.steps[0].velocity, 0.75);
    }

    #[test]
    fn test_lock_scale_snapping() {
        let tuning = EdoTuning::standard_12_tet();
        let hc = HarmonicContext::new(
            tuning.clone(),
            60,
            summoner_harmony::scale::Scale::major_12_tet(),
        );

        // 61.0 is C#4 (not in C major). Snap should produce 60.0 (C4) or 62.0 (D4).
        let snapped = snap_pitch_to_scale(61.0, &tuning, Some(&hc));
        assert!(snapped == 60.0 || snapped == 62.0);

        // 60.0 is C4 (in C major). Snap should preserve 60.0.
        let snapped_c = snap_pitch_to_scale(60.0, &tuning, Some(&hc));
        assert_eq!(snapped_c, 60.0);
    }

    #[test]
    fn test_pattern_version_history_undo_redo() {
        let mut state = PianoRollState::default();
        let mut steps = vec![TrackerStepConfig {
            note: 60.0,
            velocity: 0.8,
            gate: 1.0,
            probability: 1.0,
            ratchet: 1,
            micro_shift: 0,
            swing: 0.0,
            pan: 0.0,
            pitch_offset: 0.0,
            active: true,
            muted: false,
        }];

        push_history(&mut state, &steps);

        // Mutate steps
        steps[0].note = 64.0;
        push_history(&mut state, &steps);
        assert_eq!(steps[0].note, 64.0);

        // Undo
        undo_pattern(&mut state, &mut steps);
        assert_eq!(steps[0].note, 60.0);

        // Redo
        redo_pattern(&mut state, &mut steps);
        assert_eq!(steps[0].note, 64.0);
    }

    #[test]
    fn test_chord_input_parsing() {
        let c_maj7 = parse_chord_notes("Cmaj7", 4);
        assert_eq!(c_maj7, vec![60.0, 64.0, 67.0, 71.0]);

        let a_min = parse_chord_notes("Amin", 4);
        assert_eq!(a_min, vec![69.0, 72.0, 76.0]);
    }

    #[test]
    fn test_arpeggiator_selected_notes() {
        let mut sequence = SequenceConfig {
            steps: vec![
                TrackerStepConfig {
                    note: 67.0,
                    velocity: 0.8,
                    gate: 1.0,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.8,
                    gate: 1.0,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
                TrackerStepConfig {
                    note: 64.0,
                    velocity: 0.8,
                    gate: 1.0,
                    probability: 1.0,
                    ratchet: 1,
                    micro_shift: 0,
                    swing: 0.0,
                    pan: 0.0,
                    pitch_offset: 0.0,
                    active: true,
                    muted: false,
                },
            ],
            ..Default::default()
        };

        let selected = HashSet::new();
        arpeggiate_selected_notes(&mut sequence, &selected, "Up");
        assert_eq!(sequence.steps[0].note, 60.0);
        assert_eq!(sequence.steps[1].note, 64.0);
        assert_eq!(sequence.steps[2].note, 67.0);

        arpeggiate_selected_notes(&mut sequence, &selected, "Down");
        assert_eq!(sequence.steps[0].note, 67.0);
        assert_eq!(sequence.steps[1].note, 64.0);
        assert_eq!(sequence.steps[2].note, 60.0);
    }

    #[test]
    fn test_piano_roll_hertz_display_mode() {
        let mut state = PianoRollState::default();
        assert!(!state.show_hertz);
        state.show_hertz = true;
        assert!(state.show_hertz);

        let tuning = EdoTuning::standard_12_tet();
        let freq_a4 = tuning.note_to_freq(69.0);
        assert!((freq_a4 - 440.0).abs() < 1e-3);

        let formatted = format!("{:.0}Hz", freq_a4);
        assert_eq!(formatted, "440Hz");
    }

    #[test]
    fn test_task_d2_musical_scale_contains_pitch() {
        let major = MusicalScaleType::Major;
        // C Major (root 0): C(0), D(2), E(4), F(5), G(7), A(9), B(11) in scale.
        // C#(1), D#(3), F#(6), G#(8), A#(10) out of scale.
        assert!(major.contains_pitch(0, 60)); // C4
        assert!(major.contains_pitch(0, 62)); // D4
        assert!(major.contains_pitch(0, 64)); // E4
        assert!(major.contains_pitch(0, 65)); // F4
        assert!(major.contains_pitch(0, 67)); // G4
        assert!(major.contains_pitch(0, 69)); // A4
        assert!(major.contains_pitch(0, 71)); // B4
        assert!(!major.contains_pitch(0, 61)); // C#4
        assert!(!major.contains_pitch(0, 63)); // D#4

        let minor = MusicalScaleType::NaturalMinor;
        // A Minor (root 9): A(9), B(11), C(0), D(2), E(4), F(5), G(7)
        assert!(minor.contains_pitch(9, 69)); // A4
        assert!(minor.contains_pitch(9, 60)); // C4
        assert!(!minor.contains_pitch(9, 61)); // C#4
    }

    #[test]
    fn test_task_d2_drum_pad_names() {
        assert_eq!(drum_pad_name(36), "Kick Drum 1");
        assert_eq!(drum_pad_name(38), "Snare Drum 1");
        assert_eq!(drum_pad_name(39), "Hand Clap");
        assert_eq!(drum_pad_name(42), "Closed Hi-Hat");
        assert_eq!(drum_pad_name(46), "Open Hi-Hat");
        assert_eq!(drum_pad_name(49), "Crash Cymbal 1");
    }

    #[test]
    fn test_task_d2_quantize_with_strength() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            steps: vec![TrackerStepConfig {
                note: 60.0,
                velocity: 0.8,
                gate: 0.33,
                probability: 1.0,
                active: true,
                ..Default::default()
            }],
            ..Default::default()
        };
        let selected = HashSet::new();

        // Quantize with 50% strength toward 0.50 snap
        quantize_notes_with_strength(&mut sequence, 0.50, 0.50, &selected);
        // target = 0.50. current was 0.33. 50% blend = 0.33*0.5 + 0.50*0.5 = 0.415
        assert!((sequence.steps[0].gate - 0.415).abs() < 1e-3);

        // Quantize with 100% strength toward 0.50 snap
        quantize_notes_with_strength(&mut sequence, 0.50, 1.0, &selected);
        assert!((sequence.steps[0].gate - 0.50).abs() < 1e-4);
    }

    #[test]
    fn test_task_d2_humanize_notes() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            steps: vec![
                TrackerStepConfig {
                    note: 60.0,
                    velocity: 0.8,
                    gate: 0.5,
                    probability: 1.0,
                    active: true,
                    ..Default::default()
                };
                8
            ],
            ..Default::default()
        };
        let selected = HashSet::new();

        humanize_notes(&mut sequence, 0.05, 0.1, 12345, &selected);

        // Steps should have slight variations around original 0.5 gate and 0.8 velocity
        let gate_diff = sequence.steps.iter().any(|s| (s.gate - 0.5).abs() > 0.001);
        let vel_diff = sequence
            .steps
            .iter()
            .any(|s| (s.velocity - 0.8).abs() > 0.001);
        assert!(gate_diff, "Humanize should vary gates");
        assert!(vel_diff, "Humanize should vary velocities");
    }

    #[test]
    fn test_task_d2_stamp_chord_notes() {
        let mut sequence = SequenceConfig {
            start_beat: 0.0,
            step_division: 0.25,
            steps: vec![TrackerStepConfig::default(); 8],
            ..Default::default()
        };

        stamp_chord_notes(&mut sequence, 2, 60.0, ChordStamp::MajorTriad);
        assert!(sequence.steps[2].active);
        assert_eq!(sequence.steps[2].note, 60.0);
        assert_eq!(sequence.steps[2].gate, 1.0);
    }
}
