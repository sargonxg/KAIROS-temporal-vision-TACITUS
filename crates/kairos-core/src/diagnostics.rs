use crate::aco::ExtractedCommitment;
use crate::episode::Episode;
use crate::extract::DateMention;
use crate::friction::{Friction, FrictionKind, FrictionTrajectory};
use crate::relations::{AllenRelation, EpisodeRelation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AnalysisMetadata {
    pub schema_version: String,
    pub provider: String,
    pub model: String,
    pub mode: String,
    pub input_chars: usize,
    pub elapsed_ms: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TemporalDiagnostics {
    pub warnings: Vec<String>,
    pub relation_counts: BTreeMap<String, usize>,
    pub non_trivial_relations: usize,
    pub open_ended_episodes: usize,
    pub dense_overlap_pairs: Vec<DiagnosticPair>,
    pub deadline_commitments: usize,
    pub unresolved_dates: usize,
    pub friction_count: usize,
    pub escalating_friction_count: usize,
    pub high_intensity_friction_count: usize,
    pub friction_by_kind: BTreeMap<String, usize>,
    pub confidence: ConfidenceSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagnosticPair {
    pub left: String,
    pub right: String,
    pub relation: AllenRelation,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ConfidenceSummary {
    pub episode_min: Option<f32>,
    pub episode_avg: Option<f32>,
    pub episode_max: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ValidationRequest {
    #[serde(default)]
    pub dates: Vec<DateMention>,
    #[serde(default)]
    pub commitments: Vec<ExtractedCommitment>,
    #[serde(default)]
    pub frictions: Vec<Friction>,
    #[serde(default)]
    pub episodes: Vec<Episode>,
    #[serde(default)]
    pub relations: Vec<EpisodeRelation>,
}

pub fn analyze_temporal_diagnostics(
    dates: &[DateMention],
    commitments: &[ExtractedCommitment],
    frictions: &[Friction],
    episodes: &[Episode],
    relations: &[EpisodeRelation],
) -> TemporalDiagnostics {
    let mut diagnostics = TemporalDiagnostics::default();

    for relation in relations {
        *diagnostics
            .relation_counts
            .entry(format!("{:?}", relation.relation))
            .or_default() += 1;
        if !matches!(
            relation.relation,
            AllenRelation::Before | AllenRelation::After
        ) {
            diagnostics.non_trivial_relations += 1;
        }
        if matches!(
            relation.relation,
            AllenRelation::Overlaps
                | AllenRelation::OverlappedBy
                | AllenRelation::Contains
                | AllenRelation::During
                | AllenRelation::Starts
                | AllenRelation::StartedBy
                | AllenRelation::Finishes
                | AllenRelation::FinishedBy
                | AllenRelation::Equals
        ) {
            diagnostics.dense_overlap_pairs.push(DiagnosticPair {
                left: relation.from_episode.clone(),
                right: relation.to_episode.clone(),
                relation: relation.relation,
            });
        }
    }

    diagnostics.open_ended_episodes = episodes
        .iter()
        .filter(|episode| episode.interval.to.is_none())
        .count();
    diagnostics.unresolved_dates = dates.iter().filter(|date| date.resolved.is_none()).count();
    diagnostics.deadline_commitments = commitments
        .iter()
        .filter(|commitment| {
            let haystack = format!(
                "{} {} {:?}",
                commitment.summary, commitment.state, commitment.attrs
            )
            .to_lowercase();
            haystack.contains("deadline")
                || haystack.contains("by ")
                || haystack.contains("before")
                || haystack.contains("until")
                || haystack.contains("review")
                || haystack.contains("monthly")
        })
        .count();
    diagnostics.friction_count = frictions.len();
    diagnostics.escalating_friction_count = frictions
        .iter()
        .filter(|friction| friction.trajectory == FrictionTrajectory::Escalating)
        .count();
    diagnostics.high_intensity_friction_count = frictions
        .iter()
        .filter(|friction| friction.intensity >= 0.75)
        .count();
    for friction in frictions {
        *diagnostics
            .friction_by_kind
            .entry(friction_kind_label(&friction.kind).to_string())
            .or_default() += 1;
    }

    diagnostics.confidence = confidence_summary(episodes);
    diagnostics.warnings = validation_warnings(dates, commitments, frictions, episodes);
    diagnostics
}

pub fn validate_graph(request: &ValidationRequest) -> TemporalDiagnostics {
    analyze_temporal_diagnostics(
        &request.dates,
        &request.commitments,
        &request.frictions,
        &request.episodes,
        &request.relations,
    )
}

fn confidence_summary(episodes: &[Episode]) -> ConfidenceSummary {
    if episodes.is_empty() {
        return ConfidenceSummary::default();
    }
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    let mut sum = 0.0;
    for episode in episodes {
        min = min.min(episode.confidence);
        max = max.max(episode.confidence);
        sum += episode.confidence;
    }
    ConfidenceSummary {
        episode_min: Some(round3(min)),
        episode_avg: Some(round3(sum / episodes.len() as f32)),
        episode_max: Some(round3(max)),
    }
}

fn validation_warnings(
    dates: &[DateMention],
    commitments: &[ExtractedCommitment],
    frictions: &[Friction],
    episodes: &[Episode],
) -> Vec<String> {
    let mut warnings = Vec::new();

    for episode in episodes {
        if let Some(to) = episode.interval.to {
            if to < episode.interval.from {
                warnings.push(format!("episode '{}' ends before it starts", episode.title));
            }
        }
    }

    let mut titles = HashSet::new();
    for episode in episodes {
        let normalized = episode.title.trim().to_lowercase();
        if !normalized.is_empty() && !titles.insert(normalized) {
            warnings.push(format!("duplicate episode title '{}'", episode.title));
        }
    }

    for commitment in commitments {
        if commitment.summary.trim().is_empty()
            || commitment.committer.trim().is_empty()
            || commitment.committee.trim().is_empty()
            || commitment.state.trim().is_empty()
        {
            warnings.push(format!(
                "commitment '{}' has missing actor, target, state, or summary",
                commitment.id
            ));
        }
    }

    for friction in frictions {
        if friction.evidence_spans.is_empty() {
            warnings.push(format!("friction '{}' has no evidence spans", friction.id));
        }
        if friction.actors_involved.is_empty() {
            warnings.push(format!("friction '{}' has no linked actors", friction.id));
        }
    }

    if frictions
        .iter()
        .any(|friction| friction.summary.to_lowercase().contains("premature"))
    {
        warnings.push(
            "contradiction: terminal reopening claim conflicts with incomplete audit evidence"
                .to_string(),
        );
    }
    if frictions.iter().any(|friction| {
        friction.summary.to_lowercase().contains("administrative")
            || friction
                .competing_hypotheses
                .iter()
                .any(|hypothesis| hypothesis.to_lowercase().contains("political"))
    }) {
        warnings.push(
            "warning: administrative-delay and political-delay explanations remain competing hypotheses"
                .to_string(),
        );
    }
    if frictions.len() >= 10 {
        warnings.push(
            "warning: dense friction field requires analyst review before operational use"
                .to_string(),
        );
    }

    for date in dates {
        if date.resolved.is_none() {
            warnings.push(format!("date '{}' could not be resolved", date.text));
        }
    }

    warnings.sort();
    warnings.dedup();
    warnings
}

fn friction_kind_label(kind: &FrictionKind) -> &'static str {
    match kind {
        FrictionKind::CommitmentFailure => "commitment_failure",
        FrictionKind::ProceduralObstruction => "procedural_obstruction",
        FrictionKind::InstitutionalDrift => "institutional_drift",
        FrictionKind::TrustLoss => "trust_loss",
        FrictionKind::PolicyDivergence => "policy_divergence",
        FrictionKind::LeadershipTransition => "leadership_transition",
        FrictionKind::LegalBlockage => "legal_blockage",
        FrictionKind::ImplementationGap => "implementation_gap",
        FrictionKind::Escalation => "escalation",
        FrictionKind::Custom => "custom",
    }
}

fn round3(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}
