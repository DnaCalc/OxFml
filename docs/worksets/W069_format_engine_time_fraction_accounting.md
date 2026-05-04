# W069: Format Engine Time, Fraction, And Accounting Expansion

## Purpose

Process DNA OneCalc's `HANDOFF_OXFML_FORMAT_ENGINE_TIME_FRACTION_ACCOUNTING.md` by moving common user-authored custom number formats from silent fallback or unsupported-code behavior into OxFml's format engine.

The immediate user-facing defect was `=NOW()` with custom code `HH:mm:ss` rendering as a date-only fallback. The ownership issue is OxFml-local: the host passed the explicit format code through `VerificationPublicationContext.number_format_code`, but `render_with_code(...)` rejected the code before publication fell back to the `DateLike` default.

## Scope

### In Scope

1. Render common time tokens: `h`, `hh`, `m`, `mm`, `s`, `ss`, `AM/PM`, `am/pm`.
2. Render datetime composites such as `m/d/yyyy h:mm` and `yyyy-mm-dd hh:mm:ss`.
3. Render elapsed-time tokens `[h]`, `[m]`, and `[s]`.
4. Render simple fraction patterns: `?/?`, `??/??`, `# ?/?`, `# ??/??`, and `0/0`.
5. Preserve accounting parentheses behavior for common currency section patterns.
6. Ensure publication uses a recognized user-supplied time code rather than falling back to a presentation hint.
7. Convert the prior FTC-0654 unsupported-fraction evidence into positive formatter/evaluator/runtime/adapter evidence.
8. File a DNA OneCalc-facing landing note.

### Out Of Scope

1. Full Excel custom-format grammar parity.
2. Locale-specific month/day names beyond the current OxFml locale table.
3. General text-section formatting parity.
4. Exact pixel/alignment behavior for UI accounting columns.
5. DnaOneCalc format-picker UI marker removal.

## Implementation Summary

1. `format::datetime` now tokenizes and renders shared date/time sections rather than rejecting sections that contain time tokens.
2. `format::number` now routes date/time sections through that renderer.
3. Bracketed elapsed-time tokens are preserved by `strip_condition_and_color_tokens(...)` instead of being mistaken for color tokens.
4. Simple fraction patterns render through a bounded rational approximation path before generic numeric parsing.
5. Publication-surface evidence proves `HH:mm:ss` is honored when supplied by the user.

## Evidence

Focused deterministic tests:

1. `crates/oxfml_core/tests/format_time_fraction_accounting_tests.rs`
   - time token and AM/PM output,
   - datetime composite output,
   - elapsed `[h]`, `[m]`, `[s]` output,
   - simple fraction output,
   - accounting parentheses patterns,
   - publication-surface explicit time-format behavior,
   - runtime `TEXT(...,"# ?/?")` behavior.
2. `crates/oxfml_core/tests/ftc_0654_fraction_format_engine_tests.rs`
   - formatter/evaluator/runtime/adapter fraction behavior.
3. `crates/oxfml_core/src/publication/mod.rs`
   - existing unit coverage now expects `m/d/yyyy h:mm` to render rather than fail.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | Yes - workset and DNA OneCalc notes updated; no shared seam spec shape changed. |
| 2 | Conformance matrix rows updated? | Yes - `docs/IN_PROGRESS_FEATURE_WORKLIST.md` records the new formatting floor. |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | Yes - formatter, publication, runtime, evaluator, and adapter tests exercise the claimed behavior. |
| 4 | Cross-repo impact assessed and handoff filed if needed? | Yes - DNA OneCalc landing handoff filed. |
| 5 | All required tests pass? | Yes. |
| 6 | No known semantic gaps remain in declared scope? | Yes - within the bounded W069 scope; full Excel custom-format grammar remains out of scope. |
| 7 | Completion language audit passed? | Yes. |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | Yes. |
| 9 | CURRENT_BLOCKERS.md updated? | Yes - no new blocker entry required. |

## Completion Claim Self-Audit

Result: passed.

1. Declared scope is bounded to common time/datetime/fraction/accounting format-code families, not full Excel format grammar parity.
2. Evidence includes direct formatter tests and publication/runtime/evaluator/adapter paths.
3. DnaOneCalc UI cleanup remains downstream-owned.
4. No new shared coordinator-facing seam packet was introduced.

Validation commands:
1. `cargo fmt --all -- --check` - passed.
2. `cargo test -p oxfml_core --test format_time_fraction_accounting_tests` - passed, 7 tests.
3. `cargo test -p oxfml_core --test ftc_0654_fraction_format_engine_tests` - passed, 5 tests.
4. `cargo test -p oxfml_core number_format_code_heuristics_cover_grouping_percent_date_and_negative_sections` - passed.
5. `cargo test -p oxfml_core` - passed.
6. `git diff --check` - passed with line-ending warnings only.

## Status

- execution_state: validated
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
