# BUGREP-FML-006: OxFunc Worksheet Errors Become Fatal Evaluation Failures

## Intake
- **Report id**: `BUGREP-FML-006`
- **Filed**: 2026-04-08
- **Source channel**: upstream handoff
- **Reporter/source**: `OxFunc` via `HO-FN-008` plus local OxFml code review
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reported against kind**: commit
- **Canonical bug id**: `BUG-FML-009`
- **Status**: triaged

## Observed Symptom
Ordinary worksheet-visible OxFunc errors such as `ABS("x") -> #VALUE!` are
still promoted by OxFml eval into fatal `EvaluationError` failures on one path,
instead of becoming `EvalValue::Error(WorksheetErrorCode::Value)`.

## Reproduction
1. Evaluate `=ABS("x")` through current OxFml head.
2. Evaluate `=IF("",1,2)` through current OxFml head.
3. Observe that OxFunc returns ordinary worksheet errors, but OxFml turns them
   into fatal evaluation failures in `evaluate_function_call(...)`.

## Initial Ownership Read
- **Initial classification**: OxFml-owned bug
- **Reason**: the OxFunc surface already uses worksheet-error return codes as
  normal worksheet-visible outcomes; the OxFml eval seam was still promoting
  those codes to fatal adapter failure.

## Links
1. `docs/bugs/streams/BUG-FML-009_oxfunc_worksheet_errors_are_promoted_to_fatal_failures.md`
2. `docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
3. `../OxFunc/docs/handoffs/HO-FN-008_corpus_if_correction_and_numeric_comparison_tolerance.md`

## Triage Notes
1. This explains why `=IF("",1,2)` initially appeared to be an OxFunc semantic
   mismatch from the corpus lane.
2. The real bug was wider than `IF`: ordinary worksheet-visible OxFunc errors
   were not being normalized into `EvalValue::Error(...)` on the main eval path.
