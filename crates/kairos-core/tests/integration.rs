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

    assert!(result.dates.len() >= 8, "expected many date mentions");
    assert!(result.episodes.len() >= 3, "expected episode bands");
    assert!(
        result
            .relations
            .iter()
            .any(|r| !matches!(r.relation, AllenRelation::Before | AllenRelation::After)),
        "expected at least one non-trivial Allen relation"
    );
    assert!(!result.actors.is_empty(), "expected ACO actors");
    assert!(!result.commitments.is_empty(), "expected ACO commitments");
}
