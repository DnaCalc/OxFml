# BUG-FML-003: Ordinary Operator Semantics Should Dispatch To OxFunc

## Summary
- **Bug id**: `BUG-FML-003`
- **Opened**: 2026-04-07
- **Status**: validated_local
- **Owner workset**: `W059`

## Source Refs
- **Reported against ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Reproduced on ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `working-tree-uncommitted`

## Ownership And Root Cause
- **Ownership class**: OxFml-owned bug
- **Root cause class**: initial_impl_gap
- **Root cause summary**: OxFml retained local semantic execution for ordinary operators even after the seam doctrine and OxFunc operator catalog established that operator semantic truth should live in OxFunc.

## Reproduction
1. Inspect local arithmetic execution in `crates/oxfml_core/src/eval/mod.rs`.
2. Inspect OxFunc operator rows and dispatch support in:
   - `../OxFunc/crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
   - `../OxFunc/crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
   - `../OxFunc/crates/oxfunc_core/src/functions/operator_reference_family.rs`
   - `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
3. Observe that OxFml currently computes ordinary operator results locally instead of lowering to `FUNC.OP_*` rows.

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
  2. `OPERATIONS.md`
  3. `AGENTS.md`
- **Spec state at intake**: correct_and_not_implemented
- **Notes**: the seam doctrine already says OxFml owns grammar/precedence and OxFunc owns operator semantic truth; the code drifted away from that split.

## Investigation Log
1. 2026-04-07: confirmed OxFml local arithmetic execution still exists in `crates/oxfml_core/src/eval/mod.rs`.
2. 2026-04-07: confirmed OxFunc already exports `FUNC.OP_ADD`, `FUNC.OP_SUBTRACT`, `FUNC.OP_MULTIPLY`, `FUNC.OP_DIVIDE`, `FUNC.OP_POWER`, `FUNC.OP_PERCENT`, `FUNC.OP_CONCAT`, comparison rows, and reference-operator rows.
3. 2026-04-07: confirmed the current OxFunc library-context export marks these operator rows as `built_in_operator` with `preparation_owner = oxfml_then_oxfunc`.
4. 2026-04-07: replaced OxFml-local binary arithmetic execution with OxFunc `FUNC.OP_ADD`, `FUNC.OP_SUBTRACT`, `FUNC.OP_MULTIPLY`, `FUNC.OP_DIVIDE`, and `FUNC.OP_POWER` dispatch in `crates/oxfml_core/src/eval/mod.rs`.
5. 2026-04-07: added evaluator trace assertions proving retained exponentiation rows now lower through `FUNC.OP_*` rather than local OxFml arithmetic helpers.
6. 2026-04-07: widened OxFml grammar/binding to admit postfix percent, concatenation, and ordinary comparison tiers while keeping semantic execution on OxFunc rows.
7. 2026-04-07: removed local unary prefix lowering and now bind `+x` / `-x` as explicit unary nodes that dispatch through `FUNC.OP_UNARY_PLUS` and `FUNC.OP_NEGATE`.
8. 2026-04-07: removed the temporary local array-arithmetic fallback after confirming the current OxFunc operator surfaces now lift admitted array rows in this workspace.
9. 2026-04-07: widened explicit reference-operator dispatch so intersection, union, and spill now travel through OxFunc `FUNC.OP_*_REF` rows instead of stopping inside OxFml.

## Fix Plan
1. preserve OxFml ownership of grammar, precedence, and bind structure while narrowing semantic execution,
2. add deterministic tests proving the admitted operator families still yield the same observed outcomes after seam correction,
3. keep any residual multi-area/reference-model limitations explicit rather than hiding them inside local semantic fallbacks.

## Similar-Risk Scan
### Adjacent families to check
1. unary arithmetic operators
2. postfix percent operator
3. concatenation and comparison operators
4. reference operators (`OP_RANGE_REF`, `OP_INTERSECTION_REF`, `OP_UNION_REF`)

### Check method
1. compare OxFml local evaluator logic with OxFunc `FUNC.OP_*` surface inventory,
2. inspect current parser/binder coverage against the exported operator catalog,
3. add operator-family regression tests at the OxFml consumer/evaluator layer.

### Results
1. the ordinary operator family seam correction is now in the working tree and locally validated across unary, binary arithmetic, postfix percent, concat, comparisons, and explicit reference operators.
2. array-involved ordinary arithmetic no longer requires the earlier local OxFml compatibility fallback in this workspace; see `BUG-FML-004`.
3. concat, comparison, and explicit reference operators now dispatch through OxFunc rows rather than remaining note-only follow-on candidates.
4. remaining residual risk is no longer local semantic execution drift; it is the narrower question of multi-area/reference-model representation policy.

## Linked Reports
1. `BUGREP-FML-003`

## Evidence
1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
2. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
3. `../OxFunc/crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
4. `../OxFunc/crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
5. `../OxFunc/crates/oxfunc_core/src/functions/operator_reference_family.rs`
6. `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
7. `crates/oxfml_core/src/eval/mod.rs`
8. `crates/oxfml_core/tests/evaluator_tests.rs`
9. `cargo test -p oxfml_core`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [x] spec/matrix updated if required
- [ ] handoff filed if required
- [x] linked reports updated
