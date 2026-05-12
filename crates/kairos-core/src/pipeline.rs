use crate::aco::{ExtractedActor, ExtractedCommitment, TemporalEvent};
use crate::actors::canonical::{build_actor_registry, ActorRegistry};
use crate::detect::reconcile::reconcile;
use crate::detect::trait_::EpisodeDetector;
use crate::detect::LlmJudgeDetector;
use crate::diagnostics::{analyze_temporal_diagnostics, AnalysisMetadata, TemporalDiagnostics};
use crate::episode::Episode;
use crate::extract::aco_extract::extract_aco;
use crate::extract::events::extract_events;
use crate::extract::friction_extract::extract_frictions;
use crate::extract::{DateExtractor, DateMention};
use crate::friction::Friction;
use crate::graph::{build_graph, GraphExport, GraphInput, GraphSummary};
use crate::hypothesis::{infer_hypotheses, FrictionHypothesis};
use crate::llm::LlmClient;
use crate::pre_read::{pre_read, PreReadReport};
use crate::relations::{compute, EpisodeRelation};
use crate::source::index::SourceIndex;
use crate::store::KairosStore;
use crate::{KairosError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub text: String,
    #[serde(default)]
    pub gemini_api_key: Option<String>,
    #[serde(default)]
    pub gemini_model: Option<String>,
    #[serde(default)]
    pub document_id: Option<String>,
    #[serde(default)]
    pub document_created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub analysis_mode: AnalysisMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    #[default]
    Auto,
    Single,
    Dossier,
    Corpus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub session_id: String,
    pub metadata: AnalysisMetadata,
    pub diagnostics: TemporalDiagnostics,
    pub dates: Vec<DateMention>,
    pub events: Vec<TemporalEvent>,
    pub actors: Vec<ExtractedActor>,
    pub commitments: Vec<ExtractedCommitment>,
    pub frictions: Vec<Friction>,
    pub episodes: Vec<Episode>,
    pub relations: Vec<EpisodeRelation>,
    pub pre_read: PreReadReport,
    pub actor_registry: ActorRegistry,
    pub source_index: SourceIndex,
    pub hypotheses: Vec<FrictionHypothesis>,
    pub graph_summary: GraphSummary,
    pub graph: GraphExport,
}

pub struct AnalysisContext<'a> {
    pub text: &'a str,
    pub dates: &'a [DateMention],
    pub events: &'a [TemporalEvent],
    pub actors: &'a [ExtractedActor],
    pub commitments: &'a [ExtractedCommitment],
}

#[derive(Clone)]
pub struct Kairos {
    llm: LlmClient,
}

impl Kairos {
    pub fn from_env() -> Self {
        Self {
            llm: LlmClient::from_env(),
        }
    }

    pub fn mock() -> Self {
        Self {
            llm: LlmClient::mock(),
        }
    }

    pub async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResult> {
        let started = Instant::now();
        let text = request.text.trim();
        if text.len() < 10 {
            return Err(KairosError::Invalid(
                "text must contain at least 10 characters".to_string(),
            ));
        }

        let llm = request
            .gemini_api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(|key| {
                LlmClient::gemini_with_key(
                    key,
                    request
                        .gemini_model
                        .as_deref()
                        .map(str::trim)
                        .filter(|model| !model.is_empty())
                        .map(ToString::to_string),
                )
            })
            .unwrap_or_else(|| self.llm.clone());
        let provider = llm.provider_name().to_string();
        let model = llm.model_name();
        let mode = if llm.is_mock() {
            "deterministic_mock".to_string()
        } else {
            "llm_extraction".to_string()
        };

        let dates = DateExtractor::new().extract_with_context(
            text,
            request.document_id.as_deref(),
            request.document_created_at,
        );
        let pre_read = pre_read(text, &dates, request.document_created_at);
        let (events, aco) =
            tokio::join!(extract_events(&llm, text, &dates), extract_aco(&llm, text));
        let events = events?;
        let aco = aco?;
        let actor_registry = build_actor_registry(text, &aco.actors);

        let ctx = AnalysisContext {
            text,
            dates: &dates,
            events: &events,
            actors: &aco.actors,
            commitments: &aco.commitments,
        };
        let detector = LlmJudgeDetector::new(llm.clone());
        let proposals = detector.propose(&ctx).await?;
        let episodes = reconcile(proposals);
        let relations = relations_for(&episodes);
        let frictions = extract_frictions(
            &llm,
            text,
            &events,
            &aco.actors,
            &aco.commitments,
            &episodes,
        )
        .await?;
        let hypotheses = infer_hypotheses(&frictions);
        let source_index = SourceIndex::build(
            request.document_id.clone(),
            &pre_read.segments,
            &events,
            &aco.commitments,
            &frictions,
        );
        let document_id = request
            .document_id
            .clone()
            .unwrap_or_else(|| "document".to_string());
        let graph = build_graph(GraphInput {
            document_id: &document_id,
            dates: &dates,
            events: &events,
            actor_registry: &actor_registry,
            commitments: &aco.commitments,
            frictions: &frictions,
            episodes: &episodes,
            relations: &relations,
            hypotheses: &hypotheses,
        });
        let graph_summary = graph.summary.clone();
        let diagnostics = analyze_temporal_diagnostics(
            &dates,
            &aco.commitments,
            &frictions,
            &episodes,
            &relations,
        );
        let metadata = AnalysisMetadata {
            schema_version: "kairos.analysis.v1".to_string(),
            provider,
            model,
            mode,
            input_chars: text.chars().count(),
            elapsed_ms: started.elapsed().as_millis(),
        };

        if let Ok(store) = KairosStore::memory() {
            for episode in &episodes {
                if let Err(err) = store.insert_episode(episode) {
                    tracing::warn!(error = %err, episode = %episode.id, "episode storage failed");
                }
            }
        }

        Ok(AnalysisResult {
            session_id: format!("sess_{}", Uuid::now_v7()),
            metadata,
            diagnostics,
            dates,
            events,
            actors: aco.actors,
            commitments: aco.commitments,
            frictions,
            episodes,
            relations,
            pre_read,
            actor_registry,
            source_index,
            hypotheses,
            graph_summary,
            graph,
        })
    }
}

fn relations_for(episodes: &[Episode]) -> Vec<EpisodeRelation> {
    let mut out = Vec::new();
    for (i, a) in episodes.iter().enumerate() {
        for (j, b) in episodes.iter().enumerate() {
            if i == j {
                continue;
            }
            out.push(EpisodeRelation {
                from_episode: a.id.clone(),
                to_episode: b.id.clone(),
                relation: compute(&a.interval, &b.interval),
            });
        }
    }
    out
}
