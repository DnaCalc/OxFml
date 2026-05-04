*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-010: W072 Aggregate Predicate Slice

Status: `filed`
Source repo/workset: `OxFml/W072`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_CF_AGGREGATE_VISUALIZATION_RULES.md`

## Purpose

Record the first W072 aggregate-context landing over the W071 per-cell carrier. This slice handles aggregate predicate families that can be evaluated with the current scalar rule shape.

## OxFml Landing Summary

OxFml now evaluates these aggregate-context rule families per array cell:

1. `aboveAverage`
2. `belowAverage`
3. `top`
4. `bottom`
5. `uniqueValues`
6. `duplicateValues`

Implemented aggregate context:

1. numeric mean for `aboveAverage` / `belowAverage`,
2. sorted numeric values for `top` / `bottom`,
3. visible-value counts for `uniqueValues` / `duplicateValues`,
4. count and percent thresholds for `top` / `bottom`.

The result is published through `VerificationPublicationSurface.array_cell_format`.

## Evidence

Relevant OxFml files:

1. `crates/oxfml_core/src/publication/mod.rs`
2. `crates/oxfml_core/tests/conditional_formatting_array_tests.rs`

Focused validation:

1. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 7 tests.

Covered cases:

1. above/below average over `[1,2,3,4,5]`,
2. top-5 over `[1..10]`,
3. bottom-20% over `[1..10]`,
4. unique values over `[1,2,1,3]`,
5. duplicate visible text over `["x","y","x"]`.

## Remaining W072 Lanes

Still open:

1. `colorScale`,
2. `dataBar`,
3. `iconSet`,
4. min/max/stddev aggregate context,
5. richer average/rank flags such as equal-average and stddev offsets,
6. richer visualization rule payload shape.

## DnaOneCalc Guidance

DNA OneCalc can consume this as an incremental W072 subset for aggregate predicate rules. Keep `<NOT IMPL>` markers for color scales, data bars, icon sets, and any UI options that require the richer payload shape above.

## Non-Claims

This handoff does not claim color-scale, data-bar, icon-set, or full Excel aggregate visualization parity.
