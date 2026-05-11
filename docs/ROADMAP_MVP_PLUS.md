# KAIROS MVP+ Roadmap

This backlog converts the temporal-vision research brief into an implementation sequence for KAIROS as a TACITUS backbone component.

## Shipped In MVP+

- Versioned API alias: `/api/v1/analyze`.
- Graph validation API: `/api/v1/validate`.
- Analysis metadata and temporal diagnostics.
- Request-scoped Gemini key/model testing from the web screen.
- Schema-aware Gemini calls for events, ACO primitives, and episode detection.
- Workbench panels for diagnostics, temporal brief, actor lanes, relation filters, and analyst corrections.
- Smoke verification script for local and Cloud Run targets.

## Next 1 Week

1. **Provider-grade schema enforcement**
   - Add provider-specific schema adapters for Gemini, OpenAI, Anthropic, and Ollama/local models.
   - Acceptance: malformed LLM items are counted and surfaced in diagnostics without failing an entire analysis.

2. **Source-span provenance**
   - Add source spans to events, actors, commitments, and episode anchors.
   - Acceptance: every major output can point back to exact text.

3. **Temporal contradiction MVP**
   - Add a small Allen consistency checker for impossible interval claims and inverted episode intervals.
   - Acceptance: synthetic contradiction fixtures produce deterministic warnings.

4. **Demo screenshot and API examples**
   - Add public-facing screenshots and richer curl examples to README.
   - Acceptance: a new developer can run, test, and deploy without chat context.

## Next 2-4 Weeks

1. **Persistent session store**
   - Persist analysis results and analyst corrections in a small local/Cloud Run compatible store.
   - Acceptance: a session can be reloaded by ID.

2. **Human correction API**
   - Add endpoints to accept, modify, or reject episodes and relations.
   - Acceptance: corrections update diagnostics and export state.

3. **Benchmark harness**
   - Add deterministic temporal stress fixtures plus optional TempEval/MATRES import scripts.
   - Acceptance: CI reports extraction counts, contradiction detection, and graph latency.

4. **LLM cost and abuse guardrails**
   - Add input limits, request timing, clearer error messages, and optional per-IP rate limits.
   - Acceptance: public demo is hard to abuse accidentally.

## Next 1-2 Months

1. **Multi-document timeline fusion**
   - Add document IDs, event coreference, duplicate merging, and canonical timeline generation.
   - Acceptance: multiple reports collapse into one explainable episode graph.

2. **Actor commitment ledger**
   - Track actor promises, deadlines, fulfillment state, and review windows as first-class temporal objects.
   - Acceptance: API can answer “what is due, by whom, by when?”

3. **Temporal RAG interface**
   - Export timeline-aware context packs for PRAXIS, DIALECTICA, and other TACITUS agents.
   - Acceptance: downstream products can retrieve by episode, valid time, relation, and actor.

4. **Python and WASM surfaces**
   - Add PyO3 bindings for analyst workflows and WASM for browser-side graph validation.
   - Acceptance: core temporal graph logic is reusable outside the server.

## Research Bets

- Full TimeML/TIMEX3/TLINK import and export.
- Advanced Allen CSP propagation beyond pairwise relation computation.
- Episodic memory learning from accepted corrections.
- Cross-document event coreference over large policy corpora.
- Policy-window detection from temporal clusters and commitment drift.

## Risks To Track

- LLM hallucinated chronology.
- Ambiguous relative dates without a document creation time.
- Dense timelines causing UI overload.
- Public demo cost exposure when users provide server-side keys.
- Over-claiming research capabilities before they are backed by tests and live behavior.
