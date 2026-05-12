use crate::aco::{ExtractedCommitment, TemporalEvent};
use crate::friction::Friction;
use crate::pre_read::Segment;
use crate::source::SourceSpan;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SourceIndex {
    pub document_id: Option<String>,
    pub spans_by_object: BTreeMap<String, Vec<SourceSpan>>,
    pub segment_ids_by_object: BTreeMap<String, Vec<String>>,
}

impl SourceIndex {
    pub fn build(
        document_id: Option<String>,
        segments: &[Segment],
        events: &[TemporalEvent],
        commitments: &[ExtractedCommitment],
        frictions: &[Friction],
    ) -> Self {
        let mut index = Self {
            document_id,
            ..Default::default()
        };
        for event in events {
            if let Some(span) = &event.source_span {
                index.add(&event.id, span.clone(), segments);
            }
        }
        for commitment in commitments {
            for span in &commitment.evidence_spans {
                index.add(&commitment.id, span.clone(), segments);
            }
        }
        for friction in frictions {
            for span in &friction.evidence_spans {
                index.add(&friction.id, span.clone(), segments);
            }
        }
        index
    }

    fn add(&mut self, object_id: &str, span: SourceSpan, segments: &[Segment]) {
        self.spans_by_object
            .entry(object_id.to_string())
            .or_default()
            .push(span.clone());
        let segment_ids = self
            .segment_ids_by_object
            .entry(object_id.to_string())
            .or_default();
        for segment in segments {
            if span.char_start < segment.char_end && span.char_end > segment.char_start {
                segment_ids.push(segment.id.clone());
            }
        }
        segment_ids.sort();
        segment_ids.dedup();
    }
}
