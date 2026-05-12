use crate::aco::TemporalEvent;
use crate::extract::DateMention;
use crate::llm::LlmClient;
use crate::{KairosError, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn extract_events(
    llm: &LlmClient,
    text: &str,
    dates: &[DateMention],
) -> Result<Vec<TemporalEvent>> {
    if llm.is_mock() {
        return Ok(mock_events(dates));
    }

    let prompt = format!(
        "Extract dated events from this text as a JSON array matching the schema. \
         Use only events supported by the source text. Use RFC3339 UTC timestamps in `at`. \
         If a date is fuzzy, choose the best normalized instant and set fuzziness_secs. \
         Output JSON only.\n\nTEXT:\n{text}"
    );
    let value = llm
        .complete_json_with_schema(
            &prompt,
            json!({
                "type": "ARRAY",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "id": {"type": "STRING"},
                        "mention": {"type": "STRING"},
                        "canonical_name": {"type": "STRING"},
                        "at": {"type": "STRING"},
                        "fuzziness_secs": {"type": "INTEGER"},
                        "actor_ids": {"type": "ARRAY", "items": {"type": "STRING"}},
                        "event_kind": {"type": "STRING"},
                        "source_span": {"type": "OBJECT"}
                    },
                    "required": ["mention", "canonical_name", "at", "actor_ids"]
                }
            }),
        )
        .await?;
    parse_events(value)
}

fn parse_events(value: Value) -> Result<Vec<TemporalEvent>> {
    let arr = value
        .as_array()
        .ok_or_else(|| KairosError::Extract("events response was not an array".to_string()))?;
    let mut events = Vec::new();
    for item in arr {
        let at = item["at"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc));
        if let Some(at) = at {
            events.push(TemporalEvent {
                id: item["id"]
                    .as_str()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| format!("evt_{}", Uuid::now_v7())),
                mention: item["mention"].as_str().unwrap_or_default().to_string(),
                canonical_name: item["canonical_name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                at,
                fuzziness_secs: item["fuzziness_secs"].as_u64().unwrap_or(0) as u32,
                actor_ids: item["actor_ids"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
                event_kind: item["event_kind"].as_str().map(ToString::to_string),
                source_span: item
                    .get("source_span")
                    .and_then(|span| serde_json::from_value(span.clone()).ok()),
            });
        }
    }
    Ok(events)
}

fn mock_events(dates: &[DateMention]) -> Vec<TemporalEvent> {
    let labels = [
        ("Okoye orders a temporary export pause", "emergency_order"),
        ("Selene port unions begin rolling strikes", "labor_action"),
        (
            "Okoye convenes the first Meridian Compact round",
            "negotiation_opening",
        ),
        (
            "Cabinet authorizes a $1.2B resilience package",
            "commitment",
        ),
        (
            "Okoye announces targeted sanctions on water brokers",
            "sanction",
        ),
        ("Haddad opens the Northbridge backchannel", "backchannel"),
        (
            "Court blocks part of the license suspension",
            "legal_constraint",
        ),
        ("Parties sign the Meridian Compact", "agreement"),
        ("Vale declares the compact in breach", "breach_claim"),
        ("Flash flood damages the canal inspection route", "shock"),
        ("Selene reopens one export terminal", "implementation"),
        ("Okoye expands the sanctions list", "sanction_expansion"),
        ("Haddad proposes a winter review window", "review_design"),
        (
            "Ibarra enters the national leadership race",
            "political_pivot",
        ),
        ("Tomas Reed freezes new sanctions", "leadership_transition"),
        (
            "Reed commits to a public implementation ledger",
            "learning_commitment",
        ),
        (
            "Meridian Authority Bill creates a temporal-monitoring unit",
            "institutionalization",
        ),
    ];
    dates
        .iter()
        .filter_map(|d| d.resolved.map(|at| (d, at)))
        .take(labels.len())
        .zip(labels)
        .map(|((d, at), (name, kind))| TemporalEvent {
            id: format!("evt_{}", Uuid::now_v7()),
            mention: name.to_string(),
            canonical_name: name.to_string(),
            at,
            fuzziness_secs: 0,
            actor_ids: vec![],
            event_kind: Some(kind.to_string()),
            source_span: d.source_span.clone(),
        })
        .collect()
}
