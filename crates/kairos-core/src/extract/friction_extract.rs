use crate::aco::{ExtractedActor, ExtractedCommitment, TemporalEvent};
use crate::episode::Episode;
use crate::friction::{Friction, FrictionKind, FrictionTrajectory};
use crate::llm::LlmClient;
use crate::source::SourceSpan;
use crate::temporal::TemporalInterval;
use crate::{KairosError, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};

pub async fn extract_frictions(
    llm: &LlmClient,
    text: &str,
    events: &[TemporalEvent],
    actors: &[ExtractedActor],
    commitments: &[ExtractedCommitment],
    episodes: &[Episode],
) -> Result<Vec<Friction>> {
    if llm.is_mock() {
        return Ok(mock_frictions(text, events, commitments, episodes));
    }

    let prompt = format!(
        "Extract human and institutional friction from the text as a JSON array matching the schema. \
         Friction means commitment failure, obstruction, drift, trust loss, policy divergence, legal blockage, \
         implementation gap, leadership transition, or escalation. Use only evidence grounded in the source. \
         Link actors, events, commitments, and episodes by existing IDs when possible. Output JSON only.\n\n\
         EVENTS:{events}\nACTORS:{actors}\nCOMMITMENTS:{commitments}\nEPISODES:{episodes}\nTEXT:\n{text}",
        events = serde_json::to_string(events).unwrap_or_default(),
        actors = serde_json::to_string(actors).unwrap_or_default(),
        commitments = serde_json::to_string(commitments).unwrap_or_default(),
        episodes = serde_json::to_string(episodes).unwrap_or_default(),
    );

    parse_frictions(
        llm.complete_json_with_schema(
            &prompt,
            json!({
                "type": "ARRAY",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "id": {"type": "STRING"},
                        "kind": {"type": "STRING"},
                        "summary": {"type": "STRING"},
                        "actors_involved": {"type": "ARRAY", "items": {"type": "STRING"}},
                        "triggering_event": {"type": "STRING"},
                        "interval": {"type": "OBJECT"},
                        "evidence_spans": {"type": "ARRAY", "items": {"type": "OBJECT"}},
                        "intensity": {"type": "NUMBER"},
                        "confidence": {"type": "NUMBER"},
                        "commitment_ids": {"type": "ARRAY", "items": {"type": "STRING"}},
                        "episode_ids": {"type": "ARRAY", "items": {"type": "STRING"}},
                        "trajectory": {"type": "STRING"}
                    },
                    "required": ["kind", "summary", "actors_involved", "interval", "evidence_spans", "intensity", "confidence", "trajectory"]
                }
            }),
        )
        .await?,
    )
}

fn parse_frictions(value: Value) -> Result<Vec<Friction>> {
    let arr = value
        .as_array()
        .ok_or_else(|| KairosError::Extract("friction response was not an array".to_string()))?;
    Ok(arr
        .iter()
        .filter_map(|item| serde_json::from_value::<Friction>(item.clone()).ok())
        .filter_map(Friction::normalize)
        .collect())
}

fn mock_frictions(
    text: &str,
    events: &[TemporalEvent],
    commitments: &[ExtractedCommitment],
    episodes: &[Episode],
) -> Vec<Friction> {
    let find_span = |needle: &str| {
        text.find(needle)
            .map(|start| SourceSpan::new(None, start, start + needle.len(), text))
            .into_iter()
            .collect::<Vec<_>>()
    };
    let event_id = |idx: usize| events.get(idx).map(|event| event.id.clone());
    let commitment_id = |needle: &str| {
        commitments
            .iter()
            .find(|c| c.summary.to_lowercase().contains(needle))
            .map(|c| c.id.clone())
            .into_iter()
            .collect::<Vec<_>>()
    };
    let episode_ids = |needle: &str| {
        episodes
            .iter()
            .filter(|episode| episode.title.to_lowercase().contains(needle))
            .map(|episode| episode.id.clone())
            .collect::<Vec<_>>()
    };

    vec![
        mock_friction(
            FrictionKind::ProceduralObstruction,
            "No actor accepts responsibility for compensating workers or farmers during the first monitoring period.",
            vec!["actor_okoye", "actor_ibarra", "actor_vale", "actor_soren"],
            event_id(2),
            "2024-02-05T00:00:00Z",
            "2024-02-19T00:00:00Z",
            find_span("no one accepts responsibility for compensating workers or farmers"),
            0.72,
            commitment_id("fund desalination"),
            episode_ids("negotiation"),
            FrictionTrajectory::Escalating,
        ),
        mock_friction(
            FrictionKind::CommitmentFailure,
            "Farmer compensation misses the compact clock, allowing Vale to declare breach and threaten renewed blockades.",
            vec!["actor_vale", "actor_okoye"],
            event_id(8),
            "2024-06-17T00:00:00Z",
            "2024-07-09T00:00:00Z",
            find_span("only 42 percent of eligible farmers have received compensation"),
            0.88,
            commitment_id("farmer payments"),
            episode_ids("implementation"),
            FrictionTrajectory::Escalating,
        ),
        mock_friction(
            FrictionKind::ImplementationGap,
            "Flood damage turns a formal audit milestone into an uncertain implementation gap.",
            vec!["actor_haddad", "actor_vale", "actor_okoye"],
            event_id(9),
            "2024-07-09T00:00:00Z",
            "2024-08-22T00:00:00Z",
            find_span("pushes the July 31 canal-audit milestone into uncertainty"),
            0.77,
            commitment_id("canal audits"),
            episode_ids("implementation"),
            FrictionTrajectory::Mutating,
        ),
        mock_friction(
            FrictionKind::TrustLoss,
            "The provisional terminal reopening splits Ibarra's victory claim from Vale's view that the audit remains incomplete.",
            vec!["actor_ibarra", "actor_vale", "actor_soren"],
            event_id(10),
            "2024-08-22T00:00:00Z",
            "2024-09-18T00:00:00Z",
            find_span("Vale calls the reopening premature because the canal audit remains incomplete"),
            0.81,
            commitment_id("terminal reopening"),
            episode_ids("implementation"),
            FrictionTrajectory::Freezing,
        ),
        mock_friction(
            FrictionKind::InstitutionalDrift,
            "The verification cell emerges because sanctioned brokers continue shaping farm behavior through shell companies.",
            vec!["actor_okoye", "actor_brokers"],
            event_id(11),
            "2024-09-18T00:00:00Z",
            "2025-02-14T00:00:00Z",
            find_span("continued advising private farms through shell companies"),
            0.84,
            vec![],
            episode_ids("temporal"),
            FrictionTrajectory::Mutating,
        ),
        mock_friction(
            FrictionKind::LeadershipTransition,
            "Ibarra's campaign move and Okoye's resignation shift the compact from governance instrument to election asset.",
            vec!["actor_ibarra", "actor_okoye", "actor_reed"],
            event_id(13),
            "2024-11-12T00:00:00Z",
            "2024-12-03T00:00:00Z",
            find_span("turning the compact into an election asset"),
            0.79,
            vec![],
            episode_ids("okoye"),
            FrictionTrajectory::Mutating,
        ),
    ]
    .into_iter()
    .filter_map(Friction::normalize)
    .collect()
}

#[allow(clippy::too_many_arguments)]
fn mock_friction(
    kind: FrictionKind,
    summary: &str,
    actors: Vec<&str>,
    triggering_event: Option<String>,
    from: &str,
    to: &str,
    evidence_spans: Vec<SourceSpan>,
    intensity: f32,
    commitment_ids: Vec<String>,
    episode_ids: Vec<String>,
    trajectory: FrictionTrajectory,
) -> Friction {
    Friction {
        id: Friction::new_id(),
        kind,
        summary: summary.to_string(),
        actors_involved: actors.into_iter().map(ToString::to_string).collect(),
        triggering_event,
        interval: TemporalInterval {
            from: parse_dt(from),
            to: Some(parse_dt(to)),
            fuzziness_secs: 60 * 60 * 24,
        },
        evidence_spans,
        intensity,
        confidence: 0.9,
        commitment_ids,
        episode_ids,
        trajectory,
        attrs: Default::default(),
    }
}

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc.timestamp_opt(0, 0).unwrap())
}
