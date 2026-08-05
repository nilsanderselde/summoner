// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// 3D Spatial Panner Visualizer & VR/AR HMD Companion View (Steps 1064, 1074).

use summoner_core::audio::ChannelLayout;
use summoner_dsp::spatial_audio::{HeadTrackerReceiver, Position3D};

/// 3D Spatial Panner GUI View (Step 1064).
#[derive(Debug, Clone)]
pub struct SpatialPannerView {
    pub layout: ChannelLayout,
    pub listener_pos: Position3D,
    pub head_tracker: HeadTrackerReceiver,
    pub sources: Vec<(String, Position3D)>,
    pub grid_bounds: (f32, f32, f32), // Width, Depth, Height
    pub is_hmd_active: bool,          // Step 1074 VR/AR HMD companion view active flag
}

impl SpatialPannerView {
    pub fn new(layout: ChannelLayout) -> Self {
        Self {
            layout,
            listener_pos: Position3D::zero(),
            head_tracker: HeadTrackerReceiver::new(),
            sources: vec![
                ("Vocals".into(), Position3D::new(0.0, 1.5, 0.2)),
                ("Guitar".into(), Position3D::new(-1.2, 2.0, 0.0)),
                ("Synth".into(), Position3D::new(1.2, 2.0, 0.0)),
            ],
            grid_bounds: (10.0, 10.0, 4.0),
            is_hmd_active: false,
        }
    }

    pub fn add_source(&mut self, name: impl Into<String>, pos: Position3D) {
        self.sources.push((name.into(), pos));
    }

    pub fn set_hmd_active(&mut self, active: bool) {
        self.is_hmd_active = active;
    }

    /// Render ASCII/CLI representation of 3D spatial room.
    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "[3D Spatial Panner View - Layout: {:?}]\n",
            self.layout
        ));
        out.push_str(&format!(
            "Listener (Head Tracker): Yaw {:.1} deg | Pitch {:.1} deg | Roll {:.1} deg\n",
            self.head_tracker.yaw_deg, self.head_tracker.pitch_deg, self.head_tracker.roll_deg
        ));
        out.push_str(&format!(
            "HMD Companion Mode: {}\n",
            if self.is_hmd_active {
                "ACTIVE (OpenXR)"
            } else {
                "OFF"
            }
        ));
        out.push_str("Sources:\n");
        for (name, pos) in &self.sources {
            out.push_str(&format!(
                " - {:<12}: X={:+.2}m, Y={:+.2}m, Z={:+.2}m (Az: {:.1}deg, Dist: {:.2}m)\n",
                name,
                pos.x,
                pos.y,
                pos.z,
                pos.azimuth() * 57.2958,
                pos.distance()
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_panner_view_ascii_render() {
        let mut view = SpatialPannerView::new(ChannelLayout::Surround7_1_4);
        view.head_tracker.yaw_deg = 15.0;
        view.set_hmd_active(true);
        let ascii = view.render_ascii();
        assert!(ascii.contains("7_1_4"));
        assert!(ascii.contains("ACTIVE (OpenXR)"));
        assert!(ascii.contains("Vocals"));
    }
}
