# OxFml/OxFunc Library Context Runtime Interface

## Purpose
Define the preferred OxFml/OxFunc integration shape for built-in catalog truth and runtime catalog extension.

This document is not a claim that the current cross-repo interface is fully locked.
It is the OxFml-side proposal for the next stabilization round:
1. the normative interface should be a runtime-ingested, versioned library-context interface,
2. file exports such as `W044` should remain conformance, pinning, and mismatch-discovery artifacts,
3. runtime registration and removal of user-defined or host-provided functions must fit the same model without hidden global state.

## Scope
This proposal covers:
1. built-in catalog ingestion for OxFml parse, bind, semantic planning, and evaluation,
2. dynamic registration and removal of later callable or external surfaces,
3. snapshot identity and replay correlation,
4. minimum object families and invariants.

This proposal does not lock:
1. Rust trait names,
2. transport encoding for cross-process cases,
3. a final generalized provider/subscription contract,
4. final UDF execution ABI.

## Boundary Position
The OxFml/OxFunc boundary should not be a build-time catalog-file ingestion contract.

The preferred shape is:
1. OxFunc owns catalog truth and catalog-generation logic,
2. OxFml consumes immutable library-context snapshots through a formal runtime interface,
3. snapshot exports such as `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` remain:
   - conformance artifacts,
   - test-pinning artifacts,
   - mismatch-discovery artifacts,
   - evidence for replay correlation,
   not the normative runtime contract itself.

## Core Runtime Object Families
The minimum formal object families OxFml wants are:

### 1. LibraryContextProvider
An externally supplied provider that can hand OxFml a concrete versioned snapshot.

Minimum required semantic operations:
1. `current_snapshot() -> LibraryContextSnapshot`
2. `snapshot_by_identity(snapshot_ref) -> Option<LibraryContextSnapshot>`
3. `lookup_surface(snapshot_ref, surface_key) -> Option<LibraryContextEntry>`

Optional but desirable later operations:
1. `subscribe_snapshot_changes()`
2. `diff_snapshots(old_ref, new_ref)`

### 2. LibraryContextSnapshot
An immutable versioned catalog view suitable for parse, bind, semantic planning, evaluation gating, and replay correlation.

Minimum required properties:
1. `snapshot_id`
2. `snapshot_generation`
3. source provenance:
   - `source_commit_short`
   - `source_commit_full`
   - `source_tree_state`
4. stable lane identity
5. stable entry collection keyed by `surface_stable_id`

### 3. LibraryContextEntry
A single built-in or registered surface row projected into runtime-consumable form.

Minimum required semantic fields:
1. `surface_stable_id`
2. `entry_kind`
3. `registration_source_kind`
4. `canonical_surface_name`
5. `name_resolution_table_ref`
6. `semantic_trait_profile_ref`
7. `gating_profile_ref`
8. `metadata_status`
9. `special_interface_kind`
10. `admission_interface_kind`
11. `preparation_owner`
12. `runtime_boundary_kind`
13. `interface_contract_ref`

Compatibility/interoperability metadata should remain attachable without replacing primary identity:
1. `xlcall_builtin_symbol`
2. `xlcall_builtin_code`

### 4. RegistrationDescriptor
A host/OxFml-supplied descriptor for runtime-added surfaces.

Minimum semantic intent:
1. stable registration/catalog id or stable pending id,
2. surface name,
3. registration source kind,
4. declared arity/signature metadata,
5. callable or provider/runtime boundary posture,
6. origin kind sufficient to distinguish built-in from registered external/function host surfaces.

### 5. SnapshotUpdate
The result of successful registration or removal.

Minimum semantic intent:
1. old snapshot ref,
2. new snapshot ref,
3. reason class:
   - registration_added
   - registration_removed
   - catalog_refresh
   - profile_gate_change
4. affected surface ids.

## Runtime Lifecycle Model
The preferred runtime lifecycle is:
1. OxFml acquires a `LibraryContextSnapshot` through the provider,
2. parse/bind/semantic-plan artifacts preserve the snapshot ref explicitly,
3. evaluation and higher-level session work use that pinned snapshot ref rather than querying hidden mutable global state,
4. registration or removal produces a new snapshot generation rather than mutating an already-pinned snapshot in place,
5. later sessions or recompilations may adopt the newer snapshot explicitly.

Working rule:
1. snapshot generations are explicit semantic facts,
2. snapshot drift must not be hidden inside evaluation,
3. replay and proving artifacts must be able to name the exact snapshot generation used.

## Dynamic Registration Direction
For runtime extension, OxFml prefers this split:
1. host or OxFml receives registration/unregistration requests,
2. OxFunc remains steward of the catalog identity and semantic descriptor shape,
3. the host remains owner of raw external invocation/runtime exposure,
4. successful registration or removal yields a new immutable `LibraryContextSnapshot`.

This allows:
1. built-ins and registered rows to share one consumable snapshot world,
2. runtime add/remove semantics without build-time regeneration dependence,
3. deterministic replay pinning to a specific snapshot generation.

## Invariants
The runtime interface should preserve these invariants:
1. snapshots are immutable once published,
2. snapshot generations are monotonic and explicitly identified,
3. parse/bind/semantic-plan artifacts always know which snapshot they consumed,
4. session/evaluation work never silently switches snapshots mid-flight,
5. `surface_stable_id` remains the primary semantic identity,
6. legacy `xlf*` metadata remains compatibility metadata, not replacement identity,
7. registration source kind remains explicit for built-in versus registered surfaces,
8. hidden global mutable registry state is not required to explain semantic outcomes.

## Current Relationship To W044
`W044` remains useful and should continue.

Current OxFml reading:
1. `W044` is the best current mismatch-discovery and test-pinning artifact,
2. OxFml should keep consuming it in tests,
3. but OxFml does not want that CSV export to become the normative runtime interface boundary.

Preferred role split:
1. runtime interface:
   - normative
   - versioned
   - used by implementations
2. export artifact:
   - descriptive
   - diffable
   - test-pinnable
   - replay-correlatable

## Current First-Freeze Working Rule
OxFml now reads the latest OxFunc note as convergent on a two-track working rule:
1. consume the committed `W044` snapshot/export now for pinning, test synthesis, and semantic-plan validation,
2. model the normative runtime seam in parallel as:
   - `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - generation-producing registration/removal,
3. use concrete mismatches between those two tracks as the trigger for narrower seam changes,
4. do not wait for the full runtime provider shape to be coded before consuming the committed snapshot in OxFml tests and semantic-plan fixtures.

Current OxFml reading is that these are compatible rather than competing paths.

## Current Coverage Goal
The target for this seam-hardening round is:
1. the full Excel cell-formula language as scoped in OxFml,
2. nearly all built-in functions currently covered in OxFunc,
3. a working parse -> bind -> semantic-plan -> evaluate cycle ready for implementation use,
4. explicit deferral only for the few OxFunc-local packets still intentionally deferred.

The runtime library-context interface is important because that scope is too large and too dynamic to manage honestly through build-time catalog-file ingestion alone.

## Formalization Candidates
These are good candidates for later Lean/TLA+ support:
1. snapshot immutability,
2. generation monotonicity,
3. session pinning to snapshot generation,
4. no mid-session hidden catalog drift,
5. registration/removal yielding explicit new snapshot refs,
6. stable-id preservation across export and runtime surfaces.

## Current OxFml Ask To OxFunc
For the next sync, OxFml wants OxFunc to respond to this direction directly:
1. can OxFunc support a formal runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` model,
2. which `W044` fields are already part of that runtime truth versus export-only description,
3. what minimum runtime registration descriptor OxFunc needs for add/remove support,
4. whether any currently proposed field families should move out of the runtime interface and remain export-only,
5. whether OxFunc sees any semantic reason the normative interface should remain file-export ingestion rather than runtime snapshot ingestion.

Current processed OxFunc response:
1. yes, OxFunc supports the long-term runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` direction,
2. yes, the committed snapshot/export should be used now as the immediate pinning artifact,
3. yes, registration/removal should produce explicit new snapshot generations,
4. built-in `xlf*` metadata should remain compatibility metadata rather than replacing `surface_stable_id`,
5. current first-pass callable-minimum facts may remain in contract docs for now rather than requiring immediate direct snapshot columns.

## Current OxFml First-Pass Freeze Answers
For the current successor-packet round, OxFml's first-pass answers are:
1. the runtime consumer/model shape should be cleaner runtime-only structure plus an explicit CSV/export mapping layer, not a runtime object model forced to mirror every export column,
2. the committed `W044` export remains the current pinning and mismatch artifact for tests, validation, and cross-repo correlation,
3. runtime-semantic truth should be modeled primarily through:
   - `snapshot_id`
   - `snapshot_generation`
   - source provenance
   - `surface_stable_id`
   - `entry_kind`
   - `registration_source_kind`
   - `canonical_surface_name`
   - `name_resolution_table_ref`
   - `semantic_trait_profile_ref`
   - `gating_profile_ref`
   - `metadata_status`
   - `special_interface_kind`
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `interface_contract_ref`,
4. compatibility/export description such as `xlf*` metadata, snapshot formatting details, or explanatory notes may remain export-facing side metadata as long as they do not replace the runtime-semantic fields above,
5. callable-minimum semantic facts remain acceptable in contract/interface docs for one more round rather than direct snapshot columns.

Current processed OxFunc confirmation:
1. OxFunc now also reads the seam as close enough to work toward a first freezable application seam for the already-covered scope,
2. OxFunc agrees that the remaining work is primarily:
   - typed context/query bundle freeze,
   - return-surface and publication-hint freeze,
   - runtime provider/snapshot consumer modeling,
   rather than another callable-row sufficiency round,
3. OxFunc still prefers the committed `W044` export as the immediate shared pinning artifact while the runtime-only consumer model is being shaped in parallel,
4. OxFunc still treats callable-minimum facts as semantic truth that may stay in contract/interface documentation for now rather than requiring immediate direct snapshot columns.

## Current Next-Lock Questions
The next bounded OxFml/OxFunc interface locks should therefore be:
1. the first shared typed context/query bundle for the already-covered seam-heavy rows,
2. the first shared returned-value and publication-aware split,
3. the first real OxFml consumer/model packet for the runtime `LibraryContextProvider` and immutable `LibraryContextSnapshot` direction.
