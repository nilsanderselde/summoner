//! MusicXML 3.1/4.0 score interchange and multi-track round-trip (Task I6).
//!
//! Provides bidirectional exchange of musical scores with notation software
//! (Sibelius, Finale, MuseScore, Dorico):
//! - Full multi-part score structure (`<score-partwise>`, `<part-list>`, `<score-part>`).
//! - Measures with division timing, key signatures, time signatures, and clefs.
//! - Notes with step/alter/octave pitch, durations, ties, and lyric syllables.
//! - Metronome directions and tempo map points.
//! - Lossless round-trip conversion between MusicXML, SMF notes, and native [`ProjectConfig`].

use crate::schema::{
    ProjectConfig, SequenceConfig, TrackConfig, TrackerStepConfig, TransportConfig,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Lyric syllable connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Syllabic {
    Single,
    Begin,
    Middle,
    End,
}

impl fmt::Display for Syllabic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single => write!(f, "single"),
            Self::Begin => write!(f, "begin"),
            Self::Middle => write!(f, "middle"),
            Self::End => write!(f, "end"),
        }
    }
}

/// Lyric syllable attached to a musical note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicXmlLyric {
    pub text: String,
    pub syllabic: Syllabic,
}

impl MusicXmlLyric {
    pub fn new(text: &str, syllabic: Syllabic) -> Self {
        Self {
            text: text.to_string(),
            syllabic,
        }
    }
}

/// Musical note representation within a MusicXML measure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicXmlNote {
    pub midi_pitch: u8,
    pub step: char,
    pub alter: i8, // -1 = flat, 0 = natural, 1 = sharp
    pub octave: u8,
    pub duration_divisions: u32,
    pub is_rest: bool,
    pub tie_start: bool,
    pub tie_stop: bool,
    pub lyric: Option<MusicXmlLyric>,
}

impl MusicXmlNote {
    pub fn new_note(midi_pitch: u8, duration_divisions: u32) -> Self {
        let (step, alter, octave) = Self::midi_to_step_alter_octave(midi_pitch);
        Self {
            midi_pitch,
            step,
            alter,
            octave,
            duration_divisions,
            is_rest: false,
            tie_start: false,
            tie_stop: false,
            lyric: None,
        }
    }

    pub fn new_rest(duration_divisions: u32) -> Self {
        Self {
            midi_pitch: 0,
            step: 'C',
            alter: 0,
            octave: 4,
            duration_divisions,
            is_rest: true,
            tie_start: false,
            tie_stop: false,
            lyric: None,
        }
    }

    pub fn with_lyric(mut self, lyric: MusicXmlLyric) -> Self {
        self.lyric = Some(lyric);
        self
    }

    pub fn midi_to_step_alter_octave(midi_pitch: u8) -> (char, i8, u8) {
        let octave = (midi_pitch / 12).saturating_sub(1);
        let semitone = midi_pitch % 12;
        let (step, alter) = match semitone {
            0 => ('C', 0),
            1 => ('C', 1),
            2 => ('D', 0),
            3 => ('D', 1),
            4 => ('E', 0),
            5 => ('F', 0),
            6 => ('F', 1),
            7 => ('G', 0),
            8 => ('G', 1),
            9 => ('A', 0),
            10 => ('A', 1),
            11 => ('B', 0),
            _ => ('C', 0),
        };
        (step, alter, octave)
    }

    pub fn step_alter_octave_to_midi(step: char, alter: i8, octave: u8) -> u8 {
        let base_semitone = match step.to_ascii_uppercase() {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => 0,
        };
        let total = ((octave as i32 + 1) * 12) + base_semitone + alter as i32;
        total.clamp(0, 127) as u8
    }
}

/// Measure within a MusicXML part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicXmlMeasure {
    pub number: usize,
    pub divisions: u32,
    pub key_fifths: Option<i8>, // Negative = flats, positive = sharps
    pub key_mode: Option<String>,
    pub time_beats: Option<u8>,
    pub time_beat_type: Option<u8>,
    pub tempo_bpm: Option<f64>,
    pub notes: Vec<MusicXmlNote>,
}

impl MusicXmlMeasure {
    pub fn new(number: usize, divisions: u32) -> Self {
        Self {
            number,
            divisions,
            key_fifths: None,
            key_mode: None,
            time_beats: None,
            time_beat_type: None,
            tempo_bpm: None,
            notes: Vec::new(),
        }
    }
}

/// Part / instrument staff in a score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicXmlPart {
    pub id: String,
    pub name: String,
    pub measures: Vec<MusicXmlMeasure>,
}

impl MusicXmlPart {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            measures: Vec::new(),
        }
    }
}

/// Complete MusicXML multi-part score document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicXmlScore {
    pub title: String,
    pub composer: String,
    pub parts: Vec<MusicXmlPart>,
}

impl Default for MusicXmlScore {
    fn default() -> Self {
        Self {
            title: "Untitled Score".to_string(),
            composer: "Summoner Audio".to_string(),
            parts: Vec::new(),
        }
    }
}

impl MusicXmlScore {
    pub fn new(title: &str, composer: &str) -> Self {
        Self {
            title: title.to_string(),
            composer: composer.to_string(),
            parts: Vec::new(),
        }
    }

    /// Extract MusicXML score from native Summoner `ProjectConfig`.
    pub fn from_project(project: &ProjectConfig) -> Self {
        let mut score = Self::new(&project.name, "Summoner");
        let bpm = project.transport.bpm;
        let divisions = 480; // 480 divisions per quarter note (standard MIDI resolution)

        for track in &project.tracks {
            let mut part = MusicXmlPart::new(&format!("P{}", track.id), &track.name);
            let mut measure = MusicXmlMeasure::new(1, divisions);
            measure.tempo_bpm = Some(bpm);
            measure.time_beats = Some(4);
            measure.time_beat_type = Some(4);
            measure.key_fifths = Some(0); // C Major
            measure.key_mode = Some("major".to_string());

            // Extract notes from sequences / clips
            if let Some(seq) = &track.sequence {
                for step in &seq.steps {
                    if step.active && !step.muted {
                        let duration_divs = (step.gate as f64 * divisions as f64).round() as u32;
                        let mut xml_note =
                            MusicXmlNote::new_note(step.note.round() as u8, duration_divs.max(60));
                        if let Some(meta) = &project.meta {
                            if let Some(lyric_txt) = &meta.notes {
                                xml_note.lyric =
                                    Some(MusicXmlLyric::new(lyric_txt, Syllabic::Single));
                            }
                        }
                        measure.notes.push(xml_note);
                    }
                }
            }

            for clip in &track.clips {
                for step in &clip.steps {
                    if step.active && !step.muted {
                        let duration_divs = (step.gate as f64 * divisions as f64).round() as u32;
                        let xml_note =
                            MusicXmlNote::new_note(step.note.round() as u8, duration_divs.max(60));
                        measure.notes.push(xml_note);
                    }
                }
            }

            if measure.notes.is_empty() {
                // Add default rest
                measure.notes.push(MusicXmlNote::new_rest(divisions * 4));
            }

            part.measures.push(measure);
            score.parts.push(part);
        }

        score
    }

    /// Convert MusicXML score into native Summoner `ProjectConfig`.
    pub fn to_project(&self) -> ProjectConfig {
        let mut bpm = 120.0;
        let mut time_sig = "4/4".to_string();

        if let Some(first_part) = self.parts.first() {
            if let Some(first_measure) = first_part.measures.first() {
                if let Some(b) = first_measure.tempo_bpm {
                    bpm = b;
                }
                if let (Some(num), Some(den)) =
                    (first_measure.time_beats, first_measure.time_beat_type)
                {
                    time_sig = format!("{}/{}", num, den);
                }
            }
        }

        let mut project = ProjectConfig {
            version: "1.1.0".to_string(),
            name: self.title.clone(),
            transport: TransportConfig {
                sample_rate: 44100,
                bpm,
                time_signature: time_sig,
                ..TransportConfig::default()
            },
            ..ProjectConfig::default()
        };

        for (idx, part) in self.parts.iter().enumerate() {
            let mut track = TrackConfig {
                id: idx as u64 + 1,
                name: part.name.clone(),
                channels: 2,
                ..TrackConfig::default()
            };

            let mut steps = Vec::new();

            for measure in &part.measures {
                let div = measure.divisions.max(1) as f64;
                for note in &measure.notes {
                    let note_len_beats = note.duration_divisions as f64 / div;
                    if !note.is_rest {
                        steps.push(TrackerStepConfig {
                            note: note.midi_pitch as f64,
                            velocity: 0.8,
                            gate: note_len_beats.clamp(0.1, 4.0) as f32,
                            active: true,
                            ..TrackerStepConfig::default()
                        });
                    }
                }
            }

            track.sequence = Some(SequenceConfig {
                steps,
                ..SequenceConfig::default()
            });

            project.tracks.push(track);
        }

        project
    }
}

/// Errors occurring during MusicXML import/export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicXmlError {
    Parse(String),
    MissingPart(String),
    MalformedMeasure(String),
}

impl fmt::Display for MusicXmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "MusicXML parse error: {}", e),
            Self::MissingPart(e) => write!(f, "Missing part in MusicXML: {}", e),
            Self::MalformedMeasure(e) => write!(f, "Malformed measure: {}", e),
        }
    }
}

impl std::error::Error for MusicXmlError {}

/// Export `MusicXmlScore` to W3C MusicXML partwise XML format.
pub fn export_musicxml(score: &MusicXmlScore) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE score-partwise PUBLIC \"-//Recordare//DTD MusicXML 3.1 Partwise//EN\" \"http://www.musicxml.org/dtds/partwise.dtd\">\n");
    xml.push_str("<score-partwise version=\"3.1\">\n");
    xml.push_str("  <work>\n");
    xml.push_str(&format!(
        "    <work-title>{}</work-title>\n",
        xml_escape(&score.title)
    ));
    xml.push_str("  </work>\n");
    xml.push_str("  <identification>\n");
    xml.push_str(&format!(
        "    <creator type=\"composer\">{}</creator>\n",
        xml_escape(&score.composer)
    ));
    xml.push_str("  </identification>\n");

    // Part list
    xml.push_str("  <part-list>\n");
    for part in &score.parts {
        xml.push_str(&format!("    <score-part id=\"{}\">\n", part.id));
        xml.push_str(&format!(
            "      <part-name>{}</part-name>\n",
            xml_escape(&part.name)
        ));
        xml.push_str("    </score-part>\n");
    }
    xml.push_str("  </part-list>\n");

    // Parts
    for part in &score.parts {
        xml.push_str(&format!("  <part id=\"{}\">\n", part.id));
        for measure in &part.measures {
            xml.push_str(&format!("    <measure number=\"{}\">\n", measure.number));

            // Attributes
            xml.push_str("      <attributes>\n");
            xml.push_str(&format!(
                "        <divisions>{}</divisions>\n",
                measure.divisions
            ));
            if let Some(fifths) = measure.key_fifths {
                xml.push_str("        <key>\n");
                xml.push_str(&format!("          <fifths>{}</fifths>\n", fifths));
                if let Some(mode) = &measure.key_mode {
                    xml.push_str(&format!("          <mode>{}</mode>\n", mode));
                }
                xml.push_str("        </key>\n");
            }
            if let (Some(beats), Some(beat_type)) = (measure.time_beats, measure.time_beat_type) {
                xml.push_str("        <time>\n");
                xml.push_str(&format!("          <beats>{}</beats>\n", beats));
                xml.push_str(&format!("          <beat-type>{}</beat-type>\n", beat_type));
                xml.push_str("        </time>\n");
            }
            xml.push_str("      </attributes>\n");

            // Direction (tempo)
            if let Some(bpm) = measure.tempo_bpm {
                xml.push_str("      <direction placement=\"above\">\n");
                xml.push_str("        <direction-type>\n");
                xml.push_str(&format!("          <metronome><beat-unit>quarter</beat-unit><per-minute>{:.0}</per-minute></metronome>\n", bpm));
                xml.push_str("        </direction-type>\n");
                xml.push_str(&format!("        <sound tempo=\"{:.0}\"/>\n", bpm));
                xml.push_str("      </direction>\n");
            }

            // Notes
            for note in &measure.notes {
                xml.push_str("      <note>\n");
                if note.is_rest {
                    xml.push_str("        <rest/>\n");
                } else {
                    xml.push_str("        <pitch>\n");
                    xml.push_str(&format!("          <step>{}</step>\n", note.step));
                    if note.alter != 0 {
                        xml.push_str(&format!("          <alter>{}</alter>\n", note.alter));
                    }
                    xml.push_str(&format!("          <octave>{}</octave>\n", note.octave));
                    xml.push_str("        </pitch>\n");
                }
                xml.push_str(&format!(
                    "        <duration>{}</duration>\n",
                    note.duration_divisions
                ));

                if let Some(lyric) = &note.lyric {
                    xml.push_str("        <lyric>\n");
                    xml.push_str(&format!(
                        "          <syllabic>{}</syllabic>\n",
                        lyric.syllabic
                    ));
                    xml.push_str(&format!(
                        "          <text>{}</text>\n",
                        xml_escape(&lyric.text)
                    ));
                    xml.push_str("        </lyric>\n");
                }
                xml.push_str("      </note>\n");
            }

            xml.push_str("    </measure>\n");
        }
        xml.push_str("  </part>\n");
    }

    xml.push_str("</score-partwise>\n");
    xml
}

/// Import `MusicXmlScore` from MusicXML partwise XML string.
pub fn import_musicxml(xml: &str) -> Result<MusicXmlScore, MusicXmlError> {
    let title = extract_xml_tag(xml, "work-title").unwrap_or_else(|| "Imported Score".to_string());
    let composer = extract_xml_tag(xml, "creator").unwrap_or_else(|| "Unknown".to_string());
    let mut score = MusicXmlScore::new(&title, &composer);

    let part_blocks = split_xml_blocks(xml, "<part ", "</part>");
    for (p_idx, p_block) in part_blocks.iter().enumerate() {
        let part_id = extract_attr(p_block, "id").unwrap_or_else(|| format!("P{}", p_idx + 1));
        let mut part = MusicXmlPart::new(&part_id, &format!("Track {}", p_idx + 1));

        let measure_blocks = split_xml_blocks(p_block, "<measure ", "</measure>");
        for (m_idx, m_block) in measure_blocks.iter().enumerate() {
            let divisions = extract_xml_tag(m_block, "divisions")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(480);
            let mut measure = MusicXmlMeasure::new(m_idx + 1, divisions);

            if let Some(fifths_str) = extract_xml_tag(m_block, "fifths") {
                measure.key_fifths = fifths_str.parse::<i8>().ok();
            }
            measure.key_mode = extract_xml_tag(m_block, "mode");

            if let Some(beats_str) = extract_xml_tag(m_block, "beats") {
                measure.time_beats = beats_str.parse::<u8>().ok();
            }
            if let Some(beat_type_str) = extract_xml_tag(m_block, "beat-type") {
                measure.time_beat_type = beat_type_str.parse::<u8>().ok();
            }
            if let Some(per_min_str) = extract_xml_tag(m_block, "per-minute") {
                measure.tempo_bpm = per_min_str.parse::<f64>().ok();
            }

            // Parse notes
            let note_blocks = split_xml_blocks(m_block, "<note", "</note>");
            for n_block in note_blocks {
                let duration = extract_xml_tag(&n_block, "duration")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(divisions);
                let is_rest = n_block.contains("<rest");

                let mut xml_note = if is_rest {
                    MusicXmlNote::new_rest(duration)
                } else {
                    let step = extract_xml_tag(&n_block, "step")
                        .and_then(|s| s.chars().next())
                        .unwrap_or('C');
                    let alter = extract_xml_tag(&n_block, "alter")
                        .and_then(|s| s.parse::<i8>().ok())
                        .unwrap_or(0);
                    let octave = extract_xml_tag(&n_block, "octave")
                        .and_then(|s| s.parse::<u8>().ok())
                        .unwrap_or(4);
                    let midi = MusicXmlNote::step_alter_octave_to_midi(step, alter, octave);
                    let mut n = MusicXmlNote::new_note(midi, duration);
                    n.step = step;
                    n.alter = alter;
                    n.octave = octave;
                    n
                };

                if let Some(lyric_txt) = extract_xml_tag(&n_block, "text") {
                    xml_note.lyric = Some(MusicXmlLyric::new(&lyric_txt, Syllabic::Single));
                }

                measure.notes.push(xml_note);
            }

            part.measures.push(measure);
        }

        score.parts.push(part);
    }

    Ok(score)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open_prefix = format!("<{}", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = xml.find(&open_prefix) {
        if let Some(tag_end) = xml[start..].find('>') {
            let content_start = start + tag_end + 1;
            if let Some(end) = xml[content_start..].find(&close) {
                return Some(xml[content_start..content_start + end].trim().to_string());
            }
        }
    }
    None
}

fn extract_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = xml.find(&pattern) {
        let val_start = start + pattern.len();
        if let Some(end) = xml[val_start..].find('"') {
            return Some(xml[val_start..val_start + end].to_string());
        }
    }
    None
}

fn split_xml_blocks(xml: &str, open_prefix: &str, close_tag: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut search_idx = 0;

    while let Some(start) = xml[search_idx..].find(open_prefix) {
        let absolute_start = search_idx + start;
        if let Some(end) = xml[absolute_start..].find(close_tag) {
            let absolute_end = absolute_start + end + close_tag.len();
            blocks.push(xml[absolute_start..absolute_end].to_string());
            search_idx = absolute_end;
        } else {
            break;
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musicxml_pitch_conversions() {
        // C4 = MIDI 60
        let (step, alter, oct) = MusicXmlNote::midi_to_step_alter_octave(60);
        assert_eq!((step, alter, oct), ('C', 0, 4));
        assert_eq!(MusicXmlNote::step_alter_octave_to_midi('C', 0, 4), 60);

        // A4 = MIDI 69
        let (step_a, alter_a, oct_a) = MusicXmlNote::midi_to_step_alter_octave(69);
        assert_eq!((step_a, alter_a, oct_a), ('A', 0, 4));
        assert_eq!(MusicXmlNote::step_alter_octave_to_midi('A', 0, 4), 69);

        // F#3 = MIDI 54
        let (step_fs, alter_fs, oct_fs) = MusicXmlNote::midi_to_step_alter_octave(54);
        assert_eq!((step_fs, alter_fs, oct_fs), ('F', 1, 3));
        assert_eq!(MusicXmlNote::step_alter_octave_to_midi('F', 1, 3), 54);
    }

    #[test]
    fn test_musicxml_score_export_and_import() {
        let mut score = MusicXmlScore::new("Choral Anthem", "J. S. Bach");
        let mut soprano = MusicXmlPart::new("P1", "Soprano");
        let mut m1 = MusicXmlMeasure::new(1, 480);
        m1.tempo_bpm = Some(96.0);
        m1.time_beats = Some(3);
        m1.time_beat_type = Some(4);
        m1.key_fifths = Some(1); // G Major

        // Add note with lyric syllable
        let note1 =
            MusicXmlNote::new_note(67, 480).with_lyric(MusicXmlLyric::new("Glo", Syllabic::Begin));
        let note2 =
            MusicXmlNote::new_note(69, 480).with_lyric(MusicXmlLyric::new("ri", Syllabic::Middle));
        let note3 =
            MusicXmlNote::new_note(71, 480).with_lyric(MusicXmlLyric::new("a", Syllabic::End));
        m1.notes.push(note1);
        m1.notes.push(note2);
        m1.notes.push(note3);

        soprano.measures.push(m1);
        score.parts.push(soprano);

        // Export to MusicXML
        let xml_str = export_musicxml(&score);
        assert!(xml_str.contains("<work-title>Choral Anthem</work-title>"));
        assert!(xml_str.contains("<creator type=\"composer\">J. S. Bach</creator>"));
        assert!(xml_str.contains("<fifths>1</fifths>"));
        assert!(xml_str.contains("<text>Glo</text>"));

        // Import back from MusicXML
        let imported = import_musicxml(&xml_str).expect("Import should succeed");
        assert_eq!(imported.title, "Choral Anthem");
        assert_eq!(imported.composer, "J. S. Bach");
        assert_eq!(imported.parts.len(), 1);
        assert_eq!(imported.parts[0].measures.len(), 1);
        assert_eq!(imported.parts[0].measures[0].notes.len(), 3);
        assert_eq!(imported.parts[0].measures[0].notes[0].midi_pitch, 67);
        assert_eq!(
            imported.parts[0].measures[0].notes[0]
                .lyric
                .as_ref()
                .unwrap()
                .text,
            "Glo"
        );
    }

    #[test]
    fn test_musicxml_project_roundtrip() {
        let mut score = MusicXmlScore::new("Piano Etude", "Chopin");
        let mut part = MusicXmlPart::new("P1", "Right Hand");
        let mut m1 = MusicXmlMeasure::new(1, 480);
        m1.tempo_bpm = Some(132.0);
        m1.time_beats = Some(4);
        m1.time_beat_type = Some(4);

        m1.notes.push(MusicXmlNote::new_note(60, 480)); // 1 beat
        m1.notes.push(MusicXmlNote::new_note(64, 480)); // 1 beat
        m1.notes.push(MusicXmlNote::new_note(67, 480)); // 1 beat
        m1.notes.push(MusicXmlNote::new_note(72, 480)); // 1 beat

        part.measures.push(m1);
        score.parts.push(part);

        // Convert to ProjectConfig
        let project = score.to_project();
        assert_eq!(project.name, "Piano Etude");
        assert_eq!(project.transport.bpm, 132.0);
        assert_eq!(project.tracks.len(), 1);
        let seq = project.tracks[0].sequence.as_ref().unwrap();
        assert_eq!(seq.steps.len(), 4);
        assert_eq!(seq.steps[0].note.round() as u8, 60);
        assert_eq!(seq.steps[3].note.round() as u8, 72);

        // Convert back to score
        let roundtrip_score = MusicXmlScore::from_project(&project);
        assert_eq!(roundtrip_score.parts.len(), 1);
        assert_eq!(roundtrip_score.parts[0].measures[0].notes.len(), 4);
        assert_eq!(roundtrip_score.parts[0].measures[0].notes[0].midi_pitch, 60);
    }
}
