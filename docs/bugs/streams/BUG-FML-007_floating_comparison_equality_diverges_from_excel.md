# BUG-FML-007: Floating Comparison Equality Diverges From Excel

## Summary
- **Bug id**: `BUG-FML-007`
- **Opened**: 2026-04-08
- **Status**: validated_local
- **Owner workset**: `W061`

## Source Refs
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reproduced on ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `sibling-working-tree-after-HO-FN-008`

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: initial_impl_gap
- **Root cause summary**: the current intake reduced to the broader OxFunc
  numeric-comparison family split now documented in `HO-FN-008`: operators,
  criteria/database, and `SWITCH` share a truncation-style 15-significant-digit
  tolerance lane, while `MATCH`, `XMATCH`, and `DELTA` remain exact.

## Reproduction
1. Evaluate `=0.1+0.2=0.3` through current OxFml head plus the current sibling
   OxFunc workspace.
2. Observe that ordinary compare operators now treat the row as `TRUE`.
3. Verify the broader family split with one tolerant `SWITCH` row and one exact
   `DELTA` row.

## Spec Relationship
- **Spec references**:
  1. `../OxFunc/crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
  2. `../OxFunc/docs/handoffs/HO-FN-008_corpus_if_correction_and_numeric_comparison_tolerance.md`
- **Spec state at intake**: vague
- **Notes**: this remains semantic truth in OxFunc's comparison families, not
  an OxFml parse/bind issue.

## Investigation Log
1. 2026-04-08: initial intake isolated `=0.1+0.2=0.3` as an operator-equality
   mismatch.
2. 2026-04-08: filed OxFunc handoff `HANDOFF-OXFUNC-003`.
3. 2026-04-08: `HO-FN-008` widened the lane to operators, criteria/database,
   and `SWITCH`, while pinning `MATCH`, `XMATCH`, and `DELTA` as exact.
4. 2026-04-08: local OxFml regression uptake added operator, `SWITCH`, and
   `DELTA` coverage against the current sibling workspace.

## Fix Plan
1. OxFunc defines and implements the intended floating comparison family split.
2. OxFml consumes the landed behavior without local semantic override.
3. Cross-repo regression coverage should include operator tolerance, at least
   one broader tolerant family (`SWITCH`), and at least one exact-match
   contrast row (`DELTA`).

## Similar-Risk Scan
### Adjacent families to check
1. `<>` and ordered float comparisons
2. criteria/database numeric comparisons in OxFunc
3. exact-match contrast lanes such as `MATCH`, `XMATCH`, and `DELTA`

### Check method
1. inspect OxFunc comparison kernels and widened family note
2. add OxFml evaluator regressions for both tolerant and exact contrast lanes

### Results
1. ordinary operator equality remains OxFunc-owned
2. the live semantic lane is broader than operators and should be tracked that
   way
3. exact-match contrast lanes remain intentionally separate

## Linked Reports
1. `BUGREP-FML-005`

## Evidence
1. `../OxFunc/crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
2. `docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`
3. `../OxFunc/docs/handoffs/HO-FN-008_corpus_if_correction_and_numeric_comparison_tolerance.md`
4. `crates/oxfml_core/tests/evaluator_tests.rs`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [x] handoff filed if required
- [x] linked reports updated
