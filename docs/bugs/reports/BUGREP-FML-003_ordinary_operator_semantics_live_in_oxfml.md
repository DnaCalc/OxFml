# BUGREP-FML-003: Ordinary Operator Semantics Live In OxFml

## Intake
- **Report id**: `BUGREP-FML-003`
- **Filed**: 2026-04-07
- **Source channel**: local test
- **Reporter/source**: local boundary review after `BUG-FML-001`
- **Reported against ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Reported against kind**: commit
- **Canonical bug id**: `BUG-FML-003`
- **Status**: triaged

## Observed Symptom
Ordinary operator semantics such as arithmetic are currently executed inside the OxFml local evaluator even though the OxFml/OxFunc semantic-boundary doctrine says operator semantic truth belongs on the OxFunc side.

## Reproduction
1. Inspect `crates/oxfml_core/src/eval/mod.rs`.
2. Observe local arithmetic execution in `evaluate_binary_numeric_op` and related scalar/array helpers.
3. Compare that with OxFunc operator rows and dispatch support in `../OxFunc/crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`, `../OxFunc/crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`, and `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`.

## Initial Ownership Read
- **Initial classification**: OxFml-owned bug
- **Reason**: the defect is not missing OxFunc capability; it is OxFml continuing to hold semantic execution that should be delegated across the declared seam.

## Links
1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
2. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
3. `docs/bugs/streams/BUG-FML-001_exponentiation_formulas_diverge_from_excel.md`

## Triage Notes
This stream is the ownership-correction follow-on exposed while investigating and fixing the exponentiation admission bug. It should be treated as the canonical parent stream for ordinary operator seam narrowing from OxFml into OxFunc.
