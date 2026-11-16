use super::*;

#[test]
fn test_dsp_node_registry_inventory_completeness() {
    let registry = DspNodeRegistry::new();
    let inventory = DspNodeRegistry::inventory();
    let mut missing = Vec::new();
    for (name, _, _) in &inventory {
        if registry.get(name).is_none() {
            missing.push(*name);
        }
    }
    if !missing.is_empty() {
        panic!("Missing {} descriptors: {:?}", missing.len(), missing);
    }
    assert!(registry.list_all().len() >= 670, "Registry should contain >= 670 descriptors, found {}", registry.list_all().len());
    assert!(inventory.len() >= 670, "Inventory should contain >= 670 entries, found {}", inventory.len());

    // Verify newly registered unmapped nodes exist
    assert!(registry.get("TruePeakLimiter").is_some());
    assert!(registry.get("NamAmpNode").is_some());
    assert!(registry.get("RnnoiseNode").is_some());
    assert!(registry.get("GranularCloudNode").is_some());
    assert!(registry.get("FreeReedNode").is_some());
    assert!(registry.get("Koto").is_some());
    assert!(registry.get("SpatialRoomNode").is_some());
    assert!(registry.get("ContinuousSpatialPannerDoppler3D").is_some());
    assert!(registry.get("SpectrogramArtNode").is_some());
    assert!(registry.get("AudiogramLoudnessModel").is_some());
    assert!(registry.get("PluginAudioNode").is_some());
    assert!(registry.get("AudioChannelRoutingMatrix").is_some());
    assert!(registry.get("GrandPianoFeltHammer").is_some());
    assert!(registry.get("SpruceSoundboardMode").is_some());
    assert!(registry.get("ToneholeJunction").is_some());
    assert!(registry.get("AmbisonicsDecoder3D").is_some());
    assert!(registry.get("AmbisonicsEncoder3D").is_some());
    assert!(registry.get("AutoWah").is_some());
    assert!(registry.get("ClavinetVoice").is_some());
    assert!(registry.get("GlitchShuffle").is_some());
    assert!(registry.get("MalletStrikeGenerator").is_some());
    assert!(registry.get("OversampledClipper").is_some());
    assert!(registry.get("PartitionedBinauralHrtfConvolver").is_some());
    assert!(registry.get("TruePeakDetector").is_some());
    assert!(registry.get("TarabStringResonator").is_some());
    assert!(registry.get("SitarGourdBody").is_some());
    assert!(registry.get("NeuralIrSynthesizer").is_some());
    assert!(registry.get("NeuralMidiTranscriber").is_some());
    assert!(registry.get("DrumTranscriptor").is_some());
    assert!(registry.get("AiChordGenerator").is_some());
    assert!(registry.get("AiMixAssistant").is_some());
    assert!(registry.get("GrandPianoStringDuplexCoupler").is_some());
    assert!(registry.get("BellowsDynamicsCoupler").is_some());
    assert!(registry.get("FormantVowelMatrix").is_some());
    assert!(registry.get("PreampToneControl").is_some());
    assert!(registry.get("ClavinetAnvilModel").is_some());
    assert!(registry.get("YarnDamper").is_some());
    assert!(registry.get("BesselMembraneMode").is_some());
    assert!(registry.get("KotoStringVoice").is_some());
    assert!(registry.get("HurdyGurdyStringVoice").is_some());
    assert!(registry.get("GlassArmonicaBowl").is_some());
    assert!(registry.get("EurorackCvGateInterface").is_some());
    assert!(registry.get("PushControllerDriver").is_some());
    assert!(registry.get("LaunchpadProDriver").is_some());
    assert!(registry.get("NksIntegrationDriver").is_some());
    assert!(registry.get("McuControllerDriver").is_some());
    assert!(registry.get("MidiClockCalibrator").is_some());
    assert!(registry.get("CvGateGenerator").is_some());
    assert!(registry.get("DinSyncGenerator").is_some());
    assert!(registry.get("BleMidiController").is_some());
    assert!(registry.get("FreeReedOscillator").is_some());
    assert!(registry.get("PipeOrganStops").is_some());
    assert!(registry.get("CassottoChamber").is_some());
    assert!(registry.get("BodyResonatorMode").is_some());
    assert!(registry.get("SitarStringWaveguide").is_some());
    assert!(registry.get("GrandPianoVoice").is_some());
    assert!(registry.get("OledDisplayDriver").is_some());
    assert!(registry.get("GpioDriver").is_some());
    assert!(registry.get("BatteryMonitor").is_some());
    assert!(registry.get("BrainstemPitchTracker").is_some());
    assert!(registry.get("NeuroFeedbackOscillator").is_some());
    assert!(registry.get("AcousticHologramFilter").is_some());
    assert!(registry.get("HapticTransducerNode").is_some());
    assert!(registry.get("SubHarmonicQuantumTunnelingFilter").is_some());
    assert!(registry.get("HyperbolicReverbNode").is_some());
    assert!(registry.get("QuantumEntanglementRouter").is_some());
    assert!(registry.get("NeuralQuantumAnnealer").is_some());
    assert!(registry.get("QuantumTeleportationBufferBus").is_some());
    assert!(registry.get("QuantumErrorCorrectionCodec").is_some());
    assert!(registry.get("HyperDimensionalHrtfLoader").is_some());
    assert!(registry.get("MhdPlasmaWaveModulatorNode").is_some());
    assert!(registry.get("AcousticCloakingSpatializerNode").is_some());
    assert!(registry.get("CrystalModalFilter").is_some());
    assert!(registry.get("HurdyGurdyDispersionFilter").is_some());
    assert!(registry.get("SpectrogramArtEngine").is_some());
    assert!(registry.get("SpectrogramArtMorpher").is_some());
    assert!(registry.get("AudioReverse").is_some());
    assert!(registry.get("GlitchGate").is_some());
    assert!(registry.get("PeakHeadroomAnalyzer").is_some());
    assert!(registry.get("EbuR128LoudnessMeter").is_some());
    assert!(registry.get("StemMetadataParser").is_some());
    assert!(registry.get("DrumReplacementTrigger").is_some());
    assert!(registry.get("SurroundStemSplitterBedObject").is_some());
    assert!(registry.get("HardwareControlEditorState").is_some());
    assert!(registry.get("HeadTrackerReceiver").is_some());
    assert!(registry.get("ProceduralSpatialIrGenerator").is_some());
    assert!(registry.get("IsolatedPluginScanner").is_some());
    assert!(registry.get("MidiFilterEngine").is_some());
    assert!(registry.get("PolymetricSequencer").is_some());
    assert!(registry.get("LinkwitzRiley4WaySplitter").is_some());
    assert!(registry.get("AdaptiveBufferScaler").is_some());
    assert!(registry.get("NativeAudioDriverTuner").is_some());
    assert!(registry.get("HardwareWatchdogService").is_some());
    assert!(registry.get("ThermalThrottlingListener").is_some());
    assert!(registry.get("BypassRelayTrigger").is_some());
    assert!(registry.get("RotaryEncoderDebouncer").is_some());
    assert!(registry.get("MultiTenantRenderQueue").is_some());
    assert!(registry.get("LockFreeTuningRemapper").is_some());
    assert!(registry.get("SidechainMatrix").is_some());
    assert!(registry.get("BellowsGestureEngine").is_some());
    assert!(registry.get("BowingGestureEngine").is_some());
    assert!(registry.get("ClavinetGestureEngine").is_some());
    assert!(registry.get("EpGestureEngine").is_some());
    assert!(registry.get("GlassGestureEngine").is_some());
    assert!(registry.get("GrandPianoGestureEngine").is_some());
    assert!(registry.get("HurdyGurdyGestureEngine").is_some());
    assert!(registry.get("KotoGestureEngine").is_some());
    assert!(registry.get("MalletGestureEngine").is_some());
    assert!(registry.get("MarkovSequenceMutator").is_some());
    assert!(registry.get("PipeOrganGestureEngine").is_some());
    assert!(registry.get("PluckGestureEngine").is_some());
    assert!(registry.get("RotaryGestureEngine").is_some());
    assert!(registry.get("ShakuhachiGestureEngine").is_some());
    assert!(registry.get("SitarGestureEngine").is_some());
    assert!(registry.get("SpatialAutomationEngine").is_some());
    assert!(registry.get("SpringGestureEngine").is_some());
    assert!(registry.get("WoodwindFingeringEngine").is_some());
    assert!(registry.get("BellowsBus").is_some());
    assert!(registry.get("SpatialBus").is_some());
    assert!(registry.get("ArticulationBus").is_some());
    assert!(registry.get("BreathBus").is_some());
    assert!(registry.get("CadenceBus").is_some());
    assert!(registry.get("ClavinetBus").is_some());
    assert!(registry.get("EpBus").is_some());
    assert!(registry.get("FormantBus").is_some());
    assert!(registry.get("GlassBus").is_some());
    assert!(registry.get("GrandPianoBus").is_some());
    assert!(registry.get("HurdyGurdyBus").is_some());
    assert!(registry.get("KotoBus").is_some());
    assert!(registry.get("MalletBus").is_some());
    assert!(registry.get("PipeOrganBus").is_some());
    assert!(registry.get("PlectrumBus").is_some());
    assert!(registry.get("RotaryBus").is_some());
    assert!(registry.get("ShakuhachiBus").is_some());
    assert!(registry.get("SitarBus").is_some());
    assert!(registry.get("SpringBus").is_some());
    assert!(registry.get("BootToSynthEngine").is_some());
    assert!(registry.get("MidiUsbGadgetMode").is_some());
    assert!(registry.get("MultiBusEventRouter").is_some());
    assert!(registry.get("SampleEditor").is_some());
    assert!(registry.get("SpatialObjectAutomation").is_some());
    assert!(registry.get("Vst3WindowEmbedder").is_some());
    assert!(registry.get("ParameterAutomapper").is_some());
    assert!(registry.get("TruePeakMeter").is_some());
    assert!(registry.get("AudioTagger").is_some());
    assert!(registry.get("NeuralWavetableInterpolator").is_some());
    assert!(registry.get("KarplusStrongString").is_some());
    assert!(registry.get("TonebarCoupling").is_some());
    assert!(registry.get("InductivePickup").is_some());
    assert!(registry.get("HammerModel").is_some());
    assert!(registry.get("ImageReflection").is_some());
    assert!(registry.get("TapeChannelState").is_some());
    assert!(registry.get("TubeChannelState").is_some());
    assert!(registry.get("MpeRouter").is_some());
    assert!(registry.get("MidiUartSerialDriver").is_some());
    assert!(registry.get("EepromPresetStore").is_some());
    assert!(registry.get("WebConfigDashboard").is_some());
    assert!(registry.get("WasapiDriver").is_some());
    assert!(registry.get("StereoPanner").is_some());
    assert!(registry.get("MultitrackSessionLooper").is_some());
    assert!(registry.get("AlsaDriver").is_some());
    assert!(registry.get("AudioUnitDriver").is_some());
    assert!(registry.get("AAudioDriver").is_some());
    assert!(registry.get("AsapiDriver").is_some());
    assert!(registry.get("BleMidiPeripheral").is_some());
    assert!(registry.get("HardwareEmulationHarness").is_some());
    assert!(registry.get("MidiClockGenerator").is_some());
    assert!(registry.get("MidiClockReceiver").is_some());
    assert!(registry.get("MidiInputFilter").is_some());
    assert!(registry.get("KeyboardSplit").is_some());
    assert!(registry.get("KeyboardLayering").is_some());
    assert!(registry.get("NamWaveNetEngine").is_some());
    assert!(registry.get("FreeReedVoice").is_some());
    assert!(registry.get("ArmonicaChassisMode").is_some());
    assert!(registry.get("PaulowniaBodyMode").is_some());
    assert!(registry.get("DispersionAllpassChain").is_some());
    assert!(registry.get("TineDispersionAllpass").is_some());
    assert!(registry.get("SitarDispersionAllpass").is_some());
    assert!(registry.get("BandDynamics").is_some());
    assert!(registry.get("GlottalPulseNode").is_some());
    assert!(registry.get("AirReed").is_some());
    assert!(registry.get("StruckIdiophoneResonatorNode").is_some());
    assert!(registry.get("FrictionModel").is_some());
    assert!(registry.get("JawariBridge").is_some());
    assert!(registry.get("JiBridgeJunction").is_some());
    assert!(registry.get("TrompetteChienJunction").is_some());
    assert!(registry.get("BridgeWaveCoupler").is_some());
    assert!(registry.get("SnareRattleModel").is_some());
    assert!(registry.get("Windchest").is_some());
    assert!(registry.get("NeuralHarmonicExciterNode").is_some());
    assert!(registry.get("NeuralTransientShaperNode").is_some());
    assert!(registry.get("NeuralBassGeneratorNode").is_some());
    assert!(registry.get("HeadBumpFilter").is_some());
    assert!(registry.get("LiveSessionRecorder").is_some());
    assert!(registry.get("VisualizerIntegrationEngine").is_some());
    assert!(registry.get("TrackStereoCorrelationMeter").is_some());
    assert!(registry.get("SpectrumMatchingAnalyzer").is_some());
    assert!(registry.get("MasterBusTrim").is_some());
    assert!(registry.get("BowedStringNode").is_some());
    assert!(registry.get("PercussionMembraneNode").is_some());
    assert!(registry.get("PipeOrganNode").is_some());
    assert!(registry.get("CommutedPluckedString").is_some());
    assert!(registry.get("RotarySpeakerNode").is_some());
    assert!(registry.get("SamplerNode").is_some());
    assert!(registry.get("ShakuhachiVoice").is_some());
    assert!(registry.get("SitarVoice").is_some());
    assert!(registry.get("TonewheelOrganNode").is_some());
    assert!(registry.get("WaveguideBrassNode").is_some());
    assert!(registry.get("WaveguideMeshNode").is_some());
    assert!(registry.get("WoodwindJetNode").is_some());
    assert!(registry.get("MolecularVibrationResonatorNode").is_some());
    assert!(registry.get("PlasmaArcSynthesizerNode").is_some());
    assert!(registry.get("MetamaterialRefractionFilterNode").is_some());
    assert!(registry.get("IsmShockwaveReverbNode").is_some());
    assert!(registry.get("FusionResonanceSynthNode").is_some());
    assert!(registry.get("MasterLimiter").is_some());
    assert!(registry.get("ClavinetToneSwitches").is_some());
    assert!(registry.get("OrganVoice").is_some());
    assert!(registry.get("BowedString").is_some());
    assert!(registry.get("Clavinet").is_some());
    assert!(registry.get("ElectricPiano").is_some());
    assert!(registry.get("FreeReed").is_some());
    assert!(registry.get("GlottalPulse").is_some());
    assert!(registry.get("ConcertGrandPiano").is_some());
    assert!(registry.get("PlateTank").is_some());
    assert!(registry.get("Shakuhachi").is_some());
    assert!(registry.get("SitarNode").is_some());
    assert!(registry.get("SpringLattice").is_some());
    assert!(registry.get("VocalTract").is_some());
    assert!(registry.get("StereoWidth").is_some());
    assert!(registry.get("MemoryEstimator").is_some());
    assert!(registry.get("LFO").is_some());
    assert!(registry.get("MacroKnob").is_some());
    assert!(registry.get("Butterworth2ndOrder").is_some());
    assert!(registry.get("ModalFilterSection").is_some());
    assert!(registry.get("DispersionAllpass").is_some());
    assert!(registry.get("SoundboardMode").is_some());
    assert!(registry.get("ConsoleChannelState").is_some());
    assert!(registry.get("ConsoleBiquad").is_some());
    assert!(registry.get("CadenceGraph").is_some());
    assert!(registry.get("CadenceEngine").is_some());
    assert!(registry.get("HarmonicTensionAnalyzer").is_some());
    assert!(registry.get("VoiceLeadingOptimizer").is_some());
    assert!(registry.get("IsomorphicRouter").is_some());
    assert!(registry.get("VowelGraph").is_some());
    assert!(registry.get("EdoTuning").is_some());
    assert!(registry.get("SclTuning").is_some());
    assert!(registry.get("KbmMapping").is_some());
    assert!(registry.get("HarmonicBusBridge").is_some());
    assert!(registry.get("ClipMatrix").is_some());
    assert!(registry.get("AutomationTimeline").is_some());
    assert!(registry.get("CompTrack").is_some());
    assert!(registry.get("GenerativeEngine").is_some());
    assert!(registry.get("TimelineArranger").is_some());
    assert!(registry.get("Arpeggiator").is_some());
    assert!(registry.get("ChordMemory").is_some());
    assert!(registry.get("Strummer").is_some());
    assert!(registry.get("SidechainRoute").is_some());
    assert!(registry.get("SpatialTrajectoryTrack").is_some());
    assert!(registry.get("TrackerStep").is_some());
    assert!(registry.get("CadenceFlow").is_some());
    assert!(registry.get("ConcordanceLattice").is_some());
    assert!(registry.get("EmbouchureAngle").is_some());
    assert!(registry.get("FrictionOrbit").is_some());
    assert!(registry.get("HarmonicTensionMap").is_some());
    assert!(registry.get("Hoa4Spatializer").is_some());
    assert!(registry.get("IdiophoneSpectrum").is_some());
    assert!(registry.get("IsomorphicLattice").is_some());
    assert!(registry.get("IsomorphicTuningKeyboard").is_some());
    assert!(registry.get("LadderFilterBode").is_some());
    assert!(registry.get("MembraneCavity").is_some());
    assert!(registry.get("MpeghSpatializer").is_some());
    assert!(registry.get("PhaseAlign").is_some());
    assert!(registry.get("RotaryDoppler").is_some());
    assert!(registry.get("ToneholeMatrix").is_some());
    assert!(registry.get("TrompetteBridge").is_some());
    assert!(registry.get("VowelSpace").is_some());
    assert!(registry.get("BellowsChamber").is_some());
    assert!(registry.get("ConvolutionImpulse").is_some());
    assert!(registry.get("ConvolutionMorph").is_some());
    assert!(registry.get("BezierAutomation").is_some());
    assert!(registry.get("NamModelConfig").is_some());
    assert!(registry.get("Vst3HostConfig").is_some());
    assert!(registry.get("ClapAutomationHost").is_some());
    assert!(registry.get("SpatialIrConfig").is_some());
    assert!(registry.get("SpectralMorphEngine").is_some());
    assert!(registry.get("SpectrogramSoundGenerator").is_some());
    assert!(registry.get("MultiSampleBank").is_some());
    assert!(registry.get("SamplerMacroStrip").is_some());
    assert!(registry.get("AirReedConfig").is_some());
    assert!(registry.get("WindchestReservoir").is_some());
    assert!(registry.get("QuantumTomographyVisualizer").is_some());
    assert!(registry.get("NeuroAffectiveEngine").is_some());
    assert!(registry.get("OscMappingRouter").is_some());
    assert!(registry.get("MaskingCollisionAnalyzer").is_some());
    assert!(registry.get("AiMixRecommendationEngine").is_some());
    assert!(registry.get("AetherMacroStrip").is_some());
    assert!(registry.get("DrumMacroStrip").is_some());
    assert!(registry.get("SimdVoiceAllocator").is_some());
    assert!(registry.get("AudioWarpMarker").is_some());
    assert!(registry.get("StickSlipPhysicsModel").is_some());
    assert!(registry.get("PianoHammerMechanics").is_some());
    assert!(registry.get("GlassArmonicaBus").is_some());
    assert!(registry.get("SfzPresetPatch").is_some());
    assert!(registry.get("LuaScriptEngine").is_some());
    assert!(registry.get("PluginSandbox").is_some());
    assert!(registry.get("PluginLatencyCompensation").is_some());
    assert!(registry.get("PluginCrashGuard").is_some());
    assert!(registry.get("AudioGraphBenchmarkSuite").is_some());
    assert!(registry.get("ProjectAutoSaveManager").is_some());
    assert!(registry.get("SessionMarkerNavigationManager").is_some());
    assert!(registry.get("ScratchAudioCache").is_some());
    assert!(registry.get("WorkspaceDependencyAuditor").is_some());
    assert!(registry.get("CrashDumpAnalyzer").is_some());
    assert!(registry.get("ExportPresetManager").is_some());
    assert!(registry.get("DistributedRenderFarm").is_some());
    assert!(registry.get("LivePerformanceSync").is_some());
    assert!(registry.get("CloudPresetHub").is_some());
    assert!(registry.get("FederatedMarketplace").is_some());
    assert!(registry.get("PeerMeshNetwork").is_some());
    assert!(registry.get("ZkPatchVerifier").is_some());
    assert!(registry.get("ContinuousBackupEngine").is_some());
    assert!(registry.get("MidiFileParser").is_some());
    assert!(registry.get("FederatedMixLearner").is_some());
    assert!(registry.get("PresenceSyncServer").is_some());
    assert!(registry.get("IpfsAssetStore").is_some());
    assert!(registry.get("TomlMergeDriver").is_some());
    assert!(registry.get("SessionChatCrypto").is_some());
    assert!(registry.get("WasmPluginRunner").is_some());
    assert!(registry.get("DidAuthenticator").is_some());
    assert!(registry.get("PtpSyncClock").is_some());
    assert!(registry.get("AdaptiveBandwidthManager").is_some());
    assert!(registry.get("MultiTakeCompManager").is_some());
    assert!(registry.get("GrooveQuantizeEngine").is_some());
    assert!(registry.get("CadencePathResolver").is_some());
    assert!(registry.get("PolyphonicVoicePool").is_some());
    assert!(registry.get("MidiMonitorLog").is_some());
    assert!(registry.get("LooperTrackManager").is_some());
    assert!(registry.get("DynamicParameterSmoother").is_some());
    assert!(registry.get("NodeGraphScheduler").is_some());
    assert!(registry.get("VowelCylinderAreaSynthesizer").is_some());
    assert!(registry.get("Rank2TemperamentGenerator").is_some());
    assert!(registry.get("PhaseFlipPolarityInverter").is_some());
    assert!(registry.get("InputOutputGainTrim").is_some());
    assert!(registry.get("Vst3PluginScanner").is_some());
    assert!(registry.get("AudioWorkletExporter").is_some());
    assert!(registry.get("TransientShaperMatrix").is_some());
    assert!(registry.get("DitherNoiseShaper").is_some());
    assert!(registry.get("EqualPowerCrossfader").is_some());
    assert!(registry.get("BesselFilterBank").is_some());
    assert!(registry.get("StateVariableOversampler").is_some());
    assert!(registry.get("ParametricDynamicEq").is_some());
    assert!(registry.get("BinauralHrtfInterpolator").is_some());
    assert!(registry.get("HarmonicTensionFlow").is_some());
    assert!(registry.get("EuclideanPolyrhythmGenerator").is_some());
    assert!(registry.get("MidiPatternHumanizer").is_some());
    assert!(registry.get("SidechainSpectralDucker").is_some());
    assert!(registry.get("LinearPhaseDeEsser").is_some());
    assert!(registry.get("ResonantBodyFormant").is_some());
    assert!(registry.get("ConvolutionCabinetSimulator").is_some());
    assert!(registry.get("StereoGoniometerVisualizer").is_some());
    assert!(registry.get("SurroundDownmixer").is_some());
    assert!(registry.get("PitchDriftModulator").is_some());
    assert!(registry.get("ZeroLatencyBufferBridge").is_some());
    assert!(registry.get("MasterSafetyClipper").is_some());
    assert!(registry.get("CrdtEngine").is_some());
    assert!(registry.get("CloudProjectManager").is_some());
    assert!(registry.get("AutomationRegistry").is_some());
    assert!(registry.get("LooperTrack").is_some());
    assert!(registry.get("GraphSchedule").is_some());
    assert!(registry.get("LuaDspGraphBuilder").is_some());
    assert!(registry.get("LuaLspServer").is_some());
    assert!(registry.get("LuaHotReloader").is_some());
    assert!(registry.get("LuaOscServer").is_some());
    assert!(registry.get("FollowAction").is_some());
    assert!(registry.get("HarmonicContext").is_some());
    assert!(registry.get("IsomorphicVoiceState").is_some());
    assert!(registry.get("FormantTrajectoryPoint").is_some());
    assert!(registry.get("AudioMeshPacket").is_some());
    assert!(registry.get("BellowsWaypoint").is_some());
    assert!(registry.get("BowingWaypoint").is_some());
    assert!(registry.get("KotoWaypoint").is_some());
    assert!(registry.get("GrandPianoWaypoint").is_some());
    assert!(registry.get("ClavinetWaypoint").is_some());
    assert!(registry.get("AtmosphericDensity").is_some());
    assert!(registry.get("AcousticLevitationTrap").is_some());
    assert!(registry.get("NodeGraph").is_some());
    assert!(registry.get("Track").is_some());
    assert!(registry.get("Transport").is_some());
    assert!(registry.get("VoicePool").is_some());
    assert!(registry.get("WavWriter").is_some());
    assert!(registry.get("ParamBus").is_some());
    assert!(registry.get("FixedAudioBuffer").is_some());
    assert!(registry.get("MultichannelAudioBuffer").is_some());
    assert!(registry.get("SequenceTrack").is_some());
    assert!(registry.get("SequenceEvent").is_some());
    assert!(registry.get("StemMetadata").is_some());
    assert!(registry.get("SpectrogramArtConfig").is_some());
    assert!(registry.get("SpectrogramImage").is_some());
    assert!(registry.get("ImageNoteTrigger").is_some());
    assert!(registry.get("SpectralMorphConfig").is_some());
    assert!(registry.get("LatticePointMass").is_some());
    assert!(registry.get("NamModel").is_some());
    assert!(registry.get("OnnxCpuSimdExecutionProvider").is_some());
    assert!(registry.get("DrumStepEvent").is_some());
    assert!(registry.get("MixSuggestions").is_some());
    assert!(registry.get("TranscribedNote").is_some());
    assert!(registry.get("ClipSlot").is_some());
    assert!(registry.get("Scene").is_some());
    assert!(registry.get("CompLane").is_some());
    assert!(registry.get("CompRegion").is_some());
    assert!(registry.get("AudioTake").is_some());
    assert!(registry.get("LooperLayer").is_some());
    assert!(registry.get("SpatialWaypoint").is_some());
    assert!(registry.get("MarkovMutatorConfig").is_some());
    assert!(registry.get("MarkovTransitionMatrix").is_some());
    assert!(registry.get("MarkovStepEvent").is_some());
    assert!(registry.get("AutomationLane").is_some());
    assert!(registry.get("AutomationCurve").is_some());
    assert!(registry.get("AutomationPoint").is_some());
    assert!(registry.get("SessionMarker").is_some());
    assert!(registry.get("ExportPreset").is_some());
    assert!(registry.get("StemExportReport").is_some());
    assert!(registry.get("BatchConvertReport").is_some());
    assert!(registry.get("SfzRegionConfig").is_some());
    assert!(registry.get("MicroCommit").is_some());
    assert!(registry.get("MidiControllerMapping").is_some());
    assert!(registry.get("PluginResourceBudget").is_some());
    assert!(registry.get("GrandPianoBusSnapshot").is_some());
    assert!(registry.get("PipeOrganBusSnapshot").is_some());
    assert!(registry.get("HurdyGurdyBusSnapshot").is_some());
    assert!(registry.get("KotoBusSnapshot").is_some());
    assert!(registry.get("ShakuhachiBusSnapshot").is_some());
    assert!(registry.get("SitarBusSnapshot").is_some());
    assert!(registry.get("GlassArmonicaBusSnapshot").is_some());
    assert!(registry.get("ClavinetBusSnapshot").is_some());
    assert!(registry.get("BellowsBusSnapshot").is_some());
    assert!(registry.get("BreathBusSnapshot").is_some());
    assert!(registry.get("CadenceBusSnapshot").is_some());
    assert!(registry.get("MalletBusSnapshot").is_some());
    assert!(registry.get("PlectrumBusSnapshot").is_some());
    assert!(registry.get("FormantBusSnapshot").is_some());
    assert!(registry.get("EpBusSnapshot").is_some());
    assert!(registry.get("SpringBusSnapshot").is_some());
    assert!(registry.get("RotaryBusSnapshot").is_some());
    assert!(registry.get("AcousticProfile").is_some());
    assert!(registry.get("HarmonicProfile").is_some());
    assert!(registry.get("HammerParameters").is_some());
    assert!(registry.get("FrictionParameters").is_some());
    assert!(registry.get("HurdyGurdyWaypoint").is_some());
    assert!(registry.get("GlassWaypoint").is_some());
    assert!(registry.get("PipeOrganWaypoint").is_some());
    assert!(registry.get("PluckWaypoint").is_some());
    assert!(registry.get("EpWaypoint").is_some());
    assert!(registry.get("MalletWaypoint").is_some());
    assert!(registry.get("RotaryWaypoint").is_some());
    assert!(registry.get("ShakuhachiWaypoint").is_some());
    assert!(registry.get("SitarWaypoint").is_some());
    assert!(registry.get("SpringWaypoint").is_some());
    assert!(registry.get("WoodwindWaypoint").is_some());
    assert!(registry.get("ArticulationBusSnapshot").is_some());
    assert!(registry.get("EmbeddedHardwareConfig").is_some());
    assert!(registry.get("MpeVoiceState").is_some());
    assert!(registry.get("SidechainConfig").is_some());
    assert!(registry.get("HarmonicTension").is_some());
    assert!(registry.get("CadenceEdge").is_some());
    assert!(registry.get("HarmonicPathStep").is_some());
    assert!(registry.get("HexCoordinate").is_some());
    assert!(registry.get("VowelNode").is_some());
    assert!(registry.get("ToneholeParams").is_some());
    assert!(registry.get("RecordingStats").is_some());
    assert!(registry.get("FastPrng").is_some());
    assert!(registry.get("MaskingReport").is_some());
    assert!(registry.get("StrikeContactResult").is_some());
    assert!(registry.get("ToneholeScatteringResult").is_some());
    assert!(registry.get("SampleBuffer").is_some());
    assert!(registry.get("SampleRegion").is_some());
    assert!(registry.get("CloudGrain").is_some());
    assert!(registry.get("AlignmentResult").is_some());
    assert!(registry.get("ExtractedChordEvent").is_some());
    assert!(registry.get("FrictionResult").is_some());
    assert!(registry.get("HarmonicPartial").is_some());
    assert!(registry.get("QuantumTomographyData").is_some());
    assert!(registry.get("WindchestConfig").is_some());
    assert!(registry.get("TunerResult").is_some());
    assert!(registry.get("VisualizerFrameData").is_some());
    assert!(registry.get("VisualizerPreset").is_some());
    assert!(registry.get("SampleMarker").is_some());
    assert!(registry.get("AllocGuard").is_some());
    assert!(registry.get("EegBands").is_some());
    assert!(registry.get("PluginStateConfig").is_some());
    assert!(registry.get("AtomicSpatialSlot").is_some());
    assert!(registry.get("EqBand").is_some());
    assert!(registry.get("Grain").is_some());
    assert!(registry.get("ClapMpeEvent").is_some());
    assert!(registry.get("ClapProcessBuffer").is_some());
    assert!(registry.get("ClapSampleAccurateAutomation").is_some());
    assert!(registry.get("DrumPad").is_some());
    assert!(registry.get("HarmonicNode").is_some());
    assert!(registry.get("MarkovChain2").is_some());
    assert!(registry.get("MidiStepEvent").is_some());
    assert!(registry.get("ModulationAssignment").is_some());
    assert!(registry.get("ModulationTarget").is_some());
    assert!(registry.get("PadTriggerEvent").is_some());
    assert!(registry.get("PatternClip").is_some());
    assert!(registry.get("PluginDescriptor").is_some());
    assert!(registry.get("PluginParamInfo").is_some());
    assert!(registry.get("Position3D").is_some());
    assert!(registry.get("ProcessContext").is_some());
    assert!(registry.get("RenderJob").is_some());
    assert!(registry.get("SampleSlice").is_some());
    assert!(registry.get("SmoothParam").is_some());
    assert!(registry.get("AetherMacroView").is_some());
    assert!(registry.get("Complex32").is_some());
    assert!(registry.get("DrumMacroView").is_some());
    assert!(registry.get("ModulationTargetId").is_some());
    assert!(registry.get("NeuroAffectiveState").is_some());
    assert!(registry.get("OscMappingRule").is_some());
    assert!(registry.get("SamplerMacroView").is_some());
    assert!(registry.get("SimdPolyVoice").is_some());
    assert!(registry.get("SliceMarker").is_some());
    assert!(registry.get("SongSection").is_some());
    assert!(registry.get("SpatialImpulseResponse").is_some());
    assert!(registry.get("WarpMarker").is_some());
    assert!(registry.get("AtomicParam").is_some());
    assert!(registry.get("ControlEvent").is_some());
    assert!(registry.get("SidechainBusId").is_some());
    assert!(registry.get("SpatialVector3D").is_some());
    assert!(registry.get("TimeSignature").is_some());
    assert!(registry.get("AutomationEvent").is_some());
    assert!(registry.get("AutomationEventBuffer").is_some());
    assert!(registry.get("Chord").is_some());
    assert!(registry.get("Scale").is_some());
    assert!(registry.get("Clip").is_some());
    assert!(registry.get("NoteEvent").is_some());
    assert!(registry.get("PatternStep").is_some());
    assert!(registry.get("PatternNote").is_some());
    assert!(registry.get("TimelineEvaluation").is_some());
    assert!(registry.get("MidiLogEntry").is_some());
    assert!(registry.get("ParamId").is_some());
    assert!(registry.get("SampleHash").is_some());
    assert!(registry.get("RenderResult").is_some());
    assert!(registry.get("Edge").is_some());
    assert!(registry.get("Frame").is_some());
    assert!(registry.get("EpGesturePattern").is_some());
    assert!(registry.get("BowingGesturePattern").is_some());
    assert!(registry.get("KotoGesturePattern").is_some());
    assert!(registry.get("MalletGesturePattern").is_some());
    assert!(registry.get("PipeOrganGesturePattern").is_some());
    assert!(registry.get("RotaryGesturePattern").is_some());
    assert!(registry.get("ShakuhachiGesturePattern").is_some());
    assert!(registry.get("SitarGesturePattern").is_some());
    assert!(registry.get("SpringGesturePattern").is_some());
    assert!(registry.get("WoodwindGesturePattern").is_some());
assert!(registry.get("BellowsGesturePattern").is_some());
    assert!(registry.get("ClavinetGesturePattern").is_some());
    assert!(registry.get("GlassGesturePattern").is_some());
    assert!(registry.get("GrandPianoGesturePattern").is_some());
    assert!(registry.get("HurdyGurdyGesturePattern").is_some());
    assert!(registry.get("PluckGesturePattern").is_some());
    assert!(registry.get("ArmonicaProfile").is_some());
    assert!(registry.get("CassottoProfile").is_some());
    assert!(registry.get("ChienMaterialProfile").is_some());
    assert!(registry.get("ClavinetProfile").is_some());
    assert!(registry.get("ElectricPianoProfile").is_some());
    assert!(registry.get("FreeReedProfile").is_some());
    assert!(registry.get("CrystalMaterialProfile").is_some());
    assert!(registry.get("CrystallineLattice").is_some());
    assert!(registry.get("AcousticMaterial").is_some());
    assert!(registry.get("AudioGraphBenchmarkConfig").is_some());
    assert!(registry.get("AudioGraphBenchmarkReport").is_some());
    assert!(registry.get("ExportSettings").is_some());
    assert!(registry.get("DevicePreset").is_some());
    assert!(registry.get("CrashAnalysisResult").is_some());
    assert!(registry.get("CollaborationSession").is_some());
    assert!(registry.get("PipeOrganProfile").is_some());
    assert!(registry.get("PlateReverbProfile").is_some());
    assert!(registry.get("PluckedInstrumentProfile").is_some());
    assert!(registry.get("MembraneInstrumentProfile").is_some());
    assert!(registry.get("MeshMaterialProfile").is_some());
    assert!(registry.get("RotaryCabinetModel").is_some());
    assert!(registry.get("ShakuhachiProfile").is_some());
    assert!(registry.get("SitarProfile").is_some());
    assert!(registry.get("SoundboardProfile").is_some());
    assert!(registry.get("SpringLatticeProfile").is_some());
    assert!(registry.get("TonewheelOrganProfile").is_some());
    assert!(registry.get("WoodwindInstrumentProfile").is_some());
    assert!(registry.get("BellowsInterpolationCurve").is_some());
    assert!(registry.get("BowingInterpolationCurve").is_some());
    assert!(registry.get("ClavinetInterpolationCurve").is_some());
    assert!(registry.get("EpInterpolationCurve").is_some());
    assert!(registry.get("GlassInterpolationCurve").is_some());
    assert!(registry.get("GrandPianoInterpolationCurve").is_some());
    assert!(registry.get("HurdyGurdyInterpolationCurve").is_some());
    assert!(registry.get("KotoInterpolationCurve").is_some());
    assert!(registry.get("MalletInterpolationCurve").is_some());
    assert!(registry.get("MacroRackLuaDevice").is_some());
    assert!(registry.get("LuaDspContext").is_some());
    assert!(registry.get("LuaRandomEngine").is_some());
    assert!(registry.get("LuaMidiMessage").is_some());
    assert!(registry.get("LuaDspObjectMetatable").is_some());
    assert!(registry.get("LuaEventSystem").is_some());
    assert!(registry.get("LuaTimer").is_some());
    assert!(registry.get("LuaUiWidget").is_some());
    assert!(registry.get("LuaUiLayout").is_some());
    assert!(registry.get("LuaPainterBuffer").is_some());
    assert!(registry.get("LuaMidiInputSubscriber").is_some());
    assert!(registry.get("LuaCoroutinePattern").is_some());
    assert!(registry.get("BlockSizePerformance").is_some());
    assert!(registry.get("PeerNode").is_some());
    assert!(registry.get("ZkPatchProof").is_some());
    assert!(registry.get("UserPresence").is_some());
    assert!(registry.get("RenderTaskBlock").is_some());
    assert!(registry.get("MarketplacePreset").is_some());
    assert!(registry.get("PermissionToken").is_some());
    assert!(registry.get("PendingMutation").is_some());
    assert!(registry.get("OfflineCrdtQueue").is_some());
    assert!(registry.get("ToneholeGrid").is_some());
    assert!(registry.get("IdiophoneResonator").is_some());
    assert!(registry.get("AutoUpdateChecker").is_some());
    assert!(registry.get("UpdateInstaller").is_some());
    assert!(registry.get("RollbackManager").is_some());
    assert!(registry.get("CrashReporter").is_some());
    assert!(registry.get("UsageTelemetry").is_some());
    assert!(registry.get("PrivacyPolicyProvider").is_some());
    assert!(registry.get("GdprComplianceExporter").is_some());
    assert!(registry.get("FactoryResetManager").is_some());
    assert!(registry.get("SystemSettingsBackup").is_some());
    assert!(registry.get("PersistentLogin").is_some());
    assert!(registry.get("CloudRenderJobSubmission").is_some());
    assert!(registry.get("CloudCollaborationWorkspace").is_some());
    assert!(registry.get("OfflineModeIndicator").is_some());
    assert!(registry.get("CloudStorageQuota").is_some());
    assert!(registry.get("PluginRatingReview").is_some());
    assert!(registry.get("PluginBlacklist").is_some());
    assert!(registry.get("PluginConflictDetector").is_some());
    assert!(registry.get("PluginStateBackup").is_some());
    assert!(registry.get("OfflineLocalProjectManager").is_some());
    assert!(registry.get("GrandPianoArticulation").is_some());
    assert!(registry.get("PipeOrganArticulation").is_some());
    assert!(registry.get("MalletArticulation").is_some());
    assert!(registry.get("EpModelType").is_some());
    assert!(registry.get("PluckArticulation").is_some());
    assert!(registry.get("RotaryArticulation").is_some());
    assert!(registry.get("ShakuhachiArticulation").is_some());
    assert!(registry.get("SitarArticulation").is_some());
    assert!(registry.get("SpringArticulation").is_some());
    assert!(registry.get("WoodwindArticulation").is_some());
    assert!(registry.get("JiBridgeProfile").is_some());
    assert!(registry.get("AutoWahFilterMode").is_some());
    assert!(registry.get("DrumClassClassification").is_some());
    assert!(registry.get("PluginMarketplace").is_some());
    assert!(registry.get("PluginLatencyCompensationEngine").is_some());
    assert!(registry.get("LuaScriptMarketplace").is_some());
    assert!(registry.get("LuaScriptSafeMode").is_some());
    assert!(registry.get("RemoteUserCursorSync").is_some());
    assert!(registry.get("VoiceMemoAnnotation").is_some());
    assert!(registry.get("SynchronizedPatternTrigger").is_some());
    assert!(registry.get("WebRtcMidiPacket").is_some());
    assert!(registry.get("BowedInstrumentModel").is_some());
    assert!(registry.get("ClavinetPickupSelector").is_some());
    assert!(registry.get("RosinFrictionProfile").is_some());
    assert!(registry.get("GlottalVoicingMode").is_some());
    assert!(registry.get("GrandPianoAcousticProfile").is_some());
    assert!(registry.get("GrainWindowProfile").is_some());
    assert!(registry.get("HammerExcitationType").is_some());
    assert!(registry.get("HornFlareContour").is_some());
    assert!(registry.get("HornAcousticMute").is_some());
    assert!(registry.get("RagaScaleTuning").is_some());
    assert!(registry.get("JawariCurvatureProfile").is_some());
    assert!(registry.get("KotoTuningSchema").is_some());
    assert!(registry.get("ModalExcitationEngine").is_some());
    assert!(registry.get("RoomAcousticModel").is_some());
    assert!(registry.get("WallAcousticMaterial").is_some());
    assert!(registry.get("SpectrogramFrequencyMapping").is_some());
    assert!(registry.get("SpectrogramColorMapping").is_some());
    assert!(registry.get("SpectralMorphMode").is_some());
    assert!(registry.get("VacuumTubeTopology").is_some());
    assert!(registry.get("BrassInstrumentProfile").is_some());
    assert!(registry.get("WaveguideMeshBoundary").is_some());
    assert!(registry.get("AudioStylePreset").is_some());
    assert!(registry.get("AutoWahDirection").is_some());
    assert!(registry.get("BellowsArticulation").is_some());
    assert!(registry.get("BowingTechnique").is_some());
    assert!(registry.get("BrassInstrumentType").is_some());
    assert!(registry.get("CadenceResolutionType").is_some());
    assert!(registry.get("ChordQuality").is_some());
    assert!(registry.get("ClavinetArticulation").is_some());
    assert!(registry.get("ClavinetPickupMode").is_some());
    assert!(registry.get("ConsoleMode").is_some());
    assert!(registry.get("DynamicsBandIndex").is_some());
    assert!(registry.get("EpArticulation").is_some());
    assert!(registry.get("GlassArticulation").is_some());
    assert!(registry.get("HurdyGurdyArticulation").is_some());
    assert!(registry.get("HurdyGurdyProfile").is_some());
    assert!(registry.get("KotoArticulation").is_some());
    assert!(registry.get("PercussionHarmonic").is_some());
    assert!(registry.get("RecordingFormat").is_some());
    assert!(registry.get("TapeSpeed").is_some());
    assert!(registry.get("VibratoChorusMode").is_some());
    assert!(registry.get("VowelPreset").is_some());
}

#[test]
fn test_dsp_node_descriptor_parameters_and_macro_bindings() {
    let registry = DspNodeRegistry::new();
    let aether = registry.get("AetherSynth").expect("AetherSynth must exist");
    assert_eq!(aether.category, DspNodeCategory::CompositeSynth);
    assert!(aether.params.len() >= 3);

    let shakuhachi = registry.get("ShakuhachiModel").expect("ShakuhachiModel must exist");
    assert_eq!(shakuhachi.category, DspNodeCategory::AcousticPhysicalModel);
    assert!(shakuhachi.params.len() >= 3);
}

#[test]
fn test_dsp_category_colors_and_icons() {
    let cat = DspNodeCategory::Oscillator;
    assert_eq!(cat.icon(), "🌊");
    assert_eq!(cat.color_rgb(), (56, 189, 248));

    let cat_phys = DspNodeCategory::AcousticPhysicalModel;
    assert_eq!(cat_phys.icon(), "🎻");
    assert_eq!(cat_phys.color_rgb(), (245, 158, 11));
}

#[test]
fn test_tier117_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1072, "Inventory must contain at least 1072 DSP modules");
    assert!(registry.list_all().len() >= 1072, "Registry list_all must be >= 1072");

    let tier117_nodes = [
        ("ToneholeGrid", DspNodeCategory::AcousticPhysicalModel),
        ("IdiophoneResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AutoUpdateChecker", DspNodeCategory::Utility),
        ("UpdateInstaller", DspNodeCategory::Utility),
        ("RollbackManager", DspNodeCategory::Utility),
        ("CrashReporter", DspNodeCategory::Utility),
        ("UsageTelemetry", DspNodeCategory::Utility),
        ("PrivacyPolicyProvider", DspNodeCategory::Utility),
        ("GdprComplianceExporter", DspNodeCategory::Utility),
        ("FactoryResetManager", DspNodeCategory::Utility),
        ("SystemSettingsBackup", DspNodeCategory::Utility),
        ("PersistentLogin", DspNodeCategory::Utility),
        ("CloudRenderJobSubmission", DspNodeCategory::Utility),
        ("CloudCollaborationWorkspace", DspNodeCategory::Utility),
        ("OfflineModeIndicator", DspNodeCategory::Utility),
        ("CloudStorageQuota", DspNodeCategory::Utility),
        ("PluginRatingReview", DspNodeCategory::Utility),
        ("PluginBlacklist", DspNodeCategory::Utility),
        ("PluginConflictDetector", DspNodeCategory::Utility),
        ("PluginStateBackup", DspNodeCategory::Utility),
        ("OfflineLocalProjectManager", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier117_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("toneholegrid"), Some("ToneholeGrid"));
    assert_eq!(DspNodeRegistry::normalize_type_name("woodwindtoneholes"), Some("ToneholeGrid"));
    assert_eq!(DspNodeRegistry::normalize_type_name("idiophoneresonator"), Some("IdiophoneResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("marimbawood"), Some("IdiophoneResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("autoupdatechecker"), Some("AutoUpdateChecker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("updatechecker"), Some("AutoUpdateChecker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("updateinstaller"), Some("UpdateInstaller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rollbackmanager"), Some("RollbackManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crashreporter"), Some("CrashReporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("usagetelemetry"), Some("UsageTelemetry"));
    assert_eq!(DspNodeRegistry::normalize_type_name("privacypolicyprovider"), Some("PrivacyPolicyProvider"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gdprcomplianceexporter"), Some("GdprComplianceExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("factoryresetmanager"), Some("FactoryResetManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("systemsettingsbackup"), Some("SystemSettingsBackup"));
    assert_eq!(DspNodeRegistry::normalize_type_name("persistentlogin"), Some("PersistentLogin"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudrenderjobsubmission"), Some("CloudRenderJobSubmission"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudcollaborationworkspace"), Some("CloudCollaborationWorkspace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("offlinemodeindicator"), Some("OfflineModeIndicator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudstoragequota"), Some("CloudStorageQuota"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginratingreview"), Some("PluginRatingReview"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginblacklist"), Some("PluginBlacklist"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginconflictdetector"), Some("PluginConflictDetector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginstatebackup"), Some("PluginStateBackup"));
    assert_eq!(DspNodeRegistry::normalize_type_name("offlinelocalprojectmanager"), Some("OfflineLocalProjectManager"));
}

#[test]
fn test_tier118_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1093, "Inventory must contain at least 1093 DSP modules");
    assert!(registry.list_all().len() >= 1093, "Registry list_all must be >= 1093");

    let tier118_nodes = [
        ("GrandPianoArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("PipeOrganArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("MalletArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("EpModelType", DspNodeCategory::AcousticPhysicalModel),
        ("PluckArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("RotaryArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("ShakuhachiArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("SitarArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("SpringArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("WoodwindArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("JiBridgeProfile", DspNodeCategory::AcousticPhysicalModel),
        ("AutoWahFilterMode", DspNodeCategory::FilterEq),
        ("DrumClassClassification", DspNodeCategory::NeuralAi),
        ("PluginMarketplace", DspNodeCategory::Utility),
        ("PluginLatencyCompensationEngine", DspNodeCategory::Utility),
        ("LuaScriptMarketplace", DspNodeCategory::Utility),
        ("LuaScriptSafeMode", DspNodeCategory::Utility),
        ("RemoteUserCursorSync", DspNodeCategory::Utility),
        ("VoiceMemoAnnotation", DspNodeCategory::Utility),
        ("SynchronizedPatternTrigger", DspNodeCategory::Utility),
        ("WebRtcMidiPacket", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier118_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("grandpianoarticulation"), Some("GrandPianoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grandpianoaction"), Some("GrandPianoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pianotouch"), Some("GrandPianoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pianohammeraction"), Some("GrandPianoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pipeorganarticulation"), Some("PipeOrganArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("organtracker"), Some("PipeOrganArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pipeorganaction"), Some("PipeOrganArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("organchif"), Some("PipeOrganArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("malletarticulation"), Some("MalletArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("malletdynamics"), Some("MalletArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("malletphysics"), Some("MalletArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("idiophonestrike"), Some("MalletArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("epmodeltype"), Some("EpModelType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vintageepiano"), Some("EpModelType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rhodesmodel"), Some("EpModelType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("epianovoice"), Some("EpModelType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluckarticulation"), Some("PluckArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("plectrumdynamics"), Some("PluckArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("guitarpluck"), Some("PluckArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pickscrape"), Some("PluckArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotaryarticulation"), Some("RotaryArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lesliedynamics"), Some("RotaryArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotorspeed"), Some("RotaryArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotaryrotoraccel"), Some("RotaryArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("shakuhachiarticulation"), Some("ShakuhachiArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("shakuhachimerikari"), Some("ShakuhachiArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("muraiki"), Some("ShakuhachiArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nayashi"), Some("ShakuhachiArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sitararticulation"), Some("SitarArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sitarmeend"), Some("SitarArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sitarchikari"), Some("SitarArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tarabresonance"), Some("SitarArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springarticulation"), Some("SpringArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springtankkick"), Some("SpringArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springboing"), Some("SpringArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springdispersion"), Some("SpringArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("woodwindarticulation"), Some("WoodwindArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("woodwindembouchure"), Some("WoodwindArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fluttertongue"), Some("WoodwindArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("overblowregister"), Some("WoodwindArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jibridgeprofile"), Some("JiBridgeProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jibridgematerial"), Some("JiBridgeProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zitherbridge"), Some("JiBridgeProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("guzhengbridge"), Some("JiBridgeProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("autowahfiltermode"), Some("AutoWahFilterMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("autowahmode"), Some("AutoWahFilterMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("envelopesweepfilter"), Some("AutoWahFilterMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocalwahfilter"), Some("AutoWahFilterMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("drumclassclassification"), Some("DrumClassClassification"));
    assert_eq!(DspNodeRegistry::normalize_type_name("drumclassifier"), Some("DrumClassClassification"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aidrumtagger"), Some("DrumClassClassification"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldrumclass"), Some("DrumClassClassification"));
    assert_eq!(DspNodeRegistry::normalize_type_name("communitypluginmarket"), Some("PluginMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginmarket"), Some("PluginMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginstore"), Some("PluginMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clapmarketplace"), Some("PluginMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginlatencycompensationengine"), Some("PluginLatencyCompensationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pdcengine"), Some("PluginLatencyCompensationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("latencycompensation"), Some("PluginLatencyCompensationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginlookaheadalign"), Some("PluginLatencyCompensationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptmarketplace"), Some("LuaScriptMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luamarket"), Some("LuaScriptMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascripts"), Some("LuaScriptMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luahub"), Some("LuaScriptMarketplace"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptsafemode"), Some("LuaScriptSafeMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luasandbox"), Some("LuaScriptSafeMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luasafemode"), Some("LuaScriptSafeMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luagovernor"), Some("LuaScriptSafeMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("remoteusercursorsync"), Some("RemoteUserCursorSync"));
    assert_eq!(DspNodeRegistry::normalize_type_name("remotecursors"), Some("RemoteUserCursorSync"));
    assert_eq!(DspNodeRegistry::normalize_type_name("collabcursors"), Some("RemoteUserCursorSync"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multiuserpointers"), Some("RemoteUserCursorSync"));
    assert_eq!(DspNodeRegistry::normalize_type_name("voicememoannotation"), Some("VoiceMemoAnnotation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("voicememo"), Some("VoiceMemoAnnotation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timelinevoicenote"), Some("VoiceMemoAnnotation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiomemo"), Some("VoiceMemoAnnotation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("synchronizedpatterntrigger"), Some("SynchronizedPatternTrigger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("patterntrigger"), Some("SynchronizedPatternTrigger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantizedcliplaunch"), Some("SynchronizedPatternTrigger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("networkphaselock"), Some("SynchronizedPatternTrigger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("webrtcmidipacket"), Some("WebRtcMidiPacket"));
    assert_eq!(DspNodeRegistry::normalize_type_name("webrtcmidi"), Some("WebRtcMidiPacket"));
    assert_eq!(DspNodeRegistry::normalize_type_name("networkmidi"), Some("WebRtcMidiPacket"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mpedatachannel"), Some("WebRtcMidiPacket"));
}

#[test]
fn test_tier119_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1114, "Inventory must contain at least 1114 DSP modules");
    assert!(registry.list_all().len() >= 1114, "Registry list_all must be >= 1114");

    let tier119_nodes = [
        ("BowedInstrumentModel", DspNodeCategory::AcousticPhysicalModel),
        ("ClavinetPickupSelector", DspNodeCategory::AcousticPhysicalModel),
        ("RosinFrictionProfile", DspNodeCategory::AcousticPhysicalModel),
        ("GlottalVoicingMode", DspNodeCategory::AcousticPhysicalModel),
        ("GrandPianoAcousticProfile", DspNodeCategory::AcousticPhysicalModel),
        ("GrainWindowProfile", DspNodeCategory::CompositeSynth),
        ("HammerExcitationType", DspNodeCategory::AcousticPhysicalModel),
        ("HornFlareContour", DspNodeCategory::AcousticPhysicalModel),
        ("HornAcousticMute", DspNodeCategory::AcousticPhysicalModel),
        ("RagaScaleTuning", DspNodeCategory::Modulation),
        ("JawariCurvatureProfile", DspNodeCategory::AcousticPhysicalModel),
        ("KotoTuningSchema", DspNodeCategory::Modulation),
        ("ModalExcitationEngine", DspNodeCategory::AcousticPhysicalModel),
        ("RoomAcousticModel", DspNodeCategory::SpatialSurround),
        ("WallAcousticMaterial", DspNodeCategory::SpatialSurround),
        ("SpectrogramFrequencyMapping", DspNodeCategory::SpectralResynthesis),
        ("SpectrogramColorMapping", DspNodeCategory::SpectralResynthesis),
        ("SpectralMorphMode", DspNodeCategory::FilterEq),
        ("VacuumTubeTopology", DspNodeCategory::DistortionSaturation),
        ("BrassInstrumentProfile", DspNodeCategory::AcousticPhysicalModel),
        ("WaveguideMeshBoundary", DspNodeCategory::AcousticPhysicalModel),
    ];

    for (name, expected_cat) in tier119_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("bowedinstrumentmodel"), Some("BowedInstrumentModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowedinstrument"), Some("BowedInstrumentModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("violincello"), Some("BowedInstrumentModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("orchestralstringbody"), Some("BowedInstrumentModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetpickupselector"), Some("ClavinetPickupSelector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetpickups"), Some("ClavinetPickupSelector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetd6pickups"), Some("ClavinetPickupSelector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetswitches"), Some("ClavinetPickupSelector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rosinfrictionprofile"), Some("RosinFrictionProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rosintype"), Some("RosinFrictionProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowrosin"), Some("RosinFrictionProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sticksliprosin"), Some("RosinFrictionProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glottalvoicingmode"), Some("GlottalVoicingMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocalvoicingmode"), Some("GlottalVoicingMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phonationmode"), Some("GlottalVoicingMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glottalflowregime"), Some("GlottalVoicingMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grandpianoacousticprofile"), Some("GrandPianoAcousticProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grandpianoprofile"), Some("GrandPianoAcousticProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pianosoundboardprofile"), Some("GrandPianoAcousticProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertgrandscale"), Some("GrandPianoAcousticProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grainwindowprofile"), Some("GrainWindowProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grainwindowtype"), Some("GrainWindowProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grainenvelope"), Some("GrainWindowProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularwindowing"), Some("GrainWindowProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hammerexcitationtype"), Some("HammerExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("excitationtype"), Some("HammerExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("strikermaterial"), Some("HammerExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("strikermechanics"), Some("HammerExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hornflarecontour"), Some("HornFlareContour"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hornflarepreset"), Some("HornFlareContour"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brassbellcontour"), Some("HornFlareContour"));
    assert_eq!(DspNodeRegistry::normalize_type_name("websterhornflare"), Some("HornFlareContour"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hornacousticmute"), Some("HornAcousticMute"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hornmutetype"), Some("HornAcousticMute"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brassmute"), Some("HornAcousticMute"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonmute"), Some("HornAcousticMute"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ragascaletuning"), Some("RagaScaleTuning"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ragascale"), Some("RagaScaleTuning"));
    assert_eq!(DspNodeRegistry::normalize_type_name("shrutituning"), Some("RagaScaleTuning"));
    assert_eq!(DspNodeRegistry::normalize_type_name("indianmicrotonalscale"), Some("RagaScaleTuning"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jawaricurvatureprofile"), Some("JawariCurvatureProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jawariprofile"), Some("JawariCurvatureProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sitarjawarislope"), Some("JawariCurvatureProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grazingobstacleprofile"), Some("JawariCurvatureProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kototuningschema"), Some("KotoTuningSchema"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hirajoshituning"), Some("KotoTuningSchema"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kotomodalscale"), Some("KotoTuningSchema"));
    assert_eq!(DspNodeRegistry::normalize_type_name("japanesekototuning"), Some("KotoTuningSchema"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalexcitationengine"), Some("ModalExcitationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalexcitationtype"), Some("ModalExcitationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalexciter"), Some("ModalExcitationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonatorexciter"), Some("ModalExcitationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("roomacousticmodel"), Some("RoomAcousticModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spatialroomacoustics"), Some("RoomAcousticModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("imagesourcemodel"), Some("RoomAcousticModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("shoeboxacousticmodel"), Some("RoomAcousticModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wallacousticmaterial"), Some("WallAcousticMaterial"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wallmaterial"), Some("WallAcousticMaterial"));
    assert_eq!(DspNodeRegistry::normalize_type_name("surfaceabsorption"), Some("WallAcousticMaterial"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticmaterialpreset"), Some("WallAcousticMaterial"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogramfrequencymapping"), Some("SpectrogramFrequencyMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("frequencymapping"), Some("SpectrogramFrequencyMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogramscale"), Some("SpectrogramFrequencyMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("imagespectrogrambins"), Some("SpectrogramFrequencyMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogramcolormapping"), Some("SpectrogramColorMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("colormappingmode"), Some("SpectrogramColorMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogrampixeldecoder"), Some("SpectrogramColorMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("imageamplitudedecoder"), Some("SpectrogramColorMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralmorphmode"), Some("SpectralMorphMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralmorphalgo"), Some("SpectralMorphMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crosssynthesismorph"), Some("SpectralMorphMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralinterpolator"), Some("SpectralMorphMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vacuumtubetopology"), Some("VacuumTubeTopology"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubetopology"), Some("VacuumTubeTopology"));
    assert_eq!(DspNodeRegistry::normalize_type_name("valvetopology"), Some("VacuumTubeTopology"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vacuumtubesaturation"), Some("VacuumTubeTopology"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brassinstrumentprofile"), Some("BrassInstrumentProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brassinstrumenttype"), Some("BrassInstrumentProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brassboreprofile"), Some("BrassInstrumentProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brasswaveguide"), Some("BrassInstrumentProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveguidemeshboundary"), Some("WaveguideMeshBoundary"));
    assert_eq!(DspNodeRegistry::normalize_type_name("meshboundarytype"), Some("WaveguideMeshBoundary"));
    assert_eq!(DspNodeRegistry::normalize_type_name("drummembraneboundary"), Some("WaveguideMeshBoundary"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveguidemeshreflection"), Some("WaveguideMeshBoundary"));
}

#[test]
fn test_tier120_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1135, "Inventory must contain at least 1135 DSP modules");
    assert!(registry.list_all().len() >= 1135, "Registry list_all must be >= 1135");

    let tier120_nodes = [
        ("AudioStylePreset", DspNodeCategory::NeuralAi),
        ("AutoWahDirection", DspNodeCategory::FilterEq),
        ("BellowsArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("BowingTechnique", DspNodeCategory::AcousticPhysicalModel),
        ("BrassInstrumentType", DspNodeCategory::AcousticPhysicalModel),
        ("CadenceResolutionType", DspNodeCategory::Modulation),
        ("ChordQuality", DspNodeCategory::Modulation),
        ("ClavinetArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("ClavinetPickupMode", DspNodeCategory::AcousticPhysicalModel),
        ("ConsoleMode", DspNodeCategory::DistortionSaturation),
        ("DynamicsBandIndex", DspNodeCategory::DynamicsMaster),
        ("EpArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("GlassArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("HurdyGurdyArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("HurdyGurdyProfile", DspNodeCategory::AcousticPhysicalModel),
        ("KotoArticulation", DspNodeCategory::AcousticPhysicalModel),
        ("PercussionHarmonic", DspNodeCategory::AcousticPhysicalModel),
        ("RecordingFormat", DspNodeCategory::Utility),
        ("TapeSpeed", DspNodeCategory::DistortionSaturation),
        ("VibratoChorusMode", DspNodeCategory::Modulation),
        ("VowelPreset", DspNodeCategory::AcousticPhysicalModel),
    ];

    for (name, expected_cat) in tier120_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("audiostylepreset"), Some("AudioStylePreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiostyle"), Some("AudioStylePreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralaudiostyle"), Some("AudioStylePreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("styletypetarget"), Some("AudioStylePreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("autowahdirection"), Some("AutoWahDirection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wahdirection"), Some("AutoWahDirection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("envelopesweepdirection"), Some("AutoWahDirection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wahsweepdir"), Some("AutoWahDirection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bellowsarticulation"), Some("BellowsArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bellowsairflow"), Some("BellowsArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bellowspushpull"), Some("BellowsArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bellowsstroke"), Some("BellowsArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowingtechnique"), Some("BowingTechnique"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowingmode"), Some("BowingTechnique"));
    assert_eq!(DspNodeRegistry::normalize_type_name("violintechnique"), Some("BowingTechnique"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowarticulation"), Some("BowingTechnique"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticbrasstype"), Some("BrassInstrumentType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brasstype"), Some("BrassInstrumentType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveguidebrasstype"), Some("BrassInstrumentType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brassmodeltype"), Some("BrassInstrumentType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cadenceresolutiontype"), Some("CadenceResolutionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cadencetype"), Some("CadenceResolutionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cadencemode"), Some("CadenceResolutionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cadenceresolution"), Some("CadenceResolutionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chordquality"), Some("ChordQuality"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chordvoicing"), Some("ChordQuality"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalchordquality"), Some("ChordQuality"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chordharmonicquality"), Some("ChordQuality"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetarticulation"), Some("ClavinetArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinettangent"), Some("ClavinetArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetmechanics"), Some("ClavinetArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinettouch"), Some("ClavinetArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetpickupmode"), Some("ClavinetPickupMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetpickupsetting"), Some("ClavinetPickupMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavpickupmode"), Some("ClavinetPickupMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("d6pickupmode"), Some("ClavinetPickupMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("consolemode"), Some("ConsoleMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("consoletopology"), Some("ConsoleMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mixingconsoletype"), Some("ConsoleMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("deskmodel"), Some("ConsoleMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicsbandindex"), Some("DynamicsBandIndex"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandindex"), Some("DynamicsBandIndex"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crossoverbandindex"), Some("DynamicsBandIndex"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicscrossover"), Some("DynamicsBandIndex"));
    assert_eq!(DspNodeRegistry::normalize_type_name("eparticulation"), Some("EpArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("electricpianoarticulation"), Some("EpArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tinearticulation"), Some("EpArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("eptouch"), Some("EpArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glassarticulation"), Some("GlassArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crystalrubbing"), Some("GlassArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glassrubbing"), Some("GlassArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glassbowltouch"), Some("GlassArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hurdygurdyarticulation"), Some("HurdyGurdyArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("viellegesture"), Some("HurdyGurdyArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crankarticulation"), Some("HurdyGurdyArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chienarticulation"), Some("HurdyGurdyArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hurdygurdyprofile"), Some("HurdyGurdyProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vielleprofile"), Some("HurdyGurdyProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hurdygurdyarchetype"), Some("HurdyGurdyProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("organistrumprofile"), Some("HurdyGurdyProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kotoarticulation"), Some("KotoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kotopluck"), Some("KotoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kototsume"), Some("KotoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("oshidepitchbend"), Some("KotoArticulation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("percussionharmonic"), Some("PercussionHarmonic"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tonewheelpercussion"), Some("PercussionHarmonic"));
    assert_eq!(DspNodeRegistry::normalize_type_name("organpercussion"), Some("PercussionHarmonic"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicpercussion"), Some("PercussionHarmonic"));
    assert_eq!(DspNodeRegistry::normalize_type_name("recordingformat"), Some("RecordingFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audioexportformat"), Some("RecordingFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionrecordingformat"), Some("RecordingFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavflacformat"), Some("RecordingFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapespeed"), Some("TapeSpeed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("reeltoreelspeed"), Some("TapeSpeed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analoguetapespeed"), Some("TapeSpeed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapetransportspeed"), Some("TapeSpeed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vibratochorusmode"), Some("VibratoChorusMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scannervibrato"), Some("VibratoChorusMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hammondvibrato"), Some("VibratoChorusMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scannerchorus"), Some("VibratoChorusMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vowelpreset"), Some("VowelPreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ipavowelpreset"), Some("VowelPreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantvowel"), Some("VowelPreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocaltractvowel"), Some("VowelPreset"));
}

#[test]
fn test_tier121_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1151, "Inventory must contain at least 1151 DSP modules");
    assert!(registry.list_all().len() >= 1151, "Registry list_all must be >= 1151");

    let tier121_nodes = [
        ("ChannelLayout", DspNodeCategory::Utility),
        ("ClavinetPickupSelection", DspNodeCategory::AcousticPhysicalModel),
        ("DistortionType", DspNodeCategory::DistortionSaturation),
        ("DitherType", DspNodeCategory::DynamicsMaster),
        ("EnvState", DspNodeCategory::Modulation),
        ("ExpressionCurveType", DspNodeCategory::Modulation),
        ("FilterType", DspNodeCategory::FilterEq),
        ("GpioEvent", DspNodeCategory::Utility),
        ("IdiophoneInstrumentProfile", DspNodeCategory::AcousticPhysicalModel),
        ("IpaVowel", DspNodeCategory::AcousticPhysicalModel),
        ("KSystemScale", DspNodeCategory::DynamicsMaster),
        ("KotoProfile", DspNodeCategory::AcousticPhysicalModel),
        ("LaunchpadMode", DspNodeCategory::Utility),
        ("LfoShape", DspNodeCategory::Modulation),
        ("LoopMode", DspNodeCategory::SamplerSlicer),
        ("MalletMaterial", DspNodeCategory::AcousticPhysicalModel),
    ];

    for (name, expected_cat) in tier121_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("channellayout"), Some("ChannelLayout"));
    assert_eq!(DspNodeRegistry::normalize_type_name("surroundchannellayout"), Some("ChannelLayout"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clavinetpickupselection"), Some("ClavinetPickupSelection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("distortiontype"), Some("DistortionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dithertype"), Some("DitherType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("envstate"), Some("EnvState"));
    assert_eq!(DspNodeRegistry::normalize_type_name("expressioncurvetype"), Some("ExpressionCurveType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("filtertype"), Some("FilterType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gpioevent"), Some("GpioEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("idiophoneinstrumentprofile"), Some("IdiophoneInstrumentProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ipavowel"), Some("IpaVowel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ksystemscale"), Some("KSystemScale"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kotoprofile"), Some("KotoProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("launchpadmode"), Some("LaunchpadMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lfoshape"), Some("LfoShape"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loopmode"), Some("LoopMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("malletmaterial"), Some("MalletMaterial"));

    // Test alias mappings
    assert_eq!(DspNodeRegistry::normalize_type_name("drumclass"), Some("DrumClassClassification"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mpeevent"), Some("ClapMpeEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nativeaudiodriver"), Some("NativeAudioDriverTuner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("velocitycurve"), Some("MidiVelocityCurveShaper"));
}

#[test]
fn test_tier122_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1172, "Inventory must contain at least 1172 DSP modules");
    assert!(registry.list_all().len() >= 1172, "Registry list_all must be >= 1172");

    let tier122_nodes = [
        ("ModalMaterialPreset", DspNodeCategory::AcousticPhysicalModel),
        ("ModulationCurve", DspNodeCategory::Modulation),
        ("ModulationSourceId", DspNodeCategory::Modulation),
        ("NoiseType", DspNodeCategory::Oscillator),
        ("PluginFormat", DspNodeCategory::Utility),
        ("RingModWaveform", DspNodeCategory::Modulation),
        ("RotarySpeed", DspNodeCategory::Modulation),
        ("ShakuhachiFluteLength", DspNodeCategory::AcousticPhysicalModel),
        ("SidechainTapPoint", DspNodeCategory::Utility),
        ("SliceAlgorithm", DspNodeCategory::SamplerSlicer),
        ("SongSectionType", DspNodeCategory::Utility),
        ("TargetCurve", DspNodeCategory::NeuralAi),
        ("TargetTimbre", DspNodeCategory::NeuralAi),
        ("TuningRemapMode", DspNodeCategory::Utility),
        ("MidiEvent", DspNodeCategory::Utility),
        ("HarmonicFunction", DspNodeCategory::Utility),
        ("IsomorphicLayoutType", DspNodeCategory::Utility),
        ("LaunchQuantize", DspNodeCategory::Utility),
        ("LaunchMode", DspNodeCategory::Utility),
        ("FollowActionType", DspNodeCategory::Utility),
        ("LooperState", DspNodeCategory::SamplerSlicer),
    ];

    for (name, expected_cat) in tier122_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("modalmaterialpreset"), Some("ModalMaterialPreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalsynthesisprofile"), Some("ModalMaterialPreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modulationcurve"), Some("ModulationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modcurve"), Some("ModulationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modulationsourceid"), Some("ModulationSourceId"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modulationsource"), Some("ModulationSourceId"));
    assert_eq!(DspNodeRegistry::normalize_type_name("noisetype"), Some("NoiseType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("noisecolor"), Some("NoiseType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluginformat"), Some("PluginFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiopluginformat"), Some("PluginFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ringmodwaveform"), Some("RingModWaveform"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ringmodcarrier"), Some("RingModWaveform"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotaryspeed"), Some("RotarySpeed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lesliespeed"), Some("RotarySpeed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("shakuhachiflutelength"), Some("ShakuhachiFluteLength"));
    assert_eq!(DspNodeRegistry::normalize_type_name("shakuhachilength"), Some("ShakuhachiFluteLength"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sidechaintappoint"), Some("SidechainTapPoint"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sidechaintap"), Some("SidechainTapPoint"));
    assert_eq!(DspNodeRegistry::normalize_type_name("slicealgorithm"), Some("SliceAlgorithm"));
    assert_eq!(DspNodeRegistry::normalize_type_name("autosliceralgorithm"), Some("SliceAlgorithm"));
    assert_eq!(DspNodeRegistry::normalize_type_name("songsectiontype"), Some("SongSectionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aisongsection"), Some("SongSectionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("targetcurve"), Some("TargetCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aimasteringcurve"), Some("TargetCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("targettimbre"), Some("TargetTimbre"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralneuraltimbre"), Some("TargetTimbre"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tuningremapmode"), Some("TuningRemapMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tuningmatrixmode"), Some("TuningRemapMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midievent"), Some("MidiEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("realtimemidievent"), Some("MidiEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicfunction"), Some("HarmonicFunction"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cadenceharmonicfunction"), Some("HarmonicFunction"));
    assert_eq!(DspNodeRegistry::normalize_type_name("isomorphiclayouttype"), Some("IsomorphicLayoutType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wickihaydenlayout"), Some("IsomorphicLayoutType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("launchquantize"), Some("LaunchQuantize"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipquantize"), Some("LaunchQuantize"));
    assert_eq!(DspNodeRegistry::normalize_type_name("launchmode"), Some("LaunchMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliplaunchmode"), Some("LaunchMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("followactiontype"), Some("FollowActionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("generativefollowaction"), Some("FollowActionType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("looperstate"), Some("LooperState"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionlooperstate"), Some("LooperState"));
}

#[test]
fn test_tier123_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1193, "Inventory must contain at least 1193 DSP modules");
    assert!(registry.list_all().len() >= 1193, "Registry list_all must be >= 1193");

    let tier123_nodes = [
        ("ArpDirection", DspNodeCategory::Modulation),
        ("AutomationMode", DspNodeCategory::Utility),
        ("BitDepth", DspNodeCategory::DynamicsMaster),
        ("ChapterType", DspNodeCategory::Utility),
        ("ChatMessage", DspNodeCategory::Utility),
        ("ClipContent", DspNodeCategory::Utility),
        ("CloudBranch", DspNodeCategory::Utility),
        ("CloudProjectTemplate", DspNodeCategory::Utility),
        ("CrashSeverity", DspNodeCategory::Utility),
        ("GrooveTemplate", DspNodeCategory::Modulation),
        ("LooperCommand", DspNodeCategory::SamplerSlicer),
        ("LooperQuantize", DspNodeCategory::SamplerSlicer),
        ("LooperSyncMode", DspNodeCategory::SamplerSlicer),
        ("MarkovOrder", DspNodeCategory::Modulation),
        ("MatrixEvent", DspNodeCategory::Utility),
        ("MidiMappingType", DspNodeCategory::Utility),
        ("MutationStrategy", DspNodeCategory::Modulation),
        ("PolyrhythmicDivision", DspNodeCategory::Modulation),
        ("SlotState", DspNodeCategory::Utility),
        ("StemExportFormat", DspNodeCategory::DynamicsMaster),
        ("StrumDirection", DspNodeCategory::Modulation),
    ];

    for (name, expected_cat) in tier123_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("arpdirection"), Some("ArpDirection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("arpeggiatordirection"), Some("ArpDirection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationmode"), Some("AutomationMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("touchlatchwrite"), Some("AutomationMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bitdepth"), Some("BitDepth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantizationbitdepth"), Some("BitDepth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaptertype"), Some("ChapterType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timelinechapter"), Some("ChapterType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chatmessage"), Some("ChatMessage"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionchatmessage"), Some("ChatMessage"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipcontent"), Some("ClipContent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timelineclipcontent"), Some("ClipContent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudbranch"), Some("CloudBranch"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crdtprojectbranch"), Some("CloudBranch"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudprojecttemplate"), Some("CloudProjectTemplate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectstartertemplate"), Some("CloudProjectTemplate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crashseverity"), Some("CrashSeverity"));
    assert_eq!(DspNodeRegistry::normalize_type_name("exceptionseverity"), Some("CrashSeverity"));
    assert_eq!(DspNodeRegistry::normalize_type_name("groovetemplate"), Some("GrooveTemplate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtimingshufflegroove"), Some("GrooveTemplate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loopercommand"), Some("LooperCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionloopercommand"), Some("LooperCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("looperquantize"), Some("LooperQuantize"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loopersyncquantize"), Some("LooperQuantize"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loopersyncmode"), Some("LooperSyncMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitrackloopersync"), Some("LooperSyncMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("markovorder"), Some("MarkovOrder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("markovharmonyorder"), Some("MarkovOrder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("matrixevent"), Some("MatrixEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipmatrixevent"), Some("MatrixEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midimappingtype"), Some("MidiMappingType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("miditargetmapping"), Some("MidiMappingType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mutationstrategy"), Some("MutationStrategy"));
    assert_eq!(DspNodeRegistry::normalize_type_name("generativemutationstrategy"), Some("MutationStrategy"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyrhythmicdivision"), Some("PolyrhythmicDivision"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyrhythmicstepdivision"), Some("PolyrhythmicDivision"));
    assert_eq!(DspNodeRegistry::normalize_type_name("slotstate"), Some("SlotState"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipslotstate"), Some("SlotState"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stemexportformat"), Some("StemExportFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitrackstemformat"), Some("StemExportFormat"));
    assert_eq!(DspNodeRegistry::normalize_type_name("strumdirection"), Some("StrumDirection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chordstrumdirection"), Some("StrumDirection"));
}

#[test]
fn test_tier124_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1214, "Inventory must contain at least 1214 DSP modules");
    assert!(registry.list_all().len() >= 1214, "Registry list_all must be >= 1214");

    let tier124_nodes = [
        ("NavigationCommand", DspNodeCategory::Utility),
        ("LicenseCompliance", DspNodeCategory::Utility),
        ("LuaScriptSandboxMode", DspNodeCategory::Utility),
        ("LuaDrawCommand", DspNodeCategory::Utility),
        ("WorkspaceRole", DspNodeCategory::Utility),
        ("CrdtOperationType", DspNodeCategory::Utility),
        ("SfzLoopMode", DspNodeCategory::SamplerSlicer),
        ("AudioOnsetDetector", DspNodeCategory::Modulation),
        ("FreeReedExciter", DspNodeCategory::AcousticPhysicalModel),
        ("GitMicroCommitDag", DspNodeCategory::Utility),
        ("GlottalFlowModel", DspNodeCategory::AcousticPhysicalModel),
        ("ClapPluginExportConfig", DspNodeCategory::Utility),
        ("AssetBlake3Verifier", DspNodeCategory::Utility),
        ("HammerStrikeExciter", DspNodeCategory::AcousticPhysicalModel),
        ("JiBridgeCoupling", DspNodeCategory::AcousticPhysicalModel),
        ("KotoSoundboard", DspNodeCategory::AcousticPhysicalModel),
        ("PlateReverbTank", DspNodeCategory::TimeSpace),
        ("PluckedStringWaveguide", DspNodeCategory::AcousticPhysicalModel),
        ("SitarSoundboard", DspNodeCategory::AcousticPhysicalModel),
        ("WindchestTracker", DspNodeCategory::AcousticPhysicalModel),
        ("CassottoChamberShutter", DspNodeCategory::AcousticPhysicalModel),
    ];

    for (name, expected_cat) in tier124_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("navigationcommand"), Some("NavigationCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timelinenavigation"), Some("NavigationCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("licensecompliance"), Some("LicenseCompliance"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dependencylicenseaudit"), Some("LicenseCompliance"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptsandboxmode"), Some("LuaScriptSandboxMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luasandboxmode"), Some("LuaScriptSandboxMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luadrawcommand"), Some("LuaDrawCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gpudrawcommand"), Some("LuaDrawCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("workspacerole"), Some("WorkspaceRole"));
    assert_eq!(DspNodeRegistry::normalize_type_name("collaborativerole"), Some("WorkspaceRole"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crdtoperationtype"), Some("CrdtOperationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crdtsyncoperation"), Some("CrdtOperationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sfzloopmode"), Some("SfzLoopMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sampleloopmode"), Some("SfzLoopMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audioonsetdetector"), Some("AudioOnsetDetector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralfluxonset"), Some("AudioOnsetDetector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("freereedexciter"), Some("FreeReedExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroelasticreed"), Some("FreeReedExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gitmicrocommitdag"), Some("GitMicroCommitDag"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessioncommitdag"), Some("GitMicroCommitDag"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glottalflowmodel"), Some("GlottalFlowModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rosenbergglottalflow"), Some("GlottalFlowModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clappluginexportconfig"), Some("ClapPluginExportConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("patchexportclap"), Some("ClapPluginExportConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("assetblake3verifier"), Some("AssetBlake3Verifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiointegrityverifier"), Some("AssetBlake3Verifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hammerstrikeexciter"), Some("HammerStrikeExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("feltcontactdynamics"), Some("HammerStrikeExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jibridgecoupling"), Some("JiBridgeCoupling"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jibridgemodel"), Some("JiBridgeCoupling"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kotosoundboard"), Some("KotoSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kotoacousticbody"), Some("KotoSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("platereverbtank"), Some("PlateReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("emt140platetank"), Some("PlateReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluckedstringwaveguide"), Some("PluckedStringWaveguide"));
    assert_eq!(DspNodeRegistry::normalize_type_name("commutedwaveguidestring"), Some("PluckedStringWaveguide"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sitarsoundboard"), Some("SitarSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tablisoundboard"), Some("SitarSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("windchesttracker"), Some("WindchestTracker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("organwindpressure"), Some("WindchestTracker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cassottochambershutter"), Some("CassottoChamberShutter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tonechambershutter"), Some("CassottoChamberShutter"));
}

#[test]
fn test_tier125_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1235, "Inventory must contain at least 1235 DSP modules");
    assert!(registry.list_all().len() >= 1235, "Registry list_all must be >= 1235");

    let tier125_nodes = [
        ("TrajectoryPathType", DspNodeCategory::SpatialSurround),
        ("SpatialInterpolationCurve", DspNodeCategory::SpatialSurround),
        ("GestureInterpolationCurve", DspNodeCategory::Modulation),
        ("PipeOrganInterpolationCurve", DspNodeCategory::Modulation),
        ("PluckInterpolationCurve", DspNodeCategory::Modulation),
        ("RotaryInterpolationCurve", DspNodeCategory::Modulation),
        ("ShakuhachiInterpolationCurve", DspNodeCategory::Modulation),
        ("SitarInterpolationCurve", DspNodeCategory::Modulation),
        ("SpringInterpolationCurve", DspNodeCategory::Modulation),
        ("WoodwindInterpolationCurve", DspNodeCategory::Modulation),
        ("AutomationInterpolation", DspNodeCategory::Modulation),
        ("MarkerConfig", DspNodeCategory::Utility),
        ("AutomationLaneConfig", DspNodeCategory::Utility),
        ("AutomationEventConfig", DspNodeCategory::Modulation),
        ("MidiMappingConfig", DspNodeCategory::Utility),
        ("AssetConfig", DspNodeCategory::Utility),
        ("TrackerStepConfig", DspNodeCategory::Modulation),
        ("ConnectionConfig", DspNodeCategory::Utility),
        ("TrackConfig", DspNodeCategory::Utility),
        ("TransportConfig", DspNodeCategory::Utility),
        ("ProjectMetadata", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier125_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("trajectorypathtype"), Some("TrajectoryPathType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("trajectorypath"), Some("TrajectoryPathType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spatialinterpolationcurve"), Some("SpatialInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spatialkeyframecurve"), Some("SpatialInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gestureinterpolationcurve"), Some("GestureInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowinggesturecurve"), Some("GestureInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pipeorganinterpolationcurve"), Some("PipeOrganInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("trackerkeyframecurve"), Some("PipeOrganInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluckinterpolationcurve"), Some("PluckInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pluckgesturecurve"), Some("PluckInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotaryinterpolationcurve"), Some("RotaryInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("leslieaccelerationcurve"), Some("RotaryInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("shakuhachiinterpolationcurve"), Some("ShakuhachiInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("merikaricurve"), Some("ShakuhachiInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sitarinterpolationcurve"), Some("SitarInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("meendpullcurve"), Some("SitarInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springinterpolationcurve"), Some("SpringInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springdispersioncurve"), Some("SpringInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("woodwindinterpolationcurve"), Some("WoodwindInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("toneholetransitioncurve"), Some("WoodwindInterpolationCurve"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationinterpolation"), Some("AutomationInterpolation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timelineinterpolation"), Some("AutomationInterpolation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("markerconfig"), Some("MarkerConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timelinesessionmarker"), Some("MarkerConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationlaneconfig"), Some("AutomationLaneConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parameterautomationlane"), Some("AutomationLaneConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationeventconfig"), Some("AutomationEventConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationkeyframe"), Some("AutomationEventConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midimappingconfig"), Some("MidiMappingConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hardwaremidimapping"), Some("MidiMappingConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("assetconfig"), Some("AssetConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audioassetdescriptor"), Some("AssetConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("trackerstepconfig"), Some("TrackerStepConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fasttrackerstep"), Some("TrackerStepConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("connectionconfig"), Some("ConnectionConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("patchcordconfig"), Some("ConnectionConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("trackconfig"), Some("TrackConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mixerchannelconfig"), Some("TrackConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transportconfig"), Some("TransportConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mastertransportconfig"), Some("TransportConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectmetadata"), Some("ProjectMetadata"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectmanifest"), Some("ProjectMetadata"));
}

#[test]
fn test_tier126_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1256, "Inventory must contain at least 1256 DSP modules");
    assert!(registry.list_all().len() >= 1256, "Registry list_all must be >= 1256");

    let tier126_nodes = [
        ("ProjectConfig", DspNodeCategory::Utility),
        ("SequenceConfig", DspNodeCategory::Utility),
        ("NodeConfig", DspNodeCategory::Utility),
        ("SystemSettings", DspNodeCategory::Utility),
        ("CrashDump", DspNodeCategory::Utility),
        ("CrashDumpSummaryReport", DspNodeCategory::Utility),
        ("RecoveryPoint", DspNodeCategory::Utility),
        ("RenderJobSpec", DspNodeCategory::Utility),
        ("CrdtOp", DspNodeCategory::Utility),
        ("VectorClock", DspNodeCategory::Utility),
        ("RemoteUserCursor", DspNodeCategory::Utility),
        ("NetworkStatus", DspNodeCategory::Utility),
        ("LuaScriptConfig", DspNodeCategory::Utility),
        ("LuaScriptErrorRecovery", DspNodeCategory::Utility),
        ("LuaScriptInfo", DspNodeCategory::Utility),
        ("LuaTestRunner", DspNodeCategory::Utility),
        ("LuaTestResult", DspNodeCategory::Utility),
        ("LuaUsageAnalytics", DspNodeCategory::Utility),
        ("MarketplacePlugin", DspNodeCategory::Utility),
        ("DependencyAuditReport", DspNodeCategory::Utility),
        ("WorkspaceCrateAudit", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier126_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("projectconfig"), Some("ProjectConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectfile"), Some("ProjectConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sequenceconfig"), Some("SequenceConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("arrangementclip"), Some("SequenceConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nodeconfig"), Some("NodeConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dspnodeconfig"), Some("NodeConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("systemsettings"), Some("SystemSettings"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dawsettings"), Some("SystemSettings"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crashdump"), Some("CrashDump"));
    assert_eq!(DspNodeRegistry::normalize_type_name("telemetrycrashdump"), Some("CrashDump"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crashdumpsummaryreport"), Some("CrashDumpSummaryReport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stabilitysummary"), Some("CrashDumpSummaryReport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("recoverypoint"), Some("RecoveryPoint"));
    assert_eq!(DspNodeRegistry::normalize_type_name("backupsnapshotpoint"), Some("RecoveryPoint"));
    assert_eq!(DspNodeRegistry::normalize_type_name("renderjobspec"), Some("RenderJobSpec"),);
    assert_eq!(DspNodeRegistry::normalize_type_name("offlinerenderspec"), Some("RenderJobSpec"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crdtop"), Some("CrdtOp"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crdtmutationop"), Some("CrdtOp"));
    assert_eq!(DspNodeRegistry::normalize_type_name("collaborativeop"), Some("CrdtOp"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vectorclock"), Some("VectorClock"));
    assert_eq!(DspNodeRegistry::normalize_type_name("causalvectorclock"), Some("VectorClock"));
    assert_eq!(DspNodeRegistry::normalize_type_name("remoteusercursor"), Some("RemoteUserCursor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("peercursor"), Some("RemoteUserCursor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("networkstatus"), Some("NetworkStatus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("peernetworkstatus"), Some("NetworkStatus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptconfig"), Some("LuaScriptConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("proceduralluaconfig"), Some("LuaScriptConfig"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascripterrorrecovery"), Some("LuaScriptErrorRecovery"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luafaulttolerance"), Some("LuaScriptErrorRecovery"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptinfo"), Some("LuaScriptInfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptmanifest"), Some("LuaScriptInfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luatestrunner"), Some("LuaTestRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luadspunitrunner"), Some("LuaTestRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luatestresult"), Some("LuaTestResult"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luatestverdict"), Some("LuaTestResult"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luausageanalytics"), Some("LuaUsageAnalytics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptprofiler"), Some("LuaUsageAnalytics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luacycleprofiler"), Some("LuaUsageAnalytics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("marketplaceplugin"), Some("MarketplacePlugin"));
    assert_eq!(DspNodeRegistry::normalize_type_name("communityplugin"), Some("MarketplacePlugin"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dependencyauditreport"), Some("DependencyAuditReport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cargoauditreport"), Some("DependencyAuditReport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("workspacecrateaudit"), Some("WorkspaceCrateAudit"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crateintegrityvalidator"), Some("WorkspaceCrateAudit"));
}

#[test]
fn test_tier127_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1277, "Inventory must contain at least 1277 DSP modules");
    assert!(registry.list_all().len() >= 1277, "Registry list_all must be >= 1277");

    let tier127_nodes = [
        ("GrandPianoProfile", DspNodeCategory::AcousticPhysicalModel),
        ("JawariProfile", DspNodeCategory::AcousticPhysicalModel),
        ("ModalExcitationType", DspNodeCategory::AcousticPhysicalModel),
        ("AuditedCrateDependency", DspNodeCategory::Utility),
        ("LuaAutomationOnlyGuard", DspNodeCategory::Utility),
        ("LuaClipboard", DspNodeCategory::Utility),
        ("LuaFileWatcher", DspNodeCategory::Utility),
        ("LuaGitScriptTracker", DspNodeCategory::Utility),
        ("LuaProjectBuilder", DspNodeCategory::Utility),
        ("LuaScriptAnalytics", DspNodeCategory::Utility),
        ("LuaScriptInspectorState", DspNodeCategory::Utility),
        ("MarketplaceScriptEntry", DspNodeCategory::Utility),
        ("ScriptMergeConflict", DspNodeCategory::Utility),
        ("ProjectError", DspNodeCategory::Utility),
        ("CloudPullRequest", DspNodeCategory::Utility),
        ("PresetComment", DspNodeCategory::Utility),
        ("UserSessionMetric", DspNodeCategory::Utility),
        ("TelemetryEvent", DspNodeCategory::Utility),
        ("GitBlameEntry", DspNodeCategory::Utility),
        ("ScriptBlameInfo", DspNodeCategory::Utility),
        ("ScriptExecutionLog", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier127_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("grandpianovoicing"), Some("GrandPianoProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grandpianoacoustic"), Some("GrandPianoProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sitarjawariprofile"), Some("JawariProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jivadamping"), Some("JawariProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalexcitationmode"), Some("ModalExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalstrikepluckbow"), Some("ModalExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("auditedcratedependency"), Some("AuditedCrateDependency"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cratedependencyaudit"), Some("AuditedCrateDependency"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaautomationonlyguard"), Some("LuaAutomationOnlyGuard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationonlyguard"), Some("LuaAutomationOnlyGuard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaclipboard"), Some("LuaClipboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipboardbuffer"), Some("LuaClipboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luafilewatcher"), Some("LuaFileWatcher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptfilewatcher"), Some("LuaFileWatcher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luagitscripttracker"), Some("LuaGitScriptTracker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptgittracker"), Some("LuaGitScriptTracker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaprojectbuilder"), Some("LuaProjectBuilder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("proceduralprojectbuilder"), Some("LuaProjectBuilder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptanalytics"), Some("LuaScriptAnalytics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaperfanalytics"), Some("LuaScriptAnalytics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptinspectorstate"), Some("LuaScriptInspectorState"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptinspector"), Some("LuaScriptInspectorState"));
    assert_eq!(DspNodeRegistry::normalize_type_name("marketplacescriptentry"), Some("MarketplaceScriptEntry"));
    assert_eq!(DspNodeRegistry::normalize_type_name("communityscriptentry"), Some("MarketplaceScriptEntry"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptmergeconflict"), Some("ScriptMergeConflict"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptconflictresolver"), Some("ScriptMergeConflict"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projecterror"), Some("ProjectError"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projecttomlerror"), Some("ProjectError"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudpullrequest"), Some("CloudPullRequest"));
    assert_eq!(DspNodeRegistry::normalize_type_name("collaborativepr"), Some("CloudPullRequest"));
    assert_eq!(DspNodeRegistry::normalize_type_name("presetcomment"), Some("PresetComment"));
    assert_eq!(DspNodeRegistry::normalize_type_name("communitypresetreview"), Some("PresetComment"));
    assert_eq!(DspNodeRegistry::normalize_type_name("usersessionmetric"), Some("UserSessionMetric"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dsploadtelemetry"), Some("UserSessionMetric"));
    assert_eq!(DspNodeRegistry::normalize_type_name("telemetryevent"), Some("TelemetryEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diagnosticevent"), Some("TelemetryEvent"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gitblameentry"), Some("GitBlameEntry"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipblameinfo"), Some("GitBlameEntry"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptblameinfo"), Some("ScriptBlameInfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptblame"), Some("ScriptBlameInfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptexecutionlog"), Some("ScriptExecutionLog"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luacycleprofilerlog"), Some("ScriptExecutionLog"));
}

#[test]
fn test_tier128_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1298, "Inventory must contain at least 1298 DSP modules");
    assert!(registry.list_all().len() >= 1298, "Registry list_all must be >= 1298");

    let tier128_nodes = [
        ("BowedInstrument", DspNodeCategory::AcousticPhysicalModel),
        ("ExcitationType", DspNodeCategory::AcousticPhysicalModel),
        ("HornFlarePreset", DspNodeCategory::AcousticPhysicalModel),
        ("HornMuteType", DspNodeCategory::AcousticPhysicalModel),
        ("MeshBoundaryType", DspNodeCategory::AcousticPhysicalModel),
        ("RagaScale", DspNodeCategory::AcousticPhysicalModel),
        ("RosinType", DspNodeCategory::AcousticPhysicalModel),
        ("GrainWindowType", DspNodeCategory::SamplerSlicer),
        ("TubeTopology", DspNodeCategory::DistortionSaturation),
        ("WallMaterial", DspNodeCategory::SpatialSurround),
        ("ColorMappingMode", DspNodeCategory::SpectralResynthesis),
        ("FrequencyMapping", DspNodeCategory::SpectralResynthesis),
        ("CadenceType", DspNodeCategory::Modulation),
        ("Interpolation", DspNodeCategory::Modulation),
        ("CloudPresetEntry", DspNodeCategory::Utility),
        ("CloudRenderJob", DspNodeCategory::Utility),
        ("CrashReport", DspNodeCategory::Utility),
        ("TelemetryManager", DspNodeCategory::Utility),
        ("UserAccount", DspNodeCategory::Utility),
        ("UpdateChecker", DspNodeCategory::Utility),
        ("GdprNotice", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier128_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("bowedstringinstrument"), Some("BowedInstrument"));
    assert_eq!(DspNodeRegistry::normalize_type_name("physicalbowedinstrument"), Some("BowedInstrument"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stringexcitationtype"), Some("ExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hammerexcitationdynamics"), Some("ExcitationType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brasshornflare"), Some("HornFlarePreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bellflarepreset"), Some("HornFlarePreset"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticbrassmute"), Some("HornMuteType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bellmuteprofile"), Some("HornMuteType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("meshboundaryphysics"), Some("MeshBoundaryType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonatingmeshboundary"), Some("MeshBoundaryType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("indianragascale"), Some("RagaScale"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ragashrutituning"), Some("RagaScale"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowrosintype"), Some("RosinType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("frictionrosinformula"), Some("RosinType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularwindowtype"), Some("GrainWindowType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grainenvelopewindow"), Some("GrainWindowType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubecircuittopology"), Some("TubeTopology"));
    assert_eq!(DspNodeRegistry::normalize_type_name("valvecircuittopology"), Some("TubeTopology"));
    assert_eq!(DspNodeRegistry::normalize_type_name("roomwallmaterial"), Some("WallMaterial"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spacewallmaterial"), Some("WallMaterial"));
    assert_eq!(DspNodeRegistry::normalize_type_name("colorharmonicmapping"), Some("ColorMappingMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogramcolortoharmonic"), Some("ColorMappingMode"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralfrequencymapping"), Some("FrequencyMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogramfrequencyaxis"), Some("FrequencyMapping"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmoniccadencetype"), Some("CadenceType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cadenceresolutiontarget"), Some("CadenceType"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationinterpolationmode"), Some("Interpolation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("curvetransferinterpolation"), Some("Interpolation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudcrdtpreset"), Some("CloudPresetEntry"));
    assert_eq!(DspNodeRegistry::normalize_type_name("collaborativepresetentry"), Some("CloudPresetEntry"));
    assert_eq!(DspNodeRegistry::normalize_type_name("distributedrenderjob"), Some("CloudRenderJob"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cloudrenderfarmjob"), Some("CloudRenderJob"));
    assert_eq!(DspNodeRegistry::normalize_type_name("enginecrashreport"), Some("CrashReport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("paniccrashreport"), Some("CrashReport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("enginetelemetrymanager"), Some("TelemetryManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dsploadtelemetrymanager"), Some("TelemetryManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clouduseraccount"), Some("UserAccount"));
    assert_eq!(DspNodeRegistry::normalize_type_name("summoneruseraccount"), Some("UserAccount"));
    assert_eq!(DspNodeRegistry::normalize_type_name("softwareupdatechecker"), Some("UpdateChecker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("releaseupdatechecker"), Some("UpdateChecker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gdprprivacycompliance"), Some("GdprNotice"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gdprdataprivacy"), Some("GdprNotice"));
}

#[test]
fn test_tier129_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1319, "Inventory must contain at least 1319 DSP modules");
    assert!(registry.list_all().len() >= 1319, "Registry list_all must be >= 1319");

    let tier129_nodes = [
        ("ParamUpdate", DspNodeCategory::Utility),
        ("GraphRunner", DspNodeCategory::Utility),
        ("NodeFactory", DspNodeCategory::Utility),
        ("OscCommand", DspNodeCategory::Utility),
        ("OscServer", DspNodeCategory::Utility),
        ("CliBatchRenderEngine", DspNodeCategory::Utility),
        ("CliStemSeparatorEngine", DspNodeCategory::Utility),
        ("CliSfzPatchConverter", DspNodeCategory::Utility),
        ("CliMelodyGenerator", DspNodeCategory::Modulation),
        ("CliPatternGenerator", DspNodeCategory::Modulation),
        ("CliProjectMigrator", DspNodeCategory::Utility),
        ("CliProjectDiffEngine", DspNodeCategory::Utility),
        ("CliWasmPackager", DspNodeCategory::Utility),
        ("CliAdmSpatialExporter", DspNodeCategory::SpatialSurround),
        ("CliHarmonySuggester", DspNodeCategory::Modulation),
        ("CliMicrotoneTuner", DspNodeCategory::Utility),
        ("CliPiPlatformBuilder", DspNodeCategory::Utility),
        ("CliAssetVerifier", DspNodeCategory::Utility),
        ("CliAutoSlicer", DspNodeCategory::SamplerSlicer),
        ("CliSessionWatchDaemon", DspNodeCategory::Utility),
        ("CliProjectNormalizer", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier129_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("paramupdate"), Some("ParamUpdate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audioparamupdate"), Some("ParamUpdate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("graphrunner"), Some("GraphRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiographrunner"), Some("GraphRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nodefactory"), Some("NodeFactory"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dspnodefactory"), Some("NodeFactory"));
    assert_eq!(DspNodeRegistry::normalize_type_name("osccommand"), Some("OscCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("oscmessagerouter"), Some("OscCommand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("oscserver"), Some("OscServer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("osclistener"), Some("OscServer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clibatchrenderengine"), Some("CliBatchRenderEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("batchrenderengine"), Some("CliBatchRenderEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clistemseparatorengine"), Some("CliStemSeparatorEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stemseparatorengine"), Some("CliStemSeparatorEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clisfzpatchconverter"), Some("CliSfzPatchConverter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sfzpatchconverter"), Some("CliSfzPatchConverter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("climelodygenerator"), Some("CliMelodyGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("melodygenerator"), Some("CliMelodyGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipatterngenerator"), Some("CliPatternGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clicellularautomata"), Some("CliPatternGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliprojectmigrator"), Some("CliProjectMigrator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectmigrator"), Some("CliProjectMigrator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliprojectdiffengine"), Some("CliProjectDiffEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectdiffengine"), Some("CliProjectDiffEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliwasmpackager"), Some("CliWasmPackager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("webassemblypluginpackager"), Some("CliWasmPackager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliadmspatialexporter"), Some("CliAdmSpatialExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clispatialadm"), Some("CliAdmSpatialExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliharmonysuggester"), Some("CliHarmonySuggester"));
    assert_eq!(DspNodeRegistry::normalize_type_name("climodalharmony"), Some("CliHarmonySuggester"));
    assert_eq!(DspNodeRegistry::normalize_type_name("climicrotonetuner"), Some("CliMicrotoneTuner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonetuner"), Some("CliMicrotoneTuner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipiplatformbuilder"), Some("CliPiPlatformBuilder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("piplatformbuilder"), Some("CliPiPlatformBuilder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliassetverifier"), Some("CliAssetVerifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clisampleverifier"), Some("CliAssetVerifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliautoslicer"), Some("CliAutoSlicer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clitransientslicer"), Some("CliAutoSlicer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clisessionwatchdaemon"), Some("CliSessionWatchDaemon"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionwatchdaemon"), Some("CliSessionWatchDaemon"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliprojectnormalizer"), Some("CliProjectNormalizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliprojectformatter"), Some("CliProjectNormalizer"));
}

#[test]
fn test_tier130_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1340, "Inventory must contain at least 1340 DSP modules");
    assert!(registry.list_all().len() >= 1340, "Registry list_all must be >= 1340");

    let tier130_nodes = [
        ("CliProjectInitializer", DspNodeCategory::Utility),
        ("CliStubRenderer", DspNodeCategory::Utility),
        ("CliWavRenderer", DspNodeCategory::Utility),
        ("CliBatchManifestRenderer", DspNodeCategory::Utility),
        ("CliPatchExporter", DspNodeCategory::Utility),
        ("CliClapPluginExporter", DspNodeCategory::Utility),
        ("CliCommitHistoryViewer", DspNodeCategory::Utility),
        ("CliUndoManager", DspNodeCategory::Utility),
        ("CliRedoManager", DspNodeCategory::Utility),
        ("CliGitHubPrCreator", DspNodeCategory::Utility),
        ("CliLivePlayerEngine", DspNodeCategory::Utility),
        ("CliAssetManager", DspNodeCategory::Utility),
        ("CliProjectValidator", DspNodeCategory::Utility),
        ("CliDspProfiler", DspNodeCategory::Utility),
        ("CliPresetBaker", DspNodeCategory::Utility),
        ("CliMultiStemExporter", DspNodeCategory::Utility),
        ("CliGrooveHumanizer", DspNodeCategory::Modulation),
        ("CliAutomationThinner", DspNodeCategory::Modulation),
        ("CliAudioDeviceEnumerator", DspNodeCategory::Utility),
        ("CliTempoMapExtractor", DspNodeCategory::Modulation),
        ("CliScriptEvaluator", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier130_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("cliprojectinitializer"), Some("CliProjectInitializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectinitializer"), Some("CliProjectInitializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clistubrenderer"), Some("CliStubRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stubrenderer"), Some("CliStubRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliwavrenderer"), Some("CliWavRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavrenderer"), Some("CliWavRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clibatchmanifestrenderer"), Some("CliBatchManifestRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("batchmanifestrenderer"), Some("CliBatchManifestRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipatchexporter"), Some("CliPatchExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("patchexporter"), Some("CliPatchExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliclappluginexporter"), Some("CliClapPluginExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clappluginexporter"), Some("CliClapPluginExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clicommithistoryviewer"), Some("CliCommitHistoryViewer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("commithistoryviewer"), Some("CliCommitHistoryViewer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliundomanager"), Some("CliUndoManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("undomanager"), Some("CliUndoManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliredomanager"), Some("CliRedoManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("redomanager"), Some("CliRedoManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cligithubprcreator"), Some("CliGitHubPrCreator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("githubprcreator"), Some("CliGitHubPrCreator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliliveplayerengine"), Some("CliLivePlayerEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("liveplayerengine"), Some("CliLivePlayerEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliassetmanager"), Some("CliAssetManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("assetmanager"), Some("CliAssetManager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliprojectvalidator"), Some("CliProjectValidator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectschemavalidator"), Some("CliProjectValidator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clidspprofiler"), Some("CliDspProfiler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliaudiothreadprofiler"), Some("CliDspProfiler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clipresetbaker"), Some("CliPresetBaker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliofflinepresetbaker"), Some("CliPresetBaker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("climultistemexporter"), Some("CliMultiStemExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multistemexporter"), Some("CliMultiStemExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cligroovehumanizer"), Some("CliGrooveHumanizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clitickinghumanizer"), Some("CliGrooveHumanizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliautomationthinner"), Some("CliAutomationThinner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rdpatomationthinner"), Some("CliAutomationThinner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliaudiodeviceenumerator"), Some("CliAudioDeviceEnumerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiodeviceenumerator"), Some("CliAudioDeviceEnumerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clitempomapextractor"), Some("CliTempoMapExtractor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tempomapgridextractor"), Some("CliTempoMapExtractor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptevaluator"), Some("CliScriptEvaluator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptevaluator"), Some("CliScriptEvaluator"));
}

#[test]
fn test_tier131_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1361, "Inventory must contain at least 1361 DSP modules");
    assert!(registry.list_all().len() >= 1361, "Registry list_all must be >= 1361");

    let tier131_nodes = [
        ("CliBatchAudioConverter", DspNodeCategory::Utility),
        ("CliProjectZipBackup", DspNodeCategory::Utility),
        ("CliAbletonExporter", DspNodeCategory::Utility),
        ("CliAbletonClipImporter", DspNodeCategory::Utility),
        ("CliReaperExporter", DspNodeCategory::Utility),
        ("CliDawprojectExporter", DspNodeCategory::Utility),
        ("CliDawprojectImporter", DspNodeCategory::Utility),
        ("CliMidiFileExporter", DspNodeCategory::Utility),
        ("CliMidiFileImporter", DspNodeCategory::Utility),
        ("CliGraphSvgExporter", DspNodeCategory::Utility),
        ("CliAutomationCsvExporter", DspNodeCategory::Modulation),
        ("CliAutomationCsvImporter", DspNodeCategory::Modulation),
        ("CliAudioBenchmarkRunner", DspNodeCategory::Utility),
        ("CliScriptTestRunner", DspNodeCategory::Utility),
        ("CliScriptReplHarness", DspNodeCategory::Utility),
        ("CliScriptAutomationRunner", DspNodeCategory::Modulation),
        ("CliScriptSecurityAuditor", DspNodeCategory::Utility),
        ("CliScriptFormatter", DspNodeCategory::Utility),
        ("CliScriptLinter", DspNodeCategory::Utility),
        ("CliScriptMinifier", DspNodeCategory::Utility),
        ("CliScriptDocGenerator", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier131_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("clibatchaudioconverter"), Some("CliBatchAudioConverter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("batchaudioconverter"), Some("CliBatchAudioConverter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliprojectzipbackup"), Some("CliProjectZipBackup"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectzipbackup"), Some("CliProjectZipBackup"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliabletonexporter"), Some("CliAbletonExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("abletonexporter"), Some("CliAbletonExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliabletonclipimporter"), Some("CliAbletonClipImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("abletonclipimporter"), Some("CliAbletonClipImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clireaperexporter"), Some("CliReaperExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("reaperexporter"), Some("CliReaperExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clidawprojectexporter"), Some("CliDawprojectExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dawprojectexporter"), Some("CliDawprojectExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clidawprojectimporter"), Some("CliDawprojectImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dawprojectimporter"), Some("CliDawprojectImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("climidifileexporter"), Some("CliMidiFileExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midifileexporter"), Some("CliMidiFileExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("climidifileimporter"), Some("CliMidiFileImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midifileimporter"), Some("CliMidiFileImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cligraphsvgexporter"), Some("CliGraphSvgExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("graphsvgexporter"), Some("CliGraphSvgExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliautomationcsvexporter"), Some("CliAutomationCsvExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationcsvexporter"), Some("CliAutomationCsvExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliautomationcsvimporter"), Some("CliAutomationCsvImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automationcsvimporter"), Some("CliAutomationCsvImporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliaudiobenchmarkrunner"), Some("CliAudioBenchmarkRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiobenchmarkrunner"), Some("CliAudioBenchmarkRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscripttestrunner"), Some("CliScriptTestRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scripttestrunner"), Some("CliScriptTestRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptreplharness"), Some("CliScriptReplHarness"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptreplharness"), Some("CliScriptReplHarness"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptautomationrunner"), Some("CliScriptAutomationRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptautomationrunner"), Some("CliScriptAutomationRunner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptsecurityauditor"), Some("CliScriptSecurityAuditor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptsecurityauditor"), Some("CliScriptSecurityAuditor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptformatter"), Some("CliScriptFormatter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptformatter"), Some("CliScriptFormatter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptlinter"), Some("CliScriptLinter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptlinter"), Some("CliScriptLinter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptminifier"), Some("CliScriptMinifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptminifier"), Some("CliScriptMinifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cliscriptdocgenerator"), Some("CliScriptDocGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scriptdocgenerator"), Some("CliScriptDocGenerator"));
}

#[test]
fn test_tier132_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1382, "Inventory must contain at least 1382 DSP modules");
    assert!(registry.list_all().len() >= 1382, "Registry list_all must be >= 1382");

    let tier132_nodes = [
        ("AudioWatermarkEmbedder", DspNodeCategory::Utility),
        ("AudioWatermarkExtractor", DspNodeCategory::Utility),
        ("WaveformPngExporter", DspNodeCategory::Utility),
        ("SpectrogramPngExporter", DspNodeCategory::Utility),
        ("PianoRollPngExporter", DspNodeCategory::Utility),
        ("ProjectLayoutPdfExporter", DspNodeCategory::Utility),
        ("SessionNotesPdfExporter", DspNodeCategory::Utility),
        ("ProjectAes256Encryptor", DspNodeCategory::Utility),
        ("ProjectAes256Decryptor", DspNodeCategory::Utility),
        ("GitChangeAttributor", DspNodeCategory::Utility),
        ("TomlConflictResolver", DspNodeCategory::Utility),
        ("VideoMetadataExporter", DspNodeCategory::Utility),
        ("LuaBytecodeCompiler", DspNodeCategory::Utility),
        ("LuaAstLinter", DspNodeCategory::Utility),
        ("LuaForeignFunctionBridge", DspNodeCategory::Utility),
        ("LuaMemoryWatchdog", DspNodeCategory::Utility),
        ("LuaCustomDspSandbox", DspNodeCategory::Modulation),
        ("LuaBytecodeDisassembler", DspNodeCategory::Utility),
        ("LuaScriptMergeDetector", DspNodeCategory::Utility),
        ("LuaSpectralBufferHook", DspNodeCategory::Modulation),
        ("LuaTimerScheduler", DspNodeCategory::Modulation),
    ];

    for (name, expected_cat) in tier132_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("audiowatermarkembedder"), Some("AudioWatermarkEmbedder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("watermarkembedder"), Some("AudioWatermarkEmbedder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiowatermarkextractor"), Some("AudioWatermarkExtractor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("watermarkextractor"), Some("AudioWatermarkExtractor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveformpngexporter"), Some("WaveformPngExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveformpng"), Some("WaveformPngExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogrampngexporter"), Some("SpectrogramPngExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogrampng"), Some("SpectrogramPngExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pianorollpngexporter"), Some("PianoRollPngExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pianorollpng"), Some("PianoRollPngExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectlayoutpdfexporter"), Some("ProjectLayoutPdfExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectlayoutpdf"), Some("ProjectLayoutPdfExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionnotespdfexporter"), Some("SessionNotesPdfExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionnotespdf"), Some("SessionNotesPdfExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectaes256encryptor"), Some("ProjectAes256Encryptor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectaes256"), Some("ProjectAes256Encryptor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectaes256decryptor"), Some("ProjectAes256Decryptor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectaes256decrypt"), Some("ProjectAes256Decryptor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gitchangeattributor"), Some("GitChangeAttributor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("changeattributor"), Some("GitChangeAttributor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tomlconflictresolver"), Some("TomlConflictResolver"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tomlconflict"), Some("TomlConflictResolver"));
    assert_eq!(DspNodeRegistry::normalize_type_name("videometadataexporter"), Some("VideoMetadataExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("videometadata"), Some("VideoMetadataExporter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luabytecodecompiler"), Some("LuaBytecodeCompiler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luacompiler"), Some("LuaBytecodeCompiler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaastlinter"), Some("LuaAstLinter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lualinter"), Some("LuaAstLinter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaforeignfunctionbridge"), Some("LuaForeignFunctionBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaffibridge"), Some("LuaForeignFunctionBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luamemorywatchdog"), Some("LuaMemoryWatchdog"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luagcwatchdog"), Some("LuaMemoryWatchdog"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luacustomdspsandbox"), Some("LuaCustomDspSandbox"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luadspsandbox"), Some("LuaCustomDspSandbox"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luabytecodedisassembler"), Some("LuaBytecodeDisassembler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luadisassembler"), Some("LuaBytecodeDisassembler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luascriptmergedetector"), Some("LuaScriptMergeDetector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luamergedetector"), Some("LuaScriptMergeDetector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaspectralbufferhook"), Some("LuaSpectralBufferHook"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luaspectralhook"), Some("LuaSpectralBufferHook"));
    assert_eq!(DspNodeRegistry::normalize_type_name("luatimerscheduler"), Some("LuaTimerScheduler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("musicalbeattimer"), Some("LuaTimerScheduler"));
}

#[test]
fn test_tier133_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1403, "Inventory must contain at least 1403 DSP modules");
    assert!(registry.list_all().len() >= 1403, "Registry list_all must be >= 1403");

    let tier133_nodes = [
        ("SessionSnapshotDiffViewer", DspNodeCategory::Utility),
        ("ProjectTreeDependencyGraph", DspNodeCategory::Utility),
        ("SampleRateResamplingEngine", DspNodeCategory::Utility),
        ("BitDepthDitherQuantizer", DspNodeCategory::DynamicsMaster),
        ("LoudnessEbuR128Normalizer", DspNodeCategory::DynamicsMaster),
        ("TruePeakIntersampleLimiter", DspNodeCategory::DynamicsMaster),
        ("MultichannelStemAligner", DspNodeCategory::Utility),
        ("DcOffsetRemoverFilter", DspNodeCategory::FilterEq),
        ("AcousticImpulseResponseTruncator", DspNodeCategory::TimeSpace),
        ("BinauralHeadphoneEqualizer", DspNodeCategory::SpatialSurround),
        ("MidiSysExMessageRouter", DspNodeCategory::Utility),
        ("MidiMpeVoiceDistributor", DspNodeCategory::Modulation),
        ("GenerativeEuclideanArpeggiator", DspNodeCategory::Modulation),
        ("MicrotonalScaleQuantizer", DspNodeCategory::Modulation),
        ("HarmonicCadenceAnalyzer", DspNodeCategory::Modulation),
        ("SpectralPhaseVocoderPitchShifter", DspNodeCategory::SpectralResynthesis),
        ("ZeroLatencyConvolutionEngine", DspNodeCategory::TimeSpace),
        ("DynamicConvolutionMorpher", DspNodeCategory::TimeSpace),
        ("SubHarmonicBassSynthesizer", DspNodeCategory::Oscillator),
        ("AnalogTapeFluttersWowGenerator", DspNodeCategory::Modulation),
        ("TransientSplayStereoWidener", DspNodeCategory::SpatialSurround),
    ];

    for (name, expected_cat) in tier133_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("sessionsnapshotdiffviewer"), Some("SessionSnapshotDiffViewer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessiondiffviewer"), Some("SessionSnapshotDiffViewer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projecttreedependencygraph"), Some("ProjectTreeDependencyGraph"));
    assert_eq!(DspNodeRegistry::normalize_type_name("projectdependencygraph"), Some("ProjectTreeDependencyGraph"));
    assert_eq!(DspNodeRegistry::normalize_type_name("samplerateresamplingengine"), Some("SampleRateResamplingEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resamplingengine"), Some("SampleRateResamplingEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bitdepthditherquantizer"), Some("BitDepthDitherQuantizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ditherquantizer"), Some("BitDepthDitherQuantizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loudnessebur128normalizer"), Some("LoudnessEbuR128Normalizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ebur128normalizer"), Some("LoudnessEbuR128Normalizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakintersamplelimiter"), Some("TruePeakIntersampleLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("intersamplelimiter"), Some("TruePeakIntersampleLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelstemaligner"), Some("MultichannelStemAligner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stemaligner"), Some("MultichannelStemAligner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dcoffsetremoverfilter"), Some("DcOffsetRemoverFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dcoffsetremover"), Some("DcOffsetRemoverFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticimpulseresponsetruncator"), Some("AcousticImpulseResponseTruncator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("irtruncator"), Some("AcousticImpulseResponseTruncator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralheadphoneequalizer"), Some("BinauralHeadphoneEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("headphoneequalizer"), Some("BinauralHeadphoneEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midisysexmessagerouter"), Some("MidiSysExMessageRouter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sysexmessagerouter"), Some("MidiSysExMessageRouter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midimpevoicedistributor"), Some("MidiMpeVoiceDistributor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mpevoicedistributor"), Some("MidiMpeVoiceDistributor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("generativeeuclideanarpeggiator"), Some("GenerativeEuclideanArpeggiator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("euclideanarpeggiator"), Some("GenerativeEuclideanArpeggiator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalscalequantizer"), Some("MicrotonalScaleQuantizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalquantizer"), Some("MicrotonalScaleQuantizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmoniccadenceanalyzer"), Some("HarmonicCadenceAnalyzer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cadenceanalyzer"), Some("HarmonicCadenceAnalyzer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralphasevocoderpitchshifter"), Some("SpectralPhaseVocoderPitchShifter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasevocoderpitchshifter"), Some("SpectralPhaseVocoderPitchShifter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerolatencyconvolutionengine"), Some("ZeroLatencyConvolutionEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerolatencyconvolution"), Some("ZeroLatencyConvolutionEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicconvolutionmorpher"), Some("DynamicConvolutionMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("convolutionmorpher"), Some("DynamicConvolutionMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbasssynthesizer"), Some("SubHarmonicBassSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbass"), Some("SubHarmonicBassSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtapeflutterswowgenerator"), Some("AnalogTapeFluttersWowGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeflutterswow"), Some("AnalogTapeFluttersWowGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientsplaystereowidener"), Some("TransientSplayStereoWidener"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientstereowidener"), Some("TransientSplayStereoWidener"));
}

#[test]
fn test_tier134_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1424, "Inventory must contain at least 1424 DSP modules");
    assert!(registry.list_all().len() >= 1424, "Registry list_all must be >= 1424");

    let tier134_nodes = [
        ("MasterLinearPhaseEq", DspNodeCategory::FilterEq),
        ("DynamicResonanceSuppressor", DspNodeCategory::FilterEq),
        ("SpectralDeHumFilter", DspNodeCategory::FilterEq),
        ("TransientPhaseAligner", DspNodeCategory::Utility),
        ("MidSideSpectralHarmonizer", DspNodeCategory::SpatialSurround),
        ("BinauralAmbisonicDecoder", DspNodeCategory::SpatialSurround),
        ("DolbyAtmosBedDownmixer", DspNodeCategory::SpatialSurround),
        ("AnalogTapeHysteresisModel", DspNodeCategory::DistortionSaturation),
        ("TubeTransformerSaturator", DspNodeCategory::DistortionSaturation),
        ("DynamicMultibandGate", DspNodeCategory::DynamicsMaster),
        ("StereoPhaseCorrelationMeter", DspNodeCategory::Utility),
        ("AcousticRoomRaytracer", DspNodeCategory::SpatialSurround),
        ("GranularFreezeReverb", DspNodeCategory::TimeSpace),
        ("SpectralMorphHarmonizer", DspNodeCategory::SpectralResynthesis),
        ("AnalogSpringReverbTank", DspNodeCategory::TimeSpace),
        ("WavetablePhaseDistortionOsc", DspNodeCategory::Oscillator),
        ("GenerativeMarkovRhythmSequencer", DspNodeCategory::Modulation),
        ("MicrotonalMtsEspBroadcast", DspNodeCategory::Utility),
        ("OfflineLufsBatchNormalizer", DspNodeCategory::Utility),
        ("SessionCrashRecoveryVault", DspNodeCategory::Utility),
        ("AudioThreadJitterGuard", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier134_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("masterlinearphaseeq"), Some("MasterLinearPhaseEq"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphaseeq"), Some("MasterLinearPhaseEq"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicresonancesuppressor"), Some("DynamicResonanceSuppressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonancesuppressor"), Some("DynamicResonanceSuppressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectraldehumfilter"), Some("SpectralDeHumFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dehumfilter"), Some("SpectralDeHumFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientphasealigner"), Some("TransientPhaseAligner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientphase"), Some("TransientPhaseAligner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidespectralharmonizer"), Some("MidSideSpectralHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsideharmonizer"), Some("MidSideSpectralHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralambisonicdecoder"), Some("BinauralAmbisonicDecoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicdecoder"), Some("BinauralAmbisonicDecoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dolbyatmosbeddownmixer"), Some("DolbyAtmosBedDownmixer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("atmosbeddownmixer"), Some("DolbyAtmosBedDownmixer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtapehysteresismodel"), Some("AnalogTapeHysteresisModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapehysteresis"), Some("AnalogTapeHysteresisModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubetransformersaturator"), Some("TubeTransformerSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubetransformer"), Some("TubeTransformerSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicmultibandgate"), Some("DynamicMultibandGate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandgate"), Some("DynamicMultibandGate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereophasecorrelationmeter"), Some("StereoPhaseCorrelationMeter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasecorrelationmeter"), Some("StereoPhaseCorrelationMeter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticroomraytracer"), Some("AcousticRoomRaytracer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("roomraytracer"), Some("AcousticRoomRaytracer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularfreezereverb"), Some("GranularFreezeReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("freezereverb"), Some("GranularFreezeReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralmorphharmonizer"), Some("SpectralMorphHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphharmonizer"), Some("SpectralMorphHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogspringreverbtank"), Some("AnalogSpringReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springreverbtank"), Some("AnalogSpringReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavetablephasedistortionosc"), Some("WavetablePhaseDistortionOsc"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasedistortionosc"), Some("WavetablePhaseDistortionOsc"));
    assert_eq!(DspNodeRegistry::normalize_type_name("generativemarkovrhythmsequencer"), Some("GenerativeMarkovRhythmSequencer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("markovrhythmsequencer"), Some("GenerativeMarkovRhythmSequencer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalmtespbroadcast"), Some("MicrotonalMtsEspBroadcast"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mtespbroadcast"), Some("MicrotonalMtsEspBroadcast"));
    assert_eq!(DspNodeRegistry::normalize_type_name("offlinelufsbatchnormalizer"), Some("OfflineLufsBatchNormalizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lufsbatchnormalizer"), Some("OfflineLufsBatchNormalizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sessioncrashrecoveryvault"), Some("SessionCrashRecoveryVault"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crashrecoveryvault"), Some("SessionCrashRecoveryVault"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiothreadjitterguard"), Some("AudioThreadJitterGuard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("threadjitterguard"), Some("AudioThreadJitterGuard"));
}

#[test]
fn test_tier135_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1445, "Inventory must contain at least 1445 DSP modules");
    assert!(registry.list_all().len() >= 1445, "Registry list_all must be >= 1445");

    let tier135_nodes = [
        ("FranklinGlassArmonica", DspNodeCategory::Oscillator),
        ("FrenchHurdyGurdy", DspNodeCategory::Oscillator),
        ("NonlinearJawariBridge", DspNodeCategory::FilterEq),
        ("TrompetteChienBridge", DspNodeCategory::DistortionSaturation),
        ("CassottoToneChamber", DspNodeCategory::FilterEq),
        ("WoodwindToneholeGrid", DspNodeCategory::FilterEq),
        ("BilinearAirReedJet", DspNodeCategory::Oscillator),
        ("StribeckFrictionDriver", DspNodeCategory::Modulation),
        ("NonlinearHammerStrike", DspNodeCategory::DynamicsMaster),
        ("AcousticHornReflection", DspNodeCategory::FilterEq),
        ("ModalIdiophoneResonator", DspNodeCategory::Oscillator),
        ("TineResonatorPickup", DspNodeCategory::Oscillator),
        ("SoundboardBridgeCoupler", DspNodeCategory::FilterEq),
        ("SpringLatticeReverberator", DspNodeCategory::TimeSpace),
        ("PercussionMembrane2D", DspNodeCategory::Oscillator),
        ("PipeOrganWindchest", DspNodeCategory::Utility),
        ("WaveguideBrassMouthpiece", DspNodeCategory::Oscillator),
        ("RectilinearWaveguideMesh", DspNodeCategory::TimeSpace),
        ("ZeroGravityFluidResonator", DspNodeCategory::SpectralResynthesis),
        ("GlottalPulseGenerator", DspNodeCategory::Oscillator),
        ("JiJustIntonationBridge", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier135_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("franklinglassarmonica"), Some("FranklinGlassArmonica"));
    assert_eq!(DspNodeRegistry::normalize_type_name("franklinarmonica"), Some("FranklinGlassArmonica"));
    assert_eq!(DspNodeRegistry::normalize_type_name("frenchhurdygurdy"), Some("FrenchHurdyGurdy"));
    assert_eq!(DspNodeRegistry::normalize_type_name("viellearoue"), Some("FrenchHurdyGurdy"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearjawaribridge"), Some("NonlinearJawariBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearjawari"), Some("NonlinearJawariBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("trompettechienbridge"), Some("TrompetteChienBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chiendogbridge"), Some("TrompetteChienBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cassottotonechamber"), Some("CassottoToneChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cassottoacoustic"), Some("CassottoToneChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("woodwindtoneholegrid"), Some("WoodwindToneholeGrid"));
    assert_eq!(DspNodeRegistry::normalize_type_name("woodwindtoneholelattice"), Some("WoodwindToneholeGrid"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bilinearairreedjet"), Some("BilinearAirReedJet"));
    assert_eq!(DspNodeRegistry::normalize_type_name("airreedjet"), Some("BilinearAirReedJet"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stribeckfrictiondriver"), Some("StribeckFrictionDriver"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stribeckdriver"), Some("StribeckFrictionDriver"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pianohammerstrike"), Some("NonlinearHammerStrike"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pianohammerimpact"), Some("NonlinearHammerStrike"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acoustichornreflection"), Some("AcousticHornReflection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hornreflection"), Some("AcousticHornReflection"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalidiophoneresonator"), Some("ModalIdiophoneResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("marimbabarresonator"), Some("ModalIdiophoneResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tineresonatorpickup"), Some("TineResonatorPickup"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rhodestinepickup"), Some("TineResonatorPickup"));
    assert_eq!(DspNodeRegistry::normalize_type_name("soundboardbridgecoupler"), Some("SoundboardBridgeCoupler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bridgecoupler"), Some("SoundboardBridgeCoupler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springlatticereverberator"), Some("SpringLatticeReverberator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springreverblatticefx"), Some("SpringLatticeReverberator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("percussionmembrane2d"), Some("PercussionMembrane2D"));
    assert_eq!(DspNodeRegistry::normalize_type_name("drummembrane2d"), Some("PercussionMembrane2D"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pipeorganwindchest"), Some("PipeOrganWindchest"));
    assert_eq!(DspNodeRegistry::normalize_type_name("windchestengine"), Some("PipeOrganWindchest"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveguidebrassmouthpiece"), Some("WaveguideBrassMouthpiece"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lipreedmouthpiece"), Some("WaveguideBrassMouthpiece"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rectilinearwaveguidemesh"), Some("RectilinearWaveguideMesh"));
    assert_eq!(DspNodeRegistry::normalize_type_name("meshplate2d"), Some("RectilinearWaveguideMesh"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerogravityfluidresonator"), Some("ZeroGravityFluidResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bubblecavitation"), Some("ZeroGravityFluidResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glottalpulsegenerator"), Some("GlottalPulseGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glottalpulsegene"), Some("GlottalPulseGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jijustintonationbridge"), Some("JiJustIntonationBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("justintonationbridge"), Some("JiJustIntonationBridge"));
}

#[test]
fn test_tier136_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1466, "Inventory must contain at least 1466 DSP modules");
    assert!(registry.list_all().len() >= 1466, "Registry list_all must be >= 1466");

    let tier136_nodes = [
        ("ElectroAcousticResonator", DspNodeCategory::Oscillator),
        ("CryogenicLatticeReverb", DspNodeCategory::TimeSpace),
        ("HarmonicExcitanceSynthesizer", DspNodeCategory::SpectralResynthesis),
        ("SpectralTransientIsolator", DspNodeCategory::FilterEq),
        ("MorphologicalWaveTerrainOsc", DspNodeCategory::Oscillator),
        ("AdaptiveNeuralDeEsser", DspNodeCategory::DynamicsMaster),
        ("SpatialBinauralPanner3D", DspNodeCategory::SpatialSurround),
        ("DynamicConvolutionCabinet", DspNodeCategory::DistortionSaturation),
        ("StochasticGranularCloud", DspNodeCategory::TimeSpace),
        ("BesselLinearPhaseCrossover", DspNodeCategory::FilterEq),
        ("AnalogTapeDeckTransport", DspNodeCategory::Modulation),
        ("SpectralFormantMorpher", DspNodeCategory::SpectralResynthesis),
        ("MultibandUpwardCompressor", DspNodeCategory::DynamicsMaster),
        ("ChaoticAttractorModulator", DspNodeCategory::Modulation),
        ("AcousticSoundboardResonator", DspNodeCategory::Oscillator),
        ("DynamicPhaseRotator", DspNodeCategory::FilterEq),
        ("VectorWaveSynthesisEngine", DspNodeCategory::Oscillator),
        ("SubharmonicTransientExciter", DspNodeCategory::DynamicsMaster),
        ("AmbisonicRotatorBinaural", DspNodeCategory::SpatialSurround),
        ("LofiVinylArtifactGenerator", DspNodeCategory::DistortionSaturation),
        ("AudioTelemetryStreamer", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier136_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("electroacousticresonator"), Some("ElectroAcousticResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("electroacoustic"), Some("ElectroAcousticResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cryogeniclatticereverb"), Some("CryogenicLatticeReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cryogenicreverb"), Some("CryogenicLatticeReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicexcitancesynthesizer"), Some("HarmonicExcitanceSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicexcitance"), Some("HarmonicExcitanceSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectraltransientisolator"), Some("SpectralTransientIsolator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientisolator"), Some("SpectralTransientIsolator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphologicalwaveterrainosc"), Some("MorphologicalWaveTerrainOsc"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveterrainosc"), Some("MorphologicalWaveTerrainOsc"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptiveneuraldeesser"), Some("AdaptiveNeuralDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldeesser"), Some("AdaptiveNeuralDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spatialbinauralpanner3d"), Some("SpatialBinauralPanner3D"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralpanner3d"), Some("SpatialBinauralPanner3D"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicconvolutioncabinet"), Some("DynamicConvolutionCabinet"));
    assert_eq!(DspNodeRegistry::normalize_type_name("convolutioncabinet"), Some("DynamicConvolutionCabinet"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticgranularcloud"), Some("StochasticGranularCloud"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticcloud"), Some("StochasticGranularCloud"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bessellinearphasecrossover"), Some("BesselLinearPhaseCrossover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselcrossover"), Some("BesselLinearPhaseCrossover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtapedecktransport"), Some("AnalogTapeDeckTransport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapedecktransport"), Some("AnalogTapeDeckTransport"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralformantmorpher"), Some("SpectralFormantMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantmorpher"), Some("SpectralFormantMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandupwardcompressor"), Some("MultibandUpwardCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("upwardcompressor"), Some("MultibandUpwardCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticattractormodulator"), Some("ChaoticAttractorModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticattractor"), Some("ChaoticAttractorModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticsoundboardresonator"), Some("AcousticSoundboardResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("soundboardresonator"), Some("AcousticSoundboardResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicphaserotator"), Some("DynamicPhaseRotator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phaserotator"), Some("DynamicPhaseRotator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vectorwavesynthesisengine"), Some("VectorWaveSynthesisEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vectorwaveosc"), Some("VectorWaveSynthesisEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonictransientexciter"), Some("SubharmonicTransientExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subtransientexciter"), Some("SubharmonicTransientExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicrotatorbinaural"), Some("AmbisonicRotatorBinaural"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicrotator"), Some("AmbisonicRotatorBinaural"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lofivinylartifactgenerator"), Some("LofiVinylArtifactGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vinylartifactgenerator"), Some("LofiVinylArtifactGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("audiotelemetrystreamer"), Some("AudioTelemetryStreamer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("telemetrystreamer"), Some("AudioTelemetryStreamer"));
}

#[test]
fn test_tier137_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1487, "Inventory must contain at least 1487 DSP modules");
    assert!(registry.list_all().len() >= 1487, "Registry list_all must be >= 1487");

    let tier137_nodes = [
        ("SpectralInharmonicResonator", DspNodeCategory::SpectralResynthesis),
        ("TruePeakOversampledLimiter", DspNodeCategory::DynamicsMaster),
        ("MultibandTransientSaturator", DspNodeCategory::DistortionSaturation),
        ("BinauralNearfieldSimulator", DspNodeCategory::SpatialSurround),
        ("VariableTapeHeadAzimuth", DspNodeCategory::DistortionSaturation),
        ("FormantVowelMorphMatrix", DspNodeCategory::SpectralResynthesis),
        ("ResonantCombLatticeFilter", DspNodeCategory::FilterEq),
        ("AeroacousticWhistleExciter", DspNodeCategory::AcousticPhysicalModel),
        ("HarmonicStereoSpreadWidener", DspNodeCategory::SpatialSurround),
        ("DynamicTransientDeEsser", DspNodeCategory::DynamicsMaster),
        ("AnalogBucketBrigadeChorus", DspNodeCategory::TimeSpace),
        ("StochasticMarkovArpeggiator", DspNodeCategory::Modulation),
        ("MultichannelSurroundBedUpmixer", DspNodeCategory::SpatialSurround),
        ("SubharmonicBassReinforcer", DspNodeCategory::DynamicsMaster),
        ("PolyphonicAudioPitchTracker", DspNodeCategory::Utility),
        ("ConvolutionImpulseReverbMorpher", DspNodeCategory::TimeSpace),
        ("NonlinearTubeBiasTremolo", DspNodeCategory::Modulation),
        ("AcousticStringSympatheticResonator", DspNodeCategory::AcousticPhysicalModel),
        ("LinearPhaseMultibandSplitter", DspNodeCategory::FilterEq),
        ("AnalogCassetteTapeWarmth", DspNodeCategory::DistortionSaturation),
        ("RealtimeLatencyCompensationEngine", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier137_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralinharmonicresonator"), Some("SpectralInharmonicResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("inharmonicresonator"), Some("SpectralInharmonicResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakoversampledlimiter"), Some("TruePeakOversampledLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringlimiter"), Some("TruePeakOversampledLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandtransientsaturator"), Some("MultibandTransientSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientsaturator"), Some("MultibandTransientSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralnearfieldsimulator"), Some("BinauralNearfieldSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nearfieldbinaural"), Some("BinauralNearfieldSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("variabletapeheadazimuth"), Some("VariableTapeHeadAzimuth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeazimuth"), Some("VariableTapeHeadAzimuth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantvowelmorphmatrix"), Some("FormantVowelMorphMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vowelmorphmatrix"), Some("FormantVowelMorphMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonantcomblatticefilter"), Some("ResonantCombLatticeFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("comblatticefilter"), Some("ResonantCombLatticeFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroacousticwhistleexciter"), Some("AeroacousticWhistleExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("whistleexciter"), Some("AeroacousticWhistleExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicstereospreadwidener"), Some("HarmonicStereoSpreadWidener"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereospreadwidener"), Some("HarmonicStereoSpreadWidener"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamictransientdeesser"), Some("DynamicTransientDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientdeesser"), Some("DynamicTransientDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogbucketbrigadechorus"), Some("AnalogBucketBrigadeChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdchorus"), Some("AnalogBucketBrigadeChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticmarkovarpeggiator"), Some("StochasticMarkovArpeggiator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("markovarpeggiator"), Some("StochasticMarkovArpeggiator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelsurroundbedupmixer"), Some("MultichannelSurroundBedUpmixer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("surroundbedupmixer"), Some("MultichannelSurroundBedUpmixer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbassreinforcer"), Some("SubharmonicBassReinforcer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicreinforcer"), Some("SubharmonicBassReinforcer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyphonicaudiopitchtracker"), Some("PolyphonicAudioPitchTracker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyphonicpitchtracker"), Some("PolyphonicAudioPitchTracker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("convolutionimpulsereverbmorpher"), Some("ConvolutionImpulseReverbMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("reverbmorpher"), Some("ConvolutionImpulseReverbMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlineartubebiastremolo"), Some("NonlinearTubeBiasTremolo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubebiastremolo"), Some("NonlinearTubeBiasTremolo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticstringsympatheticresonator"), Some("AcousticStringSympatheticResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sympatheticresonator"), Some("AcousticStringSympatheticResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphasemultibandsplitter"), Some("LinearPhaseMultibandSplitter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphasesplitter"), Some("LinearPhaseMultibandSplitter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogcassettetapewarmth"), Some("AnalogCassetteTapeWarmth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cassettetapewarmth"), Some("AnalogCassetteTapeWarmth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("realtimelatencycompensationengine"), Some("RealtimeLatencyCompensationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("realtimelatencycompensation"), Some("RealtimeLatencyCompensationEngine"));
}

#[test]
fn test_tier138_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1508, "Inventory must contain at least 1508 DSP modules");
    assert!(registry.list_all().len() >= 1508, "Registry list_all must be >= 1508");

    let tier138_nodes = [
        ("BilinearDiodeLadderFilter", DspNodeCategory::FilterEq),
        ("HarmonicRotaryDopplerHorn", DspNodeCategory::Modulation),
        ("AdaptiveSpectralDenoiser", DspNodeCategory::DynamicsMaster),
        ("GranularFreezeBuffer", DspNodeCategory::TimeSpace),
        ("DynamicAirAbsorptionAcoustics", DspNodeCategory::SpatialSurround),
        ("MidSideSpectralStereoImager", DspNodeCategory::SpatialSurround),
        ("AnalogTapeSaturatorBiasCore", DspNodeCategory::DistortionSaturation),
        ("SpectralResonanceSuppressor", DspNodeCategory::SpectralResynthesis),
        ("BowedPhysicalCelloModel", DspNodeCategory::AcousticPhysicalModel),
        ("TruePeakMasteringClipper", DspNodeCategory::DynamicsMaster),
        ("AsynchronousSampleRateConverter", DspNodeCategory::Utility),
        ("MultistagePhaserMatrix", DspNodeCategory::Modulation),
        ("HarmonicSubBassExciter", DspNodeCategory::DynamicsMaster),
        ("DiffusivePlateReverberator", DspNodeCategory::TimeSpace),
        ("AmbisonicBFormatDecoder", DspNodeCategory::SpatialSurround),
        ("OpticalVactrolCompressor", DspNodeCategory::DynamicsMaster),
        ("ModalIdiophoneChimeBank", DspNodeCategory::AcousticPhysicalModel),
        ("SpectralVocoderModulatorBank", DspNodeCategory::SpectralResynthesis),
        ("BBDAnalogFlangerChorus", DspNodeCategory::TimeSpace),
        ("MasterMultibandSaturatorExciter", DspNodeCategory::DistortionSaturation),
        ("MultichannelStemAudioRouter", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier138_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("bilineardiodeladderfilter"), Some("BilinearDiodeLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diodeladder"), Some("BilinearDiodeLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicrotarydopplerhorn"), Some("HarmonicRotaryDopplerHorn"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lesliehorn"), Some("HarmonicRotaryDopplerHorn"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivespectraldenoiser"), Some("AdaptiveSpectralDenoiser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("denoiser"), Some("AdaptiveSpectralDenoiser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularfreezebuffer"), Some("GranularFreezeBuffer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grainfreeze"), Some("GranularFreezeBuffer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicairabsorptionacoustics"), Some("DynamicAirAbsorptionAcoustics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicairabsorption"), Some("DynamicAirAbsorptionAcoustics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidespectralstereoimager"), Some("MidSideSpectralStereoImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsideimager"), Some("MidSideSpectralStereoImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtapesaturatorbiascore"), Some("AnalogTapeSaturatorBiasCore"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapebiascore"), Some("AnalogTapeSaturatorBiasCore"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralresonancesuppressor"), Some("SpectralResonanceSuppressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harshnesstamer"), Some("SpectralResonanceSuppressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bowedphysicalcellomodel"), Some("BowedPhysicalCelloModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cellomodel"), Some("BowedPhysicalCelloModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakmasteringclipper"), Some("TruePeakMasteringClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringclipper"), Some("TruePeakMasteringClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asynchronoussamplerateconverter"), Some("AsynchronousSampleRateConverter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asrcresampler"), Some("AsynchronousSampleRateConverter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multistagephasermatrix"), Some("MultistagePhaserMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogphaser"), Some("MultistagePhaserMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicsubbassexciter"), Some("HarmonicSubBassExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicsubexciter"), Some("HarmonicSubBassExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusiveplatereverberator"), Some("DiffusivePlateReverberator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusiveplatereverb"), Some("DiffusivePlateReverberator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicbformatdecoder"), Some("AmbisonicBFormatDecoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bformatdecoder"), Some("AmbisonicBFormatDecoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalvactrolcompressor"), Some("OpticalVactrolCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("optoleveler"), Some("OpticalVactrolCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("modalidiophonechimebank"), Some("ModalIdiophoneChimeBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chimebank"), Some("ModalIdiophoneChimeBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralvocodermodulatorbank"), Some("SpectralVocoderModulatorBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocoderbank"), Some("SpectralVocoderModulatorBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdanalogflangerchorus"), Some("BBDAnalogFlangerChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdflanger"), Some("BBDAnalogFlangerChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mastermultibandsaturatorexciter"), Some("MasterMultibandSaturatorExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masterexciter"), Some("MasterMultibandSaturatorExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelstemaudiorouter"), Some("MultichannelStemAudioRouter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stemrouter"), Some("MultichannelStemAudioRouter"));
}

#[test]
fn test_tier139_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1529, "Inventory must contain at least 1529 DSP modules");
    assert!(registry.list_all().len() >= 1529, "Registry list_all must be >= 1529");

    let tier139_nodes = [
        ("ZeroGravityFluidAcousticModel", DspNodeCategory::AcousticPhysicalModel),
        ("QuantumSuperpositionOscillator", DspNodeCategory::Oscillator),
        ("AiIntelligentSpectralUnmasker", DspNodeCategory::SpectralResynthesis),
        ("NeuralAmpInferenceEngine", DspNodeCategory::DistortionSaturation),
        ("NeuromorphicSpikingOscillator", DspNodeCategory::Oscillator),
        ("PiezoelectricCrystalFilter", DspNodeCategory::FilterEq),
        ("InverseFftSpectrogramSonifier", DspNodeCategory::SpectralResynthesis),
        ("MultiHelicalSpringLatticeReverb", DspNodeCategory::TimeSpace),
        ("CassottoToneChamberResonator", DspNodeCategory::AcousticPhysicalModel),
        ("SplitJetFluePipeExciter", DspNodeCategory::AcousticPhysicalModel),
        ("HurdyGurdyTrompetteBridge", DspNodeCategory::AcousticPhysicalModel),
        ("CurvedJawariSitarBridge", DspNodeCategory::AcousticPhysicalModel),
        ("JustIntonationCommaCompensator", DspNodeCategory::Modulation),
        ("LuGreBowedFrictionExciter", DspNodeCategory::AcousticPhysicalModel),
        ("EurorackCvHardwareBridge", DspNodeCategory::Utility),
        ("SandboxedPluginHostContainer", DspNodeCategory::Utility),
        ("DynamicLoudnessTargetLeveler", DspNodeCategory::DynamicsMaster),
        ("MultibandTransientHarmonizer", DspNodeCategory::DynamicsMaster),
        ("BinauralSphericalHeadModel", DspNodeCategory::SpatialSurround),
        ("PhaseAlignedStereoCrossover", DspNodeCategory::FilterEq),
        ("MasteringTruePeakDeClipper", DspNodeCategory::DynamicsMaster),
    ];

    for (name, expected_cat) in tier139_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("zerogravityfluidacousticmodel"), Some("ZeroGravityFluidAcousticModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerogravityfluid"), Some("ZeroGravityFluidAcousticModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumsuperpositionoscillator"), Some("QuantumSuperpositionOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("qubitoscillator"), Some("QuantumSuperpositionOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aiintelligentspectralunmasker"), Some("AiIntelligentSpectralUnmasker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aiunmasker"), Some("AiIntelligentSpectralUnmasker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralampinferenceengine"), Some("NeuralAmpInferenceEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralamp"), Some("NeuralAmpInferenceEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuromorphicspikingoscillator"), Some("NeuromorphicSpikingOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lifoscillator"), Some("NeuromorphicSpikingOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("piezoelectriccrystalfilter"), Some("PiezoelectricCrystalFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("piezoelectricfilter"), Some("PiezoelectricCrystalFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("inversefftspectrogramsonifier"), Some("InverseFftSpectrogramSonifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rastersonifier"), Some("InverseFftSpectrogramSonifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multihelicalspringlatticereverb"), Some("MultiHelicalSpringLatticeReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("helicalspringreverb"), Some("MultiHelicalSpringLatticeReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cassottotonechamberresonator"), Some("CassottoToneChamberResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cassottoresonator"), Some("CassottoToneChamberResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("splitjetfluepipeexciter"), Some("SplitJetFluePipeExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("splitjetexciter"), Some("SplitJetFluePipeExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hurdygurdytrompettebridge"), Some("HurdyGurdyTrompetteBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hurdychienbridge"), Some("HurdyGurdyTrompetteBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("curvedjawarisitarbridge"), Some("CurvedJawariSitarBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("curvedjawaribridge"), Some("CurvedJawariSitarBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("justintonationcommacompensator"), Some("JustIntonationCommaCompensator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("justintonationcompensator"), Some("JustIntonationCommaCompensator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lugrebowedfrictionexciter"), Some("LuGreBowedFrictionExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lugrefriction"), Some("LuGreBowedFrictionExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("eurorackcvhardwarebridge"), Some("EurorackCvHardwareBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("eurorackbridge"), Some("EurorackCvHardwareBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sandboxedpluginhostcontainer"), Some("SandboxedPluginHostContainer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sandboxedpluginhost"), Some("SandboxedPluginHostContainer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicloudnesstargetleveler"), Some("DynamicLoudnessTargetLeveler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loudnessleveler"), Some("DynamicLoudnessTargetLeveler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandtransientharmonizer"), Some("MultibandTransientHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientharmonizer"), Some("MultibandTransientHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralsphericalheadmodel"), Some("BinauralSphericalHeadModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralheadmodel"), Some("BinauralSphericalHeadModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasealignedstereocrossover"), Some("PhaseAlignedStereoCrossover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereocrossover"), Some("PhaseAlignedStereoCrossover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringtruepeakdeclipper"), Some("MasteringTruePeakDeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("declipper"), Some("MasteringTruePeakDeClipper"));
}

#[test]
fn test_tier140_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1550, "Inventory must contain at least 1550 DSP modules");
    assert!(registry.list_all().len() >= 1550, "Registry list_all must be >= 1550");

    let tier140_nodes = [
        ("BinauralDopplerMotionPanner", DspNodeCategory::SpatialSurround),
        ("GranularScatterFreezeCloud", DspNodeCategory::TimeSpace),
        ("SpectralTransientDeClicker", DspNodeCategory::DynamicsMaster),
        ("NonlinearTapeFlutterDrive", DspNodeCategory::DistortionSaturation),
        ("AcousticMembraneResonanceMatrix", DspNodeCategory::AcousticPhysicalModel),
        ("PhaseCompensatedAllpassPhaser", DspNodeCategory::Modulation),
        ("NeuralTimbreMorphSynthesizer", DspNodeCategory::SpectralResynthesis),
        ("MultichannelLoudnessRadarMeter", DspNodeCategory::DynamicsMaster),
        ("MicrotonalScaleQuantizerBridge", DspNodeCategory::Utility),
        ("TubeOptoMasterCompressor", DspNodeCategory::DynamicsMaster),
        ("BilinearStateVariableMorphFilter", DspNodeCategory::FilterEq),
        ("AeroacousticFluteJetExciter", DspNodeCategory::AcousticPhysicalModel),
        ("HarmonicResonanceCombBank", DspNodeCategory::FilterEq),
        ("SubharmonicBassSynthesizerEngine", DspNodeCategory::Oscillator),
        ("AmbisonicDiffractionEncoder", DspNodeCategory::SpatialSurround),
        ("ConvolutionCabinetMorphMatrix", DspNodeCategory::DistortionSaturation),
        ("AsynchronousJitterBufferClock", DspNodeCategory::Utility),
        ("StochasticStrummedStringMatrix", DspNodeCategory::AcousticPhysicalModel),
        ("MasteringStereoWidthMaximizer", DspNodeCategory::SpatialSurround),
        ("DynamicSidechainSpectralDucker", DspNodeCategory::DynamicsMaster),
        ("AnalogSpringMassTankReverb", DspNodeCategory::TimeSpace),
    ];

    for (name, expected_cat) in tier140_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("binauraldopplermotionpanner"), Some("BinauralDopplerMotionPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dopplermotionpanner"), Some("BinauralDopplerMotionPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularscatterfreezecloud"), Some("GranularScatterFreezeCloud"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scatterfreezecloud"), Some("GranularScatterFreezeCloud"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectraltransientdeclicker"), Some("SpectralTransientDeClicker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("declicker"), Some("SpectralTransientDeClicker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlineartapeflutterdrive"), Some("NonlinearTapeFlutterDrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeflutterdrive"), Some("NonlinearTapeFlutterDrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticmembraneresonancematrix"), Some("AcousticMembraneResonanceMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("membraneresonator"), Some("AcousticMembraneResonanceMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasecompensatedallpassphaser"), Some("PhaseCompensatedAllpassPhaser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("allpassphaser"), Some("PhaseCompensatedAllpassPhaser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltimbremorphsynthesizer"), Some("NeuralTimbreMorphSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timbremorphsynthesizer"), Some("NeuralTimbreMorphSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelloudnessradarmeter"), Some("MultichannelLoudnessRadarMeter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loudnessradar"), Some("MultichannelLoudnessRadarMeter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalscalequantizerbridge"), Some("MicrotonalScaleQuantizerBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mtespscalaquantizer"), Some("MicrotonalScaleQuantizerBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubeoptomastercompressor"), Some("TubeOptoMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("optomastercompressor"), Some("TubeOptoMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bilinearstatevariablemorphfilter"), Some("BilinearStateVariableMorphFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphfilter"), Some("BilinearStateVariableMorphFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroacousticflutejetexciter"), Some("AeroacousticFluteJetExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("flutejet"), Some("AeroacousticFluteJetExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicresonancecombbank"), Some("HarmonicResonanceCombBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonancecombbank"), Some("HarmonicResonanceCombBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbasssynthesizerengine"), Some("SubharmonicBassSynthesizerEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbasssynth"), Some("SubharmonicBassSynthesizerEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicdiffractionencoder"), Some("AmbisonicDiffractionEncoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffractionencoder"), Some("AmbisonicDiffractionEncoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("convolutioncabinetmorphmatrix"), Some("ConvolutionCabinetMorphMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("convolutioncabmorph"), Some("ConvolutionCabinetMorphMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asynchronousjitterbufferclock"), Some("AsynchronousJitterBufferClock"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jitterbufferclock"), Some("AsynchronousJitterBufferClock"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticstrummedstringmatrix"), Some("StochasticStrummedStringMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("strummedstring"), Some("StochasticStrummedStringMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringstereowidthmaximizer"), Some("MasteringStereoWidthMaximizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereowidthmaximizer"), Some("MasteringStereoWidthMaximizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicsidechainspectralducker"), Some("DynamicSidechainSpectralDucker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sidechaincarver"), Some("DynamicSidechainSpectralDucker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogspringmasstankreverb"), Some("AnalogSpringMassTankReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springmasstank"), Some("AnalogSpringMassTankReverb"));
}

#[test]
fn test_tier141_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1571, "Inventory must contain at least 1571 DSP modules");
    assert!(registry.list_all().len() >= 1571, "Registry list_all must be >= 1571");

    let tier141_nodes = [
        ("CepstralEnvelopeMorphSynthesizer", DspNodeCategory::SpectralResynthesis),
        ("TriodePentodeMasterSaturator", DspNodeCategory::DistortionSaturation),
        ("LinearPhaseMultibandIsolator", DspNodeCategory::FilterEq),
        ("BinauralRoomImpulseSpatializer", DspNodeCategory::SpatialSurround),
        ("PolyphonicGranularCloudSynthesizer", DspNodeCategory::Oscillator),
        ("VariMuMasterLimiter", DspNodeCategory::DynamicsMaster),
        ("DiffusionNetworkReverb", DspNodeCategory::TimeSpace),
        ("StereoRotaryDopplerFlanger", DspNodeCategory::Modulation),
        ("PsychoacousticSpectralEnhancer", DspNodeCategory::DistortionSaturation),
        ("MicrotonalSpectralResonatorBank", DspNodeCategory::FilterEq),
        ("TransientSustainDecomposer", DspNodeCategory::DynamicsMaster),
        ("AcousticUprightBassBodyModel", DspNodeCategory::AcousticPhysicalModel),
        ("BesselFilterArrayEqualizer", DspNodeCategory::FilterEq),
        ("MidSidePsychoacousticImager", DspNodeCategory::SpatialSurround),
        ("NonlinearTapeEchoDeck", DspNodeCategory::TimeSpace),
        ("NeuralAmpInferenceBridge", DspNodeCategory::DistortionSaturation),
        ("AdaptiveMainsHumCanceller", DspNodeCategory::Utility),
        ("SallenKeyResonantFilter", DspNodeCategory::FilterEq),
        ("AcousticSnareDrumPhysicalModel", DspNodeCategory::AcousticPhysicalModel),
        ("ParallelUpwardExpansionMaximizer", DspNodeCategory::DynamicsMaster),
        ("MasteringPsychoacousticDither", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier141_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("cepstralenvelopemorphsynthesizer"), Some("CepstralEnvelopeMorphSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cepstralmorph"), Some("CepstralEnvelopeMorphSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodepentodemastersaturator"), Some("TriodePentodeMasterSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodepentodesat"), Some("TriodePentodeMasterSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphasemultibandisolator"), Some("LinearPhaseMultibandIsolator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphasefirisolator"), Some("LinearPhaseMultibandIsolator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralroomimpulsespatializer"), Some("BinauralRoomImpulseSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralroomimpmatch"), Some("BinauralRoomImpulseSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyphonicgranularcloudsynthesizer"), Some("PolyphonicGranularCloudSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polygranularcloud"), Some("PolyphonicGranularCloudSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("varimumasterlimiter"), Some("VariMuMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("valvelimiterbus"), Some("VariMuMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusionnetworkreverb"), Some("DiffusionNetworkReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hadamardfdnreverb"), Some("DiffusionNetworkReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereorotarydopplerflanger"), Some("StereoRotaryDopplerFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dopplerrotaryflange"), Some("StereoRotaryDopplerFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticspectralenhancer"), Some("PsychoacousticSpectralEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralenhancerexciter"), Some("PsychoacousticSpectralEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalspectralresonatorbank"), Some("MicrotonalSpectralResonatorBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalfilterbank"), Some("MicrotonalSpectralResonatorBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientsustaindecomposer"), Some("TransientSustainDecomposer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientharmonicdecomposer"), Some("TransientSustainDecomposer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticuprightbassbodymodel"), Some("AcousticUprightBassBodyModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("uprightbassphysmodel"), Some("AcousticUprightBassBodyModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselfilterarrayequalizer"), Some("BesselFilterArrayEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselpolyequalizer"), Some("BesselFilterArrayEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidepsychoacousticimager"), Some("MidSidePsychoacousticImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidespatialimager"), Some("MidSidePsychoacousticImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlineartapeechodeck"), Some("NonlinearTapeEchoDeck"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitapeechodeck"), Some("NonlinearTapeEchoDeck"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralampinferencebridge"), Some("NeuralAmpInferenceBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavenetampinference"), Some("NeuralAmpInferenceBridge"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivemainshumcanceller"), Some("AdaptiveMainsHumCanceller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivehumcanceller"), Some("AdaptiveMainsHumCanceller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sallenkeyresonantfilter"), Some("SallenKeyResonantFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sallenkeyactivefilter"), Some("SallenKeyResonantFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticsnaredrumphysicalmodel"), Some("AcousticSnareDrumPhysicalModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("snaredrumphysmodel"), Some("AcousticSnareDrumPhysicalModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parallelupwardexpansionmaximizer"), Some("ParallelUpwardExpansionMaximizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("upwarddynamicecom"), Some("ParallelUpwardExpansionMaximizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringpsychoacousticdither"), Some("MasteringPsychoacousticDither"));
    assert_eq!(DspNodeRegistry::normalize_type_name("noiseshapeddither"), Some("MasteringPsychoacousticDither"));
}

#[test]
fn test_tier142_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1592, "Inventory must contain at least 1592 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1592, "Registry list_all must be >= 1592, found {}", registry.list_all().len());

    let tier142_nodes = [
        ("HarmonicAdditiveResynthesisEngine", DspNodeCategory::SpectralResynthesis),
        ("OpticalTubeCompressorLimiter", DspNodeCategory::DynamicsMaster),
        ("StateVariableMorphLadderFilter", DspNodeCategory::FilterEq),
        ("AmbisonicBinauralDecoderSpatializer", DspNodeCategory::SpatialSurround),
        ("ChaoticStrangeAttractorLfo", DspNodeCategory::Modulation),
        ("AcousticConcertGrandBodyModel", DspNodeCategory::AcousticPhysicalModel),
        ("AnalogTapeHysteresisSimulator", DspNodeCategory::DistortionSaturation),
        ("SpectralPeakTrackingFilter", DspNodeCategory::FilterEq),
        ("ConvolutionReverbTrueStereo", DspNodeCategory::TimeSpace),
        ("TransientShaperDualBand", DspNodeCategory::DynamicsMaster),
        ("AeroacousticFluteTurbulenceModel", DspNodeCategory::AcousticPhysicalModel),
        ("DynamicStereoMicroDelayPanner", DspNodeCategory::SpatialSurround),
        ("GranularFreezeReverbBuffer", DspNodeCategory::TimeSpace),
        ("SubharmonicBassExciter", DspNodeCategory::DynamicsMaster),
        ("ResonantDiodeLadderFilter", DspNodeCategory::FilterEq),
        ("PolyphonicVoiceUnisonDetuner", DspNodeCategory::Modulation),
        ("SpectralContrastCompressor", DspNodeCategory::SpectralResynthesis),
        ("AcousticCelloBodyResonator", DspNodeCategory::AcousticPhysicalModel),
        ("MultibandMidSideEqualizer", DspNodeCategory::FilterEq),
        ("AnalogBucketBrigadeFlanger", DspNodeCategory::Modulation),
        ("MasteringTruePeakDitherLimiter", DspNodeCategory::DynamicsMaster),
    ];

    for (name, expected_cat) in tier142_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicadditiveresynthesisengine"), Some("HarmonicAdditiveResynthesisEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additiveresynthesisengine"), Some("HarmonicAdditiveResynthesisEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticaltubecompressorlimiter"), Some("OpticalTubeCompressorLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticaltubecompressor"), Some("OpticalTubeCompressorLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("statevariablemorphladderfilter"), Some("StateVariableMorphLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphladderfilter"), Some("StateVariableMorphLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicbinauraldecoderspatializer"), Some("AmbisonicBinauralDecoderSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hoadecoderspatializer"), Some("AmbisonicBinauralDecoderSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticstrangeattractorlfo"), Some("ChaoticStrangeAttractorLfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("strangeattractorlfo"), Some("ChaoticStrangeAttractorLfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertgrandbodymodel"), Some("AcousticConcertGrandBodyModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grandpianobodymodel"), Some("AcousticConcertGrandBodyModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtapehysteresissimulator"), Some("AnalogTapeHysteresisSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapehysteresissimulator"), Some("AnalogTapeHysteresisSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralpeaktrackingfilter"), Some("SpectralPeakTrackingFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("peaktrackingfilter"), Some("SpectralPeakTrackingFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("convolutionreverbtruestereo"), Some("ConvolutionReverbTrueStereo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truestereoconvolutionreverb"), Some("ConvolutionReverbTrueStereo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientshaperdualband"), Some("TransientShaperDualBand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualbandtransientshaper"), Some("TransientShaperDualBand"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroacousticfluteturbulencemodel"), Some("AeroacousticFluteTurbulenceModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fluteturbulencemodel"), Some("AeroacousticFluteTurbulenceModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicstereomicrodelaypanner"), Some("DynamicStereoMicroDelayPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microdelaypanner"), Some("DynamicStereoMicroDelayPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularfreezereverbbuffer"), Some("GranularFreezeReverbBuffer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("freezereverbbuffer"), Some("GranularFreezeReverbBuffer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbassexciter"), Some("SubharmonicBassExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmexciter"), Some("SubharmonicBassExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonantdiodeladderfilter"), Some("ResonantDiodeLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonantdiodeladder"), Some("ResonantDiodeLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyphonicvoiceunisondetuner"), Some("PolyphonicVoiceUnisonDetuner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyvoiceunisondetuner"), Some("PolyphonicVoiceUnisonDetuner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralcontrastcompressor"), Some("SpectralContrastCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("contrastcompressor"), Some("SpectralContrastCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcellobodyresonator"), Some("AcousticCelloBodyResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cellobodyresonator"), Some("AcousticCelloBodyResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmidsideequalizer"), Some("MultibandMidSideEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmidsideeq"), Some("MultibandMidSideEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogbucketbrigadeflanger"), Some("AnalogBucketBrigadeFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdanalogflanger"), Some("AnalogBucketBrigadeFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringtruepeakditherlimiter"), Some("MasteringTruePeakDitherLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakditherlimiter"), Some("MasteringTruePeakDitherLimiter"));
}

#[test]
fn test_tier143_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1613, "Inventory must contain at least 1613 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1613, "Registry list_all must be >= 1613, found {}", registry.list_all().len());

    let tier143_nodes = [
        ("WaveletPacketResynthesisEngine", DspNodeCategory::SpectralResynthesis),
        ("SpectralFreezeGranularSmear", DspNodeCategory::SpectralResynthesis),
        ("MultibandUpwardDownwardCompressor", DspNodeCategory::DynamicsMaster),
        ("MasterTapeSaturatorMaximizer", DspNodeCategory::DynamicsMaster),
        ("StereoDynamicDeClipper", DspNodeCategory::DynamicsMaster),
        ("BiquadFormantMorphFilter", DspNodeCategory::FilterEq),
        ("LinearPhaseTiltEqualizer", DspNodeCategory::FilterEq),
        ("ResonantMoogLadderDualCutoff", DspNodeCategory::FilterEq),
        ("BinauralDistanceRoomRenderer", DspNodeCategory::SpatialSurround),
        ("DolbyAtmos714BedEncoder", DspNodeCategory::SpatialSurround),
        ("ThroughZeroAnalogTapeChorus", DspNodeCategory::Modulation),
        ("PolyphonicMicrotonalModulator", DspNodeCategory::Modulation),
        ("RingModulatorFrequencyShifter", DspNodeCategory::Modulation),
        ("AcousticMarimbaRosewoodModel", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticHarpSoundboardModel", DspNodeCategory::AcousticPhysicalModel),
        ("BrassLipReedInstrumentModel", DspNodeCategory::AcousticPhysicalModel),
        ("GermaniumDiodeFuzzOverdrive", DspNodeCategory::DistortionSaturation),
        ("VacuumTubePentodeOverdrive", DspNodeCategory::DistortionSaturation),
        ("MultiTapModulatedDiffuser", DspNodeCategory::TimeSpace),
        ("BucketBrigadeAnalogDelayUnit", DspNodeCategory::TimeSpace),
        ("StereoPhaseCorrelationAnalyzer", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier143_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("waveletpacketresynthesisengine"), Some("WaveletPacketResynthesisEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveletpacketresynthesis"), Some("WaveletPacketResynthesisEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralfreezegranularsmear"), Some("SpectralFreezeGranularSmear"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralfreezesmear"), Some("SpectralFreezeGranularSmear"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandupwarddownwardcompressor"), Some("MultibandUpwardDownwardCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandottcompressor"), Some("MultibandUpwardDownwardCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mastertapesaturatormaximizer"), Some("MasterTapeSaturatorMaximizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mastertapesaturator"), Some("MasterTapeSaturatorMaximizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereodynamicdeclippers"), Some("StereoDynamicDeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereodynamicdeclipper"), Some("StereoDynamicDeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("biquadformantmorphfilter"), Some("BiquadFormantMorphFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantmorphfilter"), Some("BiquadFormantMorphFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphasetiltequalizer"), Some("LinearPhaseTiltEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphasetilteq"), Some("LinearPhaseTiltEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonantmoogladderdualcutoff"), Some("ResonantMoogLadderDualCutoff"));
    assert_eq!(DspNodeRegistry::normalize_type_name("moogladderdualcutoff"), Some("ResonantMoogLadderDualCutoff"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauraldistanceroomrenderer"), Some("BinauralDistanceRoomRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauraldistancerenderer"), Some("BinauralDistanceRoomRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dolbyatmos714bedencoder"), Some("DolbyAtmos714BedEncoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("atmos714bedencoder"), Some("DolbyAtmos714BedEncoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("throughzeroanalogtapechorus"), Some("ThroughZeroAnalogTapeChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("throughzerotapechorus"), Some("ThroughZeroAnalogTapeChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyphonicmicrotonalmodulator"), Some("PolyphonicMicrotonalModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalmodulator"), Some("PolyphonicMicrotonalModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ringmodulatorfrequencyshifter"), Some("RingModulatorFrequencyShifter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ringmodfrequencyshifter"), Some("RingModulatorFrequencyShifter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticmarimbarosewoodmodel"), Some("AcousticMarimbaRosewoodModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("marimbarosewoodmodel"), Some("AcousticMarimbaRosewoodModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticharpsoundboardmodel"), Some("AcousticHarpSoundboardModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harpsoundboardmodel"), Some("AcousticHarpSoundboardModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brasslipreedinstrumentmodel"), Some("BrassLipReedInstrumentModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brasslipreedmodel"), Some("BrassLipReedInstrumentModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumdiodefuzzoverdrive"), Some("GermaniumDiodeFuzzOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumfuzzoverdrive"), Some("GermaniumDiodeFuzzOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vacuumtubepentodeoverdrive"), Some("VacuumTubePentodeOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pentodeoverdrivedsp"), Some("VacuumTubePentodeOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitapmodulateddiffuser"), Some("MultiTapModulatedDiffuser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitapdiffuser"), Some("MultiTapModulatedDiffuser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bucketbrigadeanalogdelayunit"), Some("BucketBrigadeAnalogDelayUnit"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdanalogdelayunit"), Some("BucketBrigadeAnalogDelayUnit"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereophasecorrelationanalyzer"), Some("StereoPhaseCorrelationAnalyzer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasecorrelationanalyzer"), Some("StereoPhaseCorrelationAnalyzer"));
}

#[test]
fn test_tier144_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1634, "Inventory must contain at least 1634 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1634, "Registry list_all must be >= 1634, found {}", registry.list_all().len());

    let tier144_nodes = [
        ("NavierStokesFluidSolver", DspNodeCategory::AcousticPhysicalModel),
        ("QuantumMolecularBondResonator", DspNodeCategory::SpectralResynthesis),
        ("PlasmaArcSonificationSynthesizer", DspNodeCategory::Oscillator),
        ("AtmosphericDensityAbsorption", DspNodeCategory::SpatialSurround),
        ("QuantumDotPhotonTransducer", DspNodeCategory::DistortionSaturation),
        ("AcousticLevitationStandingWaveTrap", DspNodeCategory::Modulation),
        ("SupercriticalFluidPhaseNoise", DspNodeCategory::Oscillator),
        ("MagnetohydrodynamicPlasmaModulator", DspNodeCategory::Modulation),
        ("SonoluminescenceCavitationSonifier", DspNodeCategory::DynamicsMaster),
        ("MetamaterialNegativeIndexFilter", DspNodeCategory::FilterEq),
        ("InterstellarMediumShockwaveReverb", DspNodeCategory::TimeSpace),
        ("GravitationalWaveBinaryChirp", DspNodeCategory::Oscillator),
        ("CasimirQuantumVacuumNoise", DspNodeCategory::Oscillator),
        ("AcousticCloakingTransformationPanner", DspNodeCategory::SpatialSurround),
        ("ThermonuclearFusionResonator", DspNodeCategory::Oscillator),
        ("AiDemucsStemSeparationEngine", DspNodeCategory::Utility),
        ("AiAutonomousMasteringChain", DspNodeCategory::DynamicsMaster),
        ("VocalPitchFormantRetuner", DspNodeCategory::SpectralResynthesis),
        ("NeuralAudioRestorationRepair", DspNodeCategory::Utility),
        ("AutomatedDynamicPsychoacousticEq", DspNodeCategory::FilterEq),
        ("NeuralRoomImpulseMatcher", DspNodeCategory::TimeSpace),
    ];

    for (name, expected_cat) in tier144_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("navierstokesfluidsolver"), Some("NavierStokesFluidSolver"));
    assert_eq!(DspNodeRegistry::normalize_type_name("navierstokesfluid"), Some("NavierStokesFluidSolver"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantummolecularbondresonator"), Some("QuantumMolecularBondResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantummolecularbond"), Some("QuantumMolecularBondResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("plasmaarcsonificationsynthesizer"), Some("PlasmaArcSonificationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("plasmaarcsynth"), Some("PlasmaArcSonificationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("atmosphericdensityabsorption"), Some("AtmosphericDensityAbsorption"));
    assert_eq!(DspNodeRegistry::normalize_type_name("atmosphericabsorption"), Some("AtmosphericDensityAbsorption"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumdotphotontransducer"), Some("QuantumDotPhotonTransducer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumdottransducer"), Some("QuantumDotPhotonTransducer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticlevitationstandingwavetrap"), Some("AcousticLevitationStandingWaveTrap"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticstandingwavetrap"), Some("AcousticLevitationStandingWaveTrap"));
    assert_eq!(DspNodeRegistry::normalize_type_name("supercriticalfluidphasenoise"), Some("SupercriticalFluidPhaseNoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("supercriticalfluidnoise"), Some("SupercriticalFluidPhaseNoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("magnetohydrodynamicplasmamodulator"), Some("MagnetohydrodynamicPlasmaModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mhdplasmamodulator"), Some("MagnetohydrodynamicPlasmaModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sonoluminescencecavitationsonifier"), Some("SonoluminescenceCavitationSonifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sonoluminescencesonifier"), Some("SonoluminescenceCavitationSonifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("metamaterialnegativeindexfilter"), Some("MetamaterialNegativeIndexFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("metamaterialfilter"), Some("MetamaterialNegativeIndexFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("interstellarmediumshockwavereverb"), Some("InterstellarMediumShockwaveReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("interstellarshockreverb"), Some("InterstellarMediumShockwaveReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gravitationalwavebinarychirp"), Some("GravitationalWaveBinaryChirp"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gravitationalwavechirp"), Some("GravitationalWaveBinaryChirp"));
    assert_eq!(DspNodeRegistry::normalize_type_name("casimirquantumvacuumnoise"), Some("CasimirQuantumVacuumNoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("casimirvacuumnoise"), Some("CasimirQuantumVacuumNoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcloakingtransformationpanner"), Some("AcousticCloakingTransformationPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcloakingpanner"), Some("AcousticCloakingTransformationPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("thermonuclearfusionresonator"), Some("ThermonuclearFusionResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("thermonuclearsynth"), Some("ThermonuclearFusionResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aidemucsstemseparationengine"), Some("AiDemucsStemSeparationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("demucsseparator"), Some("AiDemucsStemSeparationEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aiautonomousmasteringchain"), Some("AiAutonomousMasteringChain"));
    assert_eq!(DspNodeRegistry::normalize_type_name("autonomousmasteringengine"), Some("AiAutonomousMasteringChain"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocalpitchformantretuner"), Some("VocalPitchFormantRetuner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocalpitchcorrector"), Some("VocalPitchFormantRetuner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralaudiorestorationrepair"), Some("NeuralAudioRestorationRepair"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralaudiorepair"), Some("NeuralAudioRestorationRepair"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automateddynamicpsychoacousticeq"), Some("AutomatedDynamicPsychoacousticEq"));
    assert_eq!(DspNodeRegistry::normalize_type_name("automateddynamiceq"), Some("AutomatedDynamicPsychoacousticEq"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralroomimpulsematcher"), Some("NeuralRoomImpulseMatcher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraliracousticmatcher"), Some("NeuralRoomImpulseMatcher"));
}

#[test]
fn test_tier145_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1655, "Inventory must contain at least 1655 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1655, "Registry list_all must be >= 1655, found {}", registry.list_all().len());

    let tier145_nodes = [
        ("QuantumAnnealingOptimizerSynth", DspNodeCategory::Oscillator),
        ("SupersonicVortexWhistleModel", DspNodeCategory::AcousticPhysicalModel),
        ("DynamicLobeBeamformerSpatializer", DspNodeCategory::SpatialSurround),
        ("SpectralEntropyTransientGate", DspNodeCategory::DynamicsMaster),
        ("OptoelectronicVactrolOptocoupler", DspNodeCategory::DynamicsMaster),
        ("NonlinearMagneticCoreInductor", DspNodeCategory::DistortionSaturation),
        ("RiemannSurfaceComplexOscillator", DspNodeCategory::Oscillator),
        ("BicubicInterpolatedWavetableMorph", DspNodeCategory::Oscillator),
        ("ChaoticChuaCircuitOscillator", DspNodeCategory::Oscillator),
        ("MicrotonalPolychromaticMatrix", DspNodeCategory::Modulation),
        ("AcousticWindInstrumentBoreProfile", DspNodeCategory::AcousticPhysicalModel),
        ("PhasedArrayUltrasoundTransducer", DspNodeCategory::SpatialSurround),
        ("MultibandSpectralContrastExpander", DspNodeCategory::SpectralResynthesis),
        ("SubharmonicResonanceTransformer", DspNodeCategory::DistortionSaturation),
        ("DiffusiveSchroederAllpassDiffuser", DspNodeCategory::TimeSpace),
        ("NeuralTimbreTransferStyleEngine", DspNodeCategory::NeuralAi),
        ("AsymmetricClippingDiodeLadder", DspNodeCategory::FilterEq),
        ("AeroacousticVocalChordGlottis", DspNodeCategory::AcousticPhysicalModel),
        ("BinauralNearFieldCrosstalkCanceller", DspNodeCategory::SpatialSurround),
        ("EbuLoudnessAutoTargetLeveler", DspNodeCategory::DynamicsMaster),
        ("StemPhaseAlignmentOptimizer", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier145_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumannealingoptimizersynth"), Some("QuantumAnnealingOptimizerSynth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumannealingsynth"), Some("QuantumAnnealingOptimizerSynth"));
    assert_eq!(DspNodeRegistry::normalize_type_name("supersonicvortexwhistlemodel"), Some("SupersonicVortexWhistleModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vortexwhistlemodel"), Some("SupersonicVortexWhistleModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamiclobebeamformerspatializer"), Some("DynamicLobeBeamformerSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("beamformerspatializer"), Some("DynamicLobeBeamformerSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralentropytransientgate"), Some("SpectralEntropyTransientGate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("entropytransientgate"), Some("SpectralEntropyTransientGate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("optoelectronicvactroloptocoupler"), Some("OptoelectronicVactrolOptocoupler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vactroloptocoupler"), Some("OptoelectronicVactrolOptocoupler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearmagneticcoreinductor"), Some("NonlinearMagneticCoreInductor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("magneticcoreinductor"), Some("NonlinearMagneticCoreInductor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("riemannsurfacecomplexoscillator"), Some("RiemannSurfaceComplexOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("riemannsurfaceoscillator"), Some("RiemannSurfaceComplexOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bicubicinterpolatedwavetablemorph"), Some("BicubicInterpolatedWavetableMorph"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bicubicwavetablemorph"), Some("BicubicInterpolatedWavetableMorph"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticchuacircuitoscillator"), Some("ChaoticChuaCircuitOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chuacircuitoscillator"), Some("ChaoticChuaCircuitOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalpolychromaticmatrix"), Some("MicrotonalPolychromaticMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polychromaticmatrix"), Some("MicrotonalPolychromaticMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticwindinstrumentboreprofile"), Some("AcousticWindInstrumentBoreProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("windinstrumentboreprofile"), Some("AcousticWindInstrumentBoreProfile"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasedarrayultrasoundtransducer"), Some("PhasedArrayUltrasoundTransducer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ultrasoundtransducer"), Some("PhasedArrayUltrasoundTransducer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandspectralcontrastexpander"), Some("MultibandSpectralContrastExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralcontrastexpander"), Some("MultibandSpectralContrastExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicresonancetransformer"), Some("SubharmonicResonanceTransformer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonictransformer"), Some("SubharmonicResonanceTransformer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusiveschroederallpassdiffuser"), Some("DiffusiveSchroederAllpassDiffuser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("schroederallpassdiffuser"), Some("DiffusiveSchroederAllpassDiffuser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltimbretransferstyleengine"), Some("NeuralTimbreTransferStyleEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltimbretransfer"), Some("NeuralTimbreTransferStyleEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asymmetricclippingdiodeladder"), Some("AsymmetricClippingDiodeLadder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asymmetricclippingdiode"), Some("AsymmetricClippingDiodeLadder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroacousticvocalchordglottis"), Some("AeroacousticVocalChordGlottis"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocalchordglottis"), Some("AeroacousticVocalChordGlottis"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralnearfieldcrosstalkcanceller"), Some("BinauralNearFieldCrosstalkCanceller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crosstalkcanceller"), Some("BinauralNearFieldCrosstalkCanceller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ebuloudnessautotargetleveler"), Some("EbuLoudnessAutoTargetLeveler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ebuloudnessleveler"), Some("EbuLoudnessAutoTargetLeveler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stemphasealignmentoptimizer"), Some("StemPhaseAlignmentOptimizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasealignmentoptimizer"), Some("StemPhaseAlignmentOptimizer"));
}

#[test]
fn test_tier146_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1676, "Inventory must contain at least 1676 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1676, "Registry list_all must be >= 1676, found {}", registry.list_all().len());

    let tier146_nodes = [
        ("StochasticResonanceEnhancer", DspNodeCategory::DistortionSaturation),
        ("WaveletTransientSplayDeconstruct", DspNodeCategory::SpectralResynthesis),
        ("CavityResonanceFormantFilter", DspNodeCategory::FilterEq),
        ("GranularCloudTextureMorpher", DspNodeCategory::SamplerSlicer),
        ("TapeHissModulatedReverb", DspNodeCategory::TimeSpace),
        ("DynamicSpectralDeEsser", DspNodeCategory::DynamicsMaster),
        ("HalvorsenChaoticAttractorLfo", DspNodeCategory::Modulation),
        ("SphericalHarmonicSoundfieldRotator", DspNodeCategory::SpatialSurround),
        ("VacuumTubeTriodeClassAB", DspNodeCategory::DistortionSaturation),
        ("AdaptiveJustIntonationMapper", DspNodeCategory::Modulation),
        ("NonlinearSpringReverbTank", DspNodeCategory::TimeSpace),
        ("ResonantPhasedCombArray", DspNodeCategory::FilterEq),
        ("NeuralSpeechProsodyCloner", DspNodeCategory::NeuralAi),
        ("SubwooferHarmonicSynthesizer", DspNodeCategory::Oscillator),
        ("FrequencyShifterRingModulator", DspNodeCategory::Modulation),
        ("OpticalTremoloPanner", DspNodeCategory::Modulation),
        ("ParametricAirDampingAtmosphere", DspNodeCategory::TimeSpace),
        ("BilateralSpectralDenoise", DspNodeCategory::SpectralResynthesis),
        ("AnalogMatrixTapeDelay", DspNodeCategory::TimeSpace),
        ("VocalChantChoirSynthesizer", DspNodeCategory::CompositeSynth),
        ("MultitrackPhaseAlignMatrix", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier146_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticresonanceenhancer"), Some("StochasticResonanceEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticresonance"), Some("StochasticResonanceEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavelettransientsplaydeconstruct"), Some("WaveletTransientSplayDeconstruct"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavelettransientsplay"), Some("WaveletTransientSplayDeconstruct"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cavityresonanceformantfilter"), Some("CavityResonanceFormantFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cavityformantfilter"), Some("CavityResonanceFormantFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularcloudtexturemorpher"), Some("GranularCloudTextureMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularcloudmorpher"), Some("GranularCloudTextureMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapehissmodulatedreverb"), Some("TapeHissModulatedReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapehissreverb"), Some("TapeHissModulatedReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicspectraldeesser"), Some("DynamicSpectralDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectraldeesser"), Some("DynamicSpectralDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("halvorsenchaoticattractorlfo"), Some("HalvorsenChaoticAttractorLfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("halvorsenchaoslfo"), Some("HalvorsenChaoticAttractorLfo"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalharmonicsoundfieldrotator"), Some("SphericalHarmonicSoundfieldRotator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("soundfieldrotator"), Some("SphericalHarmonicSoundfieldRotator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vacuumtubetriodeclassab"), Some("VacuumTubeTriodeClassAB"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodeclassab"), Some("VacuumTubeTriodeClassAB"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivejustintonationmapper"), Some("AdaptiveJustIntonationMapper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("justintonationmapper"), Some("AdaptiveJustIntonationMapper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearspringreverbtank"), Some("NonlinearSpringReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearspringtank"), Some("NonlinearSpringReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonantphasedcombarray"), Some("ResonantPhasedCombArray"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasedcombarray"), Some("ResonantPhasedCombArray"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralspeechprosodycloner"), Some("NeuralSpeechProsodyCloner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("speechprosodycloner"), Some("NeuralSpeechProsodyCloner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subwooferharmonicsynthesizer"), Some("SubwooferHarmonicSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicsynth"), Some("SubwooferHarmonicSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("frequencyshifterringmodulator"), Some("FrequencyShifterRingModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("freqshifterringmod"), Some("FrequencyShifterRingModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticaltremolopanner"), Some("OpticalTremoloPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalautopanner"), Some("OpticalTremoloPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parametricairdampingatmosphere"), Some("ParametricAirDampingAtmosphere"));
    assert_eq!(DspNodeRegistry::normalize_type_name("airdampingatmosphere"), Some("ParametricAirDampingAtmosphere"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bilateralspectraldenoise"), Some("BilateralSpectralDenoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectraldenoise"), Some("BilateralSpectralDenoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogmatrixtapedelay"), Some("AnalogMatrixTapeDelay"));
    assert_eq!(DspNodeRegistry::normalize_type_name("matrixtapedelay"), Some("AnalogMatrixTapeDelay"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocalchantchoirsynthesizer"), Some("VocalChantChoirSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chantchoirsynth"), Some("VocalChantChoirSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitrackphasealignmatrix"), Some("MultitrackPhaseAlignMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasealignmatrix"), Some("MultitrackPhaseAlignMatrix"));
}

#[test]
fn test_tier147_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1697, "Inventory must contain at least 1697 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1697, "Registry list_all must be >= 1697, found {}", registry.list_all().len());

    let tier147_nodes = [
        ("NonlinearWaveguideFeedbackComb", DspNodeCategory::FilterEq),
        ("HilbertTransformStereoPanner", DspNodeCategory::SpatialSurround),
        ("MultiResonanceFormantBank", DspNodeCategory::FilterEq),
        ("PolynomialHarmonicExciter", DspNodeCategory::DistortionSaturation),
        ("GranularFreezeBufferDrone", DspNodeCategory::SamplerSlicer),
        ("DynamicSpectralTransientSplitter", DspNodeCategory::SpectralResynthesis),
        ("BinauralPinnaReflectionEngine", DspNodeCategory::SpatialSurround),
        ("AnalogTapeHeadAzimuthAligner", DspNodeCategory::Utility),
        ("AdaptiveNoiseGateTransientExpander", DspNodeCategory::DynamicsMaster),
        ("ChuaChaoticModulationOscillator", DspNodeCategory::Modulation),
        ("InfrasonicSubBassExciter", DspNodeCategory::Oscillator),
        ("OpticalRotarySpeakerHorn", DspNodeCategory::Modulation),
        ("DiffusivePlateReverbTank", DspNodeCategory::TimeSpace),
        ("PsychoacousticStereoImager", DspNodeCategory::SpatialSurround),
        ("HarmonicResonanceVocoder", DspNodeCategory::CompositeSynth),
        ("GermaniumFuzzTransistorMatrix", DspNodeCategory::DistortionSaturation),
        ("AsynchronousMultiTapDelayArray", DspNodeCategory::TimeSpace),
        ("NeuralStemInterferenceRemover", DspNodeCategory::NeuralAi),
        ("AeroacousticReedJetExciter", DspNodeCategory::AcousticPhysicalModel),
        ("MasteringTruePeakLimiterOversampled", DspNodeCategory::DynamicsMaster),
        ("MicrotonalScaleHarmonizer", DspNodeCategory::Modulation),
    ];

    for (name, expected_cat) in tier147_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearwaveguidefeedbackcomb"), Some("NonlinearWaveguideFeedbackComb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveguidefeedbackcomb"), Some("NonlinearWaveguideFeedbackComb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hilberttransformstereopanner"), Some("HilbertTransformStereoPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hilbertspatialpanner"), Some("HilbertTransformStereoPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multiresonanceformantbank"), Some("MultiResonanceFormantBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantfilterbank"), Some("MultiResonanceFormantBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polynomialharmonicexciter"), Some("PolynomialHarmonicExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polynomialexciter"), Some("PolynomialHarmonicExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularfreezebufferdrone"), Some("GranularFreezeBufferDrone"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularfreezedrone"), Some("GranularFreezeBufferDrone"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicspectraltransientsplitter"), Some("DynamicSpectralTransientSplitter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicspectraltransientsplit"), Some("DynamicSpectralTransientSplitter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralpinnareflectionengine"), Some("BinauralPinnaReflectionEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pinnareflectionengine"), Some("BinauralPinnaReflectionEngine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtapeheadazimuthaligner"), Some("AnalogTapeHeadAzimuthAligner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeheadazimuthaligner"), Some("AnalogTapeHeadAzimuthAligner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivenoisegatetransientexpander"), Some("AdaptiveNoiseGateTransientExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientdownwardexpander"), Some("AdaptiveNoiseGateTransientExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chuachaoticmodulationoscillator"), Some("ChuaChaoticModulationOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chuamodulationoscillator"), Some("ChuaChaoticModulationOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("infrasonicsubbassexciter"), Some("InfrasonicSubBassExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("infrasonicsubexciter"), Some("InfrasonicSubBassExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalrotaryspeakerhorn"), Some("OpticalRotarySpeakerHorn"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotaryspeakerhorn"), Some("OpticalRotarySpeakerHorn"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusiveplatereverbtank"), Some("DiffusivePlateReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusivesteelplatetank"), Some("DiffusivePlateReverbTank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticstereoimager"), Some("PsychoacousticStereoImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticstereomapper"), Some("PsychoacousticStereoImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicresonancevocoder"), Some("HarmonicResonanceVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonancevocodermodeler"), Some("HarmonicResonanceVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumfuzztransistormatrix"), Some("GermaniumFuzzTransistorMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumfuzzmatrix"), Some("GermaniumFuzzTransistorMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asynchronousmultitapdelayarray"), Some("AsynchronousMultiTapDelayArray"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitapdelayarray"), Some("AsynchronousMultiTapDelayArray"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralsteminterferenceremover"), Some("NeuralStemInterferenceRemover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("steminterferenceremover"), Some("NeuralStemInterferenceRemover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroacousticreedjetexciter"), Some("AeroacousticReedJetExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("reedjetexciter"), Some("AeroacousticReedJetExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringtruepeaklimiteroversampled"), Some("MasteringTruePeakLimiterOversampled"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringtruepeaklimiter"), Some("MasteringTruePeakLimiterOversampled"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalscaleharmonizer"), Some("MicrotonalScaleHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("scaleharmonizer"), Some("MicrotonalScaleHarmonizer"));
}

#[test]
fn test_tier148_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1718, "Inventory must contain at least 1718 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1718, "Registry list_all must be >= 1718, found {}", registry.list_all().len());

    let tier148_nodes = [
        ("LorenzRosslerBifurcationAttractor", DspNodeCategory::Modulation),
        ("ConcertGrandHarpPhysicalModel", DspNodeCategory::AcousticPhysicalModel),
        ("MorphologicalSpectrogramFilter", DspNodeCategory::SpectralResynthesis),
        ("SplitBandSaturatorDynamics", DspNodeCategory::DistortionSaturation),
        ("HigherOrderAmbisonicMicrophoneEncoder", DspNodeCategory::SpatialSurround),
        ("NonlinearSpeakerCabinetEmulator", DspNodeCategory::AcousticPhysicalModel),
        ("PsychoacousticFundamentalSynthesizer", DspNodeCategory::Oscillator),
        ("VariableCapstanMagneticFlanger", DspNodeCategory::TimeSpace),
        ("ZeroPhaseLinearMasteringEqualizer", DspNodeCategory::FilterEq),
        ("LookaheadDynamicDeEsserSibilance", DspNodeCategory::DynamicsMaster),
        ("GermaniumSiliconDiodeClipper", DspNodeCategory::DistortionSaturation),
        ("PhaseAlignedGranularTimeStretcher", DspNodeCategory::SamplerSlicer),
        ("NeuralHarmonicMainsHumRemover", DspNodeCategory::NeuralAi),
        ("ArticulatoryVocalTractSynthesizer", DspNodeCategory::CompositeSynth),
        ("BucketBrigadeAnalogDelayChorus", DspNodeCategory::TimeSpace),
        ("DualRotorAcousticLeslieCabinet", DspNodeCategory::Modulation),
        ("EllipticalStereoPhaseCorrelator", DspNodeCategory::SpatialSurround),
        ("AcousticSpruceViolinBodyResonator", DspNodeCategory::AcousticPhysicalModel),
        ("MultibandDownwardSpillExpander", DspNodeCategory::DynamicsMaster),
        ("VolumetricRaytracedRoomAcoustics", DspNodeCategory::SpatialSurround),
        ("GaussianStochasticGrainSynthesizer", DspNodeCategory::Oscillator),
    ];

    for (name, expected_cat) in tier148_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("lorenzrosslerbifurcationattractor"), Some("LorenzRosslerBifurcationAttractor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bifurcationattractor"), Some("LorenzRosslerBifurcationAttractor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertgrandharpphysicalmodel"), Some("ConcertGrandHarpPhysicalModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grandharpphysicalmodel"), Some("ConcertGrandHarpPhysicalModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphologicalspectrogramfilter"), Some("MorphologicalSpectrogramFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogrammorphology"), Some("MorphologicalSpectrogramFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("splitbandsaturatordynamics"), Some("SplitBandSaturatorDynamics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("splitbandsaturator"), Some("SplitBandSaturatorDynamics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("higherorderambisonicmicrophoneencoder"), Some("HigherOrderAmbisonicMicrophoneEncoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicmicrophoneencoder"), Some("HigherOrderAmbisonicMicrophoneEncoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearspeakercabinetemulator"), Some("NonlinearSpeakerCabinetEmulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("speakercabinetemulator"), Some("NonlinearSpeakerCabinetEmulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticfundamentalsynthesizer"), Some("PsychoacousticFundamentalSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fundamentalsynthesizer"), Some("PsychoacousticFundamentalSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("variablecapstanmagneticflanger"), Some("VariableCapstanMagneticFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("capstantapeflanger"), Some("VariableCapstanMagneticFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerophaselinearmasteringequalizer"), Some("ZeroPhaseLinearMasteringEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerophasemasteringeq"), Some("ZeroPhaseLinearMasteringEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lookaheaddynamicdeessersibilance"), Some("LookaheadDynamicDeEsserSibilance"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lookaheaddeesser"), Some("LookaheadDynamicDeEsserSibilance"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumsilicondiodeclipper"), Some("GermaniumSiliconDiodeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diodeclipper"), Some("GermaniumSiliconDiodeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasealignedgranulartimestretcher"), Some("PhaseAlignedGranularTimeStretcher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granulartimestretcher"), Some("PhaseAlignedGranularTimeStretcher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralharmonicmainshumremover"), Some("NeuralHarmonicMainsHumRemover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mainshumremover"), Some("NeuralHarmonicMainsHumRemover"));
    assert_eq!(DspNodeRegistry::normalize_type_name("articulatoryvocaltractsynthesizer"), Some("ArticulatoryVocalTractSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocaltractsynthesizer"), Some("ArticulatoryVocalTractSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bucketbrigadeanalogdelaychorus"), Some("BucketBrigadeAnalogDelayChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbddelaychorus"), Some("BucketBrigadeAnalogDelayChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualrotoracousticlesliecabinet"), Some("DualRotorAcousticLeslieCabinet"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lesliecabinet"), Some("DualRotorAcousticLeslieCabinet"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ellipticalstereophasecorrelator"), Some("EllipticalStereoPhaseCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereophasecorrelator"), Some("EllipticalStereoPhaseCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticspruceviolinbodyresonator"), Some("AcousticSpruceViolinBodyResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("violinbodyresonator"), Some("AcousticSpruceViolinBodyResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibanddownwardspillexpander"), Some("MultibandDownwardSpillExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("downwardspillexpander"), Some("MultibandDownwardSpillExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("volumetricraytracedroomacoustics"), Some("VolumetricRaytracedRoomAcoustics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("raytracedroomacoustics"), Some("VolumetricRaytracedRoomAcoustics"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gaussianstochasticgrainsynthesizer"), Some("GaussianStochasticGrainSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticgrainsynthesizer"), Some("GaussianStochasticGrainSynthesizer"));
}

#[test]
fn test_tier149_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1739, "Inventory must contain at least 1739 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1739, "Registry list_all must be >= 1739, found {}", registry.list_all().len());

    let tier149_nodes = [
        ("HyperdimensionalQuaternionOscillator", DspNodeCategory::Oscillator),
        ("ResonantHelmholtzVocalTractCavity", DspNodeCategory::AcousticPhysicalModel),
        ("AdaptiveSpectralSubtractDeNoiser", DspNodeCategory::SpectralResynthesis),
        ("NonlinearTapeSaturationHead", DspNodeCategory::DistortionSaturation),
        ("AmbisonicDistanceCuesBinauralizer", DspNodeCategory::SpatialSurround),
        ("MorphingStateVariableLadderFilter", DspNodeCategory::FilterEq),
        ("DynamicSpectralCompressorDeEsser", DspNodeCategory::DynamicsMaster),
        ("GermaniumOctaveFuzzDoubler", DspNodeCategory::DistortionSaturation),
        ("PhaseAlignedStereoGranularDelay", DspNodeCategory::SamplerSlicer),
        ("NeuralHumAndHarmonicsDeclicker", DspNodeCategory::NeuralAi),
        ("AcousticSpruceClassicalGuitarBody", DspNodeCategory::AcousticPhysicalModel),
        ("RotaryDualHornDopplerModulator", DspNodeCategory::Modulation),
        ("MultibandTruePeakMasterLimiter", DspNodeCategory::DynamicsMaster),
        ("SubharmonicBassReinforcementExciter", DspNodeCategory::Oscillator),
        ("PolyphonicPulsarCloudSynthesizer", DspNodeCategory::Oscillator),
        ("DynamicSpectralTransientExciter", DspNodeCategory::SpectralResynthesis),
        ("AsymmetricTriodeVacuumTubePreamp", DspNodeCategory::DistortionSaturation),
        ("MultichannelSpeakerCalibrationMatrix", DspNodeCategory::Utility),
        ("BinauralConchaReflectionSpatializer", DspNodeCategory::SpatialSurround),
        ("AnalogBucketBrigadeChorusVibrato", DspNodeCategory::TimeSpace),
        ("StochasticChaoticJitterSynthesizer", DspNodeCategory::Oscillator),
    ];

    for (name, expected_cat) in tier149_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperdimensionalquaternionoscillator"), Some("HyperdimensionalQuaternionOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quaternionoscillator"), Some("HyperdimensionalQuaternionOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonanthelmholtzvocaltractcavity"), Some("ResonantHelmholtzVocalTractCavity"));
    assert_eq!(DspNodeRegistry::normalize_type_name("helmholtzvocaltract"), Some("ResonantHelmholtzVocalTractCavity"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivespectralsubtractdenoiser"), Some("AdaptiveSpectralSubtractDeNoiser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralsubtractdenoiser"), Some("AdaptiveSpectralSubtractDeNoiser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlineartapesaturationhead"), Some("NonlinearTapeSaturationHead"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapesaturationhead"), Some("NonlinearTapeSaturationHead"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicdistancecuesbinauralizer"), Some("AmbisonicDistanceCuesBinauralizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicbinauralizer"), Some("AmbisonicDistanceCuesBinauralizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingstatevariableladderfilter"), Some("MorphingStateVariableLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingladderfilter"), Some("MorphingStateVariableLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicspectralcompressordeesser"), Some("DynamicSpectralCompressorDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralcompressordeesser"), Some("DynamicSpectralCompressorDeEsser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumoctavefuzzdoubler"), Some("GermaniumOctaveFuzzDoubler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumoctavefuzz"), Some("GermaniumOctaveFuzzDoubler"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasealignedstereogranulardelay"), Some("PhaseAlignedStereoGranularDelay"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereogranulardelay"), Some("PhaseAlignedStereoGranularDelay"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralhumandharmonicsdeclicker"), Some("NeuralHumAndHarmonicsDeclicker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralhumdeclicker"), Some("NeuralHumAndHarmonicsDeclicker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticspruceclassicalguitarbody"), Some("AcousticSpruceClassicalGuitarBody"));
    assert_eq!(DspNodeRegistry::normalize_type_name("classicalguitarbody"), Some("AcousticSpruceClassicalGuitarBody"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotarydualhorndopplermodulator"), Some("RotaryDualHornDopplerModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dopplerrotaryspeaker"), Some("RotaryDualHornDopplerModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandtruepeakmasterlimiter"), Some("MultibandTruePeakMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakmasterlimiter"), Some("MultibandTruePeakMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbassreinforcementexciter"), Some("SubharmonicBassReinforcementExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bassreinforcementexciter"), Some("SubharmonicBassReinforcementExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("polyphonicpulsarcloudsynthesizer"), Some("PolyphonicPulsarCloudSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pulsarcloudsynthesizer"), Some("PolyphonicPulsarCloudSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicspectraltransientexciter"), Some("DynamicSpectralTransientExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectraltransientexciter"), Some("DynamicSpectralTransientExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asymmetrictriodevacuumtubepreamp"), Some("AsymmetricTriodeVacuumTubePreamp"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodepreamp12ax7"), Some("AsymmetricTriodeVacuumTubePreamp"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelspeakercalibrationmatrix"), Some("MultichannelSpeakerCalibrationMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelcalibrationmatrix"), Some("MultichannelSpeakerCalibrationMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralconchareflectionspatializer"), Some("BinauralConchaReflectionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("conchaspatializer"), Some("BinauralConchaReflectionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogbucketbrigadechorusvibrato"), Some("AnalogBucketBrigadeChorusVibrato"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdchorusvibrato"), Some("AnalogBucketBrigadeChorusVibrato"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticchaoticjittersynthesizer"), Some("StochasticChaoticJitterSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticjittersynthesizer"), Some("StochasticChaoticJitterSynthesizer"));
}

#[test]
fn test_tier150_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1760, "Inventory must contain at least 1760 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1760, "Registry list_all must be >= 1760, found {}", registry.list_all().len());

    let tier150_nodes = [
        ("RelativisticDopplerLorentzTransform", DspNodeCategory::SpatialSurround),
        ("QuantumEntanglementPhaseVocoder", DspNodeCategory::SpectralResynthesis),
        ("CavitationBubbleCollapseExciter", DspNodeCategory::Oscillator),
        ("NonlinearOptocouplerOpticalCompressor", DspNodeCategory::DynamicsMaster),
        ("OrthogonalFrequencyWaveguideLattice", DspNodeCategory::TimeSpace),
        ("AcousticSnareDrumWireSnares", DspNodeCategory::AcousticPhysicalModel),
        ("AdaptiveKalmanPitchTrackingFilter", DspNodeCategory::Modulation),
        ("FerrofluidMagneticViscosityAbsorber", DspNodeCategory::FilterEq),
        ("TapeHeadAzimuthSkewCorrelator", DspNodeCategory::DistortionSaturation),
        ("MicrotonalHarmonicLatticeOscillator", DspNodeCategory::Oscillator),
        ("BiometricPupillometricEmotionMapper", DspNodeCategory::NeuralAi),
        ("ConcertHarpPedalMechanismMatrix", DspNodeCategory::AcousticPhysicalModel),
        ("MultichannelSpeakerPhaseAlignmentUtility", DspNodeCategory::Utility),
        ("PsychoacousticBarkScaleSubBassSynthesizer", DspNodeCategory::Oscillator),
        ("NonlinearDiodeLadderStereoFilter", DspNodeCategory::FilterEq),
        ("AsynchronousGranularCloudSaturator", DspNodeCategory::SamplerSlicer),
        ("RotarySpeakerAcousticDeflector", DspNodeCategory::Modulation),
        ("MultibandMidSideMasteringCompressor", DspNodeCategory::DynamicsMaster),
        ("BinauralPinnaDiffractionSpatializer", DspNodeCategory::SpatialSurround),
        ("DualTriodeTubeSaturationClipper", DspNodeCategory::DistortionSaturation),
        ("ChaoticStrangeAttractorSynthesizer", DspNodeCategory::Oscillator),
    ];

    for (name, expected_cat) in tier150_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("relativisticdopplerlorentztransform"), Some("RelativisticDopplerLorentzTransform"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lorentztransformspatializer"), Some("RelativisticDopplerLorentzTransform"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumentanglementphasevocoder"), Some("QuantumEntanglementPhaseVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("entangledphasevocoder"), Some("QuantumEntanglementPhaseVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cavitationbubblecollapseexciter"), Some("CavitationBubbleCollapseExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cavitationbubbleexciter"), Some("CavitationBubbleCollapseExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearoptocoupleropticalcompressor"), Some("NonlinearOptocouplerOpticalCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("optocouplercompressor"), Some("NonlinearOptocouplerOpticalCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("orthogonalfrequencywaveguidelattice"), Some("OrthogonalFrequencyWaveguideLattice"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveguidelatticereverb"), Some("OrthogonalFrequencyWaveguideLattice"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticsnaredrumwiresnares"), Some("AcousticSnareDrumWireSnares"));
    assert_eq!(DspNodeRegistry::normalize_type_name("snarewiresnares"), Some("AcousticSnareDrumWireSnares"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivekalmanpitchtrackingfilter"), Some("AdaptiveKalmanPitchTrackingFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("kalmanpitchtracker"), Some("AdaptiveKalmanPitchTrackingFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ferrofluidmagneticviscosityabsorber"), Some("FerrofluidMagneticViscosityAbsorber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ferrofluidfilter"), Some("FerrofluidMagneticViscosityAbsorber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeheadazimuthskewcorrelator"), Some("TapeHeadAzimuthSkewCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeazimuthskew"), Some("TapeHeadAzimuthSkewCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("microtonalharmoniclatticeoscillator"), Some("MicrotonalHarmonicLatticeOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("justintonationlattice"), Some("MicrotonalHarmonicLatticeOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("biometricpupillometricemotionmapper"), Some("BiometricPupillometricEmotionMapper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("biometricemotionmapper"), Some("BiometricPupillometricEmotionMapper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertharppedalmechanismmatrix"), Some("ConcertHarpPedalMechanismMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harppedalmechanism"), Some("ConcertHarpPedalMechanismMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelspeakerphasealignmentutility"), Some("MultichannelSpeakerPhaseAlignmentUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("speakerphasealignmentutility"), Some("MultichannelSpeakerPhaseAlignmentUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticbarkscalesubbasssynthesizer"), Some("PsychoacousticBarkScaleSubBassSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("barkscalesubbass"), Some("PsychoacousticBarkScaleSubBassSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlineardiodeladderstereofilter"), Some("NonlinearDiodeLadderStereoFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("emsdiodeladderfilter"), Some("NonlinearDiodeLadderStereoFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asynchronousgranularcloudsaturator"), Some("AsynchronousGranularCloudSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularcloudsaturator"), Some("AsynchronousGranularCloudSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotaryspeakeracousticdeflector"), Some("RotarySpeakerAcousticDeflector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lesliedeflector"), Some("RotarySpeakerAcousticDeflector"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmidsidemasteringcompressor"), Some("MultibandMidSideMasteringCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidemasteringcompressor"), Some("MultibandMidSideMasteringCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralpinnadiffractionspatializer"), Some("BinauralPinnaDiffractionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pinnaspatializer"), Some("BinauralPinnaDiffractionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualtriodetubesaturationclipper"), Some("DualTriodeTubeSaturationClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualtriodeclipper"), Some("DualTriodeTubeSaturationClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticstrangeattractorsynthesizer"), Some("ChaoticStrangeAttractorSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("strangeattractorsynthesizer"), Some("ChaoticStrangeAttractorSynthesizer"));
}

#[test]
fn test_tier151_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1781, "Inventory must contain at least 1781 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1781, "Registry list_all must be >= 1781, found {}", registry.list_all().len());

    let tier151_nodes = [
        ("AeroacousticVortexSheddingOscillator", DspNodeCategory::AcousticPhysicalModel),
        ("QuantumAnnealingHarmonicOptimizer", DspNodeCategory::NeuralAi),
        ("AnalogTriodePentodePushPullOutput", DspNodeCategory::DistortionSaturation),
        ("BinauralNearFieldAcousticScattering", DspNodeCategory::SpatialSurround),
        ("DynamicSubharmonicResonanceExciter", DspNodeCategory::Oscillator),
        ("MultibandTransientKneeExpander", DspNodeCategory::DynamicsMaster),
        ("WaveguidePlateSpringDispersionReverb", DspNodeCategory::TimeSpace),
        ("AdaptiveFormantMorphingResonator", DspNodeCategory::FilterEq),
        ("GranularSpectralGlitchDeconstruct", DspNodeCategory::SamplerSlicer),
        ("HarmonicCombModulatedChorus", DspNodeCategory::Modulation),
        ("AcousticBassClarinetWoodwindModel", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralPerceptualMaskingDenoise", DspNodeCategory::NeuralAi),
        ("AnalogStereoTapeWowFlutterSimulator", DspNodeCategory::TimeSpace),
        ("HyperbolicWaveshaperHarmonicSaturator", DspNodeCategory::DistortionSaturation),
        ("AmbisonicBFormatDopplerSpatialPanner", DspNodeCategory::SpatialSurround),
        ("SubharmonicSubwooferCrossoverAlignment", DspNodeCategory::Utility),
        ("AsynchronousGranularWavetableOscillator", DspNodeCategory::Oscillator),
        ("ResonantCapacitorSwitchedLadderFilter", DspNodeCategory::FilterEq),
        ("DualStageOpticalVactrolLimiter", DspNodeCategory::DynamicsMaster),
        ("RotaryDualDrumDopplerSpeaker", DspNodeCategory::Modulation),
        ("StochasticNoiseDrivenResonanceExciter", DspNodeCategory::DistortionSaturation),
    ];

    for (name, expected_cat) in tier151_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroacousticvortexsheddingoscillator"), Some("AeroacousticVortexSheddingOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vortexsheddingoscillator"), Some("AeroacousticVortexSheddingOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumannealingharmonicoptimizer"), Some("QuantumAnnealingHarmonicOptimizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quantumannealingoptimizer"), Some("QuantumAnnealingHarmonicOptimizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtriodepentodepushpulloutput"), Some("AnalogTriodePentodePushPullOutput"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodepentodepushpull"), Some("AnalogTriodePentodePushPullOutput"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralnearfieldacousticscattering"), Some("BinauralNearFieldAcousticScattering"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nearfieldacousticscattering"), Some("BinauralNearFieldAcousticScattering"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicsubharmonicresonanceexciter"), Some("DynamicSubharmonicResonanceExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicresonanceexciter"), Some("DynamicSubharmonicResonanceExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandtransientkneeexpander"), Some("MultibandTransientKneeExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientkneeexpander"), Some("MultibandTransientKneeExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveguideplatespringdispersionreverb"), Some("WaveguidePlateSpringDispersionReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("platespringdispersionreverb"), Some("WaveguidePlateSpringDispersionReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptiveformantmorphingresonator"), Some("AdaptiveFormantMorphingResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantmorphingresonator"), Some("AdaptiveFormantMorphingResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularspectralglitchdeconstruct"), Some("GranularSpectralGlitchDeconstruct"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralglitchdeconstruct"), Some("GranularSpectralGlitchDeconstruct"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmoniccombmodulatedchorus"), Some("HarmonicCombModulatedChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("combmodulatedchorus"), Some("HarmonicCombModulatedChorus"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticbassclarinetwoodwindmodel"), Some("AcousticBassClarinetWoodwindModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bassclarinetwoodwindmodel"), Some("AcousticBassClarinetWoodwindModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralperceptualmaskingdenoise"), Some("NeuralPerceptualMaskingDenoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("perceptualmaskingdenoise"), Some("NeuralPerceptualMaskingDenoise"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogstereotapewowfluttersimulator"), Some("AnalogStereoTapeWowFlutterSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapewowfluttersimulator"), Some("AnalogStereoTapeWowFlutterSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolicwaveshaperharmonicsaturator"), Some("HyperbolicWaveshaperHarmonicSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolicwaveshapersaturator"), Some("HyperbolicWaveshaperHarmonicSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicbformatdopplerspatialpanner"), Some("AmbisonicBFormatDopplerSpatialPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicdopplerspatialpanner"), Some("AmbisonicBFormatDopplerSpatialPanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicsubwoofercrossoveralignment"), Some("SubharmonicSubwooferCrossoverAlignment"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subwoofercrossoveralignment"), Some("SubharmonicSubwooferCrossoverAlignment"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asynchronousgranularwavetableoscillator"), Some("AsynchronousGranularWavetableOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularwavetableoscillator"), Some("AsynchronousGranularWavetableOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("resonantcapacitorswitchedladderfilter"), Some("ResonantCapacitorSwitchedLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("capacitorswitchedladderfilter"), Some("ResonantCapacitorSwitchedLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualstageopticalvactrollimiter"), Some("DualStageOpticalVactrolLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalvactrollimiter"), Some("DualStageOpticalVactrolLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotarydualdrumdopplerspeaker"), Some("RotaryDualDrumDopplerSpeaker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualdrumdopplerspeaker"), Some("RotaryDualDrumDopplerSpeaker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticnoisedrivenresonanceexciter"), Some("StochasticNoiseDrivenResonanceExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("noisedrivenresonanceexciter"), Some("StochasticNoiseDrivenResonanceExciter"));
}

#[test]
fn test_tier152_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1802, "Inventory must contain at least 1802 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1802, "Registry list_all must be >= 1802, found {}", registry.list_all().len());

    let tier152_nodes = [
        ("AcousticConcertFluteAirJetModel", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralLatentTimbreInterpolationAutoencoder", DspNodeCategory::NeuralAi),
        ("GermaniumDiodeAsymmetricClipper", DspNodeCategory::DistortionSaturation),
        ("SphericalHarmonicEigenmikeBeamformer", DspNodeCategory::SpatialSurround),
        ("AdditiveHyperbolicPartialSynthesizer", DspNodeCategory::Oscillator),
        ("MultibandUpwardDownwardDynamicExpander", DspNodeCategory::DynamicsMaster),
        ("NonlinearFeedbackDelayNetworkReverb", DspNodeCategory::TimeSpace),
        ("MorphingEllipticCauerLadderFilter", DspNodeCategory::FilterEq),
        ("AsynchronousSpectralStutterBuffer", DspNodeCategory::SamplerSlicer),
        ("QuadratureChaoticLfoModulator", DspNodeCategory::Modulation),
        ("AcousticClassicalNylonGuitarBody", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralDifferentiableIirEqualizer", DspNodeCategory::NeuralAi),
        ("MultiTapMagneticDrumEchoSimulator", DspNodeCategory::TimeSpace),
        ("SaturatingInductorTransformerHysteresis", DspNodeCategory::DistortionSaturation),
        ("BinauralRaytracedEarlyReflectionSpatializer", DspNodeCategory::SpatialSurround),
        ("StereoCorrelationPhaseVectorScopeUtility", DspNodeCategory::Utility),
        ("PhaseDistortionResonantCasioOscillator", DspNodeCategory::Oscillator),
        ("DualPeakFormantVowelBiquadFilter", DspNodeCategory::FilterEq),
        ("MasterStereoVariMuLimiter", DspNodeCategory::DynamicsMaster),
        ("BucketBrigadeAnalogFlangerModulator", DspNodeCategory::Modulation),
        ("ParametricWavefolderSymmetricHarmonizer", DspNodeCategory::DistortionSaturation),
    ];

    for (name, expected_cat) in tier152_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertfluteairjetmodel"), Some("AcousticConcertFluteAirJetModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertfluteairjetmodel"), Some("AcousticConcertFluteAirJetModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neurallatenttimbreinterpolationautoencoder"), Some("NeuralLatentTimbreInterpolationAutoencoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("latenttimbreautoencoder"), Some("NeuralLatentTimbreInterpolationAutoencoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumdiodeasymmetricclipper"), Some("GermaniumDiodeAsymmetricClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asymmetricdiodeclipper"), Some("GermaniumDiodeAsymmetricClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalharmoniceigenmikebeamformer"), Some("SphericalHarmonicEigenmikeBeamformer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("eigenmikebeamformer"), Some("SphericalHarmonicEigenmikeBeamformer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additivehyperbolicpartialsynthesizer"), Some("AdditiveHyperbolicPartialSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolicpartialsynthesizer"), Some("AdditiveHyperbolicPartialSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandupwarddownwarddynamicexpander"), Some("MultibandUpwardDownwardDynamicExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("upwarddownwardexpander"), Some("MultibandUpwardDownwardDynamicExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearfeedbackdelaynetworkreverb"), Some("NonlinearFeedbackDelayNetworkReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("householderfdnreverb"), Some("NonlinearFeedbackDelayNetworkReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingellipticcauerladderfilter"), Some("MorphingEllipticCauerLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ellipticladderfilter"), Some("MorphingEllipticCauerLadderFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asynchronousspectralstutterbuffer"), Some("AsynchronousSpectralStutterBuffer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralstutterbuffer"), Some("AsynchronousSpectralStutterBuffer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quadraturechaoticlfomodulator"), Some("QuadratureChaoticLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticlfomodulator"), Some("QuadratureChaoticLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticclassicalnylonguitarbody"), Some("AcousticClassicalNylonGuitarBody"));
    assert_eq!(DspNodeRegistry::normalize_type_name("classicalnylonguitarbody"), Some("AcousticClassicalNylonGuitarBody"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldifferentiableiirequalizer"), Some("NeuralDifferentiableIirEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("differentiableiirequalizer"), Some("NeuralDifferentiableIirEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multitapmagneticdrumechosimulator"), Some("MultiTapMagneticDrumEchoSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("magneticdrumecho"), Some("MultiTapMagneticDrumEchoSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("saturatinginductortransformerhysteresis"), Some("SaturatingInductorTransformerHysteresis"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transformerhysteresis"), Some("SaturatingInductorTransformerHysteresis"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralraytracedearlyreflectionspatializer"), Some("BinauralRaytracedEarlyReflectionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("raytracedreflectionspatializer"), Some("BinauralRaytracedEarlyReflectionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereocorrelationphasevectorscopeutility"), Some("StereoCorrelationPhaseVectorScopeUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vectorscopeutility"), Some("StereoCorrelationPhaseVectorScopeUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasedistortionresonantcasiooscillator"), Some("PhaseDistortionResonantCasioOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("casioresonantoscillator"), Some("PhaseDistortionResonantCasioOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualpeakformantvowelbiquadfilter"), Some("DualPeakFormantVowelBiquadFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantvowelbiquadfilter"), Some("DualPeakFormantVowelBiquadFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masterstereovarimulimiter"), Some("MasterStereoVariMuLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereovarimulimiter"), Some("MasterStereoVariMuLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bucketbrigadeanalogflangermodulator"), Some("BucketBrigadeAnalogFlangerModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdflangermodulator"), Some("BucketBrigadeAnalogFlangerModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parametricwavefoldersymmetricharmonizer"), Some("ParametricWavefolderSymmetricHarmonizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavefoldersymmetricharmonizer"), Some("ParametricWavefolderSymmetricHarmonizer"));
}

#[test]
fn test_tier153_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1823, "Inventory must contain at least 1823 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1823, "Registry list_all must be >= 1823, found {}", registry.list_all().len());

    let tier153_nodes = [
        ("AeroacousticWhistleVortexResonator", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralPerceptualSpectralEnhancer", DspNodeCategory::NeuralAi),
        ("SaturatingBjtTransistorDifferentialPair", DspNodeCategory::DistortionSaturation),
        ("AmbisonicRadialDistanceFilterBinaural", DspNodeCategory::SpatialSurround),
        ("FractalIteratedFunctionNoiseSynthesizer", DspNodeCategory::Oscillator),
        ("ParallelMultibandTransientMasterLimiter", DspNodeCategory::DynamicsMaster),
        ("AcousticCoupledPlateSpringChamber", DspNodeCategory::TimeSpace),
        ("SpectralFormantMorphingVowelMatrix", DspNodeCategory::FilterEq),
        ("GranularAsynchronousReverseSlicer", DspNodeCategory::SamplerSlicer),
        ("HarmonicResonanceEnvelopeFollower", DspNodeCategory::Modulation),
        ("AcousticDoubleBassSpruceBody", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralHarmonicExciterAirRestorer", DspNodeCategory::NeuralAi),
        ("MultiStageBiquadPhaserModulator", DspNodeCategory::Modulation),
        ("MagneticTapeHeadAzimuthDemagnetizer", DspNodeCategory::TimeSpace),
        ("DualDiodeBridgeCompressorLimiter", DspNodeCategory::DynamicsMaster),
        ("AmbisonicSphericalMicrophonePanner", DspNodeCategory::SpatialSurround),
        ("ParametricGoniometerPhaseCorrelator", DspNodeCategory::Utility),
        ("PneumaticPipeOrganFluteRank", DspNodeCategory::AcousticPhysicalModel),
        ("MultibandSpectralDeEsserDynamic", DspNodeCategory::FilterEq),
        ("SubharmonicBassSynthesizerSubGenerator", DspNodeCategory::Oscillator),
        ("FeedbackTapeLoopEchoFlanger", DspNodeCategory::TimeSpace),
    ];

    for (name, expected_cat) in tier153_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("aeroacousticwhistlevortexresonator"), Some("AeroacousticWhistleVortexResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("whistlevortexresonator"), Some("AeroacousticWhistleVortexResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralperceptualspectralenhancer"), Some("NeuralPerceptualSpectralEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("perceptualspectralenhancer"), Some("NeuralPerceptualSpectralEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("saturatingbjttransistordifferentialpair"), Some("SaturatingBjtTransistorDifferentialPair"));
    assert_eq!(DspNodeRegistry::normalize_type_name("differentialpairsaturator"), Some("SaturatingBjtTransistorDifferentialPair"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicradialdistancefilterbinaural"), Some("AmbisonicRadialDistanceFilterBinaural"));
    assert_eq!(DspNodeRegistry::normalize_type_name("radialdistancefilter"), Some("AmbisonicRadialDistanceFilterBinaural"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fractaliteratedfunctionnoisesynthesizer"), Some("FractalIteratedFunctionNoiseSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ifsfractalsynth"), Some("FractalIteratedFunctionNoiseSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parallelmultibandtransientmasterlimiter"), Some("ParallelMultibandTransientMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientmasterlimiter"), Some("ParallelMultibandTransientMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcoupledplatespringchamber"), Some("AcousticCoupledPlateSpringChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("platespringreverb"), Some("AcousticCoupledPlateSpringChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralformantmorphingvowelmatrix"), Some("SpectralFormantMorphingVowelMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vowelmatrixfilter"), Some("SpectralFormantMorphingVowelMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularasynchronousreverseslicer"), Some("GranularAsynchronousReverseSlicer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularreverseslicer"), Some("GranularAsynchronousReverseSlicer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicresonanceenvelopefollower"), Some("HarmonicResonanceEnvelopeFollower"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicenvelopefollower"), Some("HarmonicResonanceEnvelopeFollower"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticdoublebasssprucebody"), Some("AcousticDoubleBassSpruceBody"));
    assert_eq!(DspNodeRegistry::normalize_type_name("uprightbassbody"), Some("AcousticDoubleBassSpruceBody"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralharmonicexciterairrestorer"), Some("NeuralHarmonicExciterAirRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltubeexciter"), Some("NeuralHarmonicExciterAirRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multistagebiquadphasermodulator"), Some("MultiStageBiquadPhaserModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalbiquadphaser"), Some("MultiStageBiquadPhaserModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("magnetictapeheadazimuthdemagnetizer"), Some("MagneticTapeHeadAzimuthDemagnetizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeazimuthsimulator"), Some("MagneticTapeHeadAzimuthDemagnetizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualdiodebridgecompressorlimiter"), Some("DualDiodeBridgeCompressorLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diodebridgecompressor"), Some("DualDiodeBridgeCompressorLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicsphericalmicrophonepanner"), Some("AmbisonicSphericalMicrophonePanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spherical3dpanner"), Some("AmbisonicSphericalMicrophonePanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parametricgoniometerphasecorrelator"), Some("ParametricGoniometerPhaseCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vectorscopephasecorrelator"), Some("ParametricGoniometerPhaseCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pneumaticpipeorganfluterank"), Some("PneumaticPipeOrganFluteRank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("organfluterank"), Some("PneumaticPipeOrganFluteRank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandspectraldeesserdynamic"), Some("MultibandSpectralDeEsserDynamic"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandspectraldeesser"), Some("MultibandSpectralDeEsserDynamic"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbasssynthesizersubgenerator"), Some("SubharmonicBassSynthesizerSubGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicgenerator"), Some("SubharmonicBassSynthesizerSubGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("feedbacktapeloopechoflanger"), Some("FeedbackTapeLoopEchoFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeloopechoflanger"), Some("FeedbackTapeLoopEchoFlanger"));
}

#[test]
fn test_tier154_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1844, "Inventory must contain at least 1844 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1844, "Registry list_all must be >= 1844, found {}", registry.list_all().len());

    let tier154_nodes = [
        ("AcousticGrandMarimbaRosewoodBar", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralSpectralEnvelopeMorpher", DspNodeCategory::NeuralAi),
        ("DiscreteJfetPreampHarmonicSaturator", DspNodeCategory::DistortionSaturation),
        ("TransauralAcousticDipoleSpatializer", DspNodeCategory::SpatialSurround),
        ("SuperformulaChaoticGeometryOscillator", DspNodeCategory::Oscillator),
        ("MultibandMasteringLinearPhaseLimiter", DspNodeCategory::DynamicsMaster),
        ("BesselWaveguideCylinderAcousticChamber", DspNodeCategory::TimeSpace),
        ("MorphingStateVariableLadderCombFilter", DspNodeCategory::FilterEq),
        ("AsynchronousGranularTimeStretchWarper", DspNodeCategory::SamplerSlicer),
        ("LogarithmicBesselFilterEnvelopeFollower", DspNodeCategory::Modulation),
        ("AcousticSteelStringAcousticGuitarTop", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralPerceptualTransientEnhancer", DspNodeCategory::NeuralAi),
        ("StereoTapeHeadGapLossEqualizer", DspNodeCategory::TimeSpace),
        ("AsymmetricClippingDiodeShaper", DspNodeCategory::DistortionSaturation),
        ("AmbisonicDistanceAirAbsorptionFilter", DspNodeCategory::SpatialSurround),
        ("MasteringTruePeakDitherNoiseShaper", DspNodeCategory::Utility),
        ("AdditivePartialsResonatorBankOscillator", DspNodeCategory::Oscillator),
        ("DualSvfMorphingResonanceEqualizer", DspNodeCategory::FilterEq),
        ("OpticalVactrolPeakMasterCompressor", DspNodeCategory::DynamicsMaster),
        ("DualRotorLeslieDopplerSpeaker", DspNodeCategory::Modulation),
        ("ParametricChebyshevHarmonicFolder", DspNodeCategory::DistortionSaturation),
    ];

    for (name, expected_cat) in tier154_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticgrandmarimbarosewoodbar"), Some("AcousticGrandMarimbaRosewoodBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("grandmarimbarosewoodbar"), Some("AcousticGrandMarimbaRosewoodBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("marimbarosewoodbar"), Some("AcousticGrandMarimbaRosewoodBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralspectralenvelopemorpher"), Some("NeuralSpectralEnvelopeMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralenvelopemorpher"), Some("NeuralSpectralEnvelopeMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralformantmorpher"), Some("NeuralSpectralEnvelopeMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretejfetpreampharmonicsaturator"), Some("DiscreteJfetPreampHarmonicSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jfetpreampharmonicsaturator"), Some("DiscreteJfetPreampHarmonicSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jfetpreampsaturator"), Some("DiscreteJfetPreampHarmonicSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transauralacousticdipolespatializer"), Some("TransauralAcousticDipoleSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticdipolespatializer"), Some("TransauralAcousticDipoleSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dipolespatializer"), Some("TransauralAcousticDipoleSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("superformulachaoticgeometryoscillator"), Some("SuperformulaChaoticGeometryOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticgeometryoscillator"), Some("SuperformulaChaoticGeometryOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("superformulaoscillator"), Some("SuperformulaChaoticGeometryOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmasteringlinearphaselimiter"), Some("MultibandMasteringLinearPhaseLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringlinearphaselimiter"), Some("MultibandMasteringLinearPhaseLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("linearphaselimiting"), Some("MultibandMasteringLinearPhaseLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselwaveguidecylinderacousticchamber"), Some("BesselWaveguideCylinderAcousticChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cylinderacousticchamber"), Some("BesselWaveguideCylinderAcousticChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselwaveguidereverb"), Some("BesselWaveguideCylinderAcousticChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingstatevariableladdercombfilter"), Some("MorphingStateVariableLadderCombFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("svfladdercombfilter"), Some("MorphingStateVariableLadderCombFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("svfcombfilter"), Some("MorphingStateVariableLadderCombFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asynchronousgranulartimestretchwarper"), Some("AsynchronousGranularTimeStretchWarper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granulartimestretchwarper"), Some("AsynchronousGranularTimeStretchWarper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asyncgranulartimestretch"), Some("AsynchronousGranularTimeStretchWarper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("logarithmicbesselfilterenvelopefollower"), Some("LogarithmicBesselFilterEnvelopeFollower"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselfilterenvelopefollower"), Some("LogarithmicBesselFilterEnvelopeFollower"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselenvelopefollower"), Some("LogarithmicBesselFilterEnvelopeFollower"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticsteelstringacousticguitartop"), Some("AcousticSteelStringAcousticGuitarTop"));
    assert_eq!(DspNodeRegistry::normalize_type_name("steelstringacousticguitartop"), Some("AcousticSteelStringAcousticGuitarTop"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dreadnoughtguitartop"), Some("AcousticSteelStringAcousticGuitarTop"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralperceptualtransientenhancer"), Some("NeuralPerceptualTransientEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("perceptualtransientenhancer"), Some("NeuralPerceptualTransientEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltransientenhancer"), Some("NeuralPerceptualTransientEnhancer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereotapeheadgaplossequalizer"), Some("StereoTapeHeadGapLossEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeheadgaplossequalizer"), Some("StereoTapeHeadGapLossEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeheadgaploss"), Some("StereoTapeHeadGapLossEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asymmetricclippingdiodeshaper"), Some("AsymmetricClippingDiodeShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("clippingdiodeshaper"), Some("AsymmetricClippingDiodeShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("asymmetricdiodeshaper"), Some("AsymmetricClippingDiodeShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicdistanceairabsorptionfilter"), Some("AmbisonicDistanceAirAbsorptionFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("distanceairabsorptionfilter"), Some("AmbisonicDistanceAirAbsorptionFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("airabsorptionfilter"), Some("AmbisonicDistanceAirAbsorptionFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringtruepeakdithernoiseshaper"), Some("MasteringTruePeakDitherNoiseShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakdithernoiseshaper"), Some("MasteringTruePeakDitherNoiseShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeaknoiseshaper"), Some("MasteringTruePeakDitherNoiseShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additivepartialsresonatorbankoscillator"), Some("AdditivePartialsResonatorBankOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("partialsresonatorbankoscillator"), Some("AdditivePartialsResonatorBankOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additivepartialbank"), Some("AdditivePartialsResonatorBankOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualsvfmorphingresonanceequalizer"), Some("DualSvfMorphingResonanceEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("svfmorphingresonanceequalizer"), Some("DualSvfMorphingResonanceEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualsvfequalizer"), Some("DualSvfMorphingResonanceEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalvactrolpeakmastercompressor"), Some("OpticalVactrolPeakMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vactrolpeakmastercompressor"), Some("OpticalVactrolPeakMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vactrolmastercompressor"), Some("OpticalVactrolPeakMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualrotorlesliedopplerspeaker"), Some("DualRotorLeslieDopplerSpeaker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lesliedopplerspeaker"), Some("DualRotorLeslieDopplerSpeaker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rotarylesliespeaker"), Some("DualRotorLeslieDopplerSpeaker"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parametricchebyshevharmonicfolder"), Some("ParametricChebyshevHarmonicFolder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chebyshevharmonicfolder"), Some("ParametricChebyshevHarmonicFolder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chebyshevwavefolder"), Some("ParametricChebyshevHarmonicFolder"));
}

#[test]
fn test_tier155_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1865, "Inventory must contain at least 1865 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1865, "Registry list_all must be >= 1865, found {}", registry.list_all().len());

    let tier155_nodes = [
        ("AcousticConcertBassoonBocalDoubleReed", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticSteelStringDobroResonatorCone", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticNyckelharpaUnderstringDroneResonator", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralSpectralHarmonicRestorer", DspNodeCategory::NeuralAi),
        ("NeuralZeroPhaseTransientRestorer", DspNodeCategory::NeuralAi),
        ("DiscreteTriodeCathodeFollowerSaturator", DspNodeCategory::DistortionSaturation),
        ("TransformerCoreMagneticHysteresisSaturator", DspNodeCategory::DistortionSaturation),
        ("SphericalBinauralPinnaDiffractionSpatializer", DspNodeCategory::SpatialSurround),
        ("AcousticParabolicReflectorSpatializer", DspNodeCategory::SpatialSurround),
        ("RelativisticDopplerChaoticWaveformOscillator", DspNodeCategory::Oscillator),
        ("FractionalPuckettGranularCloudOscillator", DspNodeCategory::Oscillator),
        ("VariableMuDualSidechainMasterCompressor", DspNodeCategory::DynamicsMaster),
        ("DynamicTransientMultiBandPeakClipper", DspNodeCategory::DynamicsMaster),
        ("EllipticalAcousticPlateReverbChamber", DspNodeCategory::TimeSpace),
        ("StereoAnalogBucketBrigadeChorusFlanger", DspNodeCategory::TimeSpace),
        ("PassivePultecInductorProgramEqualizer", DspNodeCategory::FilterEq),
        ("MorphingLadderSallenKeyHybridFilter", DspNodeCategory::FilterEq),
        ("QuadratureSineCosineBesselModulator", DspNodeCategory::Modulation),
        ("MultichannelLufsTruePeakMasterMonitor", DspNodeCategory::Utility),
        ("AcousticCavityAirColumnResonator", DspNodeCategory::AcousticPhysicalModel),
        ("DynamicStereoPhaseCorrelationExpander", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier155_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertbassoonbocaldoublereed"), Some("AcousticConcertBassoonBocalDoubleReed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertbassoonbocal"), Some("AcousticConcertBassoonBocalDoubleReed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bassoonbocalreed"), Some("AcousticConcertBassoonBocalDoubleReed"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticsteelstringdobroresonatorcone"), Some("AcousticSteelStringDobroResonatorCone"));
    assert_eq!(DspNodeRegistry::normalize_type_name("steelstringdobrocone"), Some("AcousticSteelStringDobroResonatorCone"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dobroresonatorcone"), Some("AcousticSteelStringDobroResonatorCone"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticnyckelharpaunderstringdroneresonator"), Some("AcousticNyckelharpaUnderstringDroneResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nyckelharpaunderstrings"), Some("AcousticNyckelharpaUnderstringDroneResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nyckelharparesonator"), Some("AcousticNyckelharpaUnderstringDroneResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralspectralharmonicrestorer"), Some("NeuralSpectralHarmonicRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralharmonicrestorer"), Some("NeuralSpectralHarmonicRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralharmonicrestorer"), Some("NeuralSpectralHarmonicRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralzerophasetransientrestorer"), Some("NeuralZeroPhaseTransientRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerophasetransientrestorer"), Some("NeuralZeroPhaseTransientRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltransientrestorer"), Some("NeuralZeroPhaseTransientRestorer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretetriodecathodefollowersaturator"), Some("DiscreteTriodeCathodeFollowerSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodecathodefollowersaturator"), Some("DiscreteTriodeCathodeFollowerSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cathodefollowersaturator"), Some("DiscreteTriodeCathodeFollowerSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transformercoremagnetichysteresissaturator"), Some("TransformerCoreMagneticHysteresisSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transformermagnetichysteresis"), Some("TransformerCoreMagneticHysteresisSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hysteresissaturator"), Some("TransformerCoreMagneticHysteresisSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalbinauralpinnadiffractionspatializer"), Some("SphericalBinauralPinnaDiffractionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalbinauralpinnaspatializer"), Some("SphericalBinauralPinnaDiffractionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalpinnaspatializer"), Some("SphericalBinauralPinnaDiffractionSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticparabolicreflectorspatializer"), Some("AcousticParabolicReflectorSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parabolicreflectorspatializer"), Some("AcousticParabolicReflectorSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parabolicdishspatializer"), Some("AcousticParabolicReflectorSpatializer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("relativisticdopplerchaoticwaveformoscillator"), Some("RelativisticDopplerChaoticWaveformOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("relativisticdoppleroscillator"), Some("RelativisticDopplerChaoticWaveformOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lorentzchaoticoscillator"), Some("RelativisticDopplerChaoticWaveformOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fractionalpuckettgranularcloudoscillator"), Some("FractionalPuckettGranularCloudOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("puckettgranularcloudoscillator"), Some("FractionalPuckettGranularCloudOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fractionalgranularoscillator"), Some("FractionalPuckettGranularCloudOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("variablemudualsidechainmastercompressor"), Some("VariableMuDualSidechainMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("variablemumastercompressor"), Some("VariableMuDualSidechainMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("variablemucompressor"), Some("VariableMuDualSidechainMasterCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamictransientmultibandpeakclipper"), Some("DynamicTransientMultiBandPeakClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandpeakclipper"), Some("DynamicTransientMultiBandPeakClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicpeakclipper"), Some("DynamicTransientMultiBandPeakClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ellipticalacousticplatereverbchamber"), Some("EllipticalAcousticPlateReverbChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ellipticalplatereverb"), Some("EllipticalAcousticPlateReverbChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ellipticalplatereverbchamber"), Some("EllipticalAcousticPlateReverbChamber"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereoanalogbucketbrigadechorusflanger"), Some("StereoAnalogBucketBrigadeChorusFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogbucketbrigadechorusflanger"), Some("StereoAnalogBucketBrigadeChorusFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bbdchorusflanger"), Some("StereoAnalogBucketBrigadeChorusFlanger"));
    assert_eq!(DspNodeRegistry::normalize_type_name("passivepultecinductorprogramequalizer"), Some("PassivePultecInductorProgramEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pultecinductorprogramequalizer"), Some("PassivePultecInductorProgramEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pultecprogrameq"), Some("PassivePultecInductorProgramEqualizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingladdersallenkeyhybridfilter"), Some("MorphingLadderSallenKeyHybridFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("laddersallenkeyhybridfilter"), Some("MorphingLadderSallenKeyHybridFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("laddersallenkeyfilter"), Some("MorphingLadderSallenKeyHybridFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quadraturesinecosinebesselmodulator"), Some("QuadratureSineCosineBesselModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quadraturesinecosinemodulator"), Some("QuadratureSineCosineBesselModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("quadraturebesselmodulator"), Some("QuadratureSineCosineBesselModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannellufstruepeakmastermonitor"), Some("MultichannelLufsTruePeakMasterMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lufstruepeakmastermonitor"), Some("MultichannelLufsTruePeakMasterMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lufsmonitormaster"), Some("MultichannelLufsTruePeakMasterMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcavityaircolumnresonator"), Some("AcousticCavityAirColumnResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cavityaircolumnresonator"), Some("AcousticCavityAirColumnResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("aircolumnresonator"), Some("AcousticCavityAirColumnResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicstereophasecorrelationexpander"), Some("DynamicStereoPhaseCorrelationExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereophasecorrelationexpander"), Some("DynamicStereoPhaseCorrelationExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasecorrelationexpander"), Some("DynamicStereoPhaseCorrelationExpander"));
}

#[test]
fn test_tier156_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1886, "Inventory must contain at least 1886 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1886, "Registry list_all must be >= 1886, found {}", registry.list_all().len());

    let tier156_nodes = [
        ("AcousticBaroqueHarpsichordJackPlectrum", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticConcertGrandTubularChime", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticTurkishOudTearSoundboard", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralSubHarmonicFundamentalSynthesizer", DspNodeCategory::NeuralAi),
        ("NeuralStereoFieldDeCorrelator", DspNodeCategory::NeuralAi),
        ("DiscreteGermaniumDiodeRingModulator", DspNodeCategory::DistortionSaturation),
        ("BipolarJunctionClassABPowerStage", DspNodeCategory::DistortionSaturation),
        ("WfsAcousticCircularArrayRenderer", DspNodeCategory::SpatialSurround),
        ("AmbisonicBinauralEarCanalOcclusion", DspNodeCategory::SpatialSurround),
        ("FractionalChaosDuffingOscillator", DspNodeCategory::Oscillator),
        ("SuperformulaStochasticMorphOscillator", DspNodeCategory::Oscillator),
        ("OpticalVactrolFeedbackMasterLimiter", DspNodeCategory::DynamicsMaster),
        ("MultibandUpwardExpansionTransientSnap", DspNodeCategory::DynamicsMaster),
        ("BesselWaveguideHelicalSpringReverb", DspNodeCategory::TimeSpace),
        ("StereoTapeHeadMagneticAzimuthEcho", DspNodeCategory::TimeSpace),
        ("DualResonantMorphingEllipticFilter", DspNodeCategory::FilterEq),
        ("ParametricStateVariableFormantPeakFilter", DspNodeCategory::FilterEq),
        ("ChaoticStrangeAttractorLfoModulator", DspNodeCategory::Modulation),
        ("MultichannelInterauralTimeDifferenceMonitor", DspNodeCategory::Utility),
        ("AcousticVocalFoldAsymmetricGlottisModel", DspNodeCategory::AcousticPhysicalModel),
        ("MasteringTruePeakNyquistDitherShaper", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier156_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticbaroqueharpsichordjackplectrum"), Some("AcousticBaroqueHarpsichordJackPlectrum"));
    assert_eq!(DspNodeRegistry::normalize_type_name("baroqueharpsichordjack"), Some("AcousticBaroqueHarpsichordJackPlectrum"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harpsichordjackplectrum"), Some("AcousticBaroqueHarpsichordJackPlectrum"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertgrandtubularchime"), Some("AcousticConcertGrandTubularChime"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertgrandtubularchime"), Some("AcousticConcertGrandTubularChime"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubularchimebell"), Some("AcousticConcertGrandTubularChime"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticturkishoudtearsoundboard"), Some("AcousticTurkishOudTearSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("turkishoudsoundboard"), Some("AcousticTurkishOudTearSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("oudtearsoundboard"), Some("AcousticTurkishOudTearSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralsubharmonicfundamentalsynthesizer"), Some("NeuralSubHarmonicFundamentalSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicfundamentalsynthesizer"), Some("NeuralSubHarmonicFundamentalSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralsubharmonicgenerator"), Some("NeuralSubHarmonicFundamentalSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralstereofielddecorrelator"), Some("NeuralStereoFieldDeCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereofielddecorrelator"), Some("NeuralStereoFieldDeCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralstereodecorrelator"), Some("NeuralStereoFieldDeCorrelator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretegermaniumdioderingmodulator"), Some("DiscreteGermaniumDiodeRingModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumdioderingmodulator"), Some("DiscreteGermaniumDiodeRingModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretegermaniumringmodulator"), Some("DiscreteGermaniumDiodeRingModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bipolarjunctionclassabpowerstage"), Some("BipolarJunctionClassABPowerStage"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bjtclassabpowerstage"), Some("BipolarJunctionClassABPowerStage"));
    assert_eq!(DspNodeRegistry::normalize_type_name("classabpowerampstage"), Some("BipolarJunctionClassABPowerStage"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wfsacousticcirculararrayrenderer"), Some("WfsAcousticCircularArrayRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcirculararrayrenderer"), Some("WfsAcousticCircularArrayRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wfscirculararrayrenderer"), Some("WfsAcousticCircularArrayRenderer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicbinauralearcanalocclusion"), Some("AmbisonicBinauralEarCanalOcclusion"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralearcanalocclusion"), Some("AmbisonicBinauralEarCanalOcclusion"));
    assert_eq!(DspNodeRegistry::normalize_type_name("earcanalocclusionmodel"), Some("AmbisonicBinauralEarCanalOcclusion"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fractionalchaosduffingoscillator"), Some("FractionalChaosDuffingOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaosduffingoscillator"), Some("FractionalChaosDuffingOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fractionalduffingoscillator"), Some("FractionalChaosDuffingOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("superformulastochasticmorphoscillator"), Some("SuperformulaStochasticMorphOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("superformulastochasticmorph"), Some("SuperformulaStochasticMorphOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gielissuperformulaoscillator"), Some("SuperformulaStochasticMorphOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalvactrolfeedbackmasterlimiter"), Some("OpticalVactrolFeedbackMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vactrolfeedbackmasterlimiter"), Some("OpticalVactrolFeedbackMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalvactrolmasterlimiter"), Some("OpticalVactrolFeedbackMasterLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandupwardexpansiontransientsnap"), Some("MultibandUpwardExpansionTransientSnap"));
    assert_eq!(DspNodeRegistry::normalize_type_name("upwardexpansiontransientsnap"), Some("MultibandUpwardExpansionTransientSnap"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandtransientsnapper"), Some("MultibandUpwardExpansionTransientSnap"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselwaveguidehelicalspringreverb"), Some("BesselWaveguideHelicalSpringReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselwaveguidehelicalspring"), Some("BesselWaveguideHelicalSpringReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("besselhelicalspringreverb"), Some("BesselWaveguideHelicalSpringReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereotapeheadmagneticazimuthecho"), Some("StereoTapeHeadMagneticAzimuthEcho"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeheadmagneticazimuthecho"), Some("StereoTapeHeadMagneticAzimuthEcho"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereotapeazimuthecho"), Some("StereoTapeHeadMagneticAzimuthEcho"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualresonantmorphingellipticfilter"), Some("DualResonantMorphingEllipticFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingellipticfilter"), Some("DualResonantMorphingEllipticFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cauerellipticdualresonantfilter"), Some("DualResonantMorphingEllipticFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parametricstatevariableformantpeakfilter"), Some("ParametricStateVariableFormantPeakFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("statevariableformantpeakfilter"), Some("ParametricStateVariableFormantPeakFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("svfformantpeakfilter"), Some("ParametricStateVariableFormantPeakFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticstrangeattractorlfomodulator"), Some("ChaoticStrangeAttractorLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("strangeattractorlfomodulator"), Some("ChaoticStrangeAttractorLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rosslerlorenzchaoticlfo"), Some("ChaoticStrangeAttractorLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelinterauraltimedifferencemonitor"), Some("MultichannelInterauralTimeDifferenceMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("interauraltimedifferencemonitor"), Some("MultichannelInterauralTimeDifferenceMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("itdildmastermonitor"), Some("MultichannelInterauralTimeDifferenceMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticvocalfoldasymmetricglottismodel"), Some("AcousticVocalFoldAsymmetricGlottisModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocalfoldasymmetricglottismodel"), Some("AcousticVocalFoldAsymmetricGlottisModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("twomassvocalfoldmodel"), Some("AcousticVocalFoldAsymmetricGlottisModel"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringtruepeaknyquistdithershaper"), Some("MasteringTruePeakNyquistDitherShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeaknyquistdithershaper"), Some("MasteringTruePeakNyquistDitherShaper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ninthordernoiseshaperdither"), Some("MasteringTruePeakNyquistDitherShaper"));
}

#[test]
fn test_tier157_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1907, "Inventory must contain at least 1907 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1907, "Registry list_all must be >= 1907, found {}", registry.list_all().len());

    let tier157_nodes = [
        ("AcousticConcertCelestaSteelBarResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticConcertBassClarinetBoehmRegister", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticFlamencoCajonSnareTensionWire", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralDiffusionLatentTransientSynthesizer", DspNodeCategory::NeuralAi),
        ("NeuralWaveletAcousticDenoisingDenoiser", DspNodeCategory::NeuralAi),
        ("DiscreteJfetCommonSourceOverdriveSaturator", DspNodeCategory::DistortionSaturation),
        ("SaturatingInductiveTransformerDiodeClipper", DspNodeCategory::DistortionSaturation),
        ("AmbisonicSphericalHeadShadowDiffraction", DspNodeCategory::SpatialSurround),
        ("WfsPlanarLinearArrayAcousticSynthesizer", DspNodeCategory::SpatialSurround),
        ("CoupledChaoticVanDerPolNonlinearOscillator", DspNodeCategory::Oscillator),
        ("GranularStochasticPoissonTextureSynthesizer", DspNodeCategory::Oscillator),
        ("ParallelFeedbackFeedforwardOptoCompressor", DspNodeCategory::DynamicsMaster),
        ("MultibandSubHarmonicPunchTransientExpander", DspNodeCategory::DynamicsMaster),
        ("HyperbolicNonEuclideanGeometricReverb", DspNodeCategory::TimeSpace),
        ("StereoAnalogBucketBrigadeMultiTapDelay", DspNodeCategory::TimeSpace),
        ("MorphingButterworthChebyshevHybridFilter", DspNodeCategory::FilterEq),
        ("FormantVocalTractNasalBranchResonator", DspNodeCategory::FilterEq),
        ("StochasticBrownianMotionRandomWalkModulator", DspNodeCategory::Modulation),
        ("MultichannelStereoVectorscopeGoniometerMonitor", DspNodeCategory::Utility),
        ("AcousticTromboneSlideWaveguideResonator", DspNodeCategory::AcousticPhysicalModel),
        ("MasteringZeroLatencyNyquistLinearPhaseFilter", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier157_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertcelestasteelbarresonator"), Some("AcousticConcertCelestaSteelBarResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertcelestasteelbar"), Some("AcousticConcertCelestaSteelBarResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("celestasteelbarresonator"), Some("AcousticConcertCelestaSteelBarResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertbassclarinetboehmregister"), Some("AcousticConcertBassClarinetBoehmRegister"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertbassclarinetregister"), Some("AcousticConcertBassClarinetBoehmRegister"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bassclarinetboehmregister"), Some("AcousticConcertBassClarinetBoehmRegister"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticflamencocajonsnaretensionwire"), Some("AcousticFlamencoCajonSnareTensionWire"));
    assert_eq!(DspNodeRegistry::normalize_type_name("flamencocajonsnare"), Some("AcousticFlamencoCajonSnareTensionWire"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cajonsnaretensionwire"), Some("AcousticFlamencoCajonSnareTensionWire"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldiffusionlatenttransientsynthesizer"), Some("NeuralDiffusionLatentTransientSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusionlatenttransientsynthesizer"), Some("NeuralDiffusionLatentTransientSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltransientsynthesizer"), Some("NeuralDiffusionLatentTransientSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralwaveletacousticdenoisingdenoiser"), Some("NeuralWaveletAcousticDenoisingDenoiser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("waveletacousticdenoiser"), Some("NeuralWaveletAcousticDenoisingDenoiser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralwaveletdenoiser"), Some("NeuralWaveletAcousticDenoisingDenoiser"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretejfetcommonsourceoverdrivesaturator"), Some("DiscreteJfetCommonSourceOverdriveSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("jfetcommonsourceoverdrive"), Some("DiscreteJfetCommonSourceOverdriveSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretejfetsaturator"), Some("DiscreteJfetCommonSourceOverdriveSaturator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("saturatinginductivetransformerdiodeclipper"), Some("SaturatingInductiveTransformerDiodeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("inductivetransformerdiodeclipper"), Some("SaturatingInductiveTransformerDiodeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transformerdiodesaturator"), Some("SaturatingInductiveTransformerDiodeClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicsphericalheadshadowdiffraction"), Some("AmbisonicSphericalHeadShadowDiffraction"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalheadshadowdiffraction"), Some("AmbisonicSphericalHeadShadowDiffraction"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicheadshadow"), Some("AmbisonicSphericalHeadShadowDiffraction"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wfsplanarlineararrayacousticsynthesizer"), Some("WfsPlanarLinearArrayAcousticSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("planarlineararrayacousticsynthesizer"), Some("WfsPlanarLinearArrayAcousticSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wfslineararraysynthesizer"), Some("WfsPlanarLinearArrayAcousticSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("coupledchaoticvanderpolnonlinearoscillator"), Some("CoupledChaoticVanDerPolNonlinearOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticvanderpolnonlinearoscillator"), Some("CoupledChaoticVanDerPolNonlinearOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vanderpolchaoticoscillator"), Some("CoupledChaoticVanDerPolNonlinearOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("granularstochasticpoissontexturesynthesizer"), Some("GranularStochasticPoissonTextureSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticpoissontexturesynthesizer"), Some("GranularStochasticPoissonTextureSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("poissongranularsynthesizer"), Some("GranularStochasticPoissonTextureSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parallelfeedbackfeedforwardoptocompressor"), Some("ParallelFeedbackFeedforwardOptoCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("feedbackfeedforwardoptocompressor"), Some("ParallelFeedbackFeedforwardOptoCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("paralleloptocompressor"), Some("ParallelFeedbackFeedforwardOptoCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandsubharmonicpunchtransientexpander"), Some("MultibandSubHarmonicPunchTransientExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicpunchtransientexpander"), Some("MultibandSubHarmonicPunchTransientExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicpunchexpander"), Some("MultibandSubHarmonicPunchTransientExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolicnoneuclideangeometricreverb"), Some("HyperbolicNonEuclideanGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("noneuclideangeometricreverb"), Some("HyperbolicNonEuclideanGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolicgeometricreverb"), Some("HyperbolicNonEuclideanGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereoanalogbucketbrigademultitapdelay"), Some("StereoAnalogBucketBrigadeMultiTapDelay"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogbucketbrigademultitapdelay"), Some("StereoAnalogBucketBrigadeMultiTapDelay"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereobbdmultitapdelay"), Some("StereoAnalogBucketBrigadeMultiTapDelay"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingbutterworthchebyshevhybridfilter"), Some("MorphingButterworthChebyshevHybridFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("butterworthchebyshevhybridfilter"), Some("MorphingButterworthChebyshevHybridFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("butterworthchebyshevfilter"), Some("MorphingButterworthChebyshevHybridFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantvocaltractnasalbranchresonator"), Some("FormantVocalTractNasalBranchResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vocaltractnasalbranchresonator"), Some("FormantVocalTractNasalBranchResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nasalbranchformantresonator"), Some("FormantVocalTractNasalBranchResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stochasticbrownianmotionrandomwalkmodulator"), Some("StochasticBrownianMotionRandomWalkModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brownianmotionrandomwalkmodulator"), Some("StochasticBrownianMotionRandomWalkModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fbmrandomwalkmodulator"), Some("StochasticBrownianMotionRandomWalkModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelstereovectorscopegoniometermonitor"), Some("MultichannelStereoVectorscopeGoniometerMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereovectorscopegoniometermonitor"), Some("MultichannelStereoVectorscopeGoniometerMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vectorscopegoniometermonitor"), Some("MultichannelStereoVectorscopeGoniometerMonitor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acoustictromboneslidewaveguideresonator"), Some("AcousticTromboneSlideWaveguideResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tromboneslidewaveguideresonator"), Some("AcousticTromboneSlideWaveguideResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tromboneslideresonator"), Some("AcousticTromboneSlideWaveguideResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringzerolatencynyquistlinearphasefilter"), Some("MasteringZeroLatencyNyquistLinearPhaseFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerolatencynyquistlinearphasefilter"), Some("MasteringZeroLatencyNyquistLinearPhaseFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nyquistlinearphasefilter"), Some("MasteringZeroLatencyNyquistLinearPhaseFilter"));
}

#[test]
fn test_tier158_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1928, "Inventory must contain at least 1928 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1928, "Registry list_all must be >= 1928, found {}", registry.list_all().len());

    let tier158_nodes = [
        ("AcousticConcertGrandTimpaniBowlResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticVibraphoneMotorizedTremoloButterfly", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticSteelTongueDrumCavityResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticRenaissanceLuteRosewoodRibBowl", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralContinuousNormalizingFlowTimbreMorpher", DspNodeCategory::NeuralAi),
        ("NeuralDifferentiableWavetablePhaseVocoder", DspNodeCategory::NeuralAi),
        ("DiscreteGermaniumTransistorFuzzFaceDrive", DspNodeCategory::DistortionSaturation),
        ("ParametricDiodeClipperSoftSaturation", DspNodeCategory::DistortionSaturation),
        ("AmbisonicSphericalMicrophoneArrayBinauralizer", DspNodeCategory::SpatialSurround),
        ("TransauralStereoDipoleCrosstalkCanceller", DspNodeCategory::SpatialSurround),
        ("HyperchaoticLorentzAttractorSynthesizer", DspNodeCategory::Oscillator),
        ("AdditiveMorphingInharmonicBellGenerator", DspNodeCategory::Oscillator),
        ("MultibandMasteringFeedForwardVcaCompressor", DspNodeCategory::DynamicsMaster),
        ("MidSidePsychoacousticPunchLimiter", DspNodeCategory::DynamicsMaster),
        ("AnalogOpticalSpringTankReverbSimulator", DspNodeCategory::TimeSpace),
        ("DualTapeHeadPingPongEchoMatrix", DspNodeCategory::TimeSpace),
        ("NonlinearMoogTransistorLadder24dbFilter", DspNodeCategory::FilterEq),
        ("MorphingFormantVowelBandpassFilterBank", DspNodeCategory::FilterEq),
        ("ChaoticDuffingOscillatorDoubleWellModulator", DspNodeCategory::Modulation),
        ("MultichannelLoudspeakerRoomCorrectionUtility", DspNodeCategory::Utility),
        ("MasteringPhaseCorrelationGoniometerUtility", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier158_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertgrandtimpanibowlresonator"), Some("AcousticConcertGrandTimpaniBowlResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertgrandtimpanibowl"), Some("AcousticConcertGrandTimpaniBowlResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("timpanibowlresonator"), Some("AcousticConcertGrandTimpaniBowlResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticvibraphonemotorizedtremolobutterfly"), Some("AcousticVibraphoneMotorizedTremoloButterfly"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vibraphonemotorizedbutterfly"), Some("AcousticVibraphoneMotorizedTremoloButterfly"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vibraphonetremolobutterfly"), Some("AcousticVibraphoneMotorizedTremoloButterfly"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticsteeltonguedrumcavityresonator"), Some("AcousticSteelTongueDrumCavityResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("steeltonguedrumresonator"), Some("AcousticSteelTongueDrumCavityResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("steeltonguedrum"), Some("AcousticSteelTongueDrumCavityResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticrenaissanceluterosewoodribbowl"), Some("AcousticRenaissanceLuteRosewoodRibBowl"));
    assert_eq!(DspNodeRegistry::normalize_type_name("renaissanceluteribbowl"), Some("AcousticRenaissanceLuteRosewoodRibBowl"));
    assert_eq!(DspNodeRegistry::normalize_type_name("renaissancelute"), Some("AcousticRenaissanceLuteRosewoodRibBowl"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralcontinuousnormalizingflowtimbremorpher"), Some("NeuralContinuousNormalizingFlowTimbreMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("continuousnormalizingflowtimbremorpher"), Some("NeuralContinuousNormalizingFlowTimbreMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraltimbremorpher"), Some("NeuralContinuousNormalizingFlowTimbreMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldifferentiablewavetablephasevocoder"), Some("NeuralDifferentiableWavetablePhaseVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("differentiablewavetablephasevocoder"), Some("NeuralDifferentiableWavetablePhaseVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralphasevocoder"), Some("NeuralDifferentiableWavetablePhaseVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretegermaniumtransistorfuzzfacedrive"), Some("DiscreteGermaniumTransistorFuzzFaceDrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumtransistorfuzzface"), Some("DiscreteGermaniumTransistorFuzzFaceDrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumfuzzface"), Some("DiscreteGermaniumTransistorFuzzFaceDrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parametricdiodeclippersoftsaturation"), Some("ParametricDiodeClipperSoftSaturation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("parametricdiodeclipper"), Some("ParametricDiodeClipperSoftSaturation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diodesoftsaturation"), Some("ParametricDiodeClipperSoftSaturation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicsphericalmicrophonearraybinauralizer"), Some("AmbisonicSphericalMicrophoneArrayBinauralizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalmicrophonearraybinauralizer"), Some("AmbisonicSphericalMicrophoneArrayBinauralizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalbinauralizer"), Some("AmbisonicSphericalMicrophoneArrayBinauralizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transauralstereodipolecrosstalkcanceller"), Some("TransauralStereoDipoleCrosstalkCanceller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereodipolecrosstalkcanceller"), Some("TransauralStereoDipoleCrosstalkCanceller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transauraldipole"), Some("TransauralStereoDipoleCrosstalkCanceller"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperchaoticlorentzattractorsynthesizer"), Some("HyperchaoticLorentzAttractorSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperchaoticlorentzattractor"), Some("HyperchaoticLorentzAttractorSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperchaoticsynthesizer"), Some("HyperchaoticLorentzAttractorSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additivemorphinginharmonicbellgenerator"), Some("AdditiveMorphingInharmonicBellGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additiveinharmonicbellgenerator"), Some("AdditiveMorphingInharmonicBellGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("inharmonicbellgenerator"), Some("AdditiveMorphingInharmonicBellGenerator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmasteringfeedforwardvcacompressor"), Some("MultibandMasteringFeedForwardVcaCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringfeedforwardvcacompressor"), Some("MultibandMasteringFeedForwardVcaCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandvcacompressor"), Some("MultibandMasteringFeedForwardVcaCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidepsychoacousticpunchlimiter"), Some("MidSidePsychoacousticPunchLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidepunchlimiter"), Some("MidSidePsychoacousticPunchLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticpunchlimiter"), Some("MidSidePsychoacousticPunchLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogopticalspringtankreverbsimulator"), Some("AnalogOpticalSpringTankReverbSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("opticalspringtankreverb"), Some("AnalogOpticalSpringTankReverbSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("springtankreverb"), Some("AnalogOpticalSpringTankReverbSimulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualtapeheadpingpongechomatrix"), Some("DualTapeHeadPingPongEchoMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tapeheadpingpongechomatrix"), Some("DualTapeHeadPingPongEchoMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pingpongtapeecho"), Some("DualTapeHeadPingPongEchoMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearmoogtransistorladder24dbfilter"), Some("NonlinearMoogTransistorLadder24dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("moogtransistorladder24dbfilter"), Some("NonlinearMoogTransistorLadder24dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("moogladder24dbfilter"), Some("NonlinearMoogTransistorLadder24dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingformantvowelbandpassfilterbank"), Some("MorphingFormantVowelBandpassFilterBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantvowelbandpassfilterbank"), Some("MorphingFormantVowelBandpassFilterBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vowelfilterbank"), Some("MorphingFormantVowelBandpassFilterBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticduffingoscillatordoublewellmodulator"), Some("ChaoticDuffingOscillatorDoubleWellModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("duffingoscillatordoublewellmodulator"), Some("ChaoticDuffingOscillatorDoubleWellModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("duffingmodulator"), Some("ChaoticDuffingOscillatorDoubleWellModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelloudspeakerroomcorrectionutility"), Some("MultichannelLoudspeakerRoomCorrectionUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("loudspeakerroomcorrectionutility"), Some("MultichannelLoudspeakerRoomCorrectionUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("roomcorrectionutility"), Some("MultichannelLoudspeakerRoomCorrectionUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringphasecorrelationgoniometerutility"), Some("MasteringPhaseCorrelationGoniometerUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasecorrelationgoniometerutility"), Some("MasteringPhaseCorrelationGoniometerUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("goniometerutility"), Some("MasteringPhaseCorrelationGoniometerUtility"));
}

#[test]
fn test_tier159_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1949, "Inventory must contain at least 1949 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1949, "Registry list_all must be >= 1949, found {}", registry.list_all().len());

    let tier159_nodes = [
        ("AcousticConcertPedalHarpSpruceSoundboard", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticConcertGlockenspielSteelBar", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticWestAfricanBalafonGourdResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticNorthIndianSarodTeakResonator", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralDifferentialWavetableLatentOscillator", DspNodeCategory::NeuralAi),
        ("NeuralAdaptiveSpectralDeReverberator", DspNodeCategory::NeuralAi),
        ("DiscretePentodeScreenGridTubeCompressor", DspNodeCategory::DistortionSaturation),
        ("AnalogBipolarCmosOperationalTransconductanceOverdrive", DspNodeCategory::DistortionSaturation),
        ("HigherOrderAmbisonicBinauralNearFieldCompensator", DspNodeCategory::SpatialSurround),
        ("TransauralAcousticCrossfeedHeadphoneOptimizer", DspNodeCategory::SpatialSurround),
        ("NonlinearCoupledVanDerPolChaosSynthesizer", DspNodeCategory::Oscillator),
        ("AdditiveSpectralInharmonicPartialsSynthesizer", DspNodeCategory::Oscillator),
        ("MultibandMasteringOpticalFeedbackLimiter", DspNodeCategory::DynamicsMaster),
        ("DynamicTransientSpectralSnapProcessor", DspNodeCategory::DynamicsMaster),
        ("AnalogMultiHeadMagneticTapeDelayMatrix", DspNodeCategory::TimeSpace),
        ("NonEuclideanSphericalChamberAcousticReverb", DspNodeCategory::TimeSpace),
        ("NonlinearDiodeLadderZdf24dbLowpassFilter", DspNodeCategory::FilterEq),
        ("MorphingSpectralVowelResonanceFilter", DspNodeCategory::FilterEq),
        ("ChaoticLorenzAttractorTripleAxisLfoModulator", DspNodeCategory::Modulation),
        ("MultichannelStereoCorrelationSpectrogramUtility", DspNodeCategory::Utility),
        ("MasteringZeroPhaseNyquistAntiAliasingFilter", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier159_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertpedalharpsprucesoundboard"), Some("AcousticConcertPedalHarpSpruceSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertpedalharpsoundboard"), Some("AcousticConcertPedalHarpSpruceSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pedalharpsoundboard"), Some("AcousticConcertPedalHarpSpruceSoundboard"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticconcertglockenspielsteelbar"), Some("AcousticConcertGlockenspielSteelBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertglockenspielsteelbar"), Some("AcousticConcertGlockenspielSteelBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("glockenspielsteelbar"), Some("AcousticConcertGlockenspielSteelBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticwestafricanbalafongourdresonator"), Some("AcousticWestAfricanBalafonGourdResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("westafricanbalafongourdresonator"), Some("AcousticWestAfricanBalafonGourdResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("balafongourdresonator"), Some("AcousticWestAfricanBalafonGourdResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticnorthindiansarodteakresonator"), Some("AcousticNorthIndianSarodTeakResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("northindiansarodteakresonator"), Some("AcousticNorthIndianSarodTeakResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sarodteakresonator"), Some("AcousticNorthIndianSarodTeakResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldifferentialwavetablelatentoscillator"), Some("NeuralDifferentialWavetableLatentOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("differentialwavetablelatentoscillator"), Some("NeuralDifferentialWavetableLatentOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralwavetablelatentoscillator"), Some("NeuralDifferentialWavetableLatentOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraladaptivespectraldereverberator"), Some("NeuralAdaptiveSpectralDeReverberator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("adaptivespectraldereverberator"), Some("NeuralAdaptiveSpectralDeReverberator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldereverberator"), Some("NeuralAdaptiveSpectralDeReverberator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretepentodescreengridtubecompressor"), Some("DiscretePentodeScreenGridTubeCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pentodescreengridtubecompressor"), Some("DiscretePentodeScreenGridTubeCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pentodetubecompressor"), Some("DiscretePentodeScreenGridTubeCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogbipolarcmosoperationaltransconductanceoverdrive"), Some("AnalogBipolarCmosOperationalTransconductanceOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bipolarcmosotaoverdrive"), Some("AnalogBipolarCmosOperationalTransconductanceOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("otaoverdrive"), Some("AnalogBipolarCmosOperationalTransconductanceOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("higherorderambisonicbinauralnearfieldcompensator"), Some("HigherOrderAmbisonicBinauralNearFieldCompensator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicbinauralnearfieldcompensator"), Some("HigherOrderAmbisonicBinauralNearFieldCompensator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nearfieldcompensator"), Some("HigherOrderAmbisonicBinauralNearFieldCompensator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transauralacousticcrossfeedheadphoneoptimizer"), Some("TransauralAcousticCrossfeedHeadphoneOptimizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcrossfeedheadphoneoptimizer"), Some("TransauralAcousticCrossfeedHeadphoneOptimizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crossfeedoptimizer"), Some("TransauralAcousticCrossfeedHeadphoneOptimizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearcoupledvanderpolchaossynthesizer"), Some("NonlinearCoupledVanDerPolChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("coupledvanderpolchaossynthesizer"), Some("NonlinearCoupledVanDerPolChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vanderpolchaossynth"), Some("NonlinearCoupledVanDerPolChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additivespectralinharmonicpartialssynthesizer"), Some("AdditiveSpectralInharmonicPartialsSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralinharmonicpartialssynthesizer"), Some("AdditiveSpectralInharmonicPartialsSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additiveinharmonicsynth"), Some("AdditiveSpectralInharmonicPartialsSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmasteringopticalfeedbacklimiter"), Some("MultibandMasteringOpticalFeedbackLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringopticalfeedbacklimiter"), Some("MultibandMasteringOpticalFeedbackLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandoptolimiter"), Some("MultibandMasteringOpticalFeedbackLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamictransientspectralsnapprocessor"), Some("DynamicTransientSpectralSnapProcessor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientspectralsnapprocessor"), Some("DynamicTransientSpectralSnapProcessor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("transientsnapprocessor"), Some("DynamicTransientSpectralSnapProcessor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogmultiheadmagnetictapedelaymatrix"), Some("AnalogMultiHeadMagneticTapeDelayMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multiheadmagnetictapedelaymatrix"), Some("AnalogMultiHeadMagneticTapeDelayMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multiheadtapedelay"), Some("AnalogMultiHeadMagneticTapeDelayMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("noneuclideansphericalchamberacousticreverb"), Some("NonEuclideanSphericalChamberAcousticReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalchamberacousticreverb"), Some("NonEuclideanSphericalChamberAcousticReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalchamberreverb"), Some("NonEuclideanSphericalChamberAcousticReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlineardiodeladderzdf24dblowpassfilter"), Some("NonlinearDiodeLadderZdf24dbLowpassFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diodeladderzdf24dblowpassfilter"), Some("NonlinearDiodeLadderZdf24dbLowpassFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diodeladderzdf"), Some("NonlinearDiodeLadderZdf24dbLowpassFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingspectralvowelresonancefilter"), Some("MorphingSpectralVowelResonanceFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralvowelresonancefilter"), Some("MorphingSpectralVowelResonanceFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vowelresonancefilter"), Some("MorphingSpectralVowelResonanceFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticlorenzattractortripleaxislfomodulator"), Some("ChaoticLorenzAttractorTripleAxisLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lorenzattractortripleaxislfomodulator"), Some("ChaoticLorenzAttractorTripleAxisLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lorenzattractorlfo"), Some("ChaoticLorenzAttractorTripleAxisLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelstereocorrelationspectrogramutility"), Some("MultichannelStereoCorrelationSpectrogramUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereocorrelationspectrogramutility"), Some("MultichannelStereoCorrelationSpectrogramUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectrogramcorrelationutility"), Some("MultichannelStereoCorrelationSpectrogramUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringzerophasenyquistantialiasingfilter"), Some("MasteringZeroPhaseNyquistAntiAliasingFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("zerophasenyquistantialiasingfilter"), Some("MasteringZeroPhaseNyquistAntiAliasingFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nyquistantialiasingfilter"), Some("MasteringZeroPhaseNyquistAntiAliasingFilter"));
}

#[test]
fn test_tier160_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1970, "Inventory must contain at least 1970 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1970, "Registry list_all must be >= 1970, found {}", registry.list_all().len());

    let tier160_nodes = [
        ("AcousticTubularBellChimeResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticOrchestralGongTamTamResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticHardangerFiddleSympatheticStrings", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticPersianTarMulberryResonator", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralRecurrentReservoirComputingSynthesizer", DspNodeCategory::NeuralAi),
        ("NeuralSubharmonicBassRestorationSynthesizer", DspNodeCategory::NeuralAi),
        ("DiscreteGermaniumDiodeAsymmetricClipper", DspNodeCategory::DistortionSaturation),
        ("AnalogCascodeJfetOverdrivePreamplifier", DspNodeCategory::DistortionSaturation),
        ("InterauralPinnaSpectralNotchFilter", DspNodeCategory::SpatialSurround),
        ("AmbisonicSphericalCapMicrophonePanner", DspNodeCategory::SpatialSurround),
        ("NonlinearDoublePendulumChaosSynthesizer", DspNodeCategory::Oscillator),
        ("WavetableFormantGranularHybridOscillator", DspNodeCategory::Oscillator),
        ("MultibandMasteringVcaFeedbackBusCompressor", DspNodeCategory::DynamicsMaster),
        ("PsychoacousticSpectralCrestFactorLimiter", DspNodeCategory::DynamicsMaster),
        ("AnalogDualBucketBrigadeChorusFlangerMatrix", DspNodeCategory::TimeSpace),
        ("ToroidalAcousticDiffuserGeometricReverb", DspNodeCategory::TimeSpace),
        ("NonlinearSallenKeyActive12dbFilter", DspNodeCategory::FilterEq),
        ("SpectralCombFilterBankFormantMorpher", DspNodeCategory::FilterEq),
        ("FractionalOrderBrownianMotionLfoModulator", DspNodeCategory::Modulation),
        ("MultichannelMidSidePhaseCorrelationScope", DspNodeCategory::Utility),
        ("MasteringTruePeakInterSampleOversamplingMeter", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier160_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acoustictubularbellchimeresonator"), Some("AcousticTubularBellChimeResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubularbellchimeresonator"), Some("AcousticTubularBellChimeResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tubularbellresonator"), Some("AcousticTubularBellChimeResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticorchestralgongtamtamresonator"), Some("AcousticOrchestralGongTamTamResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("orchestralgongtamtamresonator"), Some("AcousticOrchestralGongTamTamResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gongtamtamresonator"), Some("AcousticOrchestralGongTamTamResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acoustichardangerfiddlesympatheticstrings"), Some("AcousticHardangerFiddleSympatheticStrings"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hardangerfiddlesympatheticstrings"), Some("AcousticHardangerFiddleSympatheticStrings"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hardangerfiddlestrings"), Some("AcousticHardangerFiddleSympatheticStrings"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticpersiantarmulberryresonator"), Some("AcousticPersianTarMulberryResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("persiantarmulberryresonator"), Some("AcousticPersianTarMulberryResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("tarmulberryresonator"), Some("AcousticPersianTarMulberryResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralrecurrentreservoircomputingsynthesizer"), Some("NeuralRecurrentReservoirComputingSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("recurrentreservoircomputingsynthesizer"), Some("NeuralRecurrentReservoirComputingSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("reservoircomputingsynth"), Some("NeuralRecurrentReservoirComputingSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralsubharmonicbassrestorationsynthesizer"), Some("NeuralSubharmonicBassRestorationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbassrestorationsynthesizer"), Some("NeuralSubharmonicBassRestorationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("subharmonicbassrestoration"), Some("NeuralSubharmonicBassRestorationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretegermaniumdiodeasymmetricclipper"), Some("DiscreteGermaniumDiodeAsymmetricClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretegermaniumclipper"), Some("DiscreteGermaniumDiodeAsymmetricClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretegermaniumasymmetricclipper"), Some("DiscreteGermaniumDiodeAsymmetricClipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogcascodejfetoverdrivepreamplifier"), Some("AnalogCascodeJfetOverdrivePreamplifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cascodejfetoverdrivepreamplifier"), Some("AnalogCascodeJfetOverdrivePreamplifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("cascodejfetoverdrive"), Some("AnalogCascodeJfetOverdrivePreamplifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("interauralpinnaspectralnotchfilter"), Some("InterauralPinnaSpectralNotchFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pinnaspectralnotchfilter"), Some("InterauralPinnaSpectralNotchFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("pinnanotchfilter"), Some("InterauralPinnaSpectralNotchFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicsphericalcapmicrophonepanner"), Some("AmbisonicSphericalCapMicrophonePanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalcapmicrophonepanner"), Some("AmbisonicSphericalCapMicrophonePanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalcappanner"), Some("AmbisonicSphericalCapMicrophonePanner"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlineardoublependulumchaossynthesizer"), Some("NonlinearDoublePendulumChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("doublependulumchaossynthesizer"), Some("NonlinearDoublePendulumChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("doublependulumchaossynth"), Some("NonlinearDoublePendulumChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavetableformantgranularhybridoscillator"), Some("WavetableFormantGranularHybridOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("formantgranularhybridoscillator"), Some("WavetableFormantGranularHybridOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("wavetablegranularhybridosc"), Some("WavetableFormantGranularHybridOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmasteringvcafeedbackbuscompressor"), Some("MultibandMasteringVcaFeedbackBusCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringvcafeedbackbuscompressor"), Some("MultibandMasteringVcaFeedbackBusCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vcafeedbackbuscompressor"), Some("MultibandMasteringVcaFeedbackBusCompressor"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticspectralcrestfactorlimiter"), Some("PsychoacousticSpectralCrestFactorLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralcrestfactorlimiter"), Some("PsychoacousticSpectralCrestFactorLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("crestfactorlimiter"), Some("PsychoacousticSpectralCrestFactorLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogdualbucketbrigadechorusflangermatrix"), Some("AnalogDualBucketBrigadeChorusFlangerMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualbbdchorusflangermatrix"), Some("AnalogDualBucketBrigadeChorusFlangerMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dualbbdchorusflanger"), Some("AnalogDualBucketBrigadeChorusFlangerMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("toroidalacousticdiffusergeometricreverb"), Some("ToroidalAcousticDiffuserGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("toroidalacousticdiffuserreverb"), Some("ToroidalAcousticDiffuserGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("toroidalreverb"), Some("ToroidalAcousticDiffuserGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearsallenkeyactive12dbfilter"), Some("NonlinearSallenKeyActive12dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sallenkeyactive12dbfilter"), Some("NonlinearSallenKeyActive12dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearsallenkeyfilter"), Some("NonlinearSallenKeyActive12dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralcombfilterbankformantmorpher"), Some("SpectralCombFilterBankFormantMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("combfilterbankformantmorpher"), Some("SpectralCombFilterBankFormantMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("combformantmorpher"), Some("SpectralCombFilterBankFormantMorpher"));
    assert_eq!(DspNodeRegistry::normalize_type_name("fractionalorderbrownianmotionlfomodulator"), Some("FractionalOrderBrownianMotionLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brownianmotionlfomodulator"), Some("FractionalOrderBrownianMotionLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("brownianmotionlfo"), Some("FractionalOrderBrownianMotionLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelmidsidephasecorrelationscope"), Some("MultichannelMidSidePhaseCorrelationScope"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidephasecorrelationscope"), Some("MultichannelMidSidePhaseCorrelationScope"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidecorrelationscope"), Some("MultichannelMidSidePhaseCorrelationScope"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringtruepeakintersampleoversamplingmeter"), Some("MasteringTruePeakInterSampleOversamplingMeter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakintersampleoversamplingmeter"), Some("MasteringTruePeakInterSampleOversamplingMeter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("truepeakoversamplingmeter"), Some("MasteringTruePeakInterSampleOversamplingMeter"));
}

#[test]
fn test_tier161_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 1991, "Inventory must contain at least 1991 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 1991, "Registry list_all must be >= 1991, found {}", registry.list_all().len());

    let tier161_nodes = [
        ("AcousticCelestaResonantSteelPlate", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticOrchestralBassDrumCalfskinResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticRenaissanceViolaDaGambaResonator", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticTurkishOudWalnutBowlResonator", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralDiffusionLatentHarmonicExciter", DspNodeCategory::NeuralAi),
        ("NeuralPhaseUnwrappingStereoSynthesizer", DspNodeCategory::NeuralAi),
        ("DiscreteSiliconFuzzFaceSaturation", DspNodeCategory::DistortionSaturation),
        ("AnalogMuFollowerDualTriodeOverdrive", DspNodeCategory::DistortionSaturation),
        ("BinauralSpectralNotchElevationSynthesizer", DspNodeCategory::SpatialSurround),
        ("AmbisonicSphericalHarmonicRotatorMatrix", DspNodeCategory::SpatialSurround),
        ("NonlinearChuaCircuitAttractorSynthesizer", DspNodeCategory::Oscillator),
        ("AdditiveSpectralFormantHarmonicOscillator", DspNodeCategory::Oscillator),
        ("MasteringMidSideMultibandExpander", DspNodeCategory::DynamicsMaster),
        ("PsychoacousticMultiStageClippingLimiter", DspNodeCategory::DynamicsMaster),
        ("VintageGermaniumTapeDelayEcho", DspNodeCategory::TimeSpace),
        ("EllipsoidalWhisperingGalleryGeometricReverb", DspNodeCategory::TimeSpace),
        ("NonlinearKorg35StateVariable24dbFilter", DspNodeCategory::FilterEq),
        ("SpectralMultiNotchPhaseCombFilter", DspNodeCategory::FilterEq),
        ("ChaoticRosslerAttractorLfoModulator", DspNodeCategory::Modulation),
        ("MultichannelPhaseAlignmentDelayUtility", DspNodeCategory::Utility),
        ("MasteringLufsLoudnessTargetNormalizer", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier161_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcelestaresonantsteelplate"), Some("AcousticCelestaResonantSteelPlate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("celestaresonantsteelplate"), Some("AcousticCelestaResonantSteelPlate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("celestasteelplate"), Some("AcousticCelestaResonantSteelPlate"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticorchestralbassdrumcalfskinresonator"), Some("AcousticOrchestralBassDrumCalfskinResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("orchestralbassdrumcalfskinresonator"), Some("AcousticOrchestralBassDrumCalfskinResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("concertbassdrumresonator"), Some("AcousticOrchestralBassDrumCalfskinResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticrenaissancevioladagambaresonator"), Some("AcousticRenaissanceViolaDaGambaResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("renaissancevioladagambaresonator"), Some("AcousticRenaissanceViolaDaGambaResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("violadagambaresonator"), Some("AcousticRenaissanceViolaDaGambaResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticturkishoudwalnutbowlresonator"), Some("AcousticTurkishOudWalnutBowlResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("turkishoudwalnutbowlresonator"), Some("AcousticTurkishOudWalnutBowlResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("turkishoudresonator"), Some("AcousticTurkishOudWalnutBowlResonator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuraldiffusionlatentharmonicexciter"), Some("NeuralDiffusionLatentHarmonicExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diffusionlatentharmonicexciter"), Some("NeuralDiffusionLatentHarmonicExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("latentharmonicexciter"), Some("NeuralDiffusionLatentHarmonicExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralphaseunwrappingstereosynthesizer"), Some("NeuralPhaseUnwrappingStereoSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phaseunwrappingstereosynthesizer"), Some("NeuralPhaseUnwrappingStereoSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralphaseunwrappingsynth"), Some("NeuralPhaseUnwrappingStereoSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discretesiliconfuzzfacesaturation"), Some("DiscreteSiliconFuzzFaceSaturation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("siliconfuzzfacesaturation"), Some("DiscreteSiliconFuzzFaceSaturation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("siliconfuzzface"), Some("DiscreteSiliconFuzzFaceSaturation"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogmufollowerdualtriodeoverdrive"), Some("AnalogMuFollowerDualTriodeOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mufollowerdualtriodeoverdrive"), Some("AnalogMuFollowerDualTriodeOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("mufolloweroverdrive"), Some("AnalogMuFollowerDualTriodeOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralspectralnotchelevationsynthesizer"), Some("BinauralSpectralNotchElevationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralnotchelevationsynthesizer"), Some("BinauralSpectralNotchElevationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralelevationsynthesizer"), Some("BinauralSpectralNotchElevationSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicsphericalharmonicrotatormatrix"), Some("AmbisonicSphericalHarmonicRotatorMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("sphericalharmonicrotatormatrix"), Some("AmbisonicSphericalHarmonicRotatorMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicharmonicrotator"), Some("AmbisonicSphericalHarmonicRotatorMatrix"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearchuacircuitattractorsynthesizer"), Some("NonlinearChuaCircuitAttractorSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chuacircuitattractorsynthesizer"), Some("NonlinearChuaCircuitAttractorSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chuacircuitsynth"), Some("NonlinearChuaCircuitAttractorSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additivespectralformantharmonicoscillator"), Some("AdditiveSpectralFormantHarmonicOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralformantharmonicoscillator"), Some("AdditiveSpectralFormantHarmonicOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additiveformantoscillator"), Some("AdditiveSpectralFormantHarmonicOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringmidsidemultibandexpander"), Some("MasteringMidSideMultibandExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("midsidemultibandexpander"), Some("MasteringMidSideMultibandExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmidsideexpander"), Some("MasteringMidSideMultibandExpander"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticmultistageclippinglimiter"), Some("PsychoacousticMultiStageClippingLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multistageclippinglimiter"), Some("PsychoacousticMultiStageClippingLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacousticclippinglimiter"), Some("PsychoacousticMultiStageClippingLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vintagegermaniumtapedelayecho"), Some("VintageGermaniumTapeDelayEcho"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumtapedelayecho"), Some("VintageGermaniumTapeDelayEcho"));
    assert_eq!(DspNodeRegistry::normalize_type_name("germaniumtapeecho"), Some("VintageGermaniumTapeDelayEcho"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ellipsoidalwhisperinggallerygeometricreverb"), Some("EllipsoidalWhisperingGalleryGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("whisperinggallerygeometricreverb"), Some("EllipsoidalWhisperingGalleryGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("whisperinggalleryreverb"), Some("EllipsoidalWhisperingGalleryGeometricReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearkorg35statevariable24dbfilter"), Some("NonlinearKorg35StateVariable24dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("korg35statevariable24dbfilter"), Some("NonlinearKorg35StateVariable24dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearkorg35filter"), Some("NonlinearKorg35StateVariable24dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralmultinotchphasecombfilter"), Some("SpectralMultiNotchPhaseCombFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multinotchphasecombfilter"), Some("SpectralMultiNotchPhaseCombFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multinotchphasecomb"), Some("SpectralMultiNotchPhaseCombFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticrosslerattractorlfomodulator"), Some("ChaoticRosslerAttractorLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rosslerattractorlfomodulator"), Some("ChaoticRosslerAttractorLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rosslerattractorlfo"), Some("ChaoticRosslerAttractorLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelphasealignmentdelayutility"), Some("MultichannelPhaseAlignmentDelayUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasealignmentdelayutility"), Some("MultichannelPhaseAlignmentDelayUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelphasealigner"), Some("MultichannelPhaseAlignmentDelayUtility"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringlufsloudnesstargetnormalizer"), Some("MasteringLufsLoudnessTargetNormalizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lufsloudnesstargetnormalizer"), Some("MasteringLufsLoudnessTargetNormalizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("lufstargetnormalizer"), Some("MasteringLufsLoudnessTargetNormalizer"));
}

#[test]
fn test_tier162_inventory_count_and_parameter_completeness() {
    let registry = DspNodeRegistry::new();
    let inv = DspNodeRegistry::inventory();
    assert!(inv.len() >= 2012, "Inventory must contain at least 2012 DSP modules, found {}", inv.len());
    assert!(registry.list_all().len() >= 2012, "Registry list_all must be >= 2012, found {}", registry.list_all().len());

    let tier162_nodes = [
        ("AcousticRenaissanceCornettIvoryMouthpiece", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticJapaneseBiwaSilkStringPlectrum", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticBalineseGamelanJegoganBronzeBar", DspNodeCategory::AcousticPhysicalModel),
        ("AcousticCelticBodhranGoatskinTipper", DspNodeCategory::AcousticPhysicalModel),
        ("NeuralAutoregressivePredictiveSampleSynthesizer", DspNodeCategory::NeuralAi),
        ("NeuralSpectralCrossSynthesisVocoder", DspNodeCategory::NeuralAi),
        ("DiscreteSchottkyDiodeWavefolderOverdrive", DspNodeCategory::DistortionSaturation),
        ("AnalogTriodePentodeHybridTubePreamplifier", DspNodeCategory::DistortionSaturation),
        ("BinauralConchaReflectionPinnaFilter", DspNodeCategory::SpatialSurround),
        ("AmbisonicThirdOrderPlanarSoundfieldDecoder", DspNodeCategory::SpatialSurround),
        ("NonlinearRabinovichFabrikantChaosSynthesizer", DspNodeCategory::Oscillator),
        ("AdditiveHarmonicSpectralMorphOscillator", DspNodeCategory::Oscillator),
        ("MultibandMasteringDiodeBridgeLimiter", DspNodeCategory::DynamicsMaster),
        ("DynamicPsychoacousticTransientExciter", DspNodeCategory::DynamicsMaster),
        ("VintageCapstanFlywheelTapeEchoMachine", DspNodeCategory::TimeSpace),
        ("HyperbolicTorusGeometricAcousticReverb", DspNodeCategory::TimeSpace),
        ("NonlinearSteinerParkerSynth12dbFilter", DspNodeCategory::FilterEq),
        ("MorphingSpectralFormantResonatorBank", DspNodeCategory::FilterEq),
        ("ChaoticDoubleWellDuffingLfoModulator", DspNodeCategory::Modulation),
        ("MultichannelGoniometerPhaseVectorscope", DspNodeCategory::Utility),
        ("MasteringDynamicStereoWidthImager", DspNodeCategory::Utility),
    ];

    for (name, expected_cat) in tier162_nodes {
        let desc = registry.get(name).unwrap_or_else(|| panic!("Module {} missing from registry", name));
        assert_eq!(desc.category, expected_cat, "Category mismatch for {}", name);
        assert!(desc.params.len() >= 5, "Module {} must have >= 5 params, found {}", name, desc.params.len());
        assert!(DspNodeRegistry::create_node_ui(name).is_some(), "create_node_ui failed for {}", name);
    }

    // Test normalizations
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticrenaissancecornettivorymouthpiece"), Some("AcousticRenaissanceCornettIvoryMouthpiece"));
    assert_eq!(DspNodeRegistry::normalize_type_name("renaissancecornettivorymouthpiece"), Some("AcousticRenaissanceCornettIvoryMouthpiece"));
    assert_eq!(DspNodeRegistry::normalize_type_name("renaissancecornett"), Some("AcousticRenaissanceCornettIvoryMouthpiece"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticjapanesebiwasilkstringplectrum"), Some("AcousticJapaneseBiwaSilkStringPlectrum"));
    assert_eq!(DspNodeRegistry::normalize_type_name("japanesebiwasilkstringplectrum"), Some("AcousticJapaneseBiwaSilkStringPlectrum"));
    assert_eq!(DspNodeRegistry::normalize_type_name("biwasilkstringplectrum"), Some("AcousticJapaneseBiwaSilkStringPlectrum"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticbalinesegamelanjegoganbronzebar"), Some("AcousticBalineseGamelanJegoganBronzeBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("balinesegamelanjegoganbronzebar"), Some("AcousticBalineseGamelanJegoganBronzeBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("gamelanjegoganbronzebar"), Some("AcousticBalineseGamelanJegoganBronzeBar"));
    assert_eq!(DspNodeRegistry::normalize_type_name("acousticcelticbodhrangoatskintipper"), Some("AcousticCelticBodhranGoatskinTipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("celticbodhrangoatskintipper"), Some("AcousticCelticBodhranGoatskinTipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("bodhrangoatskintipper"), Some("AcousticCelticBodhranGoatskinTipper"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralautoregressivepredictivesamplesynthesizer"), Some("NeuralAutoregressivePredictiveSampleSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("autoregressivepredictivesamplesynthesizer"), Some("NeuralAutoregressivePredictiveSampleSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralautoregressivesynth"), Some("NeuralAutoregressivePredictiveSampleSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralspectralcrosssynthesisvocoder"), Some("NeuralSpectralCrossSynthesisVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralcrosssynthesisvocoder"), Some("NeuralSpectralCrossSynthesisVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("neuralcrosssynthesisvocoder"), Some("NeuralSpectralCrossSynthesisVocoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("discreteschottkydiodewavefolderoverdrive"), Some("DiscreteSchottkyDiodeWavefolderOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("schottkydiodewavefolderoverdrive"), Some("DiscreteSchottkyDiodeWavefolderOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("schottkydiodewavefolder"), Some("DiscreteSchottkyDiodeWavefolderOverdrive"));
    assert_eq!(DspNodeRegistry::normalize_type_name("analogtriodepentodehybridtubepreamplifier"), Some("AnalogTriodePentodeHybridTubePreamplifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodepentodehybridtubepreamplifier"), Some("AnalogTriodePentodeHybridTubePreamplifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("triodepentodepreamp"), Some("AnalogTriodePentodeHybridTubePreamplifier"));
    assert_eq!(DspNodeRegistry::normalize_type_name("binauralconchareflectionpinnafilter"), Some("BinauralConchaReflectionPinnaFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("conchareflectionpinnafilter"), Some("BinauralConchaReflectionPinnaFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("conchapinnafilter"), Some("BinauralConchaReflectionPinnaFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("ambisonicthirdorderplanarsoundfielddecoder"), Some("AmbisonicThirdOrderPlanarSoundfieldDecoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("thirdorderplanarsoundfielddecoder"), Some("AmbisonicThirdOrderPlanarSoundfieldDecoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("planarsoundfielddecoder"), Some("AmbisonicThirdOrderPlanarSoundfieldDecoder"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearrabinovichfabrikantchaossynthesizer"), Some("NonlinearRabinovichFabrikantChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rabinovichfabrikantchaossynthesizer"), Some("NonlinearRabinovichFabrikantChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("rabinovichfabrikantsynth"), Some("NonlinearRabinovichFabrikantChaosSynthesizer"));
    assert_eq!(DspNodeRegistry::normalize_type_name("additiveharmonicspectralmorphoscillator"), Some("AdditiveHarmonicSpectralMorphOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicspectralmorphoscillator"), Some("AdditiveHarmonicSpectralMorphOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("harmonicspectralmorphosc"), Some("AdditiveHarmonicSpectralMorphOscillator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multibandmasteringdiodebridgelimiter"), Some("MultibandMasteringDiodeBridgeLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringdiodebridgelimiter"), Some("MultibandMasteringDiodeBridgeLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("diodebridgelimiter"), Some("MultibandMasteringDiodeBridgeLimiter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicpsychoacoustictransientexciter"), Some("DynamicPsychoacousticTransientExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("psychoacoustictransientexciter"), Some("DynamicPsychoacousticTransientExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamictransientexciter"), Some("DynamicPsychoacousticTransientExciter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("vintagecapstanflywheeltapeechomachine"), Some("VintageCapstanFlywheelTapeEchoMachine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("capstanflywheeltapeechomachine"), Some("VintageCapstanFlywheelTapeEchoMachine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("capstantapeecho"), Some("VintageCapstanFlywheelTapeEchoMachine"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolictorusgeometricacousticreverb"), Some("HyperbolicTorusGeometricAcousticReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("torusgeometricacousticreverb"), Some("HyperbolicTorusGeometricAcousticReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("hyperbolictorusreverb"), Some("HyperbolicTorusGeometricAcousticReverb"));
    assert_eq!(DspNodeRegistry::normalize_type_name("nonlinearsteinerparkersynth12dbfilter"), Some("NonlinearSteinerParkerSynth12dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("steinerparkersynth12dbfilter"), Some("NonlinearSteinerParkerSynth12dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("steinerparkerfilter"), Some("NonlinearSteinerParkerSynth12dbFilter"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingspectralformantresonatorbank"), Some("MorphingSpectralFormantResonatorBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("spectralformantresonatorbank"), Some("MorphingSpectralFormantResonatorBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("morphingformantresonator"), Some("MorphingSpectralFormantResonatorBank"));
    assert_eq!(DspNodeRegistry::normalize_type_name("chaoticdoublewellduffinglfomodulator"), Some("ChaoticDoubleWellDuffingLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("doublewellduffinglfomodulator"), Some("ChaoticDoubleWellDuffingLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("duffinglfomodulator"), Some("ChaoticDoubleWellDuffingLfoModulator"));
    assert_eq!(DspNodeRegistry::normalize_type_name("multichannelgoniometerphasevectorscope"), Some("MultichannelGoniometerPhaseVectorscope"));
    assert_eq!(DspNodeRegistry::normalize_type_name("goniometerphasevectorscope"), Some("MultichannelGoniometerPhaseVectorscope"));
    assert_eq!(DspNodeRegistry::normalize_type_name("phasevectorscopeutility"), Some("MultichannelGoniometerPhaseVectorscope"));
    assert_eq!(DspNodeRegistry::normalize_type_name("masteringdynamicstereowidthimager"), Some("MasteringDynamicStereoWidthImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("dynamicstereowidthimager"), Some("MasteringDynamicStereoWidthImager"));
    assert_eq!(DspNodeRegistry::normalize_type_name("stereowidthimagerutility"), Some("MasteringDynamicStereoWidthImager"));
}

#[test]
#[cfg(feature = "gui")]
fn test_dsp_node_ui_rendering_headless_context() {
    let registry = DspNodeRegistry::new();
    let ctx = egui::Context::default();
    let param_bus = summoner_core::param_bus::ParamBus::new();

    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for descriptor in registry.list_all() {
                let mut params = HashMap::new();
                let mut is_bypassed = false;
                let mut is_collapsed = false;

                descriptor.render_macro_strip(ui, &mut params, Some(&param_bus), 1, 0);
                descriptor.render_pro_inspector(ui, &mut params, Some(&param_bus), 1, 0);
                descriptor.render_rack_unit(
                    ui,
                    &mut params,
                    &mut is_bypassed,
                    &mut is_collapsed,
                    Some(&param_bus),
                    1,
                    0,
                );
            }
        });
    });
}
