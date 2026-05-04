# W072: Conditional Formatting Aggregate Visualization Rules

## Purpose

Process DNA OneCalc's `HANDOFF_OXFML_CF_AGGREGATE_VISUALIZATION_RULES.md` by defining the work needed for aggregate-context conditional-formatting visualization rules over scalar and array publication results.

This workset is separate from `W071`: W071 adds the per-cell carrier and scalar/operator/predicate per-cell evaluation. W072 adds the aggregate context and kind-specific visualization computations.

## Position and Dependencies

- **Depends on**: `W071`
- **Blocks**: DNA OneCalc removal of `<NOT IMPL>` seam markers for visualization CF rule families
- **Cross-repo**: DNA OneCalc must consume the W071/W072 per-cell carrier and either emit the bounded `VerificationConditionalFormattingRule` payload convention recorded here or coordinate a later typed-payload replacement.

## Scope

### In scope

1. Define and consume kind-specific rule metadata for aggregate-context CF families.
2. Compute aggregate context once per publication result.
3. Populate the W071 per-cell carrier with color-scale fills, data bars, icons, rank/average/unique/duplicate style outcomes.
4. Preserve rule ordering/priority behavior for mixed scalar and visualization rules.
5. Add deterministic tests for the eight DNA OneCalc-requested cases.

### Out of scope

1. DNA OneCalc UI rendering of bars/icons.
2. Worksheet-range CF semantics outside formula-result publication.
3. Excel pixel-perfect gradient/bar/icon rendering.

## Bead Set

### B072-01: Aggregate rule payload shape

- **Status**: validated
- **Owner**: OxFml with DNA OneCalc bridge alignment
- **Effect**: consume bounded visualization metadata through the existing `VerificationConditionalFormattingRule` shape: color-scale stops as `thresholds` strings (`min:#RRGGBB`, `mid:#RRGGBB`, `max:#RRGGBB`, `percent:50:#RRGGBB`, `num:42:#RRGGBB`), data-bar color from `fill_color`, data-bar options from `thresholds` (`min:n`, `max:n`, `direction:right`, `showBarOnly`), icon-set kind from `thresholds[0]`, icon thresholds from later `thresholds` entries (`num:n`, `percent:n`), top/bottom count-or-percent from `thresholds[0]`, and average options from `thresholds` (`equal`, `stddev:n`).
- **Evidence**: `cargo test -p oxfml_core --test conditional_formatting_array_tests`

### B072-02: Aggregate context precompute

- **Status**: validated
- **Owner**: OxFml
- **Effect**: compute numeric min/max/mean/stddev, sorted numeric thresholds, and distinct visible-value counts once per array or scalar visualization result.
- **Evidence**: `cargo test -p oxfml_core --test conditional_formatting_array_tests`

### B072-03: Color-scale visualization

- **Status**: validated
- **Owner**: OxFml
- **Effect**: populate per-cell `effective_fill_color` using 2-stop and 3-stop interpolation from bounded threshold stop strings; scalar visualization results populate a 1x1 carrier with midpoint color.
- **Evidence**: DNA OneCalc requested `SEQUENCE(5)` red/yellow/green case and scalar degenerate color-scale case.

### B072-04: Data-bar visualization

- **Status**: validated
- **Owner**: OxFml
- **Effect**: populate `ArrayCellFormat.data_bar` with min/max fill ratio, bar color, direction, and bar-only flag.
- **Evidence**: `[10,20,30,40]` ratios `[0.0, 0.333, 0.667, 1.0]` and explicit `min:0` / `max:40` thresholds.

### B072-05: Icon-set visualization

- **Status**: validated
- **Owner**: OxFml
- **Effect**: populate `ArrayCellFormat.icon` with icon-set kind, default equal-width icon bins, and explicit numeric/percent thresholds.
- **Evidence**: `3Arrows` over `SEQUENCE(6)` bins bottom/middle/top pairs and explicit numeric threshold case.

### B072-06: Average and rank predicates

- **Status**: validated
- **Owner**: OxFml
- **Effect**: evaluate `aboveAverage` / `belowAverage` with mean, `equal`, and `stddev:n` options; evaluate count/percent `top` / `bottom` against aggregate numeric context.
- **Evidence**: above/below-average `[1,2,3,4,5]`, equal-average, stddev, top-5 `[1..10]`, and bottom-20% `[1..10]` cases.

### B072-07: Unique and duplicate predicates

- **Status**: validated
- **Owner**: OxFml
- **Effect**: evaluate `uniqueValues` and `duplicateValues` using aggregate distinct-value counts, including non-numeric visible text.
- **Evidence**: `[1,2,1,3]` unique case and `["x","y","x"]` duplicate case.

### B072-08: Mixed-rule priority

- **Status**: validated
- **Owner**: OxFml
- **Effect**: preserve post-priority output when visualization and scalar rules both apply to a cell.
- **Evidence**: color-scale plus `cell_value > 5` case where the later scalar rule contributes red font while scaled fill remains present.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | Yes - this workset and DNA OneCalc handoffs record the bounded payload convention and downstream lane. |
| 2 | Conformance matrix rows updated? | Yes - `docs/IN_PROGRESS_FEATURE_WORKLIST.md` records the W072 floor. |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | Yes - focused publication tests cover all requested aggregate families plus scalar visualization and mixed priority. |
| 4 | Cross-repo impact assessed and handoff filed if needed? | Yes - DNA OneCalc intake, predicate-slice, and visualization-slice notes are filed. |
| 5 | All required tests pass? | Yes. |
| 6 | No known semantic gaps remain in declared scope? | Yes for OxFml-local W072 bounded payload scope; downstream DNA OneCalc consumption remains open. |
| 7 | Completion language audit passed? | Yes. |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | Yes. |
| 9 | CURRENT_BLOCKERS.md updated? | Yes - no blocker required at intake. |

## Completion Claim Self-Audit

Result: passed for the OxFml-local W072 bounded payload scope.

1. All aggregate families named in the DNA OneCalc handoff have focused publication evidence.
2. Scalar visualization output is covered through the 1x1 carrier case.
3. Mixed visualization plus scalar priority is covered.
4. DNA OneCalc rendering remains downstream-owned and is not claimed by OxFml.

Validation commands:

1. `cargo fmt --all -- --check` - passed.
2. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 15 tests.
3. `cargo test -p oxfml_core` - passed.
4. `git diff --check` - passed with CRLF normalization warnings only.

## Status

- execution_state: validated
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes:
  - downstream DNA OneCalc bridge/UI consumption
- claim_confidence: validated
