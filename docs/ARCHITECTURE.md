# KAIROS Architecture

KAIROS is a temporal-vision layer for policy and institutional analysis. The goal is the same kind of jump that computer vision gave robotics: prose becomes an inspectable scene, but the scene is made of dates, actors, commitments, boundaries, episodes, and relations through time.

## Rust Crates

`kairos-core` is the library. It owns the data model, extraction pipeline, episode detector interface, reconciliation, Allen-13 relation algebra, and embedded store.

`kairos-server` is the deployable binary. It wraps `kairos-core` with Axum routes and embeds the static UI with `rust-embed`.

This split matters because TACITUS products can reuse the library without inheriting the demo server. PRAXIS, DIALECTICA, or future policy systems should call `Kairos::analyze` directly or wrap it behind their own service boundary.

## Core Pipeline

```text
AnalysisRequest
  text
    |
    +--> DateExtractor                       pure Rust regex dates
    |
    +--> extract_events                      LLM or mock temporal events
    |
    +--> extract_aco                         LLM or mock actors/commitments
    |
    +--> LlmJudgeDetector                    episode proposals
    |
    +--> reconcile                           ordered episode entities
    |
    +--> relations::compute                  Allen-13 pairwise relation graph
    |
    +--> AnalysisResult                      JSON for tools and UI
```

The MVP intentionally uses three model-facing passes: events, ACO, and episodes. Dates and Allen relations stay deterministic because they are safety-critical. This creates a strong hybrid contract: the model proposes structure, Rust validates and relates time.

## Temporal Model

KAIROS distinguishes:

- `ValidityInterval`: when a fact is true in the world.
- `TransactionInterval`: when KAIROS recorded the fact.
- `Bitemporal<T>`: a value plus validity, transaction time, provenance, and confidence.
- `TemporalInterval`: the interval used by episodes.
- `Episode`: a coherent period with a kind, title, interval, boundaries, anchors, narrative, confidence, and review state.
- `EpisodeRelation`: the Allen-13 relation between two episodes.

This is the foundation for policy-grade time reasoning. A model can ask not only "what happened?" but "what was believed then, what was true then, what changed later, and what episode did that belong to?"

## Allen-13

Allen-13 gives KAIROS a compact temporal grammar:

`before`, `after`, `meets`, `met by`, `overlaps`, `overlapped by`, `starts`, `started by`, `during`, `contains`, `finishes`, `finished by`, and `equals`.

Policy work needs this because many important claims are not simple sequences. A sanction can overlap a negotiation. A leadership tenure can contain a crisis. A review window can meet an implementation phase. A legal ruling can interrupt, but not end, an enforcement episode.

## Learning Path

The MVP does not persist long-term memory across requests. It is deliberately stateless for easy Cloud Run deployment.

The learning backbone should be layered next:

- Persist `AnalysisResult` sessions to GCS, SQLite-backed Cozo, or a TACITUS knowledge service.
- Store human edits to episode boundaries and review states.
- Build a calibration set from accepted/rejected episode proposals.
- Add retrieval over prior cases so future extractions can compare against similar policy episodes.
- Promote repeated patterns into detector rules before asking the LLM.

The current code is structured so those additions fit behind `KairosStore`, `EpisodeDetector`, and the existing `AnalysisResult` contract.

## Deployment Shape

The demo deploys as one Cloud Run service:

- no external database;
- one static binary;
- `KAIROS_LLM=mock`, `gemini`, or `ollama`;
- optional Secret Manager wiring for `GEMINI_API_KEY`;
- Artifact Registry + Cloud Build for reproducible container builds.

That makes it easy to embed in TACITUS systems as a standalone service now, and as a library later.
