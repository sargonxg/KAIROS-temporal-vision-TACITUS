use crate::aco::ExtractedActor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ActorRegistry {
    pub actors: Vec<CanonicalActor>,
    pub alias_to_actor_id: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CanonicalActor {
    pub id: String,
    pub canonical_name: String,
    pub role: String,
    pub aliases: Vec<String>,
    pub confidence: f32,
}

pub fn build_actor_registry(text: &str, actors: &[ExtractedActor]) -> ActorRegistry {
    let mut registry = ActorRegistry::default();
    for actor in actors {
        let mut aliases = candidate_aliases(&actor.name, &actor.role);
        aliases.retain(|alias| {
            let lower = alias.to_lowercase();
            !lower.trim().is_empty() && text.to_lowercase().contains(&lower)
        });
        if !aliases.iter().any(|alias| alias == &actor.name) {
            aliases.insert(0, actor.name.clone());
        }
        aliases.sort();
        aliases.dedup();
        let canonical = CanonicalActor {
            id: actor.id.clone(),
            canonical_name: actor.name.clone(),
            role: actor.role.clone(),
            aliases: aliases.clone(),
            confidence: 0.78,
        };
        for alias in aliases {
            registry
                .alias_to_actor_id
                .insert(normalize_alias(&alias), actor.id.clone());
        }
        registry.actors.push(canonical);
    }
    registry
}

fn candidate_aliases(name: &str, role: &str) -> Vec<String> {
    let mut aliases = vec![name.to_string()];
    if let Some(last) = name.split_whitespace().last() {
        if last.len() > 2 {
            aliases.push(last.to_string());
        }
    }
    for token in role.split(|c: char| !c.is_alphanumeric() && c != '-') {
        if token.len() > 4 {
            aliases.push(token.to_string());
        }
    }
    if role.contains("minister") {
        aliases.push("minister".to_string());
    }
    if role.contains("governor") {
        aliases.push("governor".to_string());
    }
    aliases
}

fn normalize_alias(alias: &str) -> String {
    alias
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
