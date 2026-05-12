use crate::source::SourceSpan;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AcoKind {
    Actor,
    Claim,
    Interest,
    Constraint,
    Leverage,
    Commitment,
    Event,
    Narrative,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalEvent {
    pub id: String,
    pub mention: String,
    pub canonical_name: String,
    pub at: DateTime<Utc>,
    pub fuzziness_secs: u32,
    pub actor_ids: Vec<String>,
    pub event_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_span: Option<SourceSpan>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExtractedActor {
    pub id: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub attrs: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ExtractedCommitment {
    pub id: String,
    pub summary: String,
    pub committer: String,
    pub committee: String,
    pub state: String,
    #[serde(default)]
    pub evidence_spans: Vec<SourceSpan>,
    #[serde(default)]
    pub attrs: serde_json::Value,
}
