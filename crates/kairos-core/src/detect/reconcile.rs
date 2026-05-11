use crate::episode::{Episode, EpisodeProposal, ReviewState};
use crate::temporal::TemporalInterval;

pub fn reconcile(mut proposals: Vec<EpisodeProposal>) -> Vec<Episode> {
    proposals.sort_by_key(|p| p.boundary_in.at);
    proposals
        .into_iter()
        .map(|p| Episode {
            id: Episode::new_id(),
            kind: p.kind,
            title: p.title,
            interval: TemporalInterval {
                from: p.boundary_in.at,
                to: p.boundary_out.as_ref().map(|b| b.at),
                fuzziness_secs: p.boundary_in.fuzziness_secs,
            },
            boundary_in: p.boundary_in,
            boundary_out: p.boundary_out,
            anchors: p.anchors,
            narrative: p.narrative,
            confidence: p.confidence,
            review_state: ReviewState::Proposed,
        })
        .collect()
}
