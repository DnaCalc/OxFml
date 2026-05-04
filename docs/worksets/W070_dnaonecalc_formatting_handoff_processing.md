# W070: DNA OneCalc Formatting Handoff Processing

## Purpose

Process three DNA OneCalc formatting handoffs as one ownership review, separating OxFml-local work from locale-profile work that belongs behind the OxFunc locale seam.

Inbound handoffs:

1. `../DnaOneCalc/docs/HANDOFF_OXFML_LOCALE_EXPANSION.md`
2. `../DnaOneCalc/docs/HANDOFF_OXFML_CF_PREDICATE_AND_RELATIVE_DATE_RULES.md`
3. `../DnaOneCalc/docs/HANDOFF_OXFML_CUSTOM_FORMAT_GRAMMAR.md`

## Position and Dependencies

- **Depends on**: `W057`, `W069`, OxFunc locale-format seam shape
- **Blocks**: downstream DNA OneCalc removal of conditional-formatting predicate warnings and locale/custom-format markers
- **Cross-repo**: `BLK-FML-005` blocks locale expansion until OxFunc exposes canonical locale profile identities and profile constants beyond the current two-profile surface

## Scope

### In scope

1. Add OxFml publication evaluation for conditional-formatting rule-kind predicates `blanks`, `noBlanks`, `errors`, `noErrors`, and `dates`.
2. Thread runtime `now_serial` from `TypedContextQueryBundle` into verification publication so relative date predicates are evaluated against the same runtime clock seed as volatile date/time functions.
3. Record locale expansion as blocked on OxFunc locale-profile API breadth instead of creating an OxFml-owned duplicate locale registry.
4. Split the custom-format grammar handoff into OxFml-local follow-up beads and locale-prefix beads blocked by `BLK-FML-005`.
5. File DNA OneCalc and OxFunc notes that identify what landed, what is blocked, and what remains in OxFml-local follow-up.

### Out of scope

1. Adding locale profile enum variants or canonical profile constants in OxFunc.
2. Full Excel custom-format grammar parity.
3. DNA OneCalc UI cleanup.
4. Migration/deprecation staging; per user direction, future work should move directly to the target end state.

## Deliverables

1. Conditional-formatting predicate evaluator with deterministic publication-surface tests.
2. Runtime publication path passes `now_serial` into relative-date predicate evaluation.
3. `BLK-FML-005` records the locale-profile dependency.
4. Handoff notes to DNA OneCalc and OxFunc.
5. Bead set for remaining locale/custom-format grammar follow-through.

## Bead Set

### B070-01: Conditional-formatting predicate publication

- **Status**: validated
- **Owner**: OxFml
- **Effect**: evaluate `blanks`, `noBlanks`, `errors`, `noErrors`, and `dates` rule-kind predicates into `conditional_formatting_applies`.
- **Evidence**: `cargo test -p oxfml_core --test conditional_formatting_predicate_tests`

### B070-02: Runtime now-serial publication threading

- **Status**: validated
- **Owner**: OxFml
- **Effect**: `SingleFormulaHost` passes `TypedContextQueryBundle.now_serial` through `build_verification_publication_surface(...)`; replay-capture conversion without a live query bundle passes `None`.
- **Evidence**: `cargo test -p oxfml_core --test conditional_formatting_predicate_tests`

### B070-03: Locale profile expansion request

- **Status**: blocked
- **Owner**: OxFunc first, then OxFml
- **Effect**: after OxFunc exposes canonical locale ids/profile constants, OxFml adds locale-keyed month/weekday names, parser branches, separators, currency, and General rendering expectations.
- **Blocker**: `BLK-FML-005`

### B070-04: Locale-prefix custom-format grammar

- **Status**: blocked
- **Owner**: OxFunc first, then OxFml
- **Effect**: parse optional locale prefixes without making OxFml the canonical locale registry owner.
- **Blocker**: `BLK-FML-005`

### B070-05: Custom-format grammar follow-up

- **Status**: planned
- **Owner**: OxFml
- **Effect**: add dedicated evidence and implementation for custom-format grammar items not handled by `W069`, including text fourth-section behavior and exposing applied color information from selected format sections where the publication seam can carry it.
- **Blocker**: none known for non-locale pieces

## Evidence

Focused deterministic tests:

1. `crates/oxfml_core/tests/conditional_formatting_predicate_tests.rs`
   - blank and nonblank predicates,
   - error and non-error predicates,
   - relative date predicates with `now_serial`,
   - unknown predicates remain unevaluated.

Changed runtime/publication paths:

1. `crates/oxfml_core/src/publication/mod.rs`
2. `crates/oxfml_core/src/host/mod.rs`
3. `crates/oxfml_core/tests/format_time_fraction_accounting_tests.rs`

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | Partial - this workset and handoff notes record the split; no shared seam spec change was required for B070-01/B070-02. |
| 2 | Conformance matrix rows updated? | Yes - `docs/IN_PROGRESS_FEATURE_WORKLIST.md` records the W070 floor and remaining lanes. |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | Partial - deterministic publication tests exist for B070-01/B070-02; locale and broader custom-format follow-up remain open. |
| 4 | Cross-repo impact assessed and handoff filed if needed? | Yes - DNA OneCalc response notes and an OxFunc locale-profile request are filed. |
| 5 | All required tests pass? | Yes - focused tests, affected W069 publication tests, and full `oxfml_core` suite passed. |
| 6 | No known semantic gaps remain in declared scope? | Partial - B070-03/B070-04 are blocked and B070-05 is planned. |
| 7 | Completion language audit passed? | Yes - this packet reports open lanes as partial/blocked/planned. |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | Yes. |
| 9 | CURRENT_BLOCKERS.md updated? | Yes - `BLK-FML-005`. |

## Completion Claim Self-Audit

Result: not applicable to the whole W070 workset because locale and broader custom-format lanes remain open.

1. The conditional-formatting predicate slice has focused evidence.
2. Locale expansion is not represented as OxFml-local implementation because OxFunc owns the canonical locale-profile registry.
3. Custom-format grammar parity is not claimed.
4. DNA OneCalc cleanup remains downstream-owned after consuming the landing notes.

Validation commands:

1. `cargo fmt --all` - passed.
2. `cargo test -p oxfml_core --test conditional_formatting_predicate_tests` - passed, 4 tests.
3. `cargo test -p oxfml_core --test format_time_fraction_accounting_tests` - passed, 7 tests.
4. `cargo fmt --all -- --check` - passed.
5. `cargo test -p oxfml_core` - passed.
6. `git diff --check` - passed with line-ending warnings only.

## Status

- execution_state: blocked
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - `BLK-FML-005` OxFunc locale-profile expansion
  - `B070-04` locale-prefix custom-format grammar
  - `B070-05` non-locale custom-format grammar follow-up
- claim_confidence: provisional
