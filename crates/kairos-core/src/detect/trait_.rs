use crate::episode::EpisodeProposal;
use crate::pipeline::AnalysisContext;
use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EpisodeDetector {
    async fn propose(&self, ctx: &AnalysisContext<'_>) -> Result<Vec<EpisodeProposal>>;
}
