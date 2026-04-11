# BUG-FML-009: OxFunc Worksheet Errors Are Promoted To Fatal Failures

## Summary
- **Bug id**: `BUG-FML-009`
- **Opened**: 2026-04-08
- **Status**: validated_local
- **Owner workset**: `W061`

## Source Refs
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reproduced on ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `working-tree-uncommitted`

## Ownership And Root Cause
- **Ownership class**: OxFml-owned bug
- **Root cause class**: initial_impl_gap
- **Root cause summary**: `evaluate_function_call(...)` converted
  `eval_surface_value_call_with_callable(...)` errors into fatal
  `EvaluationError` values, except for the narrow host-query fallback, instead
  of normalizing ordinary worksheet-visible OxFunc errors into
  `EvalValue::Error(...)`.

## Reproduction
1. Evaluate `=ABS("x")` through current OxFml head.
2. Observe a fatal OxFml evaluation failure instead of `Error(Value)`.
3. Compare with OxFunc's intended seam contract and OxFml's own adapter/host
   lanes, which already treat ordinary worksheet errors as worksheet-visible
   values.

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
  2. `crates/oxfml_core/src/oxfunc_adapter/mod.rs`
  3. `crates/oxfml_core/src/host/mod.rs`
- **Spec state at intake**: implemented inconsistently
- **Notes**: the intended seam already expected ordinary worksheet errors to
  travel as `EvalValue::Error(...)`; the main eval path had drifted away from
  that rule.

## Investigation Log
1. 2026-04-08: reviewed OxFunc packet `HO-FN-008`.
2. 2026-04-08: confirmed `evaluate_function_call(...)` in
   `crates/oxfml_core/src/eval/mod.rs` mapped OxFunc surface errors into fatal
   `EvaluationError`.
3. 2026-04-08: patched that path so ordinary worksheet errors now normalize to
   `EvalValue::Error(code)` and only host-query fallback keeps its narrowed
   `Value` mapping.
4. 2026-04-08: added focused evaluator regressions for `ABS("x")` and
   `IF("",1,2)`.

## Fix Plan
1. Normalize ordinary OxFunc surface worksheet errors to `EvalValue::Error(...)`
   in the main eval path.
2. Keep host-query fallback behavior explicit rather than using it as the only
   worksheet-error projection path.
3. Validate with direct evaluator regressions and the `oxfml_core` floor.

## Similar-Risk Scan
### Adjacent families to check
1. other ordinary worksheet-visible function errors
2. previous corpus rows that may have been mis-triaged as OxFunc semantics
3. host and adapter paths that already normalize worksheet errors correctly

### Check method
1. compare `evaluate_function_call(...)` with the adapter and host result-shape
   handling
2. add focused evaluator coverage for both a direct worksheet-error case and
   the previously mis-triaged `IF("",1,2)` row

### Results
1. adapter and host paths were already consistent with `EvalValue::Error(...)`
2. the drift was local to the main eval path
3. the earlier `IF("",1,2)` OxFunc claim was corrected to a local OxFml seam
   bug plus a corpus-read mistake

## Linked Reports
1. `BUGREP-FML-006`
2. `BUGREP-FML-005`

## Evidence
1. `crates/oxfml_core/src/eval/mod.rs`
2. `crates/oxfml_core/tests/evaluator_tests.rs`
3. `crates/oxfml_core/tests/w049_oxfunc_adapter_tests.rs`
4. `../OxFunc/docs/handoffs/HO-FN-008_corpus_if_correction_and_numeric_comparison_tolerance.md`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [x] handoff filed if required
- [x] linked reports updated
