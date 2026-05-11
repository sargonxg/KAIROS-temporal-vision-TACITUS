use crate::aco::{ExtractedActor, ExtractedCommitment, TemporalEvent};
use crate::detect::reconcile::reconcile;
use crate::detect::trait_::EpisodeDetector;
use crate::detect::LlmJudgeDetector;
use crate::diagnostics::{analyze_temporal_diagnostics, AnalysisMetadata, TemporalDiagnostics};
use crate::episode::Episode;
use crate::extract::aco_extract::extract_aco;
use crate::extract::events::extract_events;
use crate::extract::{DateExtractor, DateMention};
use crate::llm::LlmClient;
use crate::relations::{compute, EpisodeRelation};
use crate::store::KairosStore;
use crate::{KairosError, Result};
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
    pub episodes: Vec<Episode>,
    pub relations: Vec<EpisodeRelation>,
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

        let dates = DateExtractor::new().extract(text);
        let (events, aco) =
            tokio::join!(extract_events(&llm, text, &dates), extract_aco(&llm, text));
        let events = events?;
        let aco = aco?;

        let ctx = AnalysisContext {
            text,
            dates: &dates,
            events: &events,
            actors: &aco.actors,
            commitments: &aco.commitments,
        };
        let detector = LlmJudgeDetector::new(llm);
        let proposals = detector.propose(&ctx).await?;
        let episodes = reconcile(proposals);
        let relations = relations_for(&episodes);
        let diagnostics =
            analyze_temporal_diagnostics(&dates, &aco.commitments, &episodes, &relations);
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
            episodes,
            relations,
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
