# KAIROS Interop

How KAIROS exposes itself to other TACITUS services (especially DIALECTICA) and how it adopts the shared `tacitus-contracts` types.

> **Status — be honest.** This is a **specification + intent document**. KAIROS's wire format is stable today via `POST /api/v1/analyze`. The `tacitus-contracts` package referenced here is **not yet published**; KAIROS-internal types in `kairos-core` are still the source of truth. This doc declares how the migration will land.
>
> Paired with DIALECTICA's [`docs/integration/CONTRACTS.md`](https://github.com/sargonxg/A2_DIALECTICAbyTACITUS/blob/main/docs/integration/CONTRACTS.md) — that's the conductor-side spec; this is the KAIROS-side mirror.

---

## Why this doc

KAIROS is one of three repos in the [TACITUS trinity](https://github.com/sargonxg/A2_DIALECTICAbyTACITUS/tree/main/docs/integration). It runs standalone (paste-and-analyze demo, `kairos-server` binary) and as the **temporal scaffold layer** in DIALECTICA's extraction pipeline.

For trinity integration to work without rot:

1. KAIROS's wire format is **versioned and stable**.
2. KAIROS's internal types map cleanly to **shared contracts** that DIALECTICA + AGON also use.
3. KAIROS honors **cross-service conventions** (trace propagation, idempotency, contracts version negotiation).

This doc records all three.

---

## Wire surface

### Public endpoints (today)

| Endpoint | Purpose | Stability |
|---|---|---|
| `POST /api/v1/analyze` | Full temporal analysis pipeline | Stable |
| `POST /api/v1/graph` | Graph-only export | Stable |
| `POST /api/v1/validate` | Validate an existing graph payload | Stable |
| `POST /api/analyze` | Legacy alias for `/api/v1/analyze` | Deprecated, kept for one release |
| `GET  /healthz` | Liveness probe | Stable; **public even when Basic Auth enabled** |

### Versioning plan

- **`/api/v1/*`** is the explicitly-versioned namespace. Already in place.
- **Envelope with `contracts_version`** field lands once `tacitus-contracts@1.0.0` is published (target Phase B per [`../ROADMAP.md`](../ROADMAP.md)).
- **Streaming variant** `POST /api/v1/analyze/stream` (planned) emits SSE per pipeline stage; useful for DIALECTICA progress UI.

### Request envelope (target shape, post-`tacitus-contracts@1.0`)

```json
{
  "envelope": {
    "contracts_version": "1.0.0",
    "trace_id": "<propagated>",
    "workspace_id": "ws_018f...",
    "document_id": "doc_018f...",
    "source_service": "dialectica"
  },
  "text": "...",
  "document_created_at": "2024-03-12T00:00:00Z",
  "analysis_mode": "auto",
  "gemini_api_key": null,    // optional request-scoped override
  "gemini_model": null,
  "options": {
    "compute_allen_relations": true,
    "detect_episodes": true,
    "minimal_relations": false
  }
}
```

### Response envelope (target shape)

```json
{
  "envelope": {
    "contracts_version": "1.0.0",
    "trace_id": "<echoed>",
    "produced_at": "2026-05-12T22:00:00Z",
    "source_service": "kairos"
  },
  "pre_read":        { /* segments, markers, density */ },
  "dates":           [/* Date[] with fuzziness + spans */],
  "events":          [/* Event[] with timestamps + Allen anchors */],
  "actors":          [/* Actor[] */],
  "actor_registry":  { /* canonical → aliases */ },
  "commitments":     [/* Commitment[] */],
  "frictions":       [/* friction objects */],
  "hypotheses":      [/* cautious inferences */],
  "episodes":        [/* Episode[] with boundaries */],
  "relations":       [/* AllenRelation[] */],
  "source_index":    { /* object → span / segment lookup */ },
  "graph":           { /* typed nodes + edges */ },
  "graph_summary":   { /* counts by type */ },
  "diagnostics":     { /* warnings, confidence, unresolved */ }
}
```

The full output schema is reachable at runtime via `GET /api/schema` once that endpoint is added (planned).

---

## Internal types → `tacitus-contracts` mapping

KAIROS's `kairos-core` crate is the source of truth today. Migration to `tacitus-contracts` is **additive**: shared types come in as Cargo dep, `kairos-core` retains internal helpers that don't belong in the contract.

### Identity types

| KAIROS `kairos-core` | `tacitus-contracts` | Notes |
|---|---|---|
| `ActorId(String)` | `tacitus_contracts::ActorId` | Format: `act_<uuid7>`. KAIROS-local IDs preserved as `Actor.merged_from_kairos` after fusion. |
| `EventId(String)` | `tacitus_contracts::EventId` | KAIROS is canonical event minter. Format: `evt_<uuid7>`. |
| KAIROS internal `episode_id` | `tacitus_contracts::EpisodeId` | Format: `epi_<uuid7>` |
| `DocumentId` | minted by caller (DIALECTICA) | KAIROS preserves verbatim |

### `Date` and `Timestamp`

| KAIROS field | Contracts field | Notes |
|---|---|---|
| `Date.iso8601` | `Timestamp.iso8601` | Preserves precision |
| `Date.resolution` (YEAR / MONTH / DAY / HOUR / MINUTE) | `DateResolution` enum | Direct |
| `Date.fuzziness` (EXACT / APPROX / RELATIVE / UNRESOLVED) | `Fuzziness` enum | Direct |
| `Date.span` | `SourceSpan` | See span mapping below |

### `Event` → `Event`

| KAIROS | Contracts | Notes |
|---|---|---|
| `Event.canonical_label` | `canonical_label: String` | |
| `Event.timestamp` | `Timestamp` | Optional |
| `Event.fuzziness` | `Fuzziness` enum | |
| `Event.participants[]` | `participants: ActorId[]` | KAIROS-local IDs at emit time; canonicalized at fusion |
| `Event.spans[]` | `spans: SourceSpan[]` | |
| (assigned by consumer) | `plover_type: Option<PloverType>` | DIALECTICA-owned vocabulary today; may move to contracts in v1.1 |
| (assigned by consumer) | `severity: Option<int>` | Same |

### `AllenRelation` → `AllenRelation`

Direct field-by-field map. KAIROS computes the full 13-relation set; consumers can request minimal set via `options.minimal_relations`.

| KAIROS | Contracts |
|---|---|
| `relation_type` (Before / After / Meets / ... / Equals) | `AllenType` enum (13 variants) |
| `subject` | `subject: oneof { event_id / episode_id }` |
| `object` | `object: oneof { obj_event / obj_episode }` |

### `Commitment` → `Commitment`

KAIROS commitments are temporally precise (deadline-aware, state-tracked). In trinity mode, KAIROS commitments win on conflict with AGON commitments. AGON variants are logged as alternative extractions, not silently dropped.

| KAIROS | Contracts |
|---|---|
| `committer: ActorId` | same |
| `target: ActorId` | same |
| `content: String` | same |
| `state` (Pledged / Active / Fulfilled / Breached / Contested / Withdrawn) | `CommitmentState` enum (6 variants) |
| `deadline: Option<Timestamp>` | same |
| `committed_at: Option<Timestamp>` | same |
| `evidence[]` | `evidence: SourceSpan[]` |

### `Episode`

KAIROS's `Episode` maps to DIALECTICA Conflict Grammar `Phase` node (alias `Episode` in v2.1). See DIALECTICA's [`ONTOLOGY_MAPPING.md`](https://github.com/sargonxg/A2_DIALECTICAbyTACITUS/blob/main/docs/integration/ONTOLOGY_MAPPING.md) for field-by-field map.

### Span mapping

| KAIROS `Span` | `tacitus_contracts::SourceSpan` |
|---|---|
| `start`, `end` (char offsets) | `char_start`, `char_end` |
| `quote` | `text` |
| `sha256` (planned — currently optional) | `sha256` |
| `confidence_marker` | `ConfidenceMarker` enum (EXACT / NORMALIZED / UNRESOLVED) |
| segmentation reference | `segment_id` (optional) |

**Action item:** `kairos-core` currently emits spans without consistent sha256. Phase B adds mandatory hashing.

---

## Cross-service conventions

### Trace propagation

KAIROS honors `X-Trace-Id` request header. If present:
- Echoed in `envelope.trace_id` of response
- Included in all structured logs for that request
- Propagated to downstream Gemini calls

If absent: KAIROS mints `trace_id` as `kairos_<uuid7>` and includes in response.

### Idempotency

`POST /api/v1/analyze` accepts optional `Idempotency-Key` header. Same key + same body within 24h returns cached result.

Cache key: `(document_id, sha256(text), analysis_mode, options, model)`.

### Compatibility check

Caller declares supported contracts range via `X-Contracts-Range: ">=1.0.0,<2.0.0"`. KAIROS responds with `envelope.contracts_version`. Caller handles mismatch (DIALECTICA logs warning + falls back to skipping KAIROS pre-pass).

### Request-scoped Gemini keys

KAIROS supports `gemini_api_key` in request body for per-call provider key. **Never logged, never persisted, never serialized in response.** Useful for browser-initiated analysis. Trinity-internal calls from DIALECTICA use service-account auth via Vertex AI instead.

### IAM-internal mode (planned)

Deployment variant `kairos-internal`:
- Cloud Run service accepting traffic only from DIALECTICA service account
- Basic Auth disabled (IAM gate is enough)
- Same wire contract
- Used by DIALECTICA's `temporal_scaffold` pipeline node in production

Public `kairos` continues with optional Basic Auth for demos.

---

## What KAIROS does NOT promise

- **No semantic interpretation.** KAIROS extracts time + commitment structure. Meaning/intent reasoning lives in DIALECTICA.
- **No silent guesses.** If a date can't be resolved, KAIROS marks `FUZZ_UNRESOLVED`; never fabricates a timestamp.
- **No cross-document state today.** Multi-document corpus mode is planned, not shipped. Until then, each `/api/v1/analyze` call is independent.
- **No persistence guarantees across versions.** Storage layer (when added) will be opaque. Consumers use spans + IDs, not internal state.
- **No real-time streams.** Streaming variant emits per-stage progress, not per-token. Unit of work remains one document.

---

## Where this doc evolves

| Trigger | Doc update |
|---|---|
| `tacitus-contracts@0.1.0` published | Fill in concrete imports, remove "target shape" caveats |
| `POST /api/v1/analyze/stream` shipped | Add SSE event schema |
| Multi-document corpus mode shipped | Add corpus-level surface |
| New episode subtypes added | Update mapping + bump contracts minor |
| First DIALECTICA pipeline call in production | Add consumer recipes |

---

## See also

- [`../README.md`](../README.md) — KAIROS product README
- [`../ROADMAP.md`](../ROADMAP.md) — phased plan (B1-B5 are this doc's work)
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — internal architecture
- [`CAPABILITY_MAP.md`](CAPABILITY_MAP.md) — what works today vs roadmap
- DIALECTICA [`docs/integration/CONTRACTS.md`](https://github.com/sargonxg/A2_DIALECTICAbyTACITUS/blob/main/docs/integration/CONTRACTS.md) — full shared-types spec
- DIALECTICA [`docs/integration/ONTOLOGY_MAPPING.md`](https://github.com/sargonxg/A2_DIALECTICAbyTACITUS/blob/main/docs/integration/ONTOLOGY_MAPPING.md) — how KAIROS outputs land in Conflict Grammar
- AGON [`docs/INTEROP.md`](https://github.com/sargonxg/AGON/blob/main/docs/INTEROP.md) — evidence-side mirror

---

*Maintained by TACITUS. Update when wire contracts change.*
