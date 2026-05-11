use kairos_core::{AllenRelation, AnalysisRequest, Kairos};

#[tokio::test]
async fn mock_pipeline_returns_temporal_structure() {
    let text = include_str!("../../../examples/demo-text.md");
    let result = Kairos::mock()
        .analyze(AnalysisRequest {
            text: text.to_string(),
        })
        .await
        .expect("mock analysis");

    assert!(result.dates.len() >= 15, "expected many date mentions");
    assert!(result.events.len() >= 15, "expected temporal events");
    assert!(result.episodes.len() >= 8, "expected episode bands");
    assert!(
        result
            .relations
            .iter()
            .any(|r| !matches!(r.relation, AllenRelation::Before | AllenRelation::After)),
        "expected at least one non-trivial Allen relation"
    );
    assert!(result.actors.len() >= 7, "expected ACO actors");
    assert!(result.commitments.len() >= 6, "expected ACO commitments");
}
