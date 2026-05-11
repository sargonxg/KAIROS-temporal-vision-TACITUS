//! KAIROS core library: temporal extraction, episode detection, and Allen-13 relations.

pub mod aco;
pub mod detect;
pub mod diagnostics;
pub mod episode;
pub mod extract;
pub mod llm;
pub mod pipeline;
pub mod relations;
pub mod store;
pub mod temporal;

#[derive(thiserror::Error, Debug)]
pub enum KairosError {
    #[error("LLM call failed: {0}")]
    Llm(String),
    #[error("storage error: {0}")]
    Store(String),
    #[error("extraction error: {0}")]
    Extract(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, KairosError>;

pub use aco::{AcoKind, ExtractedActor, ExtractedCommitment, TemporalEvent};
pub use diagnostics::{validate_graph, AnalysisMetadata, TemporalDiagnostics, ValidationRequest};
pub use episode::{Episode, EpisodeBoundary, EpisodeKind, EpisodeProposal, ReviewState};
pub use pipeline::{AnalysisRequest, AnalysisResult, Kairos};
pub use relations::{AllenRelation, EpisodeRelation};
pub use temporal::{Bitemporal, TemporalInterval, TransactionInterval, ValidityInterval};
