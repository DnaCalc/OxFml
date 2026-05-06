*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-012: W073 Typed CF Payload First Slice

Status: `filed`
Direction: OxFml -> DNA OneCalc
Source repo/workset: `OxFml/W073`
Filed date: 2026-05-04

## Summary

OxFml has moved W073 to the target typed input payload for aggregate-context conditional-formatting metadata.

The compatibility decision is direct replacement for the W073 families:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage` options.
2. The W072 bounded string convention in `thresholds` is intentionally ignored for those families.
3. `thresholds` remains available for scalar/operator/expression rule families where threshold text is the real rule input.

## Typed Metadata Families

The first slice covers typed metadata for:

1. `colorScale`: ordered typed stops with `min`, `mid`, `max`, `percent`, `percentile`, and numeric stop positions.
2. `dataBar`: typed minimum/maximum bounds, bar color, direction, and show-bar-only flag.
3. `iconSet`: icon-set kind plus typed threshold sequence.
4. `top` / `bottom`: typed count or percent rank option.
5. `aboveAverage` / `belowAverage`: typed equal-average and stddev multiplier options.

Output carrier shape remains the W071/W072 carrier:

1. per-cell `effective_fill_color`,
2. per-cell `data_bar`,
3. per-cell `icon`.

## Validation

Focused validation:

1. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 21 tests.
2. `cargo test -p oxfml_core` - passed.
3. `cargo fmt --all -- --check` - passed.

New typed-payload test coverage:

1. typed color-scale payload drives interpolation,
2. typed data-bar payload controls bounds, direction, and show-bar-only,
3. typed icon-set payload uses explicit thresholds,
4. typed rank and average payloads drive aggregate predicates,
5. bounded visualization threshold strings are not interpreted,
6. bounded aggregate option strings are not interpreted.

## Requested DNA OneCalc Action

DNA OneCalc should update request construction for the W073 families to emit `typed_rule`.

For `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`, continuing to emit only the W072 bounded `thresholds` convention will no longer produce aggregate or visualization effects in OxFml.
