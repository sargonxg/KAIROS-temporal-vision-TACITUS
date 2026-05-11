use crate::detect::trait_::EpisodeDetector;
use crate::episode::{EpisodeBoundary, EpisodeKind, EpisodeProposal};
use crate::llm::LlmClient;
use crate::pipeline::AnalysisContext;
use crate::{KairosError, Result};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

pub struct LlmJudgeDetector {
    llm: LlmClient,
}

impl LlmJudgeDetector {
    pub fn new(llm: LlmClient) -> Self {
        Self { llm }
    }
}

#[async_trait]
impl EpisodeDetector for LlmJudgeDetector {
    async fn propose(&self, ctx: &AnalysisContext<'_>) -> Result<Vec<EpisodeProposal>> {
        if self.llm.is_mock() {
            return Ok(mock_proposals());
        }

        let prompt = format!(
            "Detect coherent temporal episodes in the text. Return JSON array. Each item fields: kind, title, boundary_in {{at,rationale,triggering_events,confidence}}, optional boundary_out, anchors, narrative, confidence. kind must be one of regime,leadership,agreement,sanction,escalation,de_escalation,pivot,crisis,custom. Use RFC3339 dates. Output only JSON.\n\nDATES:{dates}\nEVENTS:{events}\nACTORS:{actors}\nCOMMITMENTS:{commitments}\nTEXT:\n{text}",
            dates = serde_json::to_string(ctx.dates).unwrap_or_default(),
            events = serde_json::to_string(ctx.events).unwrap_or_default(),
            actors = serde_json::to_string(ctx.actors).unwrap_or_default(),
            commitments = serde_json::to_string(ctx.commitments).unwrap_or_default(),
            text = ctx.text
        );
        parse_proposals(self.llm.complete_json(&prompt).await?)
    }
}

fn parse_proposals(value: Value) -> Result<Vec<EpisodeProposal>> {
    serde_json::from_value(value)
        .map_err(|e| KairosError::Extract(format!("bad episode JSON: {e}")))
}

fn mock_proposals() -> Vec<EpisodeProposal> {
    vec![
        proposal(
            EpisodeKind::Regime,
            "Hayes Administration",
            "2024-01-01T00:00:00Z",
            Some("2025-01-01T00:00:00Z"),
            "The governing period in which Hayes owns the infrastructure and emergency-water decisions.",
            vec!["Robert Hayes".to_string()],
        ),
        proposal(
            EpisodeKind::Crisis,
            "Riverdale Drought Crisis",
            "2024-01-15T00:00:00Z",
            Some("2024-04-01T00:00:00Z"),
            "Emergency rationing, reservoir pressure, and public conflict define the crisis window.",
            vec!["rationing".to_string(), "reservoir".to_string()],
        ),
        proposal(
            EpisodeKind::Leadership,
            "Chen Water Authority Tenure",
            "2024-01-01T00:00:00Z",
            Some("2024-03-12T00:00:00Z"),
            "Chen leads the authority through the initial rationing decision before being replaced.",
            vec!["Sarah Chen".to_string()],
        ),
        proposal(
            EpisodeKind::Leadership,
            "Liu Water Authority Tenure",
            "2024-03-12T00:00:00Z",
            Some("2025-01-01T00:00:00Z"),
            "Liu executes the infrastructure pivot and negotiates the Greenfield agreement.",
            vec!["Marcus Liu".to_string(), "Greenfield".to_string()],
        ),
        proposal(
            EpisodeKind::Regime,
            "Wells Administration",
            "2025-01-01T00:00:00Z",
            Some("2025-01-15T00:00:00Z"),
            "Wells inherits the water-policy fight and opens a review of inherited decisions.",
            vec!["Patricia Wells".to_string()],
        ),
    ]
}

fn proposal(
    kind: EpisodeKind,
    title: &str,
    start: &str,
    end: Option<&str>,
    narrative: &str,
    anchors: Vec<String>,
) -> EpisodeProposal {
    let start = parse_dt(start);
    let end_dt = end.map(parse_dt);
    EpisodeProposal {
        kind,
        title: title.to_string(),
        boundary_in: boundary(start, "episode begins"),
        boundary_out: end_dt.map(|d| boundary(d, "episode closes or hands off")),
        anchors,
        narrative: narrative.to_string(),
        detector: "mock_llm_judge".to_string(),
        confidence: 0.91,
    }
}

fn boundary(at: DateTime<Utc>, rationale: &str) -> EpisodeBoundary {
    EpisodeBoundary {
        at,
        fuzziness_secs: 60 * 60 * 24,
        triggering_events: vec![],
        rationale: rationale.to_string(),
        confidence: 0.9,
    }
}

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc.timestamp_opt(0, 0).unwrap())
}
