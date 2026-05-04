*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-006: W070 Conditional-Formatting Predicate Rules Landing

Status: `filed`
Source repo/workset: `OxFml/W070`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_CF_PREDICATE_AND_RELATIVE_DATE_RULES.md`

## Purpose

Record OxFml's response to the DnaOneCalc observation that operator and expression conditional-formatting rules were evaluated in publication, while rule-kind predicates were surfaced with `applies: null`.

## OxFml Landing Summary

OxFml publication now evaluates these rule-kind predicates:

1. `blanks`
2. `noBlanks`
3. `errors`
4. `noErrors`
5. `dates`

Relative date predicates use the runtime `TypedContextQueryBundle.now_serial` value when publication is produced by `SingleFormulaHost`. Replay-capture conversion paths that do not have a live query bundle pass `None`, leaving relative date predicate applicability unknown rather than inventing a clock value.

Direct callers of `build_verification_publication_surface(...)` now pass `now_serial: Option<f64>` immediately before the publication context argument. Use `Some(serial)` for live relative-date evaluation and `None` when no deterministic clock seed is available.

Supported relative date thresholds:

1. `today`
2. `yesterday`
3. `tomorrow`
4. `last7Days`
5. `thisWeek`
6. `lastWeek`
7. `nextWeek`
8. `thisMonth`
9. `lastMonth`
10. `nextMonth`

Unknown predicates and relative date predicates without `now_serial` continue to surface `applies: null`.

## Evidence

Relevant OxFml files:

1. `crates/oxfml_core/src/publication/mod.rs`
2. `crates/oxfml_core/src/host/mod.rs`
3. `crates/oxfml_core/tests/conditional_formatting_predicate_tests.rs`

Focused validation:

1. `cargo test -p oxfml_core --test conditional_formatting_predicate_tests` - passed, 4 tests.

## Downstream Cleanup

After consuming this OxFml revision, DNA OneCalc can remove the temporary expectation that these rule-kind predicates remain `null` in OxFml publication surfaces. DNA OneCalc should still treat unknown predicates and relative-date predicates without a supplied `now_serial` as unevaluated.

## Non-Claims

This handoff does not claim:

1. icon sets, data bars, or color scales,
2. formula-evaluator parity for every possible conditional-formatting formula,
3. DNA OneCalc UI or replay-pack cleanup,
4. locale expansion or custom-format grammar parity.
