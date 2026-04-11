# BUG-FML-008: Multi-Character Comparison Tokens Advance Incorrectly

## Summary
- **Bug id**: `BUG-FML-008`
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
- **Root cause class**: code_regression
- **Root cause summary**: the lexer built two-character comparison tokens such as `<>`, `<=`, and `>=`, but left the cursor one character short of the token end, so the second character was re-lexed as an overlapping token.

## Reproduction
1. Evaluate formulas using multi-character comparison operators such as `=1<>2` or `=2>=1`.
2. Expected behavior: the operator is tokenized once and the comparison evaluates.
3. Actual behavior before the fix: the second character was re-lexed, leading to parse or evaluation failure.

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
- **Spec state at intake**: correct_and_not_implemented
- **Notes**: the operator family was already admitted in spec and partially wired in OxFml, but the lexer cursor movement was inconsistent with the token span.

## Investigation Log
1. 2026-04-08: corpus-derived regression test `=1<>2` failed on current head even though `TokenKind::NotEqual` already existed.
2. 2026-04-08: confirmed the lexer advanced only one character inside the `<` / `>` two-character branches and never normalized `index` to `token.span.end()`.
3. 2026-04-08: corrected post-token cursor advancement to honor the full token span.

## Fix Plan
1. normalize lexer cursor advancement to the emitted token span,
2. prove multi-character comparison operators through evaluator regression coverage,
3. rerun the full local validation floor.

## Similar-Risk Scan
### Adjacent families to check
1. other two-character operator tokens
2. previously reported comparison-family corpus rows

### Check method
1. evaluator regression coverage for `<>` and `>=`
2. full local test floor after lexer change

### Results
1. `<>` and `>=` are now directly exercised in `evaluator_tests.rs`.
2. the general cursor fix also protects other token branches where the branch-specific code sets a span wider than the current cursor position.

## Linked Reports
1. `BUGREP-FML-005`

## Evidence
1. `crates/oxfml_core/src/syntax/lexer.rs`
2. `crates/oxfml_core/tests/evaluator_tests.rs`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [ ] handoff filed if required
- [x] linked reports updated
