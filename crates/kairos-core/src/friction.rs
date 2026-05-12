use crate::source::SourceSpan;
use crate::temporal::TemporalInterval;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FrictionKind {
    CommitmentFailure,
    ProceduralObstruction,
    InstitutionalDrift,
    TrustLoss,
    PolicyDivergence,
    LeadershipTransition,
    LegalBlockage,
    ImplementationGap,
    Escalation,
    Custom,
}

impl Default for FrictionKind {
    fn default() -> Self {
        Self::Custom
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrictionTrajectory {
    Escalating,
    Resolving,
    Freezing,
    Mutating,
}

impl Default for FrictionTrajectory {
    fn default() -> Self {
        Self::Mutating
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrictionPolarity {
    Conflict,
    Cooperation,
    Ambiguity,
    Dependency,
    Asymmetry,
}

impl Default for FrictionPolarity {
    fn default() -> Self {
        Self::Conflict
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrictionMechanism {
    Delay,
    Noncompliance,
    LegalBlockage,
    ResourceScarcity,
    LeadershipShift,
    NarrativeSplit,
    ProceduralBlock,
    TrustLoss,
    ImplementationGap,
    Unknown,
}

impl Default for FrictionMechanism {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrictionDirectness {
    Explicit,
    Implied,
    Inferred,
}

impl Default for FrictionDirectness {
    fn default() -> Self {
        Self::Explicit
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceGrade {
    DirectQuote,
    Paraphrase,
    TemporalInference,
    CrossEventInference,
}

impl Default for EvidenceGrade {
    fn default() -> Self {
        Self::Paraphrase
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Friction {
    pub id: String,
    pub kind: FrictionKind,
    pub summary: String,
    #[serde(default)]
    pub actors_involved: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggering_event: Option<String>,
    pub interval: TemporalInterval,
    #[serde(default)]
    pub evidence_spans: Vec<SourceSpan>,
    pub intensity: f32,
    pub confidence: f32,
    #[serde(default)]
    pub polarity: FrictionPolarity,
    #[serde(default)]
    pub mechanism: FrictionMechanism,
    #[serde(default)]
    pub latent: bool,
    #[serde(default)]
    pub directness: FrictionDirectness,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counterparty_effect: Option<String>,
    #[serde(default)]
    pub evidence_grade: EvidenceGrade,
    #[serde(default)]
    pub competing_hypotheses: Vec<String>,
    #[serde(default)]
    pub commitment_ids: Vec<String>,
    #[serde(default)]
    pub episode_ids: Vec<String>,
    pub trajectory: FrictionTrajectory,
    #[serde(default)]
    pub attrs: BTreeMap<String, String>,
}

impl Friction {
    pub fn new_id() -> String {
        format!("fr_{}", Uuid::now_v7())
    }

    pub fn normalize(mut self) -> Option<Self> {
        if self.summary.trim().is_empty() {
            return None;
        }
        self.intensity = self.intensity.clamp(0.0, 1.0);
        self.confidence = self.confidence.clamp(0.0, 1.0);
        self.actors_involved
            .retain(|actor| !actor.trim().is_empty());
        self.commitment_ids.retain(|id| !id.trim().is_empty());
        self.episode_ids.retain(|id| !id.trim().is_empty());
        self.competing_hypotheses
            .retain(|hypothesis| !hypothesis.trim().is_empty());
        Some(self)
    }
}
