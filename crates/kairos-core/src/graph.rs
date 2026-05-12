use crate::aco::{ExtractedCommitment, TemporalEvent};
use crate::actors::canonical::ActorRegistry;
use crate::episode::Episode;
use crate::extract::DateMention;
use crate::friction::Friction;
use crate::hypothesis::FrictionHypothesis;
use crate::relations::EpisodeRelation;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    #[serde(default)]
    pub attrs: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub edge_type: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub attrs: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GraphSummary {
    pub node_count: usize,
    pub edge_count: usize,
    pub node_counts: BTreeMap<String, usize>,
    pub edge_counts: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GraphExport {
    pub summary: GraphSummary,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphInput<'a> {
    pub document_id: &'a str,
    pub dates: &'a [DateMention],
    pub events: &'a [TemporalEvent],
    pub actor_registry: &'a ActorRegistry,
    pub commitments: &'a [ExtractedCommitment],
    pub frictions: &'a [Friction],
    pub episodes: &'a [Episode],
    pub relations: &'a [EpisodeRelation],
    pub hypotheses: &'a [FrictionHypothesis],
}

pub fn build_graph(input: GraphInput<'_>) -> GraphExport {
    let mut graph = GraphExport::default();
    push_node(
        &mut graph,
        input.document_id.to_string(),
        "document",
        input.document_id.to_string(),
        json!({}),
    );
    for date in input.dates {
        let id = format!("timex_{}_{}", date.char_start, date.char_end);
        push_node(
            &mut graph,
            id.clone(),
            "timex",
            date.text.clone(),
            json!({"resolved": date.resolved, "kind": date.kind}),
        );
        push_edge(&mut graph, "contains", input.document_id, &id, json!({}));
    }
    for actor in &input.actor_registry.actors {
        push_node(
            &mut graph,
            actor.id.clone(),
            "actor",
            actor.canonical_name.clone(),
            json!({"role": actor.role, "confidence": actor.confidence}),
        );
        push_edge(
            &mut graph,
            "mentions",
            input.document_id,
            &actor.id,
            json!({}),
        );
        for alias in &actor.aliases {
            let alias_id = format!("alias_{}_{}", actor.id, stable_slug(alias));
            push_node(
                &mut graph,
                alias_id.clone(),
                "alias",
                alias.clone(),
                json!({}),
            );
            push_edge(&mut graph, "aliases", &alias_id, &actor.id, json!({}));
        }
    }
    for event in input.events {
        push_node(
            &mut graph,
            event.id.clone(),
            "event",
            event.canonical_name.clone(),
            json!({"at": event.at, "event_kind": event.event_kind}),
        );
        push_edge(
            &mut graph,
            "contains",
            input.document_id,
            &event.id,
            json!({}),
        );
        for actor_id in &event.actor_ids {
            push_edge(&mut graph, "involves", &event.id, actor_id, json!({}));
        }
    }
    for commitment in input.commitments {
        push_node(
            &mut graph,
            commitment.id.clone(),
            "commitment",
            commitment.summary.clone(),
            json!({"committer": commitment.committer, "committee": commitment.committee, "state": commitment.state}),
        );
        push_edge(
            &mut graph,
            "commits_to",
            input.document_id,
            &commitment.id,
            json!({}),
        );
    }
    for episode in input.episodes {
        push_node(
            &mut graph,
            episode.id.clone(),
            "episode",
            episode.title.clone(),
            json!({"kind": episode.kind, "from": episode.interval.from, "to": episode.interval.to}),
        );
        push_edge(
            &mut graph,
            "contains",
            input.document_id,
            &episode.id,
            json!({}),
        );
    }
    for friction in input.frictions {
        push_node(
            &mut graph,
            friction.id.clone(),
            "friction",
            friction.summary.clone(),
            json!({"kind": friction.kind, "polarity": friction.polarity, "mechanism": friction.mechanism, "latent": friction.latent}),
        );
        for actor_id in &friction.actors_involved {
            push_edge(&mut graph, "involves", &friction.id, actor_id, json!({}));
        }
        for commitment_id in &friction.commitment_ids {
            let edge_type = if friction.kind == crate::friction::FrictionKind::CommitmentFailure {
                "violates"
            } else {
                "disputes"
            };
            push_edge(
                &mut graph,
                edge_type,
                &friction.id,
                commitment_id,
                json!({}),
            );
        }
        for episode_id in &friction.episode_ids {
            push_edge(&mut graph, "overlaps", &friction.id, episode_id, json!({}));
        }
        if let Some(event_id) = &friction.triggering_event {
            push_edge(&mut graph, "triggers", event_id, &friction.id, json!({}));
        }
    }
    for hypothesis in input.hypotheses {
        push_node(
            &mut graph,
            hypothesis.id.clone(),
            "hypothesis",
            hypothesis.hypothesis.clone(),
            json!({"likelihood": hypothesis.likelihood, "confidence": hypothesis.confidence, "status": hypothesis.status}),
        );
        for friction_id in &hypothesis.supporting_friction_ids {
            push_edge(
                &mut graph,
                "inferred_from",
                &hypothesis.id,
                friction_id,
                json!({}),
            );
        }
    }
    for relation in input.relations {
        push_edge(
            &mut graph,
            &format!("{:?}", relation.relation).to_lowercase(),
            &relation.from_episode,
            &relation.to_episode,
            json!({"allen_relation": relation.relation}),
        );
    }
    graph.summary.node_count = graph.nodes.len();
    graph.summary.edge_count = graph.edges.len();
    graph
}

fn push_node(
    graph: &mut GraphExport,
    id: String,
    node_type: &str,
    label: String,
    attrs: serde_json::Value,
) {
    *graph
        .summary
        .node_counts
        .entry(node_type.to_string())
        .or_default() += 1;
    graph.nodes.push(GraphNode {
        id,
        node_type: node_type.to_string(),
        label,
        attrs,
    });
}

fn push_edge(
    graph: &mut GraphExport,
    edge_type: &str,
    from: &str,
    to: &str,
    attrs: serde_json::Value,
) {
    let id = format!("edge_{:05}", graph.edges.len() + 1);
    *graph
        .summary
        .edge_counts
        .entry(edge_type.to_string())
        .or_default() += 1;
    graph.edges.push(GraphEdge {
        id,
        edge_type: edge_type.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        attrs,
    });
}

fn stable_slug(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>()
}
