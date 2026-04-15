# HANDOFF-DNAONECALC-001 OxFml XML Verification Publication Response

## Purpose
Record the OxFml-side response to `C:\Work\DnaCalc\DnaOneCalc\docs\HANDOFF_OXFML_XML_VERIFICATION_REQUEST.md` against the current OxFml runtime/replay surface.

This document is an OxFml working handoff response.
It is not a closure artifact.

## Intake Summary
DNA OneCalc reported a real XML-backed verification mismatch:
1. OxFml visible/effective summary: `6`
2. Excel-observed display: `$6.00`
3. replay mismatch kind: `view_value`

OxFml accepts that this is a publication/comparison seam issue rather than a OneCalc-local formatting bug.

## Action Taken
OxFml landed a first `W056` slice that adds a typed verification publication/export packet across the runtime and replay facade path:
1. `VerificationPublicationContext`
   - host-supplied number-format, style, and conditional-formatting context for comparison-heavy verification lanes
2. `VerificationPublicationSurface`
   - OxFml-owned export packet carrying:
     - `entered_cell_text`
     - `published_value`
     - `published_value_class`
     - `visible_value_text`
     - `effective_display_text`
     - `format_profile`
     - `locale_format_context`
     - `date1904`
     - `number_format_code`
     - `style_id`
     - `style_hierarchy`
     - `format_dependency_facts`
     - `format_delta`
     - `display_delta`
     - `returned_value_surface`
     - `presentation_hint`
     - `font_color`
     - `fill_color`
     - `conditional_formatting_rules`
     - `conditional_formatting_target_ranges`
     - `conditional_formatting_rule_kind`
     - `conditional_formatting_operator`
     - `conditional_formatting_thresholds`
     - `conditional_formatting_effective_display`
3. runtime carriage
   - `RuntimeFormulaRequest` now accepts optional `VerificationPublicationContext`
   - `RuntimeFormulaResult` now exposes `verification_publication_surface`
   - `RuntimeFormulaResult` now also publishes additive `comparison_views` for:
     - `comparison_value`
     - `visible_value_text`
     - `effective_display_text`
     - `formatting_view`
     - `conditional_formatting_view`
4. replay carriage
   - the first-host replay capture packet now carries `verification_publication_surface`
   - `ReplayProjectionResult` now exposes `verification_publication_surface`
   - `ReplayProjectionResult` now also publishes additive `comparison_views` for:
     - `comparison_value`
     - `visible_value_text`
     - `effective_display_text`
     - `formatting_view`
     - `conditional_formatting_view`
5. first effective-display implementation
   - OxFml now computes a broader best-effort `effective_display_text` projection for the admitted locale/number-format slice using the OxFunc locale formatter plus local rendering heuristics for grouped/fixed numeric formats, percent formats, sectioned negative formats, date-token formats such as `m/d/yyyy`, and the earlier currency-code bridge for patterns such as `$#,##0.00`
6. best-effort conditional-formatting evaluation
   - OxFml now computes applicability plus effective font/fill/display consequences when the verification packet provides enough rule facts, including operator rules and simple current-cell expression rules such as `=A1>0`

## What DNA OneCalc Can Consume Now
For XML-backed verification, DNA OneCalc should now:
1. send extracted number-format, style, and conditional-formatting context through `VerificationPublicationContext`
2. read `verification_publication_surface` from `RuntimeFormulaResult` for ordinary runtime comparison
3. prefer `RuntimeFormulaResult.comparison_views` when ordinary direct runtime comparison wants the admitted family-oriented envelope
4. read `verification_publication_surface` from `ReplayProjectionResult` for retained-artifact and replay-facing comparison
5. prefer `ReplayProjectionResult.comparison_views` when family-oriented replay comparison is needed
6. stop inferring effective display or formatting view from raw visible value alone for the admitted first slice

## Current Limits
The landed slice is intentionally narrower than full display closure.

Open lanes remain:
1. full `MS-OE376` display-code parity is not claimed
2. full conditional-formatting formula closure, priority chains, and broad OxFml-owned conditional-formatting display evaluation are not claimed
3. current conditional-formatting export is host-supplied verification context carried through an OxFml-owned packet
4. the newly published `comparison_views` are a local OxFml replay-projection floor; downstream retained uptake and broader family promotion remain separate follow-on work
5. the newly published runtime `comparison_views` are an additive direct-host verification surface; they do not replace the richer `VerificationPublicationSurface`

## Files Updated
1. `crates/oxfml_core/src/publication/mod.rs`
2. `crates/oxfml_core/src/host/mod.rs`
3. `crates/oxfml_core/src/consumer/runtime/mod.rs`
4. `crates/oxfml_core/src/consumer/replay/mod.rs`
5. `crates/oxfml_core/tests/replay_consumer_facade_tests.rs`
6. `crates/oxfml_core/tests/fixtures/xml_verification_comparison_views_projection.json`
7. `docs/spec/OXFML_DNA_ONECALC_DOWNSTREAM_CONSUMER_CONTRACT.md`
8. `docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
9. `docs/upstream/NOTES_FOR_DNAONECALC.md`
10. `docs/worksets/W056_xml_verification_publication_and_replay_export.md`

## Validation
Focused validation run:
1. `cargo test -p oxfml_core replay_projection_service_projects_runtime_and_host_outputs -- --exact`

Observed result:
1. passed

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - broader display-code coverage beyond the current admitted locale/number-format slice
  - broader OxFml-owned conditional-formatting evaluation
  - receiving-repo acknowledgment and uptake
