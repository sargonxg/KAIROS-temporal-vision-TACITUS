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
            actor("actor_okoye", "Amara Okoye", "interior minister"),
            actor("actor_ibarra", "Rafael Ibarra", "governor of Selene"),
            actor("actor_vale", "Niko Vale", "upstream cooperative leader"),
            actor("actor_soren", "Marta Soren", "Selene port-union chair"),
            actor(
                "actor_haddad",
                "Leila Haddad",
                "Northbridge mediation envoy",
            ),
            actor("actor_reed", "Tomas Reed", "deputy water minister"),
            actor(
                "actor_brokers",
                "Water broker firms",
                "sanctioned private intermediaries",
            ),
        ],
        commitments: vec![
            commitment(
                "com_export_pause",
                "Pause high-water industrial exports while preserving food shipments",
                "Amara Okoye",
                "Meridian cabinet",
                "ordered",
            ),
            commitment(
                "com_ibarra_terminals",
                "Keep two export terminals open if compact milestones are met",
                "Rafael Ibarra",
                "Selene City",
                "announced",
            ),
            commitment(
                "com_resilience_package",
                "Fund desalination, canal repairs, and emergency crop insurance",
                "National cabinet",
                "farmers and port workers",
                "authorized",
            ),
            commitment(
                "com_meridian_compact",
                "Deliver farmer payments, canal audits, and conditional terminal reopening",
                "Meridian Compact parties",
                "basin stakeholders",
                "signed",
            ),
            commitment(
                "com_public_ledger",
                "Publish an implementation ledger and monthly satellite audits",
                "Tomas Reed",
                "public oversight bodies",
                "announced",
            ),
            commitment(
                "com_temporal_unit",
                "Convert verification into a standing temporal-monitoring unit",
                "Meridian Authority Bill",
                "future emergency governance",
                "proposed",
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
