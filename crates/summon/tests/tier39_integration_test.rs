// Summoner DAW - Tier 39 Integration Tests
// AGPLv3 License

use summoner_dsp::ai_mixing::*;
use summoner_dsp::sampler::SampleBuffer;

#[test]
fn test_tier39_integration_end_to_end_ai_mixing_pipeline() {
    let sample_rate = 44100u32;
    let duration_sec = 2;
    let total_samples = (sample_rate * duration_sec) as usize;

    let mut data = vec![0.0f32; total_samples];
    for (i, sample) in data.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;
        let sine_440 = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        let sine_100 = (2.0 * std::f32::consts::PI * 100.0 * t).sin();
        *sample = (sine_440 * 0.4 + sine_100 * 0.5) * 0.8;
    }

    let input_buffer = SampleBuffer::new(data, sample_rate, 1);

    // 1. Demucs v4 6-stem separation
    let separator = DemucsV4Separator::new();
    let stems = separator.separate_stems(&input_buffer);
    assert_eq!(stems.len(), 6);
    assert!(stems.contains_key("vocals"));
    assert!(stems.contains_key("drums"));
    assert!(stems.contains_key("bass"));

    // 2. Mix balance analysis
    let report = AiMixBalanceAnalyzer::analyze_masking(&stems["vocals"], &stems["bass"]);
    assert!(report.overall_masking_score >= 0.0);
    assert!(!report.recommendation.is_empty());

    // 3. Autonomous mastering engine
    let mastering_engine = AiAutonomousMasteringEngine::new(TargetCurve::ModernPop, -14.0, -0.3);
    let mastered = mastering_engine.master_buffer(&input_buffer);
    assert_eq!(mastered.data.len(), total_samples);

    // 4. Polyphonic chord extraction
    let chords = AiPolyphonicChordExtractor::extract_chords(&input_buffer);
    assert!(!chords.is_empty());
    assert_eq!(chords[0].midi_notes.len(), 3);

    // 5. Room acoustic matching
    let acoustic_profile = NeuralRoomAcousticMatcher::match_impulse_response(&input_buffer);
    assert!(acoustic_profile.rt60_decay_sec > 0.0);

    // 6. Song structure detector
    let sections = AiSongStructureDetector::detect_structure(&input_buffer);
    assert_eq!(sections.len(), 5);
}
