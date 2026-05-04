# W071: Conditional Formatting Array Per-Cell Publication

## Purpose

Process DNA OneCalc's `HANDOFF_OXFML_CF_ARRAY_PER_CELL.md` by making OxFml evaluate scalar conditional-formatting rules per cell when the published result is an array.

The ownership issue is OxFml-local: publication previously evaluated conditional formatting once against the whole `EvalValue::Array`, which meant operator and predicate rules compared against array summary text rather than individual array cells.

## Position and Dependencies

- **Depends on**: `W070`
- **Blocks**: DNA OneCalc result-hero array cell styling for scalar/operator/predicate CF rules
- **Cross-repo**: DNA OneCalc must consume the new `array_cell_format` carrier

## Scope

### In scope

1. Add a per-cell array formatting carrier to `VerificationPublicationSurface`.
2. Evaluate existing scalar/operator/predicate CF rules once per array cell.
3. Reuse the same `LocaleFormatContext` and `now_serial` as scalar publication.
4. Treat `ArrayCellValue::EmptyCell` as blank for `blanks` predicate evaluation.
5. Preserve scalar publication behavior and make 1x1 array per-cell output agree with whole-cell CF fields.
6. File a DNA OneCalc landing note.

### Out of scope

1. Aggregate-context visualization rules: `colorScale`, `dataBar`, `iconSet`, `aboveAverage`, `belowAverage`, `top`, `bottom`, `uniqueValues`, `duplicateValues`.
2. Worksheet-range CF semantics outside formula-result publication.
3. DNA OneCalc bridge/UI rendering.

Aggregate visualization follow-up is tracked separately as `W072`.

## Deliverables

1. `VerificationPublicationSurface.array_cell_format`.
2. Public carrier structs: `ArrayCellFormatGrid`, `ArrayCellFormat`, `DataBarFill`, `DataBarDirection`, and `CfIcon`.
3. Per-cell CF publication tests for operator, predicate, relative-date, blank, error, and 1x1-array cases.
4. DNA OneCalc handoff response.

## Gate Model

### Entry gate

- `W070` conditional-formatting predicate evaluation exists.
- DNA OneCalc per-cell array handoff has been reviewed.

### Exit gate

- Per-cell scalar/operator/predicate CF evidence passes.
- Existing W070 predicate evidence remains green.
- Full `oxfml_core` validation passes before promotion.

## Evidence

Focused deterministic tests:

1. `crates/oxfml_core/tests/conditional_formatting_array_tests.rs`
   - `cell_value > 3` over a 2x3 array applies only to cells 4, 5, and 6,
   - `errors` applies only to error array cells,
   - `blanks` applies to empty array cells and empty text cells,
   - relative `dates/today` reuses `now_serial` per cell,
   - 1x1 array per-cell fields match whole-cell CF fields.
2. `crates/oxfml_core/tests/conditional_formatting_predicate_tests.rs`
   - W070 scalar predicate floor remains green.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | Yes - W071 and DNA OneCalc landing note describe the new carrier and per-cell semantics. |
| 2 | Conformance matrix rows updated? | Yes - `docs/IN_PROGRESS_FEATURE_WORKLIST.md` records W071. |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | Yes - focused publication tests cover each in-scope behavior. |
| 4 | Cross-repo impact assessed and handoff filed if needed? | Yes - DNA OneCalc landing handoff filed. |
| 5 | All required tests pass? | Yes. |
| 6 | No known semantic gaps remain in declared scope? | Yes - aggregate visualization rules are explicitly outside W071 and tracked by W072. |
| 7 | Completion language audit passed? | Yes - aggregate work is not claimed by W071. |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | Yes. |
| 9 | CURRENT_BLOCKERS.md updated? | Yes - no new blocker required. |

## Completion Claim Self-Audit

Result: passed.

1. Declared scope is bounded to per-cell evaluation of already-supported scalar/operator/predicate CF rules.
2. Aggregate visualization families are separated into W072.
3. DNA OneCalc bridge/UI rendering remains downstream-owned.
4. The new `data_bar` and `icon` carrier slots are present for W072 consumption but are not reported as aggregate-rule implementation.

Validation commands:

1. `cargo fmt --all -- --check` - passed.
2. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 4 tests.
3. `cargo test -p oxfml_core --test conditional_formatting_predicate_tests` - passed, 4 tests.
4. `cargo test -p oxfml_core` - passed.

## Status

- execution_state: validated
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes:
  - downstream DNA OneCalc bridge/UI consumption
- claim_confidence: validated
