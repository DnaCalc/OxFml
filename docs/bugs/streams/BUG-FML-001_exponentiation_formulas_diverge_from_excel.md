# BUG-FML-001: Exponentiation Formulas Diverge From Excel

## Summary
- **Bug id**: `BUG-FML-001`
- **Opened**: 2026-04-06
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
- **Root cause summary**: the intake bug was a real OxFml grammar/bind admission gap for `^`, and the follow-up review also exposed a second ownership error: ordinary operator semantics had been kept in OxFml local evaluation instead of being lowered into OxFunc operator rows.

## Reproduction
Observed corpus rows:

| case_id | formula | oxfml_value | excel_value |
|---|---|---:|---:|
| `FTC-0003` | `=2^3^2` | `2` | `64` |
| `FTC-0004` | `=-2^2` | `-2` | `4` |
| `FTC-0008` | `=2^2*3` | `2` | `12` |
| `FTC-0010` | `=1+2*3^2` | `7` | `19` |

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
  2. `docs/spec/formula-language/archive/EXCEL_FORMULA_LANGUAGE_EMPIRICAL_BASELINES.md`
- **Spec state at intake**: correct_and_not_implemented
- **Notes**: the local precedence baseline explicitly places `^` above `*` and `/`, and the empirical baseline records `=-2^2` observed as `4`. Current code does not implement that operator.

## Investigation Log
1. 2026-04-06: confirmed `TokenKind` has no exponent token in `crates/oxfml_core/src/syntax/token.rs`.
2. 2026-04-06: confirmed lexer maps unsupported characters such as `^` to `TokenKind::Unknown` in `crates/oxfml_core/src/syntax/lexer.rs`.
3. 2026-04-06: confirmed parser only has additive and multiplicative tiers in `crates/oxfml_core/src/syntax/parser.rs`.
4. 2026-04-06: confirmed binder `BinaryOp` only supports `Add`, `Subtract`, `Multiply`, `Divide` in `crates/oxfml_core/src/binding/mod.rs`.
5. 2026-04-06: confirmed evaluator binary execution path supports only those same four operations in `crates/oxfml_core/src/eval/mod.rs`.
6. 2026-04-06: added `^` tokenization, parser tiering, binder representation, and a first local evaluator patch in the working tree.
7. 2026-04-06: added direct evaluator coverage for `=2^3^2`, `=-2^2`, `=2^2*3`, and `=1+2*3^2`.
8. 2026-04-07: revisited the fix after ownership review and classified the local evaluator patch as boundary-wrong even though it made the retained rows pass.
9. 2026-04-07: moved ordinary arithmetic operator execution onto OxFunc `FUNC.OP_*` dispatch while keeping the OxFml `^` token/parser/binder admission work.

## Fix Plan
1. add exponent tokenization, parser tiering, and binder representation for worksheet exponentiation,
2. route exponentiation execution through OxFunc operator dispatch rather than OxFml-local arithmetic semantics,
3. align unary interaction and operator-tier behavior with the local formula-language baseline and observed Excel results,
4. add deterministic parser/bind/evaluator evidence for exponentiation rows and their OxFunc operator dispatch trace.

## Similar-Risk Scan
### Adjacent families to check
1. unary `+` / unary `-` with exponentiation
2. exponentiation chained with `*`, `/`, `+`, `-`, `%`, and `&`
3. parenthesized exponentiation shapes
4. any unsupported arithmetic operators or precedence tiers beyond the currently implemented four-op set

### Check method
1. inspect token/parser/binder/evaluator operator enums and tiers,
2. expand deterministic corpus rows around `P2-FML-010`,
3. add direct evaluator and parse/bind tests for admitted exponentiation shapes.

### Results
1. the admitted exponentiation slice now evaluates correctly in the working tree for the retained corpus rows and direct evaluator tests.
2. direct evaluator trace evidence now shows exponentiation and surrounding arithmetic lowering to `FUNC.OP_*` rows instead of OxFml-local arithmetic evaluation.
3. `BUG-FML-002` remains a separate stream because unsupported operators still needed explicit execution gating even after exponentiation admission landed.

## Linked Reports
1. `BUGREP-FML-001`

## Evidence
1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
2. `docs/spec/formula-language/archive/EXCEL_FORMULA_LANGUAGE_EMPIRICAL_BASELINES.md`
3. `crates/oxfml_core/src/syntax/token.rs`
4. `crates/oxfml_core/src/syntax/lexer.rs`
5. `crates/oxfml_core/src/syntax/parser.rs`
6. `crates/oxfml_core/src/binding/mod.rs`
7. `crates/oxfml_core/src/eval/mod.rs`
8. `crates/oxfml_core/tests/evaluator_tests.rs`
9. `docs/worksets/W059_operator_semantic_dispatch_boundary_correction.md`
10. `cargo test -p oxfml_core`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [ ] handoff filed if required
- [x] linked reports updated
