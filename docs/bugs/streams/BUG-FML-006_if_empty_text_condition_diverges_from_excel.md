# BUG-FML-006: IF Empty-Text Condition Diverges From Excel

## Summary
- **Bug id**: `BUG-FML-006`
- **Opened**: 2026-04-08
- **Status**: closed
- **Owner workset**: `W061`

## Source Refs
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reproduced on ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not_applicable`

## Ownership And Root Cause
- **Ownership class**: unknown
- **Root cause class**: unknown
- **Root cause summary**: `HO-FN-008` corrected the intake. Current Excel
  replay returns `#VALUE!` for `=IF("",1,2)`, so this was not a live OxFunc
  semantic bug. The observed corpus failure reduced to the OxFml
  worksheet-error projection bug tracked under `BUG-FML-009`.

## Reproduction
1. Initial corpus intake reported `=IF("",1,2)` as if Excel returned `2`.
2. Current Excel replay from `HO-FN-008` shows `#VALUE!`.
3. Earlier local reading was incorrect and is now closed as a corrected intake
   rather than an active semantic bug.

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
  2. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
- **Spec state at intake**: corrected intake
- **Notes**: the local misread came from treating a fatal OxFml evaluation
  failure as if it were an OxFunc semantic mismatch.

## Investigation Log
1. 2026-04-08: initial corpus triage attributed `=IF("",1,2)` to OxFunc.
2. 2026-04-08: `HO-FN-008` corrected the Excel replay outcome to `#VALUE!`.
3. 2026-04-08: local review reduced the live issue to OxFml bug
   `BUG-FML-009`.

## Fix Plan
1. Close this stream as corrected intake.
2. Keep the real seam bug tracked under `BUG-FML-009`.
3. Add direct OxFml regression coverage so future corpus reads cannot re-open
   this as an OxFunc semantic lane by mistake.

## Similar-Risk Scan
### Adjacent families to check
1. other corpus rows that may actually be worksheet-error projection failures
2. OxFml paths that already normalize worksheet-visible errors correctly

### Check method
1. compare the main OxFml eval path against existing adapter and host
   result-shape handling

### Results
1. this stream is closed as a corrected intake, not as a live OxFunc defect
2. the real local issue is `BUG-FML-009`

## Linked Reports
1. `BUGREP-FML-005`
2. `BUGREP-FML-006`

## Evidence
1. `docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
2. `../OxFunc/docs/handoffs/HO-FN-008_corpus_if_correction_and_numeric_comparison_tolerance.md`
3. `docs/bugs/streams/BUG-FML-009_oxfunc_worksheet_errors_are_promoted_to_fatal_failures.md`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [x] handoff filed if required
- [x] linked reports updated
