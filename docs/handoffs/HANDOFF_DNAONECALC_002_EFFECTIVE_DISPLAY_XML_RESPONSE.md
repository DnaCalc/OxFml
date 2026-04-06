# HANDOFF-DNAONECALC-002 OxFml Effective Display XML Response

## Purpose
Record the OxFml-side response to `C:\Work\DnaCalc\DnaOneCalc\docs\HANDOFF_OXFML_EFFECTIVE_DISPLAY_XML_PROMPT.md`.

This document is an OxFml working handoff response.
It is not a closure artifact.

## Intake Summary
DNA OneCalc reported a retained SpreadsheetML XML verification mismatch for:
1. workbook: `C:\Work\DnaCalc\OxXlPlay\docs\test-corpus\excel\xlplay_capture_spreadsheetml_formatting_001\workbook.xml`
2. locator: `Input!A1`
3. formula: `=SUM(1,2,3)`
4. number format code: `$#,##0.00`
5. OxFml `effective_display_text`: `$$6.00`
6. Excel-observed `effective_display_text`: `$6.00`

The retained bundle also showed cross-lane divergence on:
1. `formatting_view`
2. `conditional_formatting_view`

## What Was Wrong
Two separate issues were present:
1. OxFml had a real display-formatting bug in the custom number-format fallback for currency-like codes such as `$#,##0.00`.
   - OxFml called the locale currency renderer and then reapplied the literal `$` prefix from the parsed format section, yielding `$$6.00`.
2. OxFml replay-facing `comparison_views` were publishing the full richer OxFml formatting and conditional-formatting envelope for this XML lane.
   - that was valid OxFml-owned data,
   - but it did not align well with the current admitted SpreadsheetML comparison envelope used by the retained OxXlPlay observation path.

## Action Taken
OxFml landed the following `W056` follow-on slice:
1. fixed the currency double-prefix bug in `crates/oxfml_core/src/publication/mod.rs`
   - `$#,##0.00` now renders as `$6.00` rather than `$$6.00`
2. kept the richer `VerificationPublicationSurface` intact
   - this remains the OxFml-owned full export packet
3. narrowed only the SpreadsheetML XML replay-facing comparison envelopes in `crates/oxfml_core/src/consumer/replay/mod.rs`
   - `formatting_view` now aligns to the current admitted comparison-friendly subset:
     - `number_format_code`
     - `style_id`
     - `font_color`
     - `fill_color`
   - `conditional_formatting_view` now aligns to the current admitted SpreadsheetML expression-rule comparison shape:
     - source-declared rules as `range` / `formula` / `value1` / `value2` / `operator` / `rule_kind` / colors
     - derived `effective_style` carrying:
       - `number_format_code`
       - `font_color`
       - `fill_color`
       - `effective_display_text`
       - `applied_rule_indexes`
       - `source_projection = spreadsheetml_expression_rules_v1`
4. preserved backward compatibility for non-XML callers
   - the narrowed comparison envelope is only used for the admitted SpreadsheetML XML verification profile

## Evidence Updated
Retained local evidence now covers the exact XML-style case:
1. `crates/oxfml_core/tests/replay_consumer_facade_tests.rs`
2. `crates/oxfml_core/tests/fixtures/xml_verification_comparison_views_projection.json`
3. `crates/oxfml_core/src/publication/mod.rs` unit test coverage for `$#,##0.00`

## Expected Downstream Result
For the retained SpreadsheetML XML case above:
1. `effective_display_text` should now align better with Excel for this case
2. `formatting_view` should now align better with the current OxXlPlay comparison-family envelope
3. `conditional_formatting_view` should now align better with the current OxXlPlay expression-rule comparison-family envelope

This does not widen OxFml capability claims to full display-code or conditional-formatting parity.

## Rerun Command
DNA OneCalc can rerun verification with:

`cargo run -p dnaonecalc-host -- verify-xml-cell --case-id xml-case-1 --workbook-xml C:\Work\DnaCalc\OxXlPlay\docs\test-corpus\excel\xlplay_capture_spreadsheetml_formatting_001\workbook.xml --locator Input!A1 --output-root C:\Work\DnaCalc\DnaOneCalc\target\onecalc-verification\manual-xml-case-live-verify2`

## Validation
Validation run:
1. `cargo test -p oxfml_core`

Observed result:
1. passed

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - broader display-code parity beyond the admitted number-format subset
  - broader conditional-formatting rule-family and priority semantics
  - downstream rerun and acknowledgment over the retained XML verification bundle
