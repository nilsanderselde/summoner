// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

use summoner_core::mpe::{MpeEvent, MpeRouter};
use summoner_core::sample::{hash_sample_data, SampleHash, SampleSlice};

#[test]
fn test_mpe_event_dispatch() {
    let mut router = MpeRouter::new();

    let note_on = MpeEvent::NoteOn {
        voice_id: 101,
        channel: 2,
        note: 60.0,
        velocity: 0.8,
    };
    router.dispatch(&note_on);

    assert!(router.voices[0].is_active);
    assert_eq!(router.voices[0].voice_id, 101);
    assert_eq!(router.voices[0].effective_note(), 60.0);

    let bend = MpeEvent::PitchBend {
        voice_id: 101,
        semitones: 2.5,
    };
    router.dispatch(&bend);
    assert_eq!(router.voices[0].effective_note(), 62.5);

    let note_off = MpeEvent::NoteOff {
        voice_id: 101,
        channel: 2,
        release_velocity: 0.5,
    };
    router.dispatch(&note_off);
    assert!(!router.voices[0].is_active);
}

#[test]
fn test_blake3_content_addressed_samples() {
    let sample_payload: Vec<f32> = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
    let hash1 = hash_sample_data(&sample_payload);
    let hash2 = hash_sample_data(&sample_payload);

    assert_eq!(hash1, hash2);

    let hex = hash1.to_hex();
    let parsed_hash = SampleHash::from_hex(&hex).expect("Failed to parse hex hash");
    assert_eq!(hash1, parsed_hash);

    let slice = SampleSlice::new(hash1.clone(), 0, 4, 1);
    assert_eq!(slice.frame_count(), 4);
    assert_eq!(slice.content_hash, hash1);
}
