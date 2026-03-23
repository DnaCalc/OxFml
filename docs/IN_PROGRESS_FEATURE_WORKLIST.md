# IN_PROGRESS_FEATURE_WORKLIST.md — OxFml

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-23.

## Status Vocabulary

- `in-progress`: partial implementation exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Formula Grammar, Parse, and Bind

- **Status**: in-progress
- **Current floor**: architectural baseline plus exercised implementation slices for formula source records, explicit formula-channel identity, tokenization, green syntax, red projections, a widened expression parser subset including additional qualified-name handling, a first local `WorksheetR1C1` floor with absolute/relative cell translation and qualified area normalization, normalized reference ADTs, bind fixture scaffolding with richer assertions, host-path incremental parse/red/bind reuse, semantic-plan compilation with helper-environment profiling, stage-aware availability summaries, external library-context snapshot refs, narrower per-surface library-context fields (`surface_stable_id`, `name_resolution_table_ref`, `semantic_trait_profile_ref`, `gating_profile_ref`) plus first-pass seam-facing export fields (`metadata_status`, `special_interface_kind`, `admission_interface_kind`, `preparation_owner`, `runtime_boundary_kind`, `arity_shape_note`, `interface_contract_ref`), a newly explicit preferred runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` interface model so implementation use does not depend on build-time catalog-file ingestion, first local `TypedContextQueryBundle` / `TypedContextQueryBundleSpec` packet types with grouped `INFO` / `CELL` / `RTD` host-run evidence, dedicated deterministic classification for accepted-unresolved-name, semantic-plan gated, runtime capability denied, and post-dispatch provider-unavailable lanes plus a checked Lean artifact for that stage split, direct local consumption of the downstream `W044` export for selected ordinary, seam-heavy, and higher-order helper rows, prepared-call/result lowering with blankness, caller-context provenance, typed callable carriers plus callable-profile detail, helper/scalarization prepared-call traces, a first `ReturnedValueSurface` packet propagated through evaluation, host, candidate, and commit carriers for ordinary-value lanes plus live typed host/provider outcome lanes for `RTD` value and capability-denied outcomes, `INFO` unsupported-query outcomes, and `CELL` provider-failure outcomes, first local restricted-carrier validation for conditional-formatting and data-validation host-managed formula lanes, local evaluation semantics for `_xlfn.SINGLE`, `LET`, callable `LAMBDA`, exact free-helper lexical capture, adopted defined-name callable transport with distinct `DefinedNameCallable` origin preservation, `ROW`, `COLUMN`, `INDIRECT`, `OFFSET`, `IFERROR`, and `RTD`, first local end-to-end higher-order callable execution for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` through a typed callable invoker, first higher-order execution through an adopted defined-name callable carrier (`MAP(...,NamedLambda)`), and semantic-plan recognition of `NameKind::MixedOrDeferred` as a first explicit `name_formula_carrier` lane.
- **Remaining gaps**: fuller Excel grammar closure, richer structured/external reference coverage, broader OxFunc catalog coverage, final shared callable transport, broader higher-order callable breadth beyond the exercised `MAP` / `REDUCE` / `SCAN` / `BYROW` / `BYCOL` / `MAKEARRAY` floor, exact host-managed name/external-name resolution boundary, and replay-backed evidence beyond the current local witness tier.
- **Why still open**: `W032`, `W038`, and `W040` still own broader catalog, host-managed name/external-name, and higher-order callable closure beyond the now-exercised `W037` and `W039` local floors, so the repo-level feature remains broader than the current exercised slice.
- **Canonical owner**: `W001` now; exercised follow-on `W002`, `W003`, `W013`, `W014`, `W019`, `W020`, `W026`, `W027`, `W031`, `W037`, and `W039`; active next owner `W032`; explicit next seam-freeze owners `W041`, `W043`, and `W045`; planned follow-on owners `W036`, `W038`, and `W040`.

### IP-02: FEC/F3E Evaluator Session

- **Status**: in-progress
- **Current floor**: OxFml-owned seam design and exercised implementation now include accepted-candidate, commit-bundle, reject-record, fence snapshots, typed no-publish fence rejection, single-formula host recalc wiring, a managed `prepare -> open_session -> capability_view -> execute -> commit` session-service slice with abort/expire handling, invalid-phase structural-conflict rejection, surfaced execution-restriction effect facts, runtime contention enforcement across sessions, async-coupled external-provider consequence surfacing, runtime-async overlay registration, and checked local formal artifacts for the external capability gate plus busy-locus session contention, retry-after-release, overlay-cleanup, pinned-epoch overlay, distributed-placement, retry-ordering fairness, and placement-deferral expiry boundaries.
- **Remaining gaps**: broader async/distributed runtime behavior beyond the local external-provider, contention, first placement-outcome floor, non-overtaking retry-order floor, and deferred-placement expiry floor, pack-grade replay/model artifacts, and broader host integration beyond the single-formula proving path.
- **Why still open**: `W029` materially widened the local async-facing runtime floor and the current pass adds checked session-contention, retry-after-release, overlay-cleanup, pinned-epoch overlay, distributed-placement, retry-ordering fairness, and placement-deferral expiry boundary models, but repo-level runtime scope still extends beyond the exercised local contention, placement, retry-order, deferral-expiry, and external-provider model.
- **Canonical owner**: `W001` now; exercised follow-on `W004`, `W015`, `W018`, `W021`, `W024`, and `W029`; planned next owners `W034` and `W035`.

### IP-03: Commit Output Contract

- **Status**: in-progress
- **Current floor**: atomic bundle, schema, and fixture-planning baseline exist in OxFml-owned docs, and the exercised implementation now constructs commit bundles from accepted candidate results under matching fences, derives seam-significant `format_delta` and `display_delta` from prepared-result hints where applicable, rejects mismatched fences with typed no-publish outcomes, and surfaces typed dependency consequence facts inside `topology_delta`.
- **Remaining gaps**: broader commit bundle construction beyond the current local publication families, wider distributed publication policy, and pack-grade replay evidence.
- **Why still open**: `W028` materially widened the local publication and topology floor, but the repo-level feature still does not represent the full evaluator publication pipeline or pack-grade coverage.
- **Canonical owner**: `W001` now; exercised follow-on `W004`, `W015`, `W017`, `W018`, `W021`, `W023`, and `W028`; planned next owner `W034`.

### IP-04: Reject Taxonomy and Trace Schema

- **Status**: in-progress
- **Current floor**: reject and trace taxonomy, minimum schemas, and formal/replay planning baseline exist, with exercised typed reject records for fence mismatch, capability denial, abort, expire, and contention-sensitive paths; local replay fixtures for semantic-plan, prepared-call/result, execution-contract, session lifecycle, FEC commit/reject, single-formula host, and empirical-oracle slices; broadened local reduced-witness artifacts; local normalized replay bundles; plus checked local Lean artifacts for session lifecycle, external-reference deferment, deferred-name-carrier classification, failure-stage split, and external-name carrier typing, and checked local TLA+ models for session lifecycle, external capability gate, higher-order callable boundary, session contention boundary, retry-after-release boundary, overlay-cleanup boundary, pinned-epoch overlay boundary, distributed-placement boundary, retry-ordering fairness boundary, and placement-deferral expiry boundary behavior.
- **Remaining gaps**: broader typed reject coverage, pack-grade deterministic replay infrastructure, and broader formal families beyond the first checked runs.
- **Why still open**: `W022` and `W023` materially widened the local witness/formal floor, but the evidence remains local and not yet promoted into pack-grade corpus or wider formal coverage.
- **Canonical owner**: `W001` now; exercised follow-on `W004`, `W005`, `W015`, `W016`, `W017`, `W022`, and `W023`; planned next owners `W033`, `W034`, and `W035`.

### IP-05: Formula-Semantic Formatting

- **Status**: in-progress
- **Current floor**: formatting behavior crossing the seam is chartered and exercised through `TEXT`, `VALUE`, `NOW`, `TODAY`, `CELL`, and `INFO` with explicit locale-format and host-query context, prepared-result format/publication hints, locale format-dependency facts surfaced through the proving host, seam-significant `format_delta` and `display_delta` publication artifacts, empirical-oracle scenarios covering formatting and host-query lanes, and a first restricted conditional-formatting/data-validation carrier floor with explicit formula-semantic host fields and restriction profiles.
- **Remaining gaps**: broader semantic formatting family coverage, fuller display-boundary closure beyond the current seam-significant subset, richer `MS-OE376` carrier parity, and pack-grade proving scenarios.
- **Why still open**: `W030` and `W039` widened the local semantic-format and non-cell carrier floor, but the repo-level feature remains much broader than the exercised slice.
- **Canonical owner**: exercised follow-on `W006`, `W014`, `W018`, `W020`, `W021`, `W024`, `W030`, `W031`, and `W039`; explicit next seam-freeze owner `W042`; planned follow-on owner `W036`.

### IP-06: Replay Appliance Adapter and Witness Governance

- **Status**: in-progress
- **Current floor**: OxFml-local replay adapter governance is written into the canonical spec set, including the adapter note, conservative capability manifest through `cap.C3.explain_valid`, additive registry bindings, witness lifecycle usage rules, passing local conformance tests, broadened local reduced-witness artifacts across FEC commit/reject, session lifecycle, execution-contract, host, and empirical-oracle outcome classes, local normalized replay bundle and pack-candidate evidence, and machine-readable promotion-readiness indices.
- **Remaining gaps**: pack-grade replay promotion, broader reduced-witness breadth beyond the current local families, and any claim toward `cap.C4.distill_valid` or `cap.C5.pack_valid` remain open.
- **Why still open**: `W025` materially widened the promotion-governance floor, but the replay evidence remains local-only and intentionally non-pack-eligible.
- **Canonical owner**: exercised follow-on `W009` through `W017`, `W022`, `W023`, and `W025`; planned next owners `W033` and `W035`.

### IP-09: Host Runtime and External Requirements Freeze

- **Status**: in-progress
- **Current floor**: host/runtime truth is currently split across `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`, `OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`, the live `W041` / `W042` / `W043` successor packets, and the outbound OxCalc seam note. A new canonical unifying draft now exists in `docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`.
- **Remaining gaps**: broader `W041` / `W042` / `W043` packet execution remains partial; caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset remains in the `W026` note lane; execution-restriction transport shape and publication/topology breadth remain narrower than final shared closure; full product-host policy and broader distributed/runtime ownership remain outside the current packet.
- **Why still open**: OxCalc now reads the unified host/runtime draft as sufficient for the first implementation slice, but the packet is still anchored to a partial local floor and has not yet been promoted into shared seam-freeze text.
- **Canonical owner**: active next owner `W045`; upstream coordination counterpart is the next bounded OxCalc seam round keyed to the new host/runtime draft.

### IP-10: First Host Implementation Packet

- **Status**: in-progress
- **Current floor**: the host/runtime draft now includes a first implementation workflow, readiness assessment, and replay-integration path; the current direct-host packet exposes `TypedContextQueryBundleSpec`, `ReturnedValueSurface`, candidate, commit, reject, and trace outputs through `HostRecalcOutput`; unsupported or unavailable `INFO` / `CELL` / `RTD` behavior is explicit for the currently exercised slice; and `HostRecalcOutput::to_first_host_replay_capture_packet()` now provides a first dedicated host-side replay-capture projection helper.
- **Remaining gaps**: broader language and built-in-function closure remain outside the first host packet; the helper packet is not yet a pack-grade replay bundle builder; broader host-query/provider families remain outside the current first-host slice.
- **Why still open**: `W046` froze the first honest implementation packet for the current exercised slice, but repo-level host implementation scope is broader than that first slice.
- **Canonical owner**: exercised next owner `W046`; broader follow-on owners remain `W041`, `W042`, `W043`, and `W045`.

### IP-11: First Host Readiness

- **Status**: in-progress
- **Current floor**: the bounded `W047` batch has now executed the immediate first-host-readiness slice:
  - `W037` first local `R1C1` channel floor,
  - `W039` first restricted `CF` / `DV` carrier floor,
  - `W046` first-host packet and replay-capture freeze.
- **Remaining gaps**: the supporting `W041` / `W042` / `W043` packet work remains partial at repo scope; broader full-Excel and broad built-in closure remain out of scope for the first-host packet.
- **Why still open**: the immediate batch blockers are no longer implicit, but broader host readiness still depends on follow-on language and seam work outside the executed batch.
- **Canonical owner**: executed batch owner `W047`; active follow-on owners remain `W041`, `W042`, `W043`, and `W045`.
