# W068: Canonical Function Registry Consumption Cleanup

## Purpose
Move OxFml to the post-migration function-metadata architecture requested by DNA OneCalc and OxFunc.

The architectural rule is now explicit: OxFunc owns the canonical runtime function registry. OxFml consumes registry entries or immutable registry-derived views for function identity, arity, display signature, parameter descriptors, source classification, and help metadata. OxFml does not maintain a comprehensive function list, does not accept host-filled string arity as source truth, and does not synthesize function signatures it cannot obtain from the registry.

This workset is a direct move to the desired end state. No compatibility deprecation lane is required for the superseded `arity_shape_note` implementation.

## Position and Dependencies
- **Depends on**:
  - `W026` library-context snapshot and availability taxonomy
  - `W043` runtime library-context provider consumer model
  - `W048` editor language service and immutable formula host plan
  - `W054` consumer-facing interface rearchitecture and facade packaging
  - `W067` diagnostic symbol spans and stage precision
  - OxFunc `W091` / `HO-FN-011` canonical registry API update
- **Blocks**:
  - DNA OneCalc removal of its host-owned default function list
  - UDF-aware editor function-help and signature-help flow
  - broader runtime capability-scoped registry-view cleanup
- **Cross-repo**:
  - Responds to `../DnaOneCalc/docs/HANDOFF_OXFML_FUNCTION_HELP_FROM_OXFUNC_REGISTRY.md`
  - Acknowledges `../OxFunc/docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md`
  - Downstream host cleanup remains DNA OneCalc-owned after this OxFml surface lands

## Reviewed Inbound Observations
1. OxFunc inbound note `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md` was reviewed.
2. OxCalc inbound note `../OxCalc/docs/upstream/NOTES_FOR_OXFML.md` was reviewed.
3. DNA OneCalc handoff `../DnaOneCalc/docs/HANDOFF_OXFML_FUNCTION_HELP_FROM_OXFUNC_REGISTRY.md` was reviewed.
4. OxFunc handoff `../OxFunc/docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md` was reviewed.

Relevant unresolved observations:
1. OxFunc requires `display_signature`, `ParameterDescriptor`, and `FunctionMeta.arity` to be the future-facing signature source.
2. DNA OneCalc must not compensate for OxFml by sending a comprehensive function-list snapshot.
3. Capability and provider availability must remain a registry-view or snapshot-overlay concern rather than deleting entries or inventing host-local list truth.

## Scope
### In scope
1. Record the OxFml acknowledgement of `HO-FN-011` and the DNA OneCalc function-help handoff.
2. Audit all current OxFml comprehensive-function-list and string-arity consumers, including:
   - `consumer/editor::build_function_help_packet`,
   - `LibraryContextSnapshotEntry.arity_shape_note`,
   - `parse_arity_shape_note`,
   - `signature_suffix`,
   - `build_argument_help`,
   - catalog snapshot export/import fixtures and tests,
   - runtime/editor/replay consumer facade tests that construct snapshot entries.
3. Add a registry-backed function metadata access path using `oxfunc_core::registry`:
   - `builtin_registry()`,
   - `FunctionRegistry::iter()`,
   - `lookup_by_surface_name()`,
   - `lookup_by_id()`,
   - `register_udf()`,
   - `unregister_udf()`,
   - `CapabilityOverlay`,
   - `with_capability_overlay()`.
4. Make `EditorEnvironment` or its provider/pinned-view equivalent carry the registry or capability-scoped registry view needed by editor function help.
5. Build `FunctionHelpPacket` from registry `display_signature` and ordered `ParameterDescriptor` entries.
6. Return no function-help packet for unknown callees rather than inventing fallback signatures.
7. Remove `LibraryContextSnapshotEntry.arity_shape_note` and all string-arity parsing/synthesis code from ordinary OxFml source and fixtures.
8. Preserve `LibraryContextSnapshot` as admission/capability/provenance overlay data, not as a function catalog.
9. Map existing snapshot/status fields from `FunctionEntry.registry_metadata` where they mirror function identity, source, or metadata truth.
10. Add deterministic tests for the DNA OneCalc regression and registry-backed built-in/UDF cases.
11. Add structural no-synthesis evidence that the old editor hot path cannot reappear silently.
12. Update spec text and downstream-facing notes to state the new ownership rule.

### Out of scope
1. DNA OneCalc deletion of its local default function list.
2. OxFunc registry implementation work.
3. Broad UDF execution semantics beyond registry mutation and editor metadata display.
4. Full product UI rendering.
5. OxCalc coordinator policy changes.
6. Long-lived compatibility shims for `arity_shape_note`.

## Deliverables
1. `HO-FN-011` OxFml acknowledgement filed under `docs/handoffs/` and registered in `docs/handoffs/HANDOFF_REGISTER.csv`.
2. Updated spec text covering the rule: function metadata flows `OxFunc -> OxFml`, never host comprehensive list `-> OxFml`.
3. Registry-backed editor function-help implementation.
4. Removal of `LibraryContextSnapshotEntry.arity_shape_note` and related fixture/test fields.
5. Removal of `parse_arity_shape_note`, `signature_suffix`, `build_argument_help`, and any editor hot-path synthetic `argN` signature generation.
6. Capability-scoped registry-view handling that preserves gated/unavailable states without deleting registry entries.
7. UDF registry mutation fixture proving editor signature display uses host-registered registry entries.
8. Deterministic test matrix:
   - `=NOW(` renders `NOW()` with no argument-help rows,
   - `=SUM(` renders canonical OxFunc registry parameters,
   - `=IF(test, ` marks the second argument active using registry parameter names,
   - `=ZZZNOTAFUNCTION(` emits no `FunctionHelpPacket`,
   - a registry-registered UDF renders its host-supplied parameter names,
   - structural source scan proves the removed synthesis helpers and strings are absent from the editor builder hot path.
9. Worklist and status updates preserving three-axis reporting.

## Gate Model
### Entry gate
1. `HO-FN-011` OxFunc registry API is available in the sibling OxFunc dependency.
2. Current OxFml editor facade tests can be run locally.
3. Current affected sources and fixtures are audited before edits.

### Implementation gate
1. Registry-backed metadata access is available to editor/runtime consumers.
2. Function help is built only from registry entries.
3. Unknown function help fallback is removed.
4. Snapshot overlay fields are separated from registry-owned function truth.

### Evidence gate
1. Focused editor facade and language-service tests pass.
2. Snapshot/interface tests compile after `arity_shape_note` removal.
3. Structural no-synthesis test passes.
4. `cargo fmt --all -- --check`, `git diff --check`, and `cargo test -p oxfml_core` pass before any completion claim.

### Cross-repo gate
1. OxFunc acknowledgement is recorded locally.
2. DNA OneCalc impact note states that downstream host cleanup can proceed after OxFml lands this surface.
3. No OxCalc coordinator-facing handoff is filed unless implementation discovers a coordinator-visible seam change.

## Bead Plan
The execution bead tree is rooted at `.beads` issue `fml-15n`.

Bead lanes:
1. `fml-15n.1` — acknowledge handoffs and sync local doctrine.
2. `fml-15n.2` — audit current function metadata and arity consumers.
3. `fml-15n.3` — wire OxFunc registry access into OxFml consumers.
4. `fml-15n.4` — rewrite editor function help from registry signatures.
5. `fml-15n.5` — remove `arity_shape_note` from snapshot schema and fixtures.
6. `fml-15n.6` — map registry metadata to library-context overlay fields.
7. `fml-15n.7` — add capability-overlay and unknown-callee tests.
8. `fml-15n.8` — add UDF registry mutation signature evidence.
9. `fml-15n.9` — add structural guard against signature synthesis.
10. `fml-15n.10` — update docs, worklist, and downstream migration note.
11. `fml-15n.11` — run validation and closure self-audit.

Dependency shape:
1. `fml-15n.2` gates registry plumbing and schema removal.
2. `fml-15n.3` gates editor rewrite and UDF registry evidence.
3. `fml-15n.4` and `fml-15n.5` gate the no-synthesis guard.
4. Documentation cleanup follows the handoff acknowledgement plus implementation/evidence lanes.
5. Validation follows all implementation, evidence, guard, and documentation lanes.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature completion wording per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## Completion Claim Self-Audit
Not run yet. This workset is in progress and cannot advance to closure until implementation and evidence gates pass.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - handoff acknowledgement filed; implementation not started
  - deterministic registry-backed editor evidence not yet produced
  - `arity_shape_note` removal not yet executed
  - downstream DNA OneCalc cleanup not yet unblocked by landed OxFml code
- claim_confidence: draft
