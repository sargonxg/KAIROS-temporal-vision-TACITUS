use crate::friction::{Friction, FrictionKind, FrictionTrajectory};
use crate::source::SourceSpan;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    Hypothesized,
    Supported,
    Disputed,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrictionHypothesis {
    pub id: String,
    pub hypothesis: String,
    pub supporting_friction_ids: Vec<String>,
    pub contradicting_evidence: Vec<SourceSpan>,
    pub likelihood: f32,
    pub confidence: f32,
    pub status: HypothesisStatus,
}

pub fn infer_hypotheses(frictions: &[Friction]) -> Vec<FrictionHypothesis> {
    let mut out = Vec::new();
    let ids_for = |kind: FrictionKind| {
        frictions
            .iter()
            .filter(|friction| friction.kind == kind)
            .map(|friction| friction.id.clone())
            .collect::<Vec<_>>()
    };

    let implementation = ids_for(FrictionKind::ImplementationGap);
    let commitments = ids_for(FrictionKind::CommitmentFailure);
    if !implementation.is_empty() || !commitments.is_empty() {
        let mut support = implementation;
        support.extend(commitments);
        out.push(FrictionHypothesis {
            id: "hyp_implementation_failure".to_string(),
            hypothesis: "Implementation failure is more likely administrative or capacity-driven than purely political.".to_string(),
            supporting_friction_ids: support,
            contradicting_evidence: vec![],
            likelihood: 0.68,
            confidence: 0.62,
            status: HypothesisStatus::Hypothesized,
        });
    }

    let trust = ids_for(FrictionKind::TrustLoss);
    let escalation = frictions
        .iter()
        .filter(|friction| friction.trajectory == FrictionTrajectory::Escalating)
        .map(|friction| friction.id.clone())
        .collect::<Vec<_>>();
    if !trust.is_empty() || escalation.len() >= 2 {
        let mut support = trust;
        support.extend(escalation);
        support.sort();
        support.dedup();
        out.push(FrictionHypothesis {
            id: "hyp_trust_loss_escalating".to_string(),
            hypothesis: "Trust loss is escalating between implementation actors and is likely to harden future bargaining positions.".to_string(),
            supporting_friction_ids: support,
            contradicting_evidence: vec![],
            likelihood: 0.73,
            confidence: 0.66,
            status: HypothesisStatus::Supported,
        });
    }

    let legal = ids_for(FrictionKind::LegalBlockage);
    let drift = ids_for(FrictionKind::InstitutionalDrift);
    if !legal.is_empty() || !drift.is_empty() {
        let mut support = legal;
        support.extend(drift);
        out.push(FrictionHypothesis {
            id: "hyp_legal_blockage_drift".to_string(),
            hypothesis: "Legal blockage is converting into institutional drift unless verification authority becomes durable.".to_string(),
            supporting_friction_ids: support,
            contradicting_evidence: vec![],
            likelihood: 0.7,
            confidence: 0.64,
            status: HypothesisStatus::Hypothesized,
        });
    }

    out
}
