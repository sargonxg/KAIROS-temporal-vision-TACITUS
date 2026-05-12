use kairos_core::{
    validate_graph, AllenRelation, AnalysisMode, AnalysisRequest, Kairos, ValidationRequest,
};

#[tokio::test]
async fn mock_pipeline_returns_temporal_structure() {
    let text = include_str!("../../../examples/demo-text.md");
    let result = Kairos::mock()
        .analyze(AnalysisRequest {
            text: text.to_string(),
            gemini_api_key: None,
            gemini_model: None,
            document_id: Some("meridian-demo".to_string()),
            document_created_at: None,
            analysis_mode: AnalysisMode::Auto,
        })
        .await
        .expect("mock analysis");

    assert!(result.dates.len() >= 15, "expected many date mentions");
    assert!(result.events.len() >= 25, "expected temporal events");
    assert!(result.episodes.len() >= 10, "expected episode bands");
    assert!(
        result
            .relations
            .iter()
            .any(|r| !matches!(r.relation, AllenRelation::Before | AllenRelation::After)),
        "expected at least one non-trivial Allen relation"
    );
    assert!(result.actors.len() >= 10, "expected ACO actors");
    assert!(result.commitments.len() >= 10, "expected ACO commitments");
    assert!(result.frictions.len() >= 10, "expected friction objects");
    assert!(
        result.hypotheses.len() >= 3,
        "expected cautious friction hypotheses"
    );
    assert!(
        result.graph_summary.node_count > 0 && result.graph_summary.edge_count > 0,
        "expected graph export summary"
    );
    assert!(
        result.pre_read.chronology_block_count >= 20,
        "expected chronology pre-read segmentation"
    );
    assert!(
        !result.actor_registry.alias_to_actor_id.is_empty(),
        "expected actor alias registry"
    );
    assert!(
        result.diagnostics.warnings.len() >= 2,
        "expected contradictions or warnings"
    );
    assert!(
        result.diagnostics.high_intensity_friction_count >= 3,
        "expected high-intensity friction signal"
    );
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
            document_id: None,
            document_created_at: None,
            analysis_mode: AnalysisMode::Auto,
        })
        .await
        .expect("mock analysis");

    let diagnostics = validate_graph(&ValidationRequest {
        dates: result.dates,
        commitments: result.commitments,
        frictions: result.frictions,
        episodes: result.episodes,
        relations: result.relations,
    });

    assert!(diagnostics.non_trivial_relations >= 1);
    assert!(diagnostics.relation_counts.contains_key("Before"));
}

#[tokio::test]
async fn dct_resolves_relative_dates() {
    let result = Kairos::mock()
        .analyze(AnalysisRequest {
            text: "March 10, 2024: The cabinet opened review. Two days later, the governor objected. Next quarter, the authority will publish a ledger.".to_string(),
            gemini_api_key: None,
            gemini_model: None,
            document_id: Some("relative-fixture".to_string()),
            document_created_at: Some(
                chrono::DateTime::parse_from_rfc3339("2024-03-10T00:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            ),
            analysis_mode: AnalysisMode::Single,
        })
        .await
        .expect("mock analysis");

    assert!(
        result
            .dates
            .iter()
            .any(|date| date.text.eq_ignore_ascii_case("Two days later")
                && date
                    .resolved
                    .map(|dt| dt.to_rfc3339().starts_with("2024-03-12"))
                    .unwrap_or(false)),
        "expected DCT anchored relative date"
    );
    assert!(
        result
            .dates
            .iter()
            .any(|date| date.text.eq_ignore_ascii_case("Next quarter")
                && date
                    .resolved
                    .map(|dt| dt.to_rfc3339().starts_with("2024-04-01"))
                    .unwrap_or(false)),
        "expected next quarter resolution"
    );
}

#[tokio::test]
async fn source_spans_and_graph_preserve_object_links() {
    let text = include_str!("../../../examples/demo-text.md");
    let result = Kairos::mock()
        .analyze(AnalysisRequest {
            text: text.to_string(),
            gemini_api_key: None,
            gemini_model: None,
            document_id: Some("source-index-fixture".to_string()),
            document_created_at: None,
            analysis_mode: AnalysisMode::Dossier,
        })
        .await
        .expect("mock analysis");

    assert!(
        result
            .source_index
            .spans_by_object
            .keys()
            .any(|id| id.starts_with("evt_") || id.starts_with("fr_")),
        "expected source index to link events or frictions"
    );
    assert!(
        result
            .graph
            .edges
            .iter()
            .any(|edge| edge.edge_type == "inferred_from"),
        "expected hypotheses to cite graph evidence"
    );
}
