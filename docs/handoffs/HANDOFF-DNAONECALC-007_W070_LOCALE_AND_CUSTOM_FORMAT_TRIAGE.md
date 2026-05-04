*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-007: W070 Locale And Custom-Format Grammar Triage

Status: `filed`
Source repo/workset: `OxFml/W070`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound:

1. `../DnaOneCalc/docs/HANDOFF_OXFML_LOCALE_EXPANSION.md`
2. `../DnaOneCalc/docs/HANDOFF_OXFML_CUSTOM_FORMAT_GRAMMAR.md`

## Purpose

Record OxFml's triage of the locale expansion and custom-format grammar handoffs after the `W069` format-engine landing and the `W070` conditional-formatting predicate slice.

## Locale Expansion

OxFml agrees this is the right target direction, but the locale-profile portion is blocked on OxFunc exposing canonical locale profile identities and constants. OxFml has filed `HANDOFF-OXFUNC-006` and recorded `BLK-FML-005`.

Until that lands, OxFml will not add a second comprehensive locale registry for month names, weekday names, separators, currency symbols, or locale-prefix parsing.

## Custom-Format Grammar Split

The custom-format grammar handoff splits into three lanes:

1. Already covered by `W069`: common time tokens, datetime composites, elapsed-time tokens, simple fractions, and common accounting parentheses.
2. OxFml-local follow-up: text fourth-section behavior and surfacing selected format-section color information where the publication seam can carry it.
3. Locale-blocked follow-up: optional locale-prefix grammar and locale-specific month/day rendering, blocked by `BLK-FML-005`.

Conditional numeric section selection is already present in OxFml's bounded format engine; additional tests should be added in the custom-format follow-up lane rather than treated as a DNA OneCalc workaround.

## Current OxFml Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - OxFunc locale-profile API breadth (`BLK-FML-005`)
  - locale-prefix custom-format grammar
  - text fourth-section custom-format behavior
  - selected color publication for custom-format sections

## Downstream Guidance

DNA OneCalc can consume the `W069` and `W070` landed slices independently:

1. remove temporary expectations around `HH:mm:ss` and other bounded `W069` format families,
2. remove temporary expectations around conditional-formatting `blanks`, `noBlanks`, `errors`, `noErrors`, and relative `dates` predicates with `now_serial`,
3. keep markers for locale expansion and the remaining custom-format grammar lanes until the corresponding OxFml/OxFunc follow-up work lands.

## Non-Claims

This handoff does not claim locale expansion, full custom-format grammar parity, or DNA OneCalc UI cleanup.
