use crate::detect::trait_::EpisodeDetector;
use crate::episode::{EpisodeBoundary, EpisodeKind, EpisodeProposal};
use crate::llm::LlmClient;
use crate::pipeline::AnalysisContext;
use crate::{KairosError, Result};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};

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
            "Detect coherent temporal episodes in the text. Return a JSON array matching the schema. \
             Episodes are meaningful policy/crisis intervals, not every single event. Use RFC3339 UTC dates. \
             Keep titles concise and narratives evidence-backed. kind must be one of \
             regime,leadership,agreement,sanction,escalation,de_escalation,pivot,crisis,custom. \
             Output JSON only.\n\nDATES:{dates}\nEVENTS:{events}\nACTORS:{actors}\nCOMMITMENTS:{commitments}\nTEXT:\n{text}",
            dates = serde_json::to_string(ctx.dates).unwrap_or_default(),
            events = serde_json::to_string(ctx.events).unwrap_or_default(),
            actors = serde_json::to_string(ctx.actors).unwrap_or_default(),
            commitments = serde_json::to_string(ctx.commitments).unwrap_or_default(),
            text = ctx.text
        );
        parse_proposals(
            self.llm
                .complete_json_with_schema(&prompt, episode_schema())
                .await?,
        )
    }
}

fn episode_schema() -> Value {
    let boundary = json!({
        "type": "OBJECT",
        "properties": {
            "at": {"type": "STRING"},
            "fuzziness_secs": {"type": "INTEGER"},
            "triggering_events": {"type": "ARRAY", "items": {"type": "STRING"}},
            "rationale": {"type": "STRING"},
            "confidence": {"type": "NUMBER"}
        },
        "required": ["at", "fuzziness_secs", "triggering_events", "rationale", "confidence"]
    });
    json!({
        "type": "ARRAY",
        "items": {
            "type": "OBJECT",
            "properties": {
                "kind": {"type": "STRING"},
                "title": {"type": "STRING"},
                "boundary_in": boundary,
                "boundary_out": {
                    "type": "OBJECT",
                    "properties": {
                        "at": {"type": "STRING"},
                        "fuzziness_secs": {"type": "INTEGER"},
                        "triggering_events": {"type": "ARRAY", "items": {"type": "STRING"}},
                        "rationale": {"type": "STRING"},
                        "confidence": {"type": "NUMBER"}
                    }
                },
                "anchors": {"type": "ARRAY", "items": {"type": "STRING"}},
                "narrative": {"type": "STRING"},
                "detector": {"type": "STRING"},
                "confidence": {"type": "NUMBER"}
            },
            "required": ["kind", "title", "boundary_in", "anchors", "narrative", "confidence"]
        }
    })
}

fn parse_proposals(value: Value) -> Result<Vec<EpisodeProposal>> {
    let arr = value
        .as_array()
        .ok_or_else(|| KairosError::Extract("episodes response was not an array".to_string()))?;
    Ok(arr
        .iter()
        .filter_map(|item| serde_json::from_value(item.clone()).ok())
        .filter(|proposal: &EpisodeProposal| !proposal.title.trim().is_empty())
        .collect())
}

fn mock_proposals() -> Vec<EpisodeProposal> {
    vec![
        proposal(
            EpisodeKind::Crisis,
            "Meridian Basin Scarcity Crisis",
            "2024-01-08T00:00:00Z",
            Some("2024-10-07T00:00:00Z"),
            "Reservoir collapse, export limits, strikes, and legality fights create the governing crisis window.",
            vec!["reservoirs".to_string(), "export pause".to_string()],
        ),
        proposal(
            EpisodeKind::Escalation,
            "Selene Port Strike Escalation",
            "2024-01-22T00:00:00Z",
            Some("2024-08-22T00:00:00Z"),
            "Port unions use rolling strikes and recall posture to pressure the cabinet while the compact is negotiated.",
            vec!["Selene".to_string(), "port unions".to_string()],
        ),
        proposal(
            EpisodeKind::Agreement,
            "Meridian Compact Negotiation",
            "2024-02-05T00:00:00Z",
            Some("2024-05-10T00:00:00Z"),
            "Official talks and the Northbridge backchannel converge into a staged compact.",
            vec!["Meridian Compact".to_string(), "Northbridge".to_string()],
        ),
        proposal(
            EpisodeKind::Sanction,
            "Water-Broker Sanctions Cycle",
            "2024-03-14T00:00:00Z",
            Some("2024-12-03T00:00:00Z"),
            "Targeted sanctions, court constraints, leaked shell-company activity, and Reed's freeze form a distinct enforcement arc.",
            vec!["sanctions".to_string(), "water brokers".to_string()],
        ),
        proposal(
            EpisodeKind::Pivot,
            "Implementation Drift And Cure Period",
            "2024-05-10T00:00:00Z",
            Some("2024-10-07T00:00:00Z"),
            "The signed compact enters a performance phase where payments, audits, flooding, and terminal reopening diverge from the formal clock.",
            vec!["payments".to_string(), "canal audits".to_string()],
        ),
        proposal(
            EpisodeKind::Leadership,
            "Okoye Water Portfolio Tenure",
            "2024-01-08T00:00:00Z",
            Some("2024-11-14T00:00:00Z"),
            "Okoye owns the emergency order, sanctions, compact signature, and implementation dispute until resigning.",
            vec!["Amara Okoye".to_string()],
        ),
        proposal(
            EpisodeKind::Leadership,
            "Reed Review And Institutionalization",
            "2024-12-03T00:00:00Z",
            Some("2025-02-14T00:00:00Z"),
            "Reed converts the crisis response into a public ledger, satellite-audit rhythm, and permanent authority bill.",
            vec!["Tomas Reed".to_string(), "implementation ledger".to_string()],
        ),
        proposal(
            EpisodeKind::Custom,
            "Temporal Monitoring Backbone Emerges",
            "2024-09-18T00:00:00Z",
            Some("2025-02-14T00:00:00Z"),
            "Verification moves from ad hoc reconciliation of evidence to a proposed standing temporal-monitoring unit.",
            vec!["verification cell".to_string(), "temporal-monitoring unit".to_string()],
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
