use crate::aco::{ExtractedActor, ExtractedCommitment};
use crate::llm::LlmClient;
use crate::{KairosError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AcoExtraction {
    pub actors: Vec<ExtractedActor>,
    pub commitments: Vec<ExtractedCommitment>,
}

pub async fn extract_aco(llm: &LlmClient, text: &str) -> Result<AcoExtraction> {
    if llm.is_mock() {
        return Ok(mock_aco());
    }
    let prompt = format!(
        "Extract TACITUS ACO primitives from text as JSON object with actors and commitments arrays. Actor fields: id,name,role. Commitment fields: id,summary,committer,committee,state. Output only JSON.\n\nTEXT:\n{text}"
    );
    parse_aco(llm.complete_json(&prompt).await?)
}

fn parse_aco(value: Value) -> Result<AcoExtraction> {
    serde_json::from_value(value).map_err(|e| KairosError::Extract(format!("bad ACO JSON: {e}")))
}

fn mock_aco() -> AcoExtraction {
    AcoExtraction {
        actors: vec![
            actor("actor_chen", "Sarah Chen", "water authority director"),
            actor("actor_hayes", "Robert Hayes", "mayor"),
            actor("actor_liu", "Marcus Liu", "water authority director"),
            actor("actor_wells", "Patricia Wells", "mayor-elect"),
        ],
        commitments: vec![
            commitment(
                "com_hayes_50m",
                "Invest $50M in new reservoir infrastructure",
                "Robert Hayes",
                "Riverdale voters",
                "announced",
            ),
            commitment(
                "com_greenfield",
                "Share the Greenfield pipeline with the neighboring district",
                "Marcus Liu",
                "Greenfield district",
                "signed",
            ),
            commitment(
                "com_wells_freeze",
                "Maintain the pipeline but freeze new construction pending environmental review",
                "Patricia Wells",
                "Riverdale water authority",
                "announced",
            ),
        ],
    }
}

fn actor(id: &str, name: &str, role: &str) -> ExtractedActor {
    ExtractedActor {
        id: id.to_string(),
        name: name.to_string(),
        role: role.to_string(),
        attrs: json!({}),
    }
}

fn commitment(
    id: &str,
    summary: &str,
    committer: &str,
    committee: &str,
    state: &str,
) -> ExtractedCommitment {
    ExtractedCommitment {
        id: id.to_string(),
        summary: summary.to_string(),
        committer: committer.to_string(),
        committee: committee.to_string(),
        state: state.to_string(),
        attrs: json!({}),
    }
}
