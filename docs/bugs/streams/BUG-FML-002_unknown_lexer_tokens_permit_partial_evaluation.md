# BUG-FML-002: Unknown Lexer Tokens Permit Partial Evaluation Instead Of Deterministic Failure

## Summary
- **Bug id**: `BUG-FML-002`
- **Opened**: 2026-04-06
- **Status**: validated_local
- **Owner workset**: `none yet`

## Source Refs
- **Reported against ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Reproduced on ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `working-tree-uncommitted`

## Ownership And Root Cause
- **Ownership class**: OxFml-owned bug
- **Root cause class**: initial_impl_gap
- **Root cause summary**: the lexer emits `Unknown` tokens for unsupported syntax, but the parser only records a trailing-token diagnostic and still returns an evaluable prefix tree; bind and evaluation then continue and return an authoritative-looking numeric result.

## Reproduction
Representative observed rows:
1. `=2^3^2` -> OxFml `2`, Excel `64`
2. `=2^2*3` -> OxFml `2`, Excel `12`
3. `=1+2*3^2` -> OxFml `7`, Excel `19`

Observed behavior shape:
1. unsupported token appears in formula,
2. syntax diagnostics are recorded,
3. execution still proceeds over the successfully parsed prefix,
4. caller receives a value instead of a deterministic parse failure or typed non-execution outcome.

## Spec Relationship
- **Spec references**:
  1. `CHARTER.md`
  2. `OPERATIONS.md`
  3. `docs/spec/formula-language/OXFML_PARSER_AND_BINDER_REALIZATION.md`
- **Spec state at intake**: vague
- **Notes**: current doctrine strongly favors deterministic authoritative results, but the explicit parser/error-gating rule for lexer-unknown tokens is not yet frozen clearly enough in local docs.

## Investigation Log
1. 2026-04-06: confirmed lexer emits `TokenKind::Unknown` for unsupported characters in `crates/oxfml_core/src/syntax/lexer.rs`.
2. 2026-04-06: confirmed `parse_formula_root` records a diagnostic for unexpected trailing tokens but still returns a green tree in `crates/oxfml_core/src/syntax/parser.rs`.
3. 2026-04-06: confirmed the runtime pipeline can continue through bind/eval and produce numeric output even when unknown-token/trailing-token diagnostics are present.
4. 2026-04-06: execution paths were changed in the working tree so one-shot host/runtime execution rejects formulas with syntax diagnostics before bind/eval publication.
5. 2026-04-06: managed runtime execution was changed in the working tree to preserve open-session diagnostics for analysis but reject execute when syntax diagnostics are present.

## Fix Plan
1. define the required deterministic behavior when lexer-unknown tokens appear in worksheet formulas,
2. prevent ordinary evaluation/publication from proceeding as if a parsed prefix were the full formula,
3. return a clear typed failure to the caller and replay/publication layers,
4. add deterministic tests proving unknown-token cases do not degrade into partial numeric evaluation.

## Similar-Risk Scan
### Adjacent families to check
1. all lexer-unknown operator characters, not only `^`
2. unsupported postfix or prefix operators
3. partial-parse trailing-token cases after a valid expression prefix
4. any execution path that ignores syntax diagnostics and still publishes ordinary values

### Check method
1. inspect lexer `Unknown` handling,
2. inspect parser trailing-token diagnostics behavior,
3. inspect runtime/evaluation gating on syntax diagnostics,
4. add negative tests for unsupported-token formulas.

### Results
1. this is broader than exponentiation and remains a separate stream.
2. the current working-tree fix chooses deterministic execution rejection with a stable error message, while still preserving parse diagnostics for analysis-oriented open-session flows.

## Linked Reports
1. `BUGREP-FML-002`

## Evidence
1. `crates/oxfml_core/src/syntax/token.rs`
2. `crates/oxfml_core/src/syntax/lexer.rs`
3. `crates/oxfml_core/src/syntax/parser.rs`
4. `crates/oxfml_core/src/host/mod.rs`
5. `crates/oxfml_core/src/consumer/runtime/mod.rs`
6. `crates/oxfml_core/tests/host_tests.rs`
7. `crates/oxfml_core/tests/runtime_consumer_facade_tests.rs`
8. `cargo test -p oxfml_core`

## Closure Checklist
- [x] fix landed
- [x] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [ ] handoff filed if required
- [x] linked reports updated
