# KAIROS Capability Map

This map is the working contract for turning KAIROS from an MVP into a temporal backbone for TACITUS products.

Latest research source: `docs/research/KAIROS_Temporal_Vision_Engine_Development.md`.
Current deep-tech build plan: `docs/DEEP_TECH_MVP_BUILD_PLAN.md`.

## Live In This Repo

| Capability | Status | Evidence |
|---|---:|---|
| Regex date extraction | Live | `DateExtractor` returns character spans, resolved timestamps, and fuzziness. |
| Temporal event extraction | Live | Gemini/Ollama prompt path plus deterministic mock path. |
| ACO actors and commitments | Live | Extracts `Actor` and `Commitment` primitives; other ACO primitives are modeled for expansion. |
| Episode proposals | Live | `EpisodeDetector` trait and `LlmJudgeDetector` implementation. |
| Episode reconciliation | Live | Current reconciler sorts and normalizes proposals into `Episode` entities. |
| Allen-13 relation graph | Live | Pairwise relation computation over all episodes. |
| Analysis metadata | Live | Every result reports schema version, provider, model, mode, input size, and elapsed time. |
| Temporal diagnostics | Live | Relation counts, overlap pairs, open-ended episodes, deadline commitments, unresolved dates, warnings, and confidence summary. |
| Validation API | Live | `/api/v1/validate` computes diagnostics without another LLM call. |
| Embedded UI | Live | Timeline, diagnostics, temporal brief, actor lanes, annotated text, ACO panel, filtered relation table, JSON export. |
| Cloud Run deployment | Live | `Dockerfile` and `deploy.sh`. |
| Public Google Cloud front door | Live | External HTTP load balancer at `http://34.54.231.53` backed by a Cloud Run serverless NEG. |
| No-key demo mode | Live | `KAIROS_LLM=mock` returns a complex deterministic policy scenario. |
| Request-scoped Gemini testing | Live | UI and API accept a temporary Gemini key for one run; it is not saved or returned. |
| Source spans | Live | Dates and mock events now carry source spans; friction evidence is grounded in source text. |
| Human friction objects | Live | Mock and Gemini extraction paths produce typed friction objects with kind, trajectory, intensity, actors, commitments, episodes, and evidence. |
| DCT-relative time | Live | Optional `document_created_at` resolves relative expressions like `two days later` and `next quarter`. |

## MVP+ Implemented

| Capability | Status | Evidence |
|---|---:|---|
| Versioned API surface | Implemented | `/api/v1/analyze` aliases the stable analysis path. |
| Browser-only analyst corrections | Implemented | Users can mark episodes approved, modified, or rejected; exported JSON includes corrections. |
| Relation filters | Implemented | UI filters all, overlap, boundary, and high-signal relations. |
| Temporal brief | Implemented | UI summarizes episode count, key episode sequence, and commitment watch. |
| Smoke verification | Implemented | `scripts/kairos-smoke.ps1` checks local or live services end to end. |

## Backbone-Grade Next Steps

| Capability | Why It Matters | Implementation Path |
|---|---|---|
| Persistent session memory | Lets TACITUS products compare new briefs against prior cases. | Store `AnalysisResult` in GCS, SQLite-backed Cozo, or a shared TACITUS knowledge service. |
| Human boundary review | Converts model guesses into trusted institutional knowledge. | Add review APIs for approving, modifying, and rejecting episode boundaries. |
| Detector learning loop | Lets KAIROS improve from accepted corrections. | Save corrections, evaluate drift, and feed them into prompt examples or deterministic rules. |
| Multi-document merge | Real policy work spans cables, memos, reports, and news. | Add document IDs, source provenance, and cross-document actor/commitment coreference. |
| Episode query API | Other products need direct temporal questions. | Add endpoints like `episodes_at`, `relations_for_actor`, `commitments_before`, and `changes_between`. |
| Confidence and contradiction scoring | Policy users need trust posture, not just output. | Compare event evidence, temporal constraints, and model confidence into a structured warning layer. |
| Rich ACO extraction | Claims, interests, constraints, leverage, events, and narratives unlock deeper conflict reasoning. | Extend `aco.rs` and `extract_aco` with typed arrays for all 8 ACO primitives. |
| Calibration pack | Enables regression tests on real policy scenarios. | Keep a suite of gold demo cases under `examples/` with expected episode/relation counts. |
| Richer friction learning | Captures broken commitments, obstruction, trust loss, institutional drift, and escalation across many domains. | Persist analyst corrections and use accepted examples as prompt/regression fixtures. |
| Broader relative time | Large policy documents use "next quarter", "two weeks later", and review windows constantly. | Expand DCT rules to more calendars, fiscal quarters, weekday references, and language variants. |
| Large-text chunking | KAIROS must handle dossiers, not only pasted paragraphs. | Add structural chunking, temporal overlap, reduce-pass merging, and no-drop source-span accounting. |

## Product Integration Pattern

KAIROS should sit behind TACITUS products as a temporal interpretation layer:

```text
product document / case file
  -> KAIROS analysis
  -> episodes + relation graph + ACO commitments
  -> product-specific reasoning, retrieval, dashboards, or agents
```

PRAXIS can use it to distinguish crisis windows from implementation windows. DIALECTICA can use it to ground deliberation in temporal order. TACITUS public demos can use it to show why policy reasoning needs time-aware structure rather than flat summaries.
