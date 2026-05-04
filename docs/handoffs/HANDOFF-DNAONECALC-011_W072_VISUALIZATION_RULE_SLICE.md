*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-011: W072 Visualization Rule Slice

Status: `filed`
Source repo/workset: `OxFml/W072`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_CF_AGGREGATE_VISUALIZATION_RULES.md`

## Purpose

Record the bounded W072 visualization-rule landing over the W071 per-cell carrier.

## OxFml Landing Summary

OxFml now publishes bounded visualization output for:

1. `colorScale`
2. `dataBar`
3. `iconSet`

The output is carried through `VerificationPublicationSurface.array_cell_format`:

1. `colorScale` sets per-cell `effective_fill_color`,
2. `dataBar` sets per-cell `data_bar`,
3. `iconSet` sets per-cell `icon`.

Mixed visualization and scalar rules preserve per-field priority: a later scalar rule can set a font color without erasing an earlier color-scale fill.

Scalar visualization results also populate `array_cell_format` as a 1x1 carrier for aggregate visualization rules.

## Bounded Payload Convention

Until a richer typed visualization payload lands, OxFml consumes these conventions from the existing `VerificationConditionalFormattingRule` shape:

1. color-scale stops in `thresholds`, e.g. `min:#F8696B`, `mid:#FFEB84`, `max:#63BE7B`, `percent:50:#FFEB84`, `num:42:#63BE7B`;
2. data-bar color from `fill_color`, with optional `thresholds` entries `min:n`, `max:n`, `direction:right`, and `showBarOnly`;
3. icon-set kind from `thresholds[0]`, defaulting to `3Arrows`;
4. icon bins use equal-width min/max numeric bins by default, or explicit later threshold entries such as `num:20` and `percent:67`;
5. average rules consume `equal` and `stddev:n` threshold entries.

## Evidence

Relevant OxFml files:

1. `crates/oxfml_core/src/publication/mod.rs`
2. `crates/oxfml_core/tests/conditional_formatting_array_tests.rs`

Focused validation:

1. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 15 tests.
2. `cargo test -p oxfml_core` - passed.
3. `git diff --check` - passed with CRLF normalization warnings only.

Covered visualization cases:

1. 3-stop color scale over `SEQUENCE(5)`,
2. data-bar ratios over `[10,20,30,40]`,
3. `3Arrows` icon-set bins over `SEQUENCE(6)`,
4. mixed color-scale plus scalar `cell_value > 5` font-color priority.
5. scalar degenerate color-scale carrier,
6. explicit data-bar min/max, direction, and bar-only flags,
7. explicit icon-set numeric thresholds,
8. equal-average and stddev-average flags.

## DnaOneCalc Guidance

DNA OneCalc can bridge the bounded payload convention now. The result hero can render `data_bar` and `icon` from the per-cell carrier for this slice.

## Non-Claims

This handoff does not claim pixel-perfect Excel rendering or DNA OneCalc UI rendering.
