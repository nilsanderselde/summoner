// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
// AGPLv3 License

//! Modern Left Dock & Asset Browser with Categories, Search & Tree Navigation.

#[cfg(feature = "gui")]
use eframe::egui::{self, Color32, FontId, RichText, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BrowserCategory {
    Instruments,
    AudioFx,
    MidiFx,
    #[default]
    Samples,
}

impl BrowserCategory {
    pub fn name(&self) -> &'static str {
        match self {
            BrowserCategory::Instruments => "Instruments",
            BrowserCategory::AudioFx => "Audio FX",
            BrowserCategory::MidiFx => "MIDI FX",
            BrowserCategory::Samples => "Samples",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BrowserCategory::Instruments => "🎹",
            BrowserCategory::AudioFx => "🎛",
            BrowserCategory::MidiFx => "🎼",
            BrowserCategory::Samples => "📁",
        }
    }

    pub fn default_folders(&self) -> &'static [(&'static str, &'static [&'static str])] {
        match self {
            BrowserCategory::Instruments => &[
                (
                    "Physical Models",
                    &[
                        "PercussionMembrane",
                        "StruckIdiophoneResonator",
                        "HurdyGurdySoundboxBody",
                        "SitarSoundboxBody",
                        "BridgeWaveCoupler",
                        "JawariBridge",
                        "TineResonator",
                        "FrictionWheelExciter",
                        "SnareRattleModel",
                        "ChikariDroneBank",
                        "ArmonicaChassisResonator",
                        "WetStickSlipExciter",
                        "YarnDamper",
                        "ClavinetAnvilModel",
                        "PipeOrgan",
                        "WaveguideBrass",
                        "WoodwindJet",
                        "GrandPianoFeltHammer",
                        "SpruceSoundboardMode",
                        "ToneholeJunction",
                        "CassottoBiquad",
                        "ClavinetVoice",
                        "TarabStringResonator",
                        "SitarGourdBody",
                        "PianoStringWaveguide",
                        "GrandPianoStringDuplexCoupler",
                        "BellowsDynamicsCoupler",
                        "BellowsGestureEngine",
                        "BowingGestureEngine",
                        "ClavinetGestureEngine",
                        "EpGestureEngine",
                        "GlassGestureEngine",
                        "GrandPianoGestureEngine",
                        "HurdyGurdyGestureEngine",
                        "KotoGestureEngine",
                        "MalletGestureEngine",
                        "PipeOrganGestureEngine",
                        "PluckGestureEngine",
                        "ShakuhachiGestureEngine",
                        "SitarGestureEngine",
                        "WoodwindFingeringEngine",
                        "ArticulationBus",
                        "BreathBus",
                        "ClavinetBus",
                        "EpBus",
                        "GlassBus",
                        "GrandPianoBus",
                        "HurdyGurdyBus",
                        "KotoBus",
                        "MalletBus",
                        "PipeOrganBus",
                        "PlectrumBus",
                        "ShakuhachiBus",
                        "SitarBus",
                        "KarplusStrongString",
                        "TonebarCoupling",
                        "InductivePickup",
                        "HammerModel",
                        "FreeReedVoice",
                        "ArmonicaChassisMode",
                        "PaulowniaBodyMode",
                        "BowedStringNode",
                        "PercussionMembraneNode",
                        "PipeOrganNode",
                        "CommutedPluckedString",
                        "ShakuhachiVoice",
                        "SitarVoice",
                        "WaveguideBrassNode",
                        "WaveguideMeshNode",
                        "WoodwindJetNode",
                        "ClavinetToneSwitches",
                        "OrganVoice",
                        "BowedString",
                        "Clavinet",
                        "ElectricPiano",
                        "FreeReed",
                        "GlottalPulse",
                        "ConcertGrandPiano",
                        "Shakuhachi",
                        "SitarNode",
                        "VocalTract",
                        "SoundboardMode",
                        "Strummer",
                        "EmbouchureAngle",
                        "FrictionOrbit",
                        "IdiophoneSpectrum",
                        "MembraneCavity",
                        "ToneholeMatrix",
                        "TrompetteBridge",
                        "BellowsChamber",
                        "AirReedConfig",
                        "WindchestReservoir",
                        "StickSlipPhysicsModel",
                        "PianoHammerMechanics",
                        "GlassArmonicaBus",
                        "ResonantBodyFormant",
                        "BellowsWaypoint",
                        "BowingWaypoint",
                        "KotoWaypoint",
                        "GrandPianoWaypoint",
                        "ClavinetWaypoint",
                        "LatticePointMass",
                    ],
                ),
                (
                    "Synthesizers",
                    &[
                        "AetherSynth",
                        "CyberpunkSubSynth",
                        "AtmosphericPadSynth",
                        "PluckSynth",
                        "Acid303",
                        "TonewheelOrgan",
                        "NeuroFeedbackOscillator",
                        "SpectrogramArtEngine",
                        "NeuralWavetableInterpolator",
                        "NamWaveNetEngine",
                        "TonewheelOrganNode",
                        "MolecularVibrationResonatorNode",
                        "PlasmaArcSynthesizerNode",
                        "FusionResonanceSynthNode",
                        "SpectrogramSoundGenerator",
                        "AetherMacroStrip",
                        "SpectrogramArtConfig",
                        "SpectrogramImage",
                        "ImageNoteTrigger",
                        "SpectralMorphConfig",
                        "VoicePool",
                    ],
                ),
                (
                    "Samplers & Slicers",
                    &[
                        "LoopSlicerNode",
                        "AutoSlicer",
                        "GranularSamplerNode",
                        "MultiSampler",
                        "AudioReverse",
                        "DrumReplacementTrigger",
                        "SampleEditor",
                        "MultitrackSessionLooper",
                        "SamplerNode",
                        "MultiSampleBank",
                        "SamplerMacroStrip",
                        "DrumMacroStrip",
                        "SfzPresetPatch",
                        "MultiSamplePresetLoader",
                        "MultiTakeCompManager",
                        "LooperTrackManager",
                    ],
                ),
            ],
            BrowserCategory::AudioFx => &[
                (
                    "Dynamics & Level",
                    &[
                        "TapeSaturation",
                        "Limiter",
                        "Compressor",
                        "SpectralGate",
                        "SpeakerCalibrationMatrix",
                        "OversampledClipper",
                        "TruePeakDetector",
                        "PeakHeadroomAnalyzer",
                        "EbuR128LoudnessMeter",
                        "SidechainMatrix",
                        "TruePeakMeter",
                        "TapeChannelState",
                        "TubeChannelState",
                        "BandDynamics",
                        "MasterLimiter",
                        "MasterBusTrim",
                        "ConsoleChannelState",
                        "SidechainRoute",
                        "NamModelConfig",
                        "ExportPresetManager",
                        "SessionAnalyticsDashboard",
                        "GoldenRenderSuite",
                        "VideoExportConfig",
                        "AiMixBalanceInspector",
                        "AudioPhaseAligner",
                        "AudioProjectNormalizer",
                        "PerceptualQualityVerifier",
                        "TransientShaperMatrix",
                        "DitherNoiseShaper",
                        "SidechainSpectralDucker",
                        "LinearPhaseDeEsser",
                        "StereoGoniometerVisualizer",
                        "MasterSafetyClipper",
                        "NamModel",
                    ],
                ),
                (
                    "Time & Reverb",
                    &[
                        "IsmShockwaveReverb",
                        "PlateReverb",
                        "StereoDelay",
                        "RotarySpeaker",
                        "TapeStop",
                        "PartitionedBinauralHrtfConvolver",
                        "AmbisonicsDecoder3D",
                        "AmbisonicsEncoder3D",
                        "HyperbolicReverbNode",
                        "AcousticCloakingSpatializerNode",
                        "AcousticHologramFilter",
                        "ProceduralSpatialIrGenerator",
                        "SurroundStemSplitterBedObject",
                        "HeadTrackerReceiver",
                        "SpringGestureEngine",
                        "SpatialAutomationEngine",
                        "SpatialBus",
                        "SpringBus",
                        "SpatialObjectAutomation",
                        "ImageReflection",
                        "StereoPanner",
                        "IsmShockwaveReverbNode",
                        "PlateTank",
                        "SpringLattice",
                        "SpatialTrajectoryTrack",
                        "Hoa4Spatializer",
                        "MpeghSpatializer",
                        "ConvolutionImpulse",
                        "ConvolutionMorph",
                        "SpatialIrConfig",
                        "AudioWarpMarker",
                        "PeerMeshNetwork",
                        "OpusAudioRelay",
                        "BinauralHrtfInterpolator",
                        "SurroundDownmixer",
                        "AtmosphericDensity",
                        "AcousticLevitationTrap",
                        "MultichannelAudioBuffer",
                    ],
                ),
                (
                    "Filters & EQ",
                    &[
                        "StateVariableFilter",
                        "MoogLadderFilter",
                        "FormantFilter",
                        "GraphicEq",
                        "FormantVowelMatrix",
                        "AllpassDispersionStage",
                        "SubHarmonicQuantumTunnelingFilter",
                        "CrystalModalFilter",
                        "HurdyGurdyDispersionFilter",
                        "DispersionAllpassChain",
                        "TineDispersionAllpass",
                        "SitarDispersionAllpass",
                        "MetamaterialRefractionFilterNode",
                        "SpectrumMatchingAnalyzer",
                        "Butterworth2ndOrder",
                        "ModalFilterSection",
                        "DispersionAllpass",
                        "ConsoleBiquad",
                        "VowelGraph",
                        "LadderFilterBode",
                        "VowelSpace",
                        "SpectralMorphEngine",
                        "BesselFilterBank",
                        "StateVariableOversampler",
                        "ParametricDynamicEq",
                    ],
                ),
                (
                    "Modulation & Pitch",
                    &[
                        "Chorus",
                        "Phaser",
                        "Flanger",
                        "CrystalResonator",
                        "DemucsV4Separator",
                        "AutoWah",
                        "OpticalTremolo",
                        "GlitchShuffle",
                        "GlitchStutter",
                        "GlitchGate",
                        "MhdPlasmaWaveModulatorNode",
                        "QuantumEntanglementRouter",
                        "LockFreeTuningRemapper",
                        "RotaryGestureEngine",
                        "RotaryBus",
                        "RotarySpeakerNode",
                        "LFO",
                        "MacroKnob",
                        "CadenceEngine",
                        "VoiceLeadingOptimizer",
                        "AutomationTimeline",
                        "GenerativeEngine",
                        "ChordMemory",
                        "TrackerStep",
                        "RotaryDoppler",
                        "BezierAutomation",
                        "ClapAutomationHost",
                        "NeuroAffectiveEngine",
                        "HarmonicTensionFlow",
                        "PitchDriftModulator",
                    ],
                ),
            ],
            BrowserCategory::MidiFx => &[
                (
                    "Generative & Sync",
                    &[
                        "HrvTempoSyncEngine",
                        "AiPolyphonicChordExtractor",
                        "AiSongStructureDetector",
                        "EuclideanSequencer",
                        "Arpeggiator",
                        "AiChordGenerator",
                        "NeuralMidiTranscriber",
                        "DrumTranscriptor",
                        "PolymetricSequencer",
                        "MarkovSequenceMutator",
                        "CadenceBus",
                        "BootToSynthEngine",
                        "AudioTagger",
                        "MidiClockGenerator",
                        "MidiClockReceiver",
                        "BleMidiPeripheral",
                        "SessionMarkerNavigationManager",
                        "LivePerformanceSync",
                        "LuaScriptEngine",
                        "MidiFileParser",
                        "WebRtcMidiStreamer",
                        "SharedAnnotationPanel",
                        "LuaDebugger",
                        "LiveClipMatrix",
                        "PolymetricTrackerSequencer",
                        "GenerativeStochasticEngine",
                        "HarmonicCadenceEngine",
                        "DynamicTempoMapAnalyzer",
                        "CurveThinningOptimizer",
                        "GrooveHumanizeTransformer",
                        "AiHarmonySuggester",
                        "BatchAutomationRunner",
                        "MidiClockPllSynchronizer",
                        "CellularAutomataRhythmGenerator",
                        "NeuralOnnxMelodyGenerator",
                        "DeterministicPatternRandomizer",
                        "PatternDensityFilter",
                        "GrooveQuantizeEngine",
                        "CadencePathResolver",
                        "Rank2TemperamentGenerator",
                        "PtpSyncClock",
                        "EuclideanPolyrhythmGenerator",
                        "MidiPatternHumanizer",
                        "FollowAction",
                        "AutomationRegistry",
                        "LooperTrack",
                        "SequenceTrack",
                        "SequenceEvent",
                        "DrumStepEvent",
                        "TranscribedNote",
                    ],
                ),
                (
                    "Transforms & Routing",
                    &[
                        "AiMixBalanceAnalyzer",
                        "AudioAlignmentTool",
                        "AiMixAssistant",
                        "NeuralIrSynthesizer",
                        "MidiTranspose",
                        "ScaleQuantizer",
                        "ChordGenerator",
                        "MidiFilterEngine",
                        "BellowsBus",
                        "FormantBus",
                        "MidiUsbGadgetMode",
                        "MultiBusEventRouter",
                        "ParameterAutomapper",
                        "MpeRouter",
                        "Vst3WindowEmbedder",
                        "MidiUartSerialDriver",
                        "EepromPresetStore",
                        "WebConfigDashboard",
                        "WasapiDriver",
                        "MidiInputFilter",
                        "KeyboardSplit",
                        "KeyboardLayering",
                        "AlsaDriver",
                        "AudioUnitDriver",
                        "AAudioDriver",
                        "AsapiDriver",
                        "HardwareEmulationHarness",
                        "TrackStereoCorrelationMeter",
                        "StereoWidth",
                        "MemoryEstimator",
                        "CadenceGraph",
                        "HarmonicTensionAnalyzer",
                        "IsomorphicRouter",
                        "EdoTuning",
                        "SclTuning",
                        "KbmMapping",
                        "HarmonicBusBridge",
                        "TimelineArranger",
                        "ClipMatrix",
                        "CompTrack",
                        "CadenceFlow",
                        "ConcordanceLattice",
                        "HarmonicTensionMap",
                        "IsomorphicLattice",
                        "IsomorphicTuningKeyboard",
                        "PhaseAlign",
                        "QuantumTomographyVisualizer",
                        "OscMappingRouter",
                        "MaskingCollisionAnalyzer",
                        "AiMixRecommendationEngine",
                        "SimdVoiceAllocator",
                        "Vst3HostConfig",
                        "PluginSandbox",
                        "PluginLatencyCompensation",
                        "PluginCrashGuard",
                        "AudioGraphBenchmarkSuite",
                        "ProjectAutoSaveManager",
                        "ScratchAudioCache",
                        "WorkspaceDependencyAuditor",
                        "CrashDumpAnalyzer",
                        "DistributedRenderFarm",
                        "CloudPresetHub",
                        "FederatedMarketplace",
                        "ZkPatchVerifier",
                        "ContinuousBackupEngine",
                        "WebSocketSyncTransport",
                        "CursorTracker",
                        "CloudBranchingManager",
                        "CloudAssetSync",
                        "AutomatedCloudBackup",
                        "AccessControlPolicy",
                        "RenderFarmDispatchApi",
                        "E2eeProjectEncryptor",
                        "OfflineChangeQueue",
                        "GitHubActionsBotIntegration",
                        "TemplateMarketplace",
                        "GitSessionDag",
                        "ApiChangelogGenerator",
                        "LuaProfiler",
                        "CompingTakeManager",
                        "MidiPerformanceHub",
                        "MidiStreamAnalyzer",
                        "AutomationTimelineManager",
                        "IsomorphicKeyboardRouter",
                        "VoiceLeadingGraph",
                        "VowelTrajectoryGraph",
                        "MidiClockSyncHub",
                        "MidiMessageFilterMatrix",
                        "DynamicMicrotonalRemapper",
                        "MultiTenantRenderManager",
                        "HardwareEncoderController",
                        "ClapPluginHostBridge",
                        "HardwareSurfaceController",
                        "AnalogSyncTransceiver",
                        "WasmPackageExporter",
                        "AdmSpatialBwfExporter",
                        "RaspberryPiApplianceBuilder",
                        "ProjectSemanticDiffer",
                        "SessionHotReloadWatcher",
                        "ProjectSchemaMigrator",
                        "PresetCachePrewarmer",
                        "SfzMultisampleConverter",
                        "MicrotonalScalaTuner",
                        "InteractiveAudioRepl",
                        "DspScriptTestHarness",
                        "BroadcastFormatTranscoder",
                        "PatchPrGenerator",
                        "MidiVelocityCurveShaper",
                        "MidiControllerMappingMatrix",
                        "VelocityLevelQuantizer",
                        "AssetIntegrityAuditor",
                        "ProjectHealthValidator",
                        "RealtimeCpuProfiler",
                        "CpalAudioDeviceInspector",
                        "DemucsNeuralStemSplitter",
                        "ClapStandaloneExporter",
                        "LuaSecuritySandboxAuditor",
                        "LuaAstLinterFormatter",
                        "LuaBundleOptimizer",
                        "LuaDocumentationGenerator",
                        "SmfMidiStreamExporter",
                        "LiveAudioMonitorEngine",
                        "ProjectScaffoldWizard",
                        "FederatedMixLearner",
                        "PresenceSyncServer",
                        "IpfsAssetStore",
                        "TomlMergeDriver",
                        "SessionChatCrypto",
                        "WasmPluginRunner",
                        "DidAuthenticator",
                        "AdaptiveBandwidthManager",
                        "PolyphonicVoicePool",
                        "MidiMonitorLog",
                        "DynamicParameterSmoother",
                        "NodeGraphScheduler",
                        "VowelCylinderAreaSynthesizer",
                        "PhaseFlipPolarityInverter",
                        "InputOutputGainTrim",
                        "Vst3PluginScanner",
                        "AudioWorkletExporter",
                        "EqualPowerCrossfader",
                        "ConvolutionCabinetSimulator",
                        "ZeroLatencyBufferBridge",
                        "CrdtEngine",
                        "CloudProjectManager",
                        "GraphSchedule",
                        "LuaDspGraphBuilder",
                        "LuaLspServer",
                        "LuaHotReloader",
                        "LuaOscServer",
                        "HarmonicContext",
                        "IsomorphicVoiceState",
                        "FormantTrajectoryPoint",
                        "AudioMeshPacket",
                        "NodeGraph",
                        "Track",
                        "Transport",
                        "WavWriter",
                        "ParamBus",
                        "FixedAudioBuffer",
                        "StemMetadata",
                        "OnnxCpuSimdExecutionProvider",
                        "MixSuggestions",
                    ],
                ),
            ],
            BrowserCategory::Samples => &[
                (
                    "Drums",
                    &[
                        "Drums/Synths.wav",
                        "Kick_Punchy.wav",
                        "Snare_Tight.wav",
                    ],
                ),
                (
                    "Synths",
                    &[
                        "Audit.midi",
                        "Lead_Aether.toml",
                        "Pad_Shimmer.toml",
                    ],
                ),
                (
                    "Bass",
                    &[
                        "Sample..midi",
                        "Sub_808.wav",
                        "Acid_303.toml",
                    ],
                ),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModernAssetBrowserState {
    pub selected_category: BrowserCategory,
    pub search_query: String,
    pub expanded_folders: Vec<String>,
    pub selected_item: Option<String>,
    pub is_collapsed: bool,
}

impl Default for ModernAssetBrowserState {
    fn default() -> Self {
        Self {
            selected_category: BrowserCategory::Samples,
            search_query: String::new(),
            expanded_folders: vec![
                "Drums".to_string(),
                "Synths".to_string(),
                "Bass".to_string(),
                "Physical Models".to_string(),
                "Synthesizers".to_string(),
                "Dynamics & Level".to_string(),
                "Time & Reverb".to_string(),
            ],
            selected_item: Some("Drums/Synths".to_string()),
            is_collapsed: false,
        }
    }
}

#[cfg(feature = "gui")]
pub fn show_modern_asset_browser(
    ui: &mut egui::Ui,
    state: &mut ModernAssetBrowserState,
    on_drag_preset: impl FnMut(&str),
) {
    let mut _on_drag = on_drag_preset;
    let total_height = ui.available_height();

    ui.horizontal(|ui| {
        // Far Left Icon Dock Strip (Width: ~40px)
        let dock_w = 40.0;
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 14, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(24, 34, 52)))
            .show(ui, |ui| {
                ui.set_width(dock_w);
                ui.set_height(total_height);
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    let icons = ["🎛", "🎚", "🎧", "🌊", "🎹", "✨", "💡", "⚙"];
                    for (i, icon) in icons.iter().enumerate() {
                        let is_active = i == 0;
                        let text_col = if is_active {
                            Color32::from_rgb(56, 189, 248)
                        } else {
                            Color32::from_rgb(100, 116, 139)
                        };
                        let fill = if is_active {
                            Color32::from_rgba_unmultiplied(56, 189, 248, 25)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let _btn = ui.add(
                            egui::Button::new(RichText::new(*icon).font(FontId::proportional(14.0)).color(text_col))
                                .fill(fill)
                                .rounding(Rounding::same(4.0))
                                .min_size(Vec2::new(28.0, 28.0))
                        );
                        ui.add_space(4.0);
                    }
                });
            });

        // Main Collapsible Asset Browser Panel (Width: ~220px)
        if !state.is_collapsed {
            let panel_w = 210.0;
            egui::Frame::none()
                .fill(Color32::from_rgb(14, 20, 32))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(8.0, 8.0))
                .show(ui, |ui| {
                    ui.set_width(panel_w);
                    ui.set_height(total_height);

                    // Header
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Asset Browser").font(FontId::proportional(12.0)).strong().color(Color32::from_rgb(241, 245, 249)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("◀").clicked() {
                                state.is_collapsed = true;
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Search Box
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍").font(FontId::proportional(10.0)).color(Color32::from_rgb(148, 163, 184)));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.search_query)
                                .hint_text("Search...")
                                .desired_width(135.0),
                        );
                        if !state.search_query.is_empty() && ui.small_button("✖").clicked() {
                            state.search_query.clear();
                        }
                    });

                    ui.add_space(6.0);

                    // Categories List
                    let categories = [
                        BrowserCategory::Instruments,
                        BrowserCategory::AudioFx,
                        BrowserCategory::MidiFx,
                        BrowserCategory::Samples,
                    ];

                    for cat in categories {
                        let is_sel = state.selected_category == cat;
                        let (bg_fill, border_stroke, text_color) = if is_sel {
                            (
                                Color32::from_rgb(45, 35, 15),
                                Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)),
                                Color32::from_rgb(245, 158, 11),
                            )
                        } else {
                            (
                                Color32::TRANSPARENT,
                                Stroke::NONE,
                                Color32::from_rgb(148, 163, 184),
                            )
                        };

                        let item_frame = egui::Frame::none()
                            .fill(bg_fill)
                            .stroke(border_stroke)
                            .rounding(Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 4.0));

                        item_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(cat.icon()).font(FontId::proportional(12.0)));
                                let resp = ui.selectable_label(is_sel, RichText::new(cat.name()).font(FontId::proportional(11.0)).color(text_color));
                                if resp.clicked() {
                                    state.selected_category = cat;
                                }
                            });
                        });
                        ui.add_space(2.0);
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Folder & Item Tree
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let folders = state.selected_category.default_folders();
                        let query = state.search_query.trim().to_lowercase();

                        for (folder_name, files) in folders {
                            let folder_matches = !query.is_empty() && folder_name.to_lowercase().contains(&query);
                            let matching_files: Vec<&'static str> = files
                                .iter()
                                .copied()
                                .filter(|file| query.is_empty() || folder_matches || file.to_lowercase().contains(&query))
                                .collect();

                            if !query.is_empty() && matching_files.is_empty() {
                                continue;
                            }

                            let is_exp = !query.is_empty() || state.expanded_folders.contains(&folder_name.to_string());
                            let arrow = if is_exp { "▼" } else { "▶" };

                            ui.horizontal(|ui| {
                                if ui.small_button(arrow).clicked() {
                                    if state.expanded_folders.contains(&folder_name.to_string()) {
                                        state.expanded_folders.retain(|f| f != *folder_name);
                                    } else {
                                        state.expanded_folders.push(folder_name.to_string());
                                    }
                                }
                                ui.label(RichText::new(format!("📁 {}", folder_name)).font(FontId::proportional(11.0)).strong().color(Color32::from_rgb(245, 158, 11)));
                            });

                            if is_exp {
                                for file in matching_files {
                                    ui.horizontal(|ui| {
                                        ui.add_space(14.0);
                                        let is_sel = state.selected_item.as_deref() == Some(file);
                                        let file_col = if is_sel { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(200, 215, 235) };
                                        let btn = ui.selectable_label(is_sel, RichText::new(format!("📄 {}", file)).font(FontId::proportional(10.0)).color(file_col));
                                        if btn.clicked() {
                                            state.selected_item = Some(file.to_string());
                                            _on_drag(file);
                                        }
                                    });
                                }
                            }
                            ui.add_space(2.0);
                        }
                    });
                });
        } else {
            // Expand button when collapsed
            if ui.button("▶").clicked() {
                state.is_collapsed = false;
            }
        }
    });
}
