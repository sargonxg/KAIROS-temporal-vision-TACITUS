# KAIROS Roadmap

What's next for KAIROS as a **standalone temporal engine** and as the **temporal scaffold** of the TACITUS trinity ([DIALECTICA](https://github.com/sargonxg/A2_DIALECTICAbyTACITUS) + [AGON](https://github.com/sargonxg/AGON) + KAIROS).

> **Practical posture.** Standalone-first. Trinity integration is additive. No big-bang rewrites.

---

## North star

KAIROS is the layer that, given any policy text, says:

> "Here is when each event happened, with what precision. Here are the intervals they live in. Here is what commitment is still pending, and what relation it has to every other event in the document."

When that's true across messy real dossiers — not just the Meridian Compact demo — KAIROS becomes the temporal substrate every TACITUS product depends on.

---

## Phase A — Hardening the standalone product (next 4-6 weeks)

### A1 — Date extraction precision

Current state: dates work well on structured prose. Edge cases miss.

- [ ] Improve relative-date resolution (`three weeks later`, `the following spring`, `in Q2`)
- [ ] Better handling of date ranges (`January through March 2024`, `Q3 2023`)
- [ ] Fiscal year vs calendar year detection
- [ ] Fuzziness scoring more conservative — flag uncertain dates as `FUZZ_UNRESOLVED` rather than guess
- [ ] Cross-cultural date formats (DD/MM/YYYY vs MM/DD/YYYY) inferred from document metadata

Acceptance: 100 golden examples covering edge cases, ≥95% correct resolution.

### A2 — Commitment state machine

Today: commitments extracted; state transitions weakly modeled.

- [ ] Explicit state transitions: `PLEDGED → ACTIVE → FULFILLED | BREACHED | CONTESTED | WITHDRAWN`
- [ ] Deadline tracking: commitment with passed deadline + no fulfillment evidence → `BREACHED`
- [ ] Multi-party commitments (A commits to B and C)
- [ ] Conditional commitments ("if X, then Y")
- [ ] Evidence anchors per state transition

### A3 — Allen-13 relation pruning

Today: relations are computed; some are redundant or low-signal.

- [ ] Minimal-relation algorithm: emit only the relations needed to reconstruct the full partial order
- [ ] Confidence on each relation (derived from date fuzziness of endpoints)
- [ ] Optional: full set vs minimal set behind request param

### A4 — Episode detection improvements

- [ ] Boundary anchoring: every episode must have at least one anchor event
- [ ] Episode types: `crisis`, `negotiation`, `enforcement`, `recovery` (vocabulary TBD)
- [ ] Sub-episode hierarchy (an enforcement episode contains a court ruling sub-episode)

### A5 — Quality + observability

- [ ] Structured JSON logs with `trace_id`, `document_id`, `event_count`, `unresolved_date_count`
- [ ] Prometheus `/metrics` endpoint
- [ ] Diagnostics shown in workbench UI

---

## Phase B — Trinity integration (in parallel, weeks 3-8)

These tasks make KAIROS callable from DIALECTICA's pipeline. See DIALECTICA's [`docs/integration/`](https://github.com/sargonxg/A2_DIALECTICAbyTACITUS/tree/main/docs/integration) for full context.

### B1 — Adopt `tacitus-contracts`

- [ ] Add `tacitus-contracts` Cargo dependency
- [ ] Map `kairos-core::Event` → `tacitus_contracts::Event` (with `Fuzziness`, `Timestamp`)
- [ ] Map `kairos-core::Actor` → `tacitus_contracts::Actor`
- [ ] Map `kairos-core::Commitment` → `tacitus_contracts::Commitment` (with `CommitmentState`)
- [ ] Map `kairos-core::AllenRelation` → `tacitus_contracts::AllenRelation`
- [ ] Emit `AnalysisEnvelope` in `/api/v1/analyze` response with `contracts_version`
- [ ] Honor `X-Trace-Id` request header

### B2 — `POST /api/v1/analyze` contract stability

- [ ] Lock request/response schema for `v1.0.0` contracts
- [ ] Document in [`docs/INTEROP.md`](docs/INTEROP.md) (planned)
- [ ] Idempotency: same `document_id` + content hash returns cached result
- [ ] Streaming variant: `POST /api/v1/analyze/stream` emits SSE per pipeline stage (lets DIALECTICA show progress)

### B3 — Compatibility checks

- [ ] Service emits `contracts_version` in envelope
- [ ] CI test matrix runs against published `tacitus-contracts` versions

### B4 — IAM and internal-only mode

- [ ] Cloud Run variant `kairos-internal` accepts traffic only from DIALECTICA service account
- [ ] Drop Basic Auth on internal variant (IAM handles)
- [ ] Public demo variant kept separate

### B5 — Optional gRPC server

- [ ] `tonic`-based gRPC server alongside Axum REST
- [ ] Same contracts, lower-latency for in-pipeline calls
- [ ] Behind cargo feature `grpc`

---

## Phase C — Capability deepening (weeks 9-20)

### C1 — Persistent multi-document memory

Today's KAIROS is single-document. Real institutional analysis spans corpora.

- [ ] `corpus` aggregate: collection of documents under one analysis
- [ ] Storage layer (Postgres or similar) for persisted events, actors, commitments
- [ ] Idempotent re-ingestion (re-running same doc updates rather than duplicates)
- [ ] Per-document confidence carried through corpus-level views

### C2 — True corpus-level graph merge

- [ ] Cross-document actor coreference (deterministic + embedding-assisted)
- [ ] Event deduplication ("same action reported in two cables")
- [ ] Episode merging when boundaries align across documents
- [ ] Conflict resolution rules: which source wins on disagreement

### C3 — Trained local date NER

- [ ] Currently dates extracted by Gemini + rules. Train a small Rust-callable NER model for the deterministic side.
- [ ] Reduces Gemini token cost on bulk ingestion
- [ ] Improves cold-start latency

### C4 — TimeML import/export

- [ ] TimeML standard parser
- [ ] Export KAIROS graph as TimeML for interop with academic NLP tools
- [ ] Import TimeML-annotated corpora for evaluation

### C5 — Hypothesis quality

Today: hypotheses are conservative but lack typing.

- [ ] Hypothesis types: `causal`, `correlational`, `temporal_anomaly`, `commitment_drift`
- [ ] Evidence grade per hypothesis (strong / moderate / weak)
- [ ] Counterfactual reasoning ("if commitment X had been fulfilled, would event Y have happened?")

---

## Phase D — Distribution + packaging (when there's a consumer)

- [ ] **PyO3 bindings.** `kairos-py` package: call `kairos-core` from Python. Useful for DIALECTICA in-process, notebooks, research.
- [ ] **WASM target.** `kairos-core` compiled to WASM for browser-side temporal analysis (privacy-preserving demos).
- [ ] **Library API.** Stable public Rust API for embedders, not just the HTTP service.
- [ ] **Crates.io publish.** `kairos-core` published as a versioned crate.

Don't ship until there's a consumer asking. PyO3 is most likely first.

---

## Phase E — Enterprise readiness (when there's a customer)

Same posture as AGON. Don't build speculatively.

- [ ] Persistent storage (Postgres or Spanner)
- [ ] RBAC + tenancy
- [ ] Audit log
- [ ] EU data residency
- [ ] SLA tier

---

## What's NOT on the roadmap (intentional)

- **General-purpose temporal reasoning.** KAIROS stays conflict-domain-focused. No tax of "should we extract any temporal pattern?"
- **Replacing Allen-13.** The interval algebra is well-defined; no need to invent a new one.
- **Verdicts or recommendations.** KAIROS outputs structure. Decision-making lives in PRAXIS or human review.
- **Custom LLM training.** Use Gemini for now; train only narrow Rust sensors (date NER, episode detector) where it's clearly cheaper or better.
- **Heavy frontend.** Workbench stays small. Rich UI is PRAXIS's job.

---

## Success criteria per phase

| Phase | Done when |
|---|---|
| A | Golden test suite hits ≥95% precision/recall on dates, commitments, Allen relations |
| B | DIALECTICA pipeline calls KAIROS in production behind feature flag, p95 < 15s |
| C | Multi-document corpora reasoning ships; cross-doc coreference accuracy validated on benchmark |
| D | At least one external consumer ships with `kairos-core` (Python or WASM) |
| E | First enterprise customer in production |

---

## Open questions

1. **Should KAIROS own actor canonicalization within a document?**
   Recommendation: yes, within a document. Cross-document canonicalization is a corpus-level concern; let DIALECTICA workspace registry be authoritative for cross-doc identity.

2. **Allen-13 minimal vs full relation set.**
   Full set is information-rich but redundant. Minimal is harder to compute. Ship full set by default; add `?relations=minimal` query param for clients that prefer compactness.

3. **Episode vocabulary.**
   `crisis`, `negotiation`, etc. need to be defined. Risk: lock-in to one domain (geopolitics) and not fit others (workplace). Mitigation: keep vocabulary additive and per-subdomain.

4. **Persistent storage choice.**
   Postgres is simple. Spanner for multi-region. Decide when first customer arrives, not before.

5. **Hypothesis confidence calibration.**
   How conservative is conservative? Need a benchmark with human-rated hypothesis quality.

---

## How to contribute

- Pick a milestone above. Open an issue first.
- Conventional commits.
- All PRs must pass: `cargo fmt --all --check`, `cargo test --release`, `cargo build --release`, `node --check web/app.js`, smoke script green.
- Golden fixture tests for any extraction or relation change.

---

*Maintained by TACITUS. Updated as phases close.*
