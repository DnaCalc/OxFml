# W049: OxFunc Preparation Adapter and Consumer Harness

## Purpose
Build the first OxFml-side adapter and consumer harness that can feed OxFunc real OxFml preparation artifacts from formula text, caller context, and deterministic fixture worlds.

## Position and Dependencies
- **Depends on**: `W032`, `W041`, `W042`, `W043`, `W045`
- **Blocks**: `W050`
- **Cross-repo**: bounded successor to the converged OxFml/OxFunc seam round; OxFunc wants a real OxFml-backed preparation/evaluation adapter rather than continued mock-only integration confidence, and has now published pinned downstream artifacts for its `W047` / `W048` / `W049` packet sets plus the consolidated seam-requirements document

## Scope
### In scope
1. Freeze the first OxFml-side adapter request/response packet for OxFunc integration tests.
2. Drive the real OxFml parse -> bind -> semantic-plan -> prepare pipeline from deterministic fixtures.
3. Preserve the current freeze-candidate seam packets under `W041`, `W042`, and `W043` in that adapter.
4. Produce structured mismatch artifacts that can drive bounded OxFml <-> OxFunc closure rounds.
5. Add deterministic local evidence for the adapter on the current first-application seam floor.
6. Use OxFunc's current pinned first-wave slice as the default integration wave; the older note-only “38-scenario” wording should be read against the currently published consolidated table, which now enumerates 45 scenario ids.

### Out of scope
1. Final production host API shape.
2. Worksheet `CALL` / `REGISTER.ID` runtime.
3. Broader pack-grade replay promotion.
4. Full cross-process or network transport ABI.
5. Expanding the first wave to currently deferred `CALL` / `REGISTER.ID` or richer publication families.

## Deliverables
1. A canonical first adapter request packet for OxFunc-facing integration tests.
2. A canonical first adapter output family covering:
   - preparation artifacts,
   - end-to-end evaluation artifacts,
   - mismatch report artifacts.
3. Local OxFml tests proving the adapter uses the real preparation pipeline rather than mock-only seams.
4. An explicit list of still-deferred seam families not admitted into the first adapter floor.
5. An explicit mapping from the adapter outputs to the pinned OxFunc-side `W047` / `W048` / `W049` packet artifacts.

## Gate Model
### Entry gate
- `W041`, `W042`, and `W043` have first local packet floors strong enough to be projected into one real OxFml-backed integration artifact.
- The OxFml/OxFunc note round has converged enough that the next useful work is artifact-driven.
- The current OxFunc pinned first-wave table and packet artifacts are available as downstream inputs.

### Exit gate
- The adapter can accept formula text, caller anchor, and deterministic fixture input and drive the real OxFml preparation path.
- The adapter preserves `W041`, `W042`, and `W043` packet meaning in structured output.
- At least one deterministic local artifact exists per admitted seam family in the first adapter floor.
- Any mismatch against OxFunc's pinned `W047` / `W048` / `W049` artifacts is reported concretely rather than as prose-only ambiguity.

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
  - the first adapter request/response packet now exists locally through `crates/oxfml_core/src/oxfunc_adapter/mod.rs`, but it is not yet frozen against the full current OxFunc first-wave artifact set
  - the real OxFml preparation pipeline is now projected into an OxFunc-facing harness artifact, but broader packet-mismatch reporting still needs widening beyond the first current fixture families
  - mismatch reporting is now structured at the packet level, but it has not yet been driven against the broader pinned OxFunc scenario corpus
  - the current pinned first-wave table is now broadly mapped into local OxFml harness cases, but two callable residuals remain deferred
  - deferred families such as `CALL` / `REGISTER.ID` still need to stay explicitly outside the first adapter floor
- claim_confidence: provisional

## Current Pinned Downstream Inputs
1. `../OxFunc/docs/upstream/OXFUNC_OXFML_SEAM_REQUIREMENTS_CONSOLIDATED.md`
2. `../OxFunc/docs/function-lane/FUNCTION_SLICE_TYPED_CONTEXT_AND_QUERY_BUNDLE_CONTRACT_PRELIM.md`
3. `../OxFunc/docs/function-lane/W47_TYPED_CONTEXT_QUERY_DEPENDENCY_MAP.csv`
4. `../OxFunc/docs/function-lane/FUNCTION_SLICE_RETURN_SURFACE_AND_PUBLICATION_HINT_CONTRACT_PRELIM.md`
5. `../OxFunc/docs/function-lane/W48_RETURN_SURFACE_CLASS_MAP.csv`
6. `../OxFunc/docs/function-lane/FUNCTION_SLICE_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL_PRELIM.md`
7. `../OxFunc/docs/function-lane/W49_RUNTIME_LIBRARY_CONTEXT_CSV_TO_RUNTIME_MAPPING.csv`
8. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

## Current OxFunc Validation Read
OxFunc now explicitly confirms:
1. the authoritative pinned first-wave table is 45 scenario ids,
2. the current local adapter/fixed-fixture floor of 45 admitted scenarios is strong integration evidence rather than a standing seam objection,
3. no further broad note round is needed for the admitted adapter wave,
4. the next new bounded OxFml/OxFunc note lane is worksheet `CALL` / `REGISTER.ID`, not another adapter-shape debate.
