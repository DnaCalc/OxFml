# IN_PROGRESS_FEATURE_WORKLIST.md — OxFml

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-15.

## Status Vocabulary

- `in-progress`: partial implementation exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Formula Grammar, Parse, and Bind

- **Status**: in-progress
- **Current floor**: architectural baseline plus first exercised implementation slices in place for formula source records, tokenization, green syntax, red projections, a small expression parser subset, normalized reference ADTs, bind fixture scaffolding, semantic-plan compilation with a first helper-environment profile for `LET`/`LAMBDA`/invocation shape and lexical-capture need, narrow prepared-call/result lowering with blankness and caller-context provenance plus helper/scalarization prepared-call traces, and first local evaluation semantics for `_xlfn.SINGLE`, `LET`, symbolic `LAMBDA` values, immediate/helper-bound `LAMBDA` invocation, and lexical helper capture that survives shadowing on top of a narrow OxFunc function registry.
- **Remaining gaps**: broader grammar definition, richer parser coverage, bind/reference normalization breadth, incremental reuse mechanics, broader OxFunc catalog coverage, richer prepared-call/result provenance families, broader helper-form semantics beyond the first callable/lexical-capture baseline, and replay-backed evidence beyond local fixture scaffolding.
- **Why still open**: implementation has started under `W002` and `W003`, but still only for a narrow parser/binder/semantic-plan subset and not yet with pack-grade replay or checked formal evidence.
- **Canonical owner**: `W001` now; active follow-on `W002` and `W003`; planned `W008`.

### IP-02: FEC/F3E Evaluator Session

- **Status**: in-progress
- **Current floor**: OxFml-owned seam design and implementation-start planning baseline is in place, with exercised accepted-candidate, commit-bundle, reject-record, fence snapshots, typed no-publish fence rejection, single-formula host recalc wiring, and a first managed `prepare -> open_session -> capability_view -> execute -> commit` session-service slice with abort/expire handling, invalid-phase structural-conflict rejection, and surfaced execution-restriction effect facts.
- **Remaining gaps**: broader contention/concurrency runtime behavior, pack-grade replay/model artifacts, and broader host integration beyond the single-formula proving path.
- **Why still open**: the seam baseline now has a narrow managed evaluator-session surface, but not the broader Stage 2 runtime semantics or pack-grade evidence.
- **Canonical owner**: `W001` now; active follow-on `W004`; planned `W007` and `W008`.

### IP-03: Commit Output Contract

- **Status**: in-progress
- **Current floor**: atomic bundle, schema, and fixture-planning baseline exist in OxFml-owned docs, and a first exercised implementation slice now constructs commit bundles from accepted candidate results under matching fences and rejects mismatched fences with typed no-publish outcomes.
- **Remaining gaps**: broader commit bundle construction, open-session-to-commit runtime wiring, richer topology-fact encoding, and replay evidence.
- **Why still open**: a minimal commit boundary slice now exists under `W004`, but it does not yet represent the full evaluator publication pipeline or replay-grade coverage.
- **Canonical owner**: `W001` now; active follow-on `W004`; planned `W008`.

### IP-04: Reject Taxonomy and Trace Schema

- **Status**: in-progress
- **Current floor**: reject and trace taxonomy, minimum schemas, and formal/replay planning baseline exist, plus exercised typed reject records for fence mismatch, capability denial, abort, and expire paths; local replay fixtures for semantic-plan, prepared-call/result, execution-contract, session lifecycle, FEC commit/reject, and single-formula host slices; and first local Lean/TLA+ skeleton artifacts.
- **Remaining gaps**: broader typed reject coverage, pack-grade deterministic replay infrastructure, and checked Lean/TLA+ coupling.
- **Why still open**: the witness and formal floor is now broader, but still local and not yet promoted into Green-owned packs or checked proof/model artifacts.
- **Canonical owner**: `W001` now; active follow-on `W004` and `W005`; planned `W007`.

### IP-05: Formula-Semantic Formatting

- **Status**: in-progress
- **Current floor**: formatting behavior crossing the seam is chartered and now exercised through `TEXT`, `VALUE`, `NOW`, `TODAY`, `CELL`, and `INFO` with explicit locale-format and host-query context, prepared-result format/publication hints, and first locale format-dependency facts surfaced through the proving host.
- **Remaining gaps**: broader semantic formatting family coverage, richer publication consequences, and pack-grade proving scenarios.
- **Why still open**: `W006` closed its initial baseline, but the repo-level feature remains much broader than that first formatting/host-query slice.
- **Canonical owner**: active `W006`.
