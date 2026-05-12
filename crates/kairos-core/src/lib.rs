//! KAIROS core library: temporal extraction, episode detection, and Allen-13 relations.

pub mod aco;
pub mod actors;
pub mod detect;
pub mod diagnostics;
pub mod episode;
pub mod extract;
pub mod friction;
pub mod graph;
pub mod hypothesis;
pub mod llm;
#[cfg(feature = "kairos-neural")]
pub mod neural;
pub mod pipeline;
pub mod pre_read;
pub mod relations;
pub mod source;
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
pub use actors::canonical::{ActorRegistry, CanonicalActor};
pub use diagnostics::{validate_graph, AnalysisMetadata, TemporalDiagnostics, ValidationRequest};
pub use episode::{Episode, EpisodeBoundary, EpisodeKind, EpisodeProposal, ReviewState};
pub use friction::{
    EvidenceGrade, Friction, FrictionDirectness, FrictionKind, FrictionMechanism, FrictionPolarity,
    FrictionTrajectory,
};
pub use graph::{GraphEdge, GraphExport, GraphNode, GraphSummary};
pub use hypothesis::{FrictionHypothesis, HypothesisStatus};
pub use pipeline::{AnalysisMode, AnalysisRequest, AnalysisResult, Kairos};
pub use pre_read::{DocumentType, PreReadReport, Segment, SegmentKind, TensionMarker};
pub use relations::{AllenRelation, EpisodeRelation};
pub use source::{index::SourceIndex, SourceSpan};
pub use temporal::{Bitemporal, TemporalInterval, TransactionInterval, ValidityInterval};
