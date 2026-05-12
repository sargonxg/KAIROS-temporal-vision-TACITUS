# KAIROS Deep-Tech MVP Build Plan

This plan distills `docs/research/KAIROS_Temporal_Vision_Engine_Development.md` into a repo-native implementation path. The goal is to turn KAIROS from a temporal demo/workbench into a serious TACITUS backbone for large-text temporal vision, actor relationships, human friction, commitments, episodes, and contradictions.

## Product Target

KAIROS should ingest dense policy, legal, crisis, and geopolitical text and return a computable temporal scene:

- what happened
- when it happened
- who was involved
- what commitments were made
- where relationships became tense
- which episodes contain or overlap other episodes
- which claims are contradictory or stale
- what downstream TACITUS agents should know before reasoning

The MVP should prove this on a 10-15 page dossier in under 60 seconds, with evidence spans and strict JSON output.

## Build Phase 1: Evidence And Friction Core

Priority: immediate.

### Data Model

Add first-class source-span and friction types to `kairos-core`:

```rust
pub struct SourceSpan {
    pub doc_id: Option<String>,
    pub char_start: usize,
    pub char_end: usize,
    pub text: String,
}

pub enum FrictionKind {
    CommitmentFailure,
    ProceduralObstruction,
    InstitutionalDrift,
    TrustLoss,
    PolicyDivergence,
    LeadershipTransition,
    LegalBlockage,
    ImplementationGap,
}

pub enum FrictionTrajectory {
    Escalating,
    Resolving,
    Freezing,
    Mutating,
}

pub struct Friction {
    pub id: String,
    pub kind: FrictionKind,
    pub actors_involved: Vec<String>,
    pub triggering_event: Option<String>,
    pub interval: TemporalInterval,
    pub evidence_spans: Vec<SourceSpan>,
    pub intensity: f32,
    pub confidence: f32,
    pub commitment_ids: Vec<String>,
    pub episode_ids: Vec<String>,
    pub trajectory: FrictionTrajectory,
    pub summary: String,
}
```

### Extraction

Add a friction extractor beside ACO extraction:

- input: source text, events, actors, commitments, episodes
- output: `Vec<Friction>`
- mode: mock fixture plus Gemini structured output
- validation: intensity/confidence clamped to `0.0..=1.0`, empty actor IDs removed, empty evidence rejected

### API

Extend `AnalysisResult` with:

- `frictions: Vec<Friction>`

Extend diagnostics with:

- `friction_count`
- `escalating_friction_count`
- `high_intensity_friction_count`
- `friction_by_kind`

### UI

Add a "Friction Map" panel:

- red/orange rows for escalating friction
- kind, actors, trajectory, intensity, evidence text
- filters for escalation, obstruction, drift, trust loss

Acceptance:

- The Meridian demo returns at least 5 friction objects.
- At least 2 are linked to commitments.
- UI shows friction without needing raw JSON.
- Response still never leaks Gemini keys.

## Build Phase 2: DCT And Relative-Time Resolution

Priority: immediate after friction.

### Request Contract

Extend `AnalysisRequest`:

```rust
pub document_id: Option<String>,
pub document_created_at: Option<DateTime<Utc>>,
```

Behavior:

- if `document_created_at` is present, resolve relative dates against it
- if absent, keep unresolved relative dates as warnings
- preserve existing request compatibility

### Date Extractor

Add support for:

- yesterday / tomorrow
- last week / next week
- next quarter
- by Friday
- two days later
- within 45 days
- after the review window

Acceptance:

- deterministic tests for DCT anchored expressions
- diagnostics report unresolved relative dates
- demo includes at least three relative-time examples

## Build Phase 3: Large-Text Pipeline

Priority: MVP+ backbone.

### Chunking Strategy

Add a large-text analysis path:

- 10 pages: single-shot global extraction
- 100 pages: chunk map-reduce with temporal overlap
- 1,000 pages: structural chunks plus temporal graph merge

Implementation shape:

```rust
pub struct DocumentChunk {
    pub doc_id: String,
    pub chunk_id: String,
    pub char_start: usize,
    pub char_end: usize,
    pub text: String,
    pub prior_context: Vec<String>,
}
```

Rules:

- split on headings/paragraphs before token count
- carry trailing events and dates into the next chunk
- merge duplicate events by actor, date, lemma, and evidence similarity
- keep contradictions rather than overwriting prior claims

Acceptance:

- 15-page synthetic fixture completes under 60 seconds in mock mode
- no chunk drops source text
- every event carries `doc_id` and source span

## Build Phase 4: Contradiction And TLINK Validator

Priority: after evidence spans exist.

### Deterministic Checks

Add contradiction objects:

```rust
pub enum ContradictionKind {
    Cycle,
    InvertedInterval,
    CauseAfterEffect,
    CommitmentBeforePromise,
    OverlappingMutuallyExclusiveStates,
}

pub struct Contradiction {
    pub id: String,
    pub kind: ContradictionKind,
    pub entity_ids: Vec<String>,
    pub explanation: String,
    pub evidence_spans: Vec<SourceSpan>,
    pub severity: f32,
}
```

Checks:

- episode end before start
- event cycle in `Before` graph
- commitment deadline before commitment date
- causal language where effect precedes cause
- mutually exclusive actor states that overlap

Acceptance:

- synthetic contradiction fixtures are caught deterministically
- diagnostics includes contradiction counts and severity
- UI warning panel links contradiction to source spans

## Build Phase 5: Graph Core And Performance

Priority: after product behavior is stable.

### Graph Module

Create an internal graph layer without over-refactoring the current workspace:

- `graph/node.rs`
- `graph/edge.rs`
- `graph/temporal_graph.rs`
- `graph/fusion.rs`

Use stable string IDs first. Only introduce `slotmap` after graph mutation pressure is real.

Graph node kinds:

- document
- source_span
- actor
- event
- timex
- commitment
- friction
- episode
- contradiction

Graph edge kinds:

- mentions
- involves
- triggers
- commits_to
- violates
- contained_by
- before / after / overlaps / during
- disputed_by

Acceptance:

- graph export endpoint returns nodes/edges
- relation computation remains under 100 ms for the demo fixture
- graph export can be consumed by downstream TACITUS tools

## Prompt Pack To Implement

Add prompt templates as Rust constants or small template files:

1. Event and TIMEX extraction with DCT.
2. Actor, commitment, and relationship extraction.
3. Friction extraction.
4. Episode detection.
5. Contradiction review.
6. Temporal brief generation.

Prompt requirements:

- ask for evidence spans
- use shallow schemas for Gemini compatibility
- include allowed enum values
- require `null` for unknowns instead of hallucinated dates
- forbid hidden inferences not grounded in text

## Evaluation Plan

Add `examples/eval/` fixtures:

- `relative_dates.md`
- `friction_commitment_failure.md`
- `contradiction_cycle.md`
- `large_dossier_15_pages.md`
- `actor_aliases.md`

Add tests:

- DCT relative date resolution
- friction schema validation
- contradiction detection
- no-key-leak serialization
- large fixture smoke in mock mode

MVP metrics:

- at least 90% precision on explicit date extraction in fixtures
- at least 5 friction objects in the Meridian demo
- at least 3 commitment links to friction
- 15-page mock fixture under 60 seconds locally
- public smoke test passes after deploy

## Implementation Order

1. Source spans on dates/events/commitments/episodes.
2. Friction schema, mock extractor, Gemini schema extractor.
3. UI friction map and diagnostics.
4. DCT fields and relative date resolver.
5. Contradiction schema and deterministic checks.
6. Large-text chunk structs and mock large-dossier path.
7. Graph export endpoint.
8. Evaluation fixtures and CI/smoke gates.

## Explicit Non-Goals For This Pass

- PyO3 bindings.
- WASM graph solver.
- Full TimeML XML import/export.
- Full cross-document coreference over hundreds of documents.
- Predictive forecasting.
- GPU/WebGPU graph processing.

These are useful research bets, but the immediate product needs evidence-backed temporal extraction, friction capture, contradiction warnings, and large-text readiness first.
