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
- **Cross-repo**: OxFunc W094 now exposes the final `FormatProfile` semantics requested by `BLK-FML-005`; OxFml consumes those profile facts locally instead of maintaining a duplicate locale registry.

## Scope

### In scope

1. Add OxFml publication evaluation for conditional-formatting rule-kind predicates `blanks`, `noBlanks`, `errors`, `noErrors`, and `dates`.
2. Thread runtime `now_serial` from `TypedContextQueryBundle` into verification publication so relative date predicates are evaluated against the same runtime clock seed as volatile date/time functions.
3. Consume OxFunc's locale-profile breadth and final `FormatProfile` semantics for locale-keyed rendering, short-date parsing, currency layout, format-code token behavior, and locale-prefix custom-format grammar.
4. Split the custom-format grammar handoff into OxFml-local follow-up beads and locale-prefix beads, then validate the locale-prefix bead after OxFunc W094 lands.
5. File DNA OneCalc and OxFunc notes that identify what landed, what is blocked, and what remains in OxFml-local follow-up.

### Out of scope

1. Adding locale profile enum variants or canonical profile constants in OxFunc.
2. Full Excel custom-format grammar parity.
3. DNA OneCalc UI cleanup.
4. Migration/deprecation staging; per user direction, future work should move directly to the target end state.

## Deliverables

1. Conditional-formatting predicate evaluator with deterministic publication-surface tests.
2. Runtime publication path passes `now_serial` into relative-date predicate evaluation.
3. `BLK-FML-005` records the resolved final `FormatProfile` dependency.
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

- **Status**: validated
- **Owner**: OxFunc first, then OxFml
- **Effect**: OxFml consumes OxFunc profile ids/profile constants plus final `FormatProfile` fields for locale-keyed month/weekday rendering, profile-aware General decimal rendering, short-date parsing, currency parsing/rendering, and invariant custom-format numeric tokens.
- **Evidence**: `cargo test -p oxfml_core --test locale_format_expansion_tests`
- **Blocker**: none; `BLK-FML-005` resolved 2026-05-06

### B070-04: Locale-prefix custom-format grammar

- **Status**: validated
- **Owner**: OxFunc first, then OxFml
- **Effect**: parse optional `[$-LCID]` locale prefixes through `LocaleProfileId::from_excel_lcid(...)` and render the selected section with the canonical OxFunc profile facts.
- **Evidence**: `cargo test -p oxfml_core --test locale_format_expansion_tests`
- **Blocker**: none; `BLK-FML-005` resolved 2026-05-06

### B070-05: Custom-format grammar follow-up

- **Status**: validated
- **Owner**: OxFml
- **Effect**: add dedicated evidence and implementation for custom-format grammar items not handled by `W069`, including text fourth-section behavior, selected section colour-token publication through `VerificationPublicationSurface.effective_font_color`, and condition/colour header ordering.
- **Evidence**: `cargo test -p oxfml_core publication::tests::custom_format`
- **Blocker**: none for non-locale pieces

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
4. `crates/oxfml_core/src/format/number.rs`
5. `crates/oxfml_core/src/publication/mod.rs`
6. `crates/oxfml_core/src/format/locale_tables.rs`
7. `crates/oxfml_core/tests/locale_format_expansion_tests.rs`
8. `crates/oxfml_core/src/format/engine.rs`

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | Yes - this workset and handoff notes record the owner split, OxFunc final profile surface, and OxFml consumption evidence. |
| 2 | Conformance matrix rows updated? | Yes - `docs/IN_PROGRESS_FEATURE_WORKLIST.md` records the W070 floor and remaining lanes. |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | Yes - deterministic publication, locale-format, and custom-format tests exist for the admitted W070 slices. |
| 4 | Cross-repo impact assessed and handoff filed if needed? | Yes - DNA OneCalc response notes and an OxFunc locale-profile request are filed. |
| 5 | All required tests pass? | Yes - focused tests, affected W069 publication tests, and full `oxfml_core` suite passed. |
| 6 | No known semantic gaps remain in declared scope? | Yes for the bounded W070 handoff-processing scope; full Excel custom-format parity remains outside this workset. |
| 7 | Completion language audit passed? | Yes - this packet distinguishes bounded W070 evidence from broader formatting parity work. |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | Yes. |
| 9 | CURRENT_BLOCKERS.md updated? | Yes - `BLK-FML-005`. |

## Completion Claim Self-Audit

Result: W070's bounded handoff-processing target is validated; broader Excel formatting parity remains outside this workset.

1. The conditional-formatting predicate slice has focused evidence.
2. Locale expansion consumes OxFunc-owned profile facts rather than adding an OxFml-owned profile registry.
3. Non-locale custom-format colour-token and text fourth-section behavior now has focused local evidence; full custom-format grammar parity is not claimed.
4. Locale-prefix grammar uses OxFunc `from_excel_lcid(...)` and deterministic OxFml evidence.
5. DNA OneCalc cleanup remains downstream-owned after consuming the landing notes.

Validation commands:

1. `cargo fmt --all` - passed.
2. `cargo test -p oxfml_core --test conditional_formatting_predicate_tests` - passed, 4 tests.
3. `cargo test -p oxfml_core --test format_time_fraction_accounting_tests` - passed, 7 tests.
4. `cargo test -p oxfml_core publication::tests::custom_format` - passed, 5 tests.
5. `cargo test -p oxfml_core --test locale_format_expansion_tests` - passed, 6 tests.
6. `cargo test -p oxfml_core --test ftc_0288_separator_context_tests --test ftc_0288_trailing_comma_separator_context_tests --test ftc_0288_adjacent_matrix_tests --test ftc_0288_rule_edge_tests` - passed, 9 tests.
7. `cargo fmt --all -- --check` - passed.
8. `cargo test -p oxfml_core` - passed.
9. `git diff --check` - passed with line-ending warnings only.

## Status

- execution_state: validated
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
- claim_confidence: evidence_backed
