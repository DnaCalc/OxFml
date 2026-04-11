# BUG-FML-005: Scientific Numeric Literals Are Not Lexed

## Summary
- **Bug id**: `BUG-FML-005`
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
- **Root cause summary**: the lexer admitted only digit-and-decimal scans and stopped before exponent markers, so literals such as `1E+308` degraded into a `Number` token followed by trailing identifier/sign tokens.

## Reproduction
1. Evaluate a formula containing a scientific numeric literal such as `=1E+3+2`.
2. Expected behavior: the literal is tokenized as one number token and the formula evaluates.
3. Actual behavior before the fix: the parser hit trailing-token diagnostics because `E+3` was not part of the numeric token.

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
- **Spec state at intake**: correct_and_not_implemented
- **Notes**: the formula-language floor already admits ordinary numeric literals; exponent notation belongs to that same literal family rather than a separate semantic exception.

## Investigation Log
1. 2026-04-08: confirmed current-head lexer scanned only digits and `.` in `crates/oxfml_core/src/syntax/lexer.rs`.
2. 2026-04-08: confirmed current-head operator parse complaints in the same corpus batch were stale, isolating the live literal gap to exponent notation.
3. 2026-04-08: widened number-token scanning to admit exponent markers and leading-decimal scientific forms.
4. 2026-04-08: added focused evaluator coverage for `=1E+3+2` and `=.5E+1`.

## Fix Plan
1. widen OxFml number-token scanning to admit exponent notation without reopening local operator semantics,
2. add deterministic tests proving scientific literals evaluate through the current OxFunc-backed operator path,
3. rerun local validation floor.

## Similar-Risk Scan
### Adjacent families to check
1. leading-decimal numeric literals
2. negative fractional power publication path
3. stale corpus reports for comparison and concat operators

### Check method
1. evaluator regression tests on representative literals and adjacent operators

### Results
1. leading-decimal scientific notation is now exercised with `=.5E+1`.
2. `=(-1)^0.5` now proves current-head `#NUM!` behavior locally and is not an open OxFml bug.
3. comparison and concat operator reports from the same batch are stale on current head and remain covered under `BUG-FML-003`.

## Linked Reports
1. `BUGREP-FML-005`

## Evidence
1. `crates/oxfml_core/tests/evaluator_tests.rs`
2. `crates/oxfml_core/src/syntax/lexer.rs`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [ ] handoff filed if required
- [ ] linked reports updated
