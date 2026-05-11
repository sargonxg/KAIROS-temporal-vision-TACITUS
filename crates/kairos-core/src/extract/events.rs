use crate::aco::TemporalEvent;
use crate::extract::DateMention;
use crate::llm::LlmClient;
use crate::{KairosError, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
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
        "Extract dated events from this text as JSON array. Each item must have mention, canonical_name, at (RFC3339), actor_ids array, event_kind. Output only JSON.\n\nTEXT:\n{text}"
    );
    let value = llm.complete_json(&prompt).await?;
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
            });
        }
    }
    Ok(events)
}

fn mock_events(dates: &[DateMention]) -> Vec<TemporalEvent> {
    let labels = [
        (
            "Sarah Chen announces emergency rationing",
            "policy_announcement",
        ),
        ("Mayor Hayes appoints Marcus Liu", "leadership_transition"),
        ("Hayes announces infrastructure commitment", "commitment"),
        (
            "Rationing ends as reservoir levels recover",
            "de_escalation",
        ),
        ("Greenfield pipeline construction begins", "implementation"),
        ("Liu signs Greenfield joint-use agreement", "agreement"),
        ("Hayes loses re-election", "regime_transition"),
        ("Wells takes office", "regime_transition"),
        (
            "Wells freezes new construction pending review",
            "policy_review",
        ),
    ];
    dates
        .iter()
        .filter_map(|d| d.resolved)
        .take(labels.len())
        .zip(labels)
        .map(|(at, (name, kind))| TemporalEvent {
            id: format!("evt_{}", Uuid::now_v7()),
            mention: name.to_string(),
            canonical_name: name.to_string(),
            at,
            fuzziness_secs: 0,
            actor_ids: vec![],
            event_kind: Some(kind.to_string()),
        })
        .collect()
}
