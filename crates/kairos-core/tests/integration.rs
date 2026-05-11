use kairos_core::{validate_graph, AllenRelation, AnalysisRequest, Kairos, ValidationRequest};

#[tokio::test]
async fn mock_pipeline_returns_temporal_structure() {
    let text = include_str!("../../../examples/demo-text.md");
    let result = Kairos::mock()
        .analyze(AnalysisRequest {
            text: text.to_string(),
            gemini_api_key: None,
            gemini_model: None,
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
    assert_eq!(result.metadata.provider, "mock");
    assert_eq!(result.metadata.schema_version, "kairos.analysis.v1");
    assert!(
        result.diagnostics.non_trivial_relations >= 1,
        "expected diagnostic relation signal"
    );
    assert!(
        result.diagnostics.deadline_commitments >= 1,
        "expected deadline-like commitments"
    );
    let serialized = serde_json::to_string(&result).expect("serialize result");
    assert!(
        !serialized.contains("gemini_api_key"),
        "response must not expose request-scoped key fields"
    );
}

#[tokio::test]
async fn validate_graph_returns_diagnostics_without_analysis() {
    let text = include_str!("../../../examples/demo-text.md");
    let result = Kairos::mock()
        .analyze(AnalysisRequest {
            text: text.to_string(),
            gemini_api_key: None,
            gemini_model: None,
        })
        .await
        .expect("mock analysis");

    let diagnostics = validate_graph(&ValidationRequest {
        dates: result.dates,
        commitments: result.commitments,
        episodes: result.episodes,
        relations: result.relations,
    });

    assert!(diagnostics.non_trivial_relations >= 1);
    assert!(diagnostics.relation_counts.contains_key("Before"));
}
