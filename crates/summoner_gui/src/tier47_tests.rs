// Summoner DAW - Tier 47 GUI & DSP Integration Unit Tests (Steps 1261-1280)

#[cfg(test)]
mod tests {
    use summoner_dsp::oscillators::{
        SimdPolyWavetableOscillator, OscWavetable,
    };
    use summoner_dsp::SignalProcessor;
    use summoner_core::node::ProcessContext;

    #[test]
    fn test_step_1261_simd_polyphonic_wavetable_oscillator_gui_integration() {
        let mut synth = SimdPolyWavetableOscillator::new(48000);
        
        // Polyphonic note trigger
        synth.note_on(48, 0.9); // C3
        synth.note_on(52, 0.8); // E3
        synth.note_on(55, 0.8); // G3
        synth.note_on(59, 0.7); // B3
        synth.note_on(62, 0.7); // D4
        assert_eq!(synth.active_voice_count(), 5);

        // Wavetable morphing setup
        let sine_table = OscWavetable::default_sine();
        let triangle_table = OscWavetable::default_triangle();
        synth = synth.with_table(sine_table).with_table2(triangle_table, 0.75);

        let mut out_buffer = vec![vec![0.0f32; 512]; 2];
        let mut slices: Vec<&mut [f32]> = out_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();
        let ctx = ProcessContext::new(48000, 120.0, 0);

        synth.process_block(&[], &mut slices, &ctx);

        for s in slices[0].iter() {
            assert!(s.is_finite());
            assert!(s.abs() <= 5.0, "Stereo output sample should remain bounded");
        }

        // Steal voice test (max voices exceed)
        for note in 60..80 {
            synth.note_on(note, 0.5);
        }
        assert!(synth.active_voice_count() <= synth.max_voices);

        // Turn all notes off
        synth.all_notes_off();
        for _ in 0..10000 {
            synth.process_sample();
        }
        assert_eq!(synth.active_voice_count(), 0);
    }
}
