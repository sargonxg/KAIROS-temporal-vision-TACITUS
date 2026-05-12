use crate::pre_read::{Segment, SegmentKind};
use regex::Regex;

pub fn segment_text(text: &str) -> Vec<Segment> {
    let speaker_rx = Regex::new(r"(?m)^([A-Z][A-Za-z .'-]{1,48}):\s+").unwrap();
    let date_rx = Regex::new(
        r"(?i)^\s*((?:January|February|March|April|May|June|July|August|September|October|November|December|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sep|Sept|Oct|Nov|Dec)\s+\d{1,2},?\s+\d{4}|\d{4}-\d{2}-\d{2})\s*:",
    )
    .unwrap();
    let heading_rx = Regex::new(r"(?m)^#{1,4}\s+(.+)$").unwrap();
    let mut segments = Vec::new();
    let mut offset = 0;

    for block in text.split_inclusive("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            offset += block.len();
            continue;
        }
        let leading_ws = block.len() - block.trim_start().len();
        let start = offset + leading_ws;
        let end = start + trimmed.len();
        let speaker = speaker_rx
            .captures(trimmed)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()));
        let heading = heading_rx
            .captures(trimmed)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()));
        let kind = if speaker.is_some() {
            SegmentKind::SpeakerTurn
        } else if date_rx.is_match(trimmed) {
            SegmentKind::ChronologyBlock
        } else if heading.is_some() {
            SegmentKind::Section
        } else {
            SegmentKind::Paragraph
        };
        let id = format!("seg_{:04}", segments.len() + 1);
        segments.push(Segment {
            id,
            kind,
            char_start: start,
            char_end: end,
            heading,
            speaker,
            text: trimmed.to_string(),
        });
        offset += block.len();
    }

    if segments.is_empty() && !text.trim().is_empty() {
        segments.push(Segment {
            id: "seg_0001".to_string(),
            kind: SegmentKind::Paragraph,
            char_start: 0,
            char_end: text.len(),
            heading: None,
            speaker: None,
            text: text.trim().to_string(),
        });
    }

    segments
}
