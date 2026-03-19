# IN_PROGRESS_FEATURE_WORKLIST.md — OxFml

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-19.

## Status Vocabulary

- `in-progress`: partial implementation exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Formula Grammar, Parse, and Bind

- **Status**: in-progress
- **Current floor**: architectural baseline plus exercised implementation slices for formula source records, tokenization, green syntax, red projections, a widened expression parser subset including additional qualified-name handling, normalized reference ADTs, bind fixture scaffolding with richer assertions, host-path incremental parse/red/bind reuse, semantic-plan compilation with helper-environment profiling, stage-aware availability summaries, external library-context snapshot refs, narrower per-surface library-context fields (`surface_stable_id`, `name_resolution_table_ref`, `semantic_trait_profile_ref`, `gating_profile_ref`), prepared-call/result lowering with blankness, caller-context provenance, typed callable carriers plus callable-profile detail, helper/scalarization prepared-call traces, and local evaluation semantics for `_xlfn.SINGLE`, `LET`, callable `LAMBDA`, exact free-helper lexical capture, adopted defined-name callable transport, `ROW`, `COLUMN`, `INDIRECT`, `OFFSET`, and `IFERROR` on top of a still-narrow OxFunc function registry.
- **Remaining gaps**: fuller Excel grammar closure, richer structured/external reference coverage, broader OxFunc catalog coverage, final shared callable transport, higher-order callable seam evidence beyond the current `LET` / `LAMBDA` floor, and replay-backed evidence beyond the current local witness tier.
- **Why still open**: `W032` has materially narrowed the minimum library-context and callable-carrier floor, but the repo-level feature remains broader than the exercised subset and still lacks pack-grade replay, fuller catalog breadth, broader higher-order callable evidence, and broader formal closure.
- **Canonical owner**: `W001` now; exercised follow-on `W002`, `W003`, `W013`, `W014`, `W019`, `W020`, `W026`, `W027`, and `W031`; active next owner `W032`; planned follow-on owners `W036`, `W037`, `W038`, and `W040`.

### IP-02: FEC/F3E Evaluator Session

- **Status**: in-progress
- **Current floor**: OxFml-owned seam design and exercised implementation now include accepted-candidate, commit-bundle, reject-record, fence snapshots, typed no-publish fence rejection, single-formula host recalc wiring, a managed `prepare -> open_session -> capability_view -> execute -> commit` session-service slice with abort/expire handling, invalid-phase structural-conflict rejection, surfaced execution-restriction effect facts, runtime contention enforcement across sessions, async-coupled external-provider consequence surfacing, runtime-async overlay registration, and checked local formal artifacts for the external capability gate.
- **Remaining gaps**: broader async/distributed runtime behavior beyond the local external-provider and contention floor, pack-grade replay/model artifacts, and broader host integration beyond the single-formula proving path.
- **Why still open**: `W029` materially widened the local async-facing runtime and formal floor, but repo-level runtime scope still extends beyond the exercised local contention and external-provider model.
- **Canonical owner**: `W001` now; exercised follow-on `W004`, `W015`, `W018`, `W021`, `W024`, and `W029`; planned next owners `W034` and `W035`.

### IP-03: Commit Output Contract

- **Status**: in-progress
- **Current floor**: atomic bundle, schema, and fixture-planning baseline exist in OxFml-owned docs, and the exercised implementation now constructs commit bundles from accepted candidate results under matching fences, derives seam-significant `format_delta` and `display_delta` from prepared-result hints where applicable, rejects mismatched fences with typed no-publish outcomes, and surfaces typed dependency consequence facts inside `topology_delta`.
- **Remaining gaps**: broader commit bundle construction beyond the current local publication families, wider distributed publication policy, and pack-grade replay evidence.
- **Why still open**: `W028` materially widened the local publication and topology floor, but the repo-level feature still does not represent the full evaluator publication pipeline or pack-grade coverage.
- **Canonical owner**: `W001` now; exercised follow-on `W004`, `W015`, `W017`, `W018`, `W021`, `W023`, and `W028`; planned next owner `W034`.

### IP-04: Reject Taxonomy and Trace Schema

- **Status**: in-progress
- **Current floor**: reject and trace taxonomy, minimum schemas, and formal/replay planning baseline exist, with exercised typed reject records for fence mismatch, capability denial, abort, expire, and contention-sensitive paths; local replay fixtures for semantic-plan, prepared-call/result, execution-contract, session lifecycle, FEC commit/reject, single-formula host, and empirical-oracle slices; broadened local reduced-witness artifacts; local normalized replay bundles; plus a checked local Lean run and checked local TLA+ model run.
- **Remaining gaps**: broader typed reject coverage, pack-grade deterministic replay infrastructure, and broader formal families beyond the first checked runs.
- **Why still open**: `W022` and `W023` materially widened the local witness/formal floor, but the evidence remains local and not yet promoted into pack-grade corpus or wider formal coverage.
- **Canonical owner**: `W001` now; exercised follow-on `W004`, `W005`, `W015`, `W016`, `W017`, `W022`, and `W023`; planned next owners `W033`, `W034`, and `W035`.

### IP-05: Formula-Semantic Formatting

- **Status**: in-progress
- **Current floor**: formatting behavior crossing the seam is chartered and exercised through `TEXT`, `VALUE`, `NOW`, `TODAY`, `CELL`, and `INFO` with explicit locale-format and host-query context, prepared-result format/publication hints, locale format-dependency facts surfaced through the proving host, seam-significant `format_delta` and `display_delta` publication artifacts, and empirical-oracle scenarios covering formatting and host-query lanes.
- **Remaining gaps**: broader semantic formatting family coverage, fuller display-boundary closure beyond the current seam-significant subset, and pack-grade proving scenarios.
- **Why still open**: `W030` materially widened the local semantic-format and display-boundary floor, but the repo-level feature remains much broader than the exercised slice.
- **Canonical owner**: exercised follow-on `W006`, `W014`, `W018`, `W020`, `W021`, `W024`, `W030`, and `W031`; planned next owners `W036` and `W039`.

### IP-06: Replay Appliance Adapter and Witness Governance

- **Status**: in-progress
- **Current floor**: OxFml-local replay adapter governance is written into the canonical spec set, including the adapter note, conservative capability manifest through `cap.C3.explain_valid`, additive registry bindings, witness lifecycle usage rules, passing local conformance tests, broadened local reduced-witness artifacts across FEC commit/reject, session lifecycle, execution-contract, host, and empirical-oracle outcome classes, local normalized replay bundle and pack-candidate evidence, and machine-readable promotion-readiness indices.
- **Remaining gaps**: pack-grade replay promotion, broader reduced-witness breadth beyond the current local families, and any claim toward `cap.C4.distill_valid` or `cap.C5.pack_valid` remain open.
- **Why still open**: `W025` materially widened the promotion-governance floor, but the replay evidence remains local-only and intentionally non-pack-eligible.
- **Canonical owner**: exercised follow-on `W009` through `W017`, `W022`, `W023`, and `W025`; planned next owners `W033` and `W035`.
