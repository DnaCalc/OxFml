*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-009: W072 CF Aggregate Visualization Intake

Status: `filed`
Source repo/workset: `OxFml/W072`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_CF_AGGREGATE_VISUALIZATION_RULES.md`

## Purpose

Acknowledge and scope the DNA OneCalc aggregate-context CF visualization handoff. This is tracked separately from W071 because aggregate rules need kind-specific metadata and precomputed aggregate context, not just the per-cell carrier.

## Current OxFml State

W071 introduces the per-cell carrier and includes optional `data_bar` and `icon` slots on `ArrayCellFormat`.

Those slots are not yet computed. W072 is the planned implementation lane for:

1. `colorScale`,
2. `dataBar`,
3. `iconSet`,
4. `aboveAverage`,
5. `belowAverage`,
6. `top`,
7. `bottom`,
8. `uniqueValues`,
9. `duplicateValues`.

## Required Next Shape Work

The current `VerificationConditionalFormattingRule` scalar fields are enough for W070/W071 operator and predicate rules, but not enough to faithfully represent aggregate visualization rules.

W072 starts with a payload-shape bead for:

1. ordered color-scale stops,
2. data-bar min/max and visual options,
3. icon-set kind and thresholds,
4. top/bottom count-vs-percent flags,
5. above/below-average stddev/equality flags,
6. unique/duplicate rule identity.

## Evidence/Tracking

OxFml tracking:

1. `docs/worksets/W072_conditional_formatting_aggregate_visualization_rules.md`
2. `docs/worksets/W071_conditional_formatting_array_per_cell_publication.md`

W071 focused carrier evidence:

1. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 4 tests.

## DnaOneCalc Guidance

DNA OneCalc can consume W071's `array_cell_format` now for scalar/operator/predicate rules. Keep `<NOT IMPL>` seam markers for aggregate visualization kinds until W072 lands.

When W072 lands, DNA OneCalc should bridge:

1. per-cell `data_bar`,
2. per-cell `icon`,
3. per-cell gradient/rank/average/unique/duplicate style outcomes.

## Non-Claims

This handoff does not claim aggregate visualization implementation. It records the accepted work lane and the payload-shape prerequisite.
