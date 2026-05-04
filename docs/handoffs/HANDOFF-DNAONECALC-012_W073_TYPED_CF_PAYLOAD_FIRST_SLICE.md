*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-012: W073 Typed CF Payload First Slice

Status: `filed`
Direction: OxFml -> DNA OneCalc
Source repo/workset: `OxFml/W073`
Filed date: 2026-05-04

## Summary

OxFml has opened and started W073 as an additive typed input payload for aggregate-context conditional-formatting metadata.

The compatibility decision for the first slice is additive:

1. `VerificationConditionalFormattingRule.typed_rule` is preferred by OxFml when present.
2. The W072 bounded string convention in `thresholds` remains supported as fallback.
3. Existing DNA OneCalc W072 request construction does not need to change to preserve current behavior.

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

1. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 19 tests.
2. `cargo test -p oxfml_core` - passed.
3. `cargo fmt --all -- --check` - passed.

New typed-payload test coverage:

1. typed color-scale payload matches the W072 bounded threshold convention,
2. typed data-bar payload controls bounds, direction, and show-bar-only,
3. typed icon-set payload uses explicit thresholds,
4. typed rank and average payloads replace threshold parsing.

## Requested DNA OneCalc Action

No urgent bridge change is required. DNA OneCalc can continue emitting the W072 bounded `thresholds` convention.

When ready, DNA OneCalc may add typed request construction for the new `typed_rule` field and keep the old convention as a compatibility fallback during its own transition.
