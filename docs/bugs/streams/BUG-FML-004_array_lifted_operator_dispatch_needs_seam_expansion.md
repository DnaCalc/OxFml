# BUG-FML-004: Array-Lifted Operator Dispatch Needs Seam Expansion

## Summary
- **Bug id**: `BUG-FML-004`
- **Opened**: 2026-04-07
- **Status**: validated_local
- **Owner workset**: `W059`

## Source Refs
- **Reported against ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Reproduced on ref**: `working-tree-uncommitted`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `working-tree-uncommitted`

## Ownership And Root Cause
- **Ownership class**: shared seam gap
- **Root cause class**: initial_impl_gap
- **Root cause summary**: the earlier OxFml/OxFunc seam read lagged behind the actual generalized OxFunc binary numeric surface in the local workspace, so OxFml retained a temporary local compatibility fallback after the downstream surface was already capable of carrying the admitted array-lifted rows.

## Reproduction
1. Inspect the earlier OxFml compatibility branch in `crates/oxfml_core/src/eval/mod.rs`.
2. Compare it with the current local OxFunc array-lift tests in:
   - `../OxFunc/crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
   - `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
3. Remove the compatibility branch and run `cargo test -p oxfml_core`.
4. Observe that admitted array-lifted arithmetic rows now pass through OxFunc-backed dispatch in the working tree.

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
  2. `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
- **Spec state at intake**: implementation_read_outdated
- **Notes**: ownership direction was correct; the local OxFml compatibility fallback had simply outlived the generalized OxFunc surface in the current workspace.

## Investigation Log
1. 2026-04-07: confirmed binary arithmetic dispatch through `FUNC.OP_ADD`, `FUNC.OP_SUBTRACT`, `FUNC.OP_MULTIPLY`, `FUNC.OP_DIVIDE`, and `FUNC.OP_POWER` works for scalar rows.
2. 2026-04-07: later local inspection showed the current OxFunc operator arithmetic and surface-dispatch tests already cover array-lifted rows in this workspace.
3. 2026-04-07: removed the temporary OxFml compatibility fallback and re-ran the local OxFml floor successfully.
4. 2026-04-07: retained evaluator evidence now proves array-lifted unary and binary arithmetic travel through `FUNC.OP_*` rows.

## Fix Plan
1. remove the stale OxFml compatibility fallback,
2. keep retained array-lift evidence proving the current OxFunc-backed path stays active,
3. narrow any future residual work to value-model questions beyond the admitted array-lifted operator rows.

## Similar-Risk Scan
### Adjacent families to check
1. unary negate and unary plus over arrays
2. postfix percent over arrays
3. concat and comparison operator array lifting
4. reference-operator dispatch that still depends on reference/value carrier breadth

### Check method
1. compare admitted evaluator rows against the current OxFunc operator surface and prepared-call value model,
2. inspect `PreparedArgValue` / surface dispatch comments and array handling in OxFunc,
3. keep array-lifted operator fixtures separate from scalar boundary-correction evidence.

### Results
1. scalar and array-involved ordinary arithmetic dispatch are now both locally validated.
2. residual open questions are no longer about admitted array-lifted operator dispatch; they are about broader multi-area/reference-model representation policy.

## Linked Reports
1. `BUGREP-FML-004`

## Evidence
1. `crates/oxfml_core/src/eval/mod.rs`
2. `../OxFunc/crates/oxfunc_core/src/functions/binary_numeric.rs`
3. `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `crates/oxfml_core/tests/evaluator_tests.rs`
5. `docs/worksets/W059_operator_semantic_dispatch_boundary_correction.md`
6. `cargo test -p oxfml_core`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix updated if required
- [x] handoff filed if required
- [x] linked reports updated
