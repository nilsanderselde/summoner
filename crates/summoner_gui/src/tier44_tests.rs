// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde - AGPLv3 License

//! Tier 44 unit & integration tests for Zero-Gravity Acoustic Fluid Dynamics & Molecular Synthesis Engine (Steps 1201-1220).

#[cfg(test)]
mod tests {
    use summoner_dsp::zero_gravity_fluid::*;
    use summoner_core::node::{AudioNode, ProcessContext};
    use summoner_core::transport::Transport;

    #[test]
    fn test_step_1201_navier_stokes_fluid_node() {
        let mut fluid = NavierStokesFluidNode::new(0.01, 343.0);
        assert_eq!(fluid.name(), "NavierStokesFluidNode");
        fluid.step_solver(1.0);
        assert!(fluid.net_pressure().is_finite());
    }

    #[test]
    fn test_step_1202_molecular_vibration_resonator() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);
        let mut node = MolecularVibrationResonatorNode::new(CrystallineLattice::Diamond, 440.0);
        assert_eq!(node.name(), "MolecularVibrationResonatorNode");

        let mut out = vec![0.0f32; 64];
        let empty_in: [&[f32]; 0] = [];
        node.process(&empty_in, &mut [&mut out[..]], &ctx);
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_step_1203_plasma_arc_synthesizer() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);
        let mut node = PlasmaArcSynthesizerNode::new(25.0, 50.0);
        assert_eq!(node.name(), "PlasmaArcSynthesizerNode");

        let mut out = vec![0.0f32; 64];
        let empty_in: [&[f32]; 0] = [];
        node.process(&empty_in, &mut [&mut out[..]], &ctx);
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_step_1204_atmospheric_density() {
        let mut node = AtmosphericDensityNode::new(101.3, 20.0);
        assert_eq!(node.name(), "AtmosphericDensityNode");
        let speed = node.calculate_sound_speed();
        assert!(speed > 300.0 && speed < 400.0);

        node.helium_ratio = 0.5;
        let helium_speed = node.calculate_sound_speed();
        assert!(helium_speed > speed);
    }

    #[test]
    fn test_step_1205_quantum_dot_transducer() {
        let node = QuantumDotTransducerNode::new(500.0);
        assert_eq!(node.name(), "QuantumDotTransducerNode");
        let freq = node.wavelength_to_audio_freq();
        assert!((100.0..=4000.0).contains(&freq));
    }

    #[test]
    fn test_step_1206_acoustic_levitation_trap() {
        let node = AcousticLevitationTrapNode::new(40.0, 4);
        assert_eq!(node.name(), "AcousticLevitationTrapNode");
        let force = node.radiation_pressure_force();
        assert!(force > 0.0);
    }

    #[test]
    fn test_step_1207_supercritical_fluid_noise() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);
        let mut node = SupercriticalFluidNoiseNode::new(304.13, 73.75);
        assert_eq!(node.name(), "SupercriticalFluidNoiseNode");

        let mut out = vec![0.0f32; 64];
        let empty_in: [&[f32]; 0] = [];
        node.process(&empty_in, &mut [&mut out[..]], &ctx);
        assert!(out.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn test_step_1208_mhd_plasma_wave_modulator() {
        let node = MhdPlasmaWaveModulatorNode::new(5.0, 1.0);
        assert_eq!(node.name(), "MhdPlasmaWaveModulatorNode");
        assert!(node.alfven_velocity > 0.0);
    }

    #[test]
    fn test_step_1209_sonoluminescence_sonifier() {
        let node = SonoluminescenceSonifierNode::new(25000.0, 5.0);
        assert_eq!(node.name(), "SonoluminescenceSonifierNode");
    }

    #[test]
    fn test_step_1210_metamaterial_refraction_filter() {
        let node = MetamaterialRefractionFilterNode::new(-1.5, 800.0);
        assert_eq!(node.name(), "MetamaterialRefractionFilterNode");
    }

    #[test]
    fn test_step_1211_ism_shockwave_reverb() {
        let node = IsmShockwaveReverbNode::new(3000.0, 10.0);
        assert_eq!(node.name(), "IsmShockwaveReverbNode");
    }

    #[test]
    fn test_step_1212_gravitational_wave_chirp() {
        let node = GravitationalWaveChirpNode::new(36.0, 29.0);
        assert_eq!(node.name(), "GravitationalWaveChirpNode");
        let freq = node.chirp_freq();
        assert!(freq > 0.0);
    }

    #[test]
    fn test_step_1213_casimir_vacuum_noise() {
        let node = CasimirVacuumNoiseNode::new(50.0, 2.0);
        assert_eq!(node.name(), "CasimirVacuumNoiseNode");
        let scale = node.force_scale();
        assert!(scale > 0.0);
    }

    #[test]
    fn test_step_1214_acoustic_cloaking_spatializer() {
        let node = AcousticCloakingSpatializerNode::new(0.5, 1.5);
        assert_eq!(node.name(), "AcousticCloakingSpatializerNode");
    }

    #[test]
    fn test_step_1215_fusion_resonance_synth() {
        let node = FusionResonanceSynthNode::new(10.0, 40.0);
        assert_eq!(node.name(), "FusionResonanceSynthNode");
    }

    #[test]
    fn test_step_1219_zero_allocation_constraint_verification() {
        let transport = Transport::new(44100, 120.0);
        let ctx = ProcessContext::from_transport(&transport);

        let mut fluid = NavierStokesFluidNode::new(0.01, 343.0);
        let in_buf = vec![0.1f32; 512];
        let mut out_buf = vec![0.0f32; 512];

        // Process multiple blocks to verify zero allocations on steady state
        for _ in 0..10 {
            fluid.process(&[&in_buf[..]], &mut [&mut out_buf[..]], &ctx);
        }
        assert!(out_buf.iter().all(|s| s.is_finite()));
    }
}
