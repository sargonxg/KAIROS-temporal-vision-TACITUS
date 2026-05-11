# KAIROS Capability Map

This map is the working contract for turning KAIROS from an MVP into a temporal backbone for TACITUS products.

## Live In This Repo

| Capability | Status | Evidence |
|---|---:|---|
| Regex date extraction | Live | `DateExtractor` returns character spans, resolved timestamps, and fuzziness. |
| Temporal event extraction | Live | Gemini/Ollama prompt path plus deterministic mock path. |
| ACO actors and commitments | Live | Extracts `Actor` and `Commitment` primitives; other ACO primitives are modeled for expansion. |
| Episode proposals | Live | `EpisodeDetector` trait and `LlmJudgeDetector` implementation. |
| Episode reconciliation | Live | Current reconciler sorts and normalizes proposals into `Episode` entities. |
| Allen-13 relation graph | Live | Pairwise relation computation over all episodes. |
| Embedded UI | Live | Annotated text, timeline, ACO panel, relation table, JSON export. |
| Cloud Run deployment | Live | `Dockerfile` and `deploy.sh`. |
| No-key demo mode | Live | `KAIROS_LLM=mock` returns a complex deterministic policy scenario. |

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

## Product Integration Pattern

KAIROS should sit behind TACITUS products as a temporal interpretation layer:

```text
product document / case file
  -> KAIROS analysis
  -> episodes + relation graph + ACO commitments
  -> product-specific reasoning, retrieval, dashboards, or agents
```

PRAXIS can use it to distinguish crisis windows from implementation windows. DIALECTICA can use it to ground deliberation in temporal order. TACITUS public demos can use it to show why policy reasoning needs time-aware structure rather than flat summaries.
