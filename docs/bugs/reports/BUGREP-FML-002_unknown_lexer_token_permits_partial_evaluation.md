# BUGREP-FML-002: Unknown Lexer Token Permits Partial Evaluation

## Intake
- **Report id**: `BUGREP-FML-002`
- **Filed**: 2026-04-06
- **Source channel**: local_investigation
- **Reporter/source**: local investigation during `BUG-FML-001`
- **Reported against ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Reported against kind**: commit
- **Canonical bug id**: `BUG-FML-002`
- **Status**: triaged

## Observed Symptom
When the lexer encounters an unsupported token such as `^`, OxFml records a syntax problem but still evaluates the successfully parsed prefix of the formula and returns a numeric result to the caller.

## Reproduction
Representative observed rows:
1. `=2^3^2` -> OxFml `2`, Excel `64`
2. `=2^2*3` -> OxFml `2`, Excel `12`
3. `=1+2*3^2` -> OxFml `7`, Excel `19`

Observed implementation path:
1. lexer emits `TokenKind::Unknown` for `^`
2. parser accepts a valid prefix expression and records trailing-token diagnostics
3. bind/eval continue over that prefix tree
4. caller receives an authoritative-looking wrong numeric result instead of a deterministic parse failure

## Initial Ownership Read
- **Initial classification**: OxFml-owned bug
- **Reason**: this is parser/diagnostic/execution-gating behavior inside OxFml, independent of whether exponentiation itself is implemented.

## Links
1. `docs/bugs/streams/BUG-FML-002_unknown_lexer_tokens_permit_partial_evaluation.md`
2. `crates/oxfml_core/src/syntax/lexer.rs`
3. `crates/oxfml_core/src/syntax/parser.rs`

## Triage Notes
This is intentionally filed as a separate bug stream from exponentiation support. Even with exponentiation unsupported, the correct behavior is likely a clear deterministic parse failure rather than partial evaluation.
