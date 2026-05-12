pub mod segments;
pub mod signals;

use crate::extract::DateMention;
use crate::pre_read::segments::segment_text;
use crate::pre_read::signals::detect_tension_markers;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Memo,
    Transcript,
    LegalText,
    CrisisBrief,
    NewsReport,
    Dossier,
    Unknown,
}

impl Default for DocumentType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SegmentKind {
    Section,
    Paragraph,
    SpeakerTurn,
    ChronologyBlock,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub kind: SegmentKind,
    pub char_start: usize,
    pub char_end: usize,
    pub heading: Option<String>,
    pub speaker: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TensionMarker {
    pub id: String,
    pub marker_type: String,
    pub phrase: String,
    pub char_start: usize,
    pub char_end: usize,
    pub weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PreReadReport {
    pub document_type: DocumentType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inferred_created_at: Option<DateTime<Utc>>,
    pub segments: Vec<Segment>,
    pub tension_markers: Vec<TensionMarker>,
    pub temporal_anchor_count: usize,
    pub relative_time_count: usize,
    pub speaker_turn_count: usize,
    pub chronology_block_count: usize,
}

pub fn pre_read(
    text: &str,
    dates: &[DateMention],
    document_created_at: Option<DateTime<Utc>>,
) -> PreReadReport {
    let segments = segment_text(text);
    let tension_markers = detect_tension_markers(text);
    let document_type = detect_document_type(text, &segments);
    let inferred_created_at =
        document_created_at.or_else(|| dates.iter().filter_map(|date| date.resolved).min());
    let relative_time_count = dates
        .iter()
        .filter(|date| {
            matches!(
                date.kind,
                crate::extract::DateMentionKind::Relative
                    | crate::extract::DateMentionKind::Duration
            )
        })
        .count();
    let speaker_turn_count = segments
        .iter()
        .filter(|segment| segment.kind == SegmentKind::SpeakerTurn)
        .count();
    let chronology_block_count = segments
        .iter()
        .filter(|segment| segment.kind == SegmentKind::ChronologyBlock)
        .count();

    PreReadReport {
        document_type,
        inferred_created_at,
        segments,
        tension_markers,
        temporal_anchor_count: dates.len(),
        relative_time_count,
        speaker_turn_count,
        chronology_block_count,
    }
}

fn detect_document_type(text: &str, segments: &[Segment]) -> DocumentType {
    let lower = text.to_lowercase();
    if segments
        .iter()
        .filter(|segment| segment.kind == SegmentKind::SpeakerTurn)
        .count()
        >= 3
        || lower.contains("transcript")
    {
        DocumentType::Transcript
    } else if lower.contains("whereas") || lower.contains("section ") || lower.contains("bill") {
        DocumentType::LegalText
    } else if lower.contains("memo") || lower.contains("memorandum") {
        DocumentType::Memo
    } else if lower.contains("crisis") || lower.contains("brief") {
        DocumentType::CrisisBrief
    } else if lower.contains("reported") || lower.contains("according to") {
        DocumentType::NewsReport
    } else if segments
        .iter()
        .filter(|segment| segment.kind == SegmentKind::ChronologyBlock)
        .count()
        >= 8
    {
        DocumentType::Dossier
    } else {
        DocumentType::Unknown
    }
}
