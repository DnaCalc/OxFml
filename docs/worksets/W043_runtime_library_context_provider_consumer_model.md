# W043: Runtime Library Context Provider Consumer Model

## Purpose
Turn the converged OxFml/OxFunc runtime library-context direction into a first real OxFml consumer/modeling packet, so the runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` interface exists as more than note-level agreement.

## Position and Dependencies
- **Depends on**: `W032`
- **Blocks**: none
- **Cross-repo**: successor packet corresponding to OxFunc `W049`; OxFunc owns catalog truth and generation logic; OxFml owns runtime consumption, snapshot pinning, and artifact correlation semantics

## Scope
### In scope
1. Define the first OxFml consumer/model shape for `LibraryContextProvider` and `LibraryContextSnapshot`.
2. Decide whether the runtime consumer shape should mirror the CSV closely or use a cleaner runtime-only shape plus explicit mapping layer.
3. Keep snapshot identity, generation, and registration/removal semantics explicit.
4. Add deterministic local evidence or model artifacts for the first consumer shape.
5. Separate runtime-semantic fields from export-only descriptive fields for the first freeze.

### Out of scope
1. Full registered-external invocation runtime.
2. Final cross-process transport ABI.
3. Full host routing logic for every built-in or external path.

## Deliverables
1. A first OxFml consumer/model shape for the runtime library-context interface.
2. An explicit statement on runtime-only shape versus CSV-mirroring shape.
3. Deterministic evidence or checked formal artifacts for snapshot pinning/generation behavior.
4. An explicit runtime-truth versus export-description field classification.

## Fresh-Eyes Reset
`W043` should now be treated as an architectural refactor lane, not only as a packet-definition lane.

The real quality target is:
1. `LibraryContextSnapshotRef` is the only durable cross-surface snapshot identity inside OxFml
2. `LibraryContextProvider` plus pinned snapshot ref is the preferred runtime lookup model
3. full `LibraryContextSnapshot` objects are used deliberately for compile-time pinning, tests, and explicit offline artifact construction, not as the default runtime transport between services
4. stringly `snapshot_id@version` carriage is design debt to remove, not a stable internal compromise

## Target End-State
The desired `W043` end-state is:
1. one canonical identity packet:
   `LibraryContextSnapshotRef`
2. one canonical runtime lookup boundary:
   `LibraryContextProvider`
3. one canonical pinned-consumer rule:
   runtime consumers execute against a pinned snapshot ref, not an ambient mutable "current snapshot"
4. one canonical mapping rule:
   CSV/export shape remains a stabilization artifact and mapping source, not the preferred runtime model

## Implementation Meaning
This work now means a smart internal refactor toward provider-and-pin semantics.

The architectural direction is:
1. `typed snapshot identity everywhere`
   no new raw compound snapshot strings on internal packet surfaces
2. `provider plus pin over full snapshot transport`
   where runtime consumers only need lookup, they should prefer provider + pinned ref instead of carrying the whole snapshot object through multiple layers
3. `explicit current-vs-pinned separation`
   `current_snapshot()` is for choosing a snapshot
   `snapshot_by_identity()` is for executing against a chosen snapshot
4. `resolved runtime views where useful`
   if repeated provider lookups become noisy or expensive, OxFml should introduce a pinned runtime view/cursor abstraction rather than spreading ad hoc lookup code everywhere

## Refactoring Direction
The conservative path would keep adding typed refs while still passing whole snapshots across many surfaces.
That is not the best end-point.

The better end-point is:
1. compile-time and offline artifact creation may still accept full `LibraryContextSnapshot`
2. runtime execution, session, editor/help, and coordinator-facing services should converge on provider + pinned ref
3. any residual whole-snapshot carriage should be justified as either:
   - compile-time construction,
   - deterministic fixture pinning,
   - or an explicit offline transform boundary

## Execution Phases
1. `Phase A: Identity normalization`
   remove remaining stringly snapshot refs and ensure typed `LibraryContextSnapshotRef` is the only durable internal identity form
2. `Phase B: Pinned consumer normalization`
   migrate remaining consumers from whole-snapshot opportunism to provider + pinned ref where runtime lookup is the real need
3. `Phase C: Generation/update behavior`
   add multi-snapshot tests proving pinned lookup stability, current-snapshot drift, and explicit invalidation behavior
4. `Phase D: Runtime-view cleanup`
   introduce a pinned runtime lookup view/cursor if it gives a cleaner and more performant architecture than repeated provider calls

## Gate Model
### Entry gate
- `W032` has converged the long-term direction toward runtime provider/snapshot rather than file-ingestion coupling.

### Exit gate
- The runtime consumer/model shape is explicit enough for implementation use.
- Snapshot/generation semantics are explicitly stated.
- Remaining open transport or registration gaps are explicitly listed.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml now has a first real local consumer/model packet:
    - `LibraryContextProvider`
    - `InMemoryLibraryContextProvider`
    - `LibraryContextSnapshotRef`
    - runtime-field classification helpers,
    and the semantic-plan compile surface, single-formula host, managed session path, and `language_service` function-help/intelligent-completion packets now all consume or preserve the typed snapshot-ref packet instead of raw compound strings, while `language_service` completion/help consumers now prefer provider + pinned-ref resolution over inline whole-snapshot opportunism; broader runtime consumers are still not fully normalized onto that same model
  - OxFml now prefers a cleaner runtime-only shape plus explicit CSV/export mapping, and that preference is backed by the first local provider model plus host consumption plus explicit session/editor-surface snapshot-ref carriage and pinned provider lookup, but not yet by broader multi-surface consumer use or an explicit pinned runtime-view abstraction
  - deterministic local evidence now exists for snapshot pinning and surface lookup through the first provider model, semantic-plan compilation and the `W044` export-consumption path now preserve the typed snapshot ref directly, the managed session path now carries `LibraryContextSnapshotRef` explicitly on prepared/opened sessions, and `language_service` now preserves the typed snapshot ref through function-help and intelligent-completion packets while using pinned provider lookup for completions, but broader generation/update behavior remains only partially exercised
  - the runtime-truth versus export-only field split is now explicit in local helper code, but OxFml still lacks a broader runtime-facing projection/view layer that would let consumers avoid repeated raw entry handling cleanly
  - the current local freeze candidate is now mirrored in `docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`, and OxFunc's `HO-FN-004` now treats the runtime consumer shape as part of the shared freeze floor for the narrowed seam families
  - deterministic local evidence for the broader compile/session/editor snapshot-ref carriage now exists in `crates/oxfml_core/tests/semantic_plan_tests.rs`, `crates/oxfml_core/tests/library_context_snapshot_tests.rs`, `crates/oxfml_core/tests/oxfunc_catalog_snapshot_export_tests.rs`, `crates/oxfml_core/tests/session_service_tests.rs`, and `crates/oxfml_core/tests/language_service_tests.rs`, including current-vs-pinned completion/help evidence through `InMemoryLibraryContextProvider::with_snapshots(...)`, but the next honest step is broader generation/update evidence across more runtime surfaces rather than more single-surface assertions
  - current OxFunc reading is that this is now a freeze-and-consumer packet rather than a broad semantic-open lane, so the remaining next step is comprehensive local normalization toward provider-plus-pin runtime use rather than more interface-shape debate
- claim_confidence: provisional
