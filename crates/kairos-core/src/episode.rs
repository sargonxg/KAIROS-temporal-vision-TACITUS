use crate::temporal::TemporalInterval;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeKind {
    Regime,
    Leadership,
    Agreement,
    Sanction,
    Escalation,
    DeEscalation,
    Pivot,
    Crisis,
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeBoundary {
    pub at: DateTime<Utc>,
    pub fuzziness_secs: u32,
    pub triggering_events: Vec<String>,
    pub rationale: String,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    Proposed,
    Approved,
    Modified,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub kind: EpisodeKind,
    pub title: String,
    pub interval: TemporalInterval,
    pub boundary_in: EpisodeBoundary,
    pub boundary_out: Option<EpisodeBoundary>,
    pub anchors: Vec<String>,
    pub narrative: String,
    pub confidence: f32,
    pub review_state: ReviewState,
}

impl Episode {
    pub fn new_id() -> String {
        format!("ep_{}", Uuid::now_v7())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeProposal {
    pub kind: EpisodeKind,
    pub title: String,
    pub boundary_in: EpisodeBoundary,
    pub boundary_out: Option<EpisodeBoundary>,
    pub anchors: Vec<String>,
    pub narrative: String,
    pub detector: String,
    pub confidence: f32,
}
