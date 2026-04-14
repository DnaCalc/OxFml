# HANDOFF_OXREPLAY_001 — XML Comparison Views Response

## Header
1. handoff id: `HANDOFF_OXREPLAY_001`
2. date: `2026-04-06`
3. from repo: `OxFml`
4. to repo: `OxReplay`
5. related workset or feature id: `W056`

## Purpose
Answer the OxReplay-side XML comparison-view request by declaring the landed OxFml replay-projection widening, the retained evidence proving it, and the limits that remain explicit.

## Landed change
OxFml replay projection now publishes additive machine-readable `comparison_views` when the replay-facing result carries `verification_publication_surface`.

Current landed family set:
1. `{ "view_family": "comparison_value", "value": ... }`
2. `{ "view_family": "visible_value_text", "value": ... }`
3. `{ "view_family": "effective_display_text", "value": ... }`
4. `{ "view_family": "formatting_view", "value": ... }`
5. `{ "view_family": "conditional_formatting_view", "value": ... }`

Current publication rule:
1. these views are emitted on `ReplayProjectionResult` for runtime-result projection and first-host-capture projection,
2. they are built directly from OxFml-owned `VerificationPublicationSurface`,
3. they are additive to the fuller `verification_publication_surface` packet rather than a replacement for it,
4. they are not host-local convenience strings and they do not move lane semantics into OxReplay.

## Current family meaning
1. `comparison_value`
   - current value: OxFml-owned typed comparison truth rooted in the OxFunc value model, including full array cell content
2. `visible_value_text`
   - current value: OxFml visible worksheet-value text
3. `effective_display_text`
   - current value: OxFml effective display text for the admitted locale/number-format slice plus broader best-effort grouped, percent, sectioned-negative, and date-token rendering
4. `formatting_view`
   - current value: machine-readable format-profile, locale-format-context, date-system flag, style identity/lineage, format dependency facts, format/display delta, presentation-hint, and color facts
5. `conditional_formatting_view`
   - current value: machine-readable conditional-formatting rule facts plus target-range, operator, threshold, applicability, effective style, and effective-display carriage from the current verification packet

## Current evidence
1. code path:
   - `crates/oxfml_core/src/consumer/replay/mod.rs`
2. focused facade test:
   - `crates/oxfml_core/tests/replay_consumer_facade_tests.rs`
3. retained local comparison-view fixture:
   - `crates/oxfml_core/tests/fixtures/xml_verification_comparison_views_projection.json`
4. existing verification publication packet source:
   - `crates/oxfml_core/src/publication/mod.rs`

## Current limits
1. this does not widen the accepted replay capability stance beyond the current local `C3` floor
2. this does not claim full number-format-code closure or broad `MS-OE376` display parity
3. this does not claim full conditional-formatting formula closure, priority/stop-if-true semantics, or broad OxFml-owned conditional-formatting semantic evaluation beyond the current restricted carrier and best-effort evaluable subset
4. this does not yet publish retained-witness or pack-grade replay families for the XML comparison-view lane

## Requested OxReplay intake
1. consume `ReplayProjectionResult.comparison_views` as the preferred family-comparison surface when it is present
2. continue treating missing family publication outside this admitted floor as a coverage gap, not as license for host-local reinterpretation
3. keep `verification_publication_surface` available as the richer OxFml-owned packet when deeper inspection is needed
