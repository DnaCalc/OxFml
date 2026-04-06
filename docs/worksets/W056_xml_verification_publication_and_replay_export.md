# W056: XML Verification Publication And Replay Export

## Purpose
Add a bounded OxFml-owned verification publication/export surface so XML-backed DNA OneCalc verification can compare formula result, effective display, formatting context, and conditional-formatting context without forcing local reinterpretation of OxFml semantics.

## Position and Dependencies
- **Depends on**: `W042`, `W046`, `W054`
- **Blocks**: first DNA OneCalc XML-backed display-faithful verification uptake over the landed `OxFml_V1` runtime/replay facades
- **Cross-repo**: DNA OneCalc consumes the verification publication/export packet; OxReplay later consumes the corresponding comparison views; OxFml remains authoritative for value/display/publication meaning while host-owned style and conditional-format inputs remain explicit host-supplied context

## Scope
### In scope
1. Define a typed OxFml-owned verification publication context for host-supplied formatting and conditional-formatting inputs needed by XML-backed verification.
2. Define a stable verification publication/export surface carried through runtime and replay projection.
3. Compute and export an OxFml-owned first effective-display text projection for the current admitted formatting slice when locale/date-system and number-format inputs are present.
4. Preserve style, format-dependency, format/display delta, and conditional-formatting facts in one comparison-friendly packet for downstream verification tooling.
5. Publish replay-facing `comparison_views` for the admitted XML verification family set when those facts are available from OxFml-owned publication surfaces.
6. Update DNA OneCalc-facing and OxReplay-facing consumer guidance and outbound handoff notes to describe the landed first slice and the remaining gaps honestly.

### Out of scope
1. Full `MS-OE376` display and formatting parity.
2. Broad conditional-formatting semantic evaluation beyond the current restricted carrier and host-supplied verification context.
3. Pack-grade replay promotion.
4. DNA OneCalc-local workbook XML extraction or persistence behavior.

## Deliverables
1. A typed verification publication context and export surface in OxFml code.
2. Runtime and replay projection carriage for the new export surface plus replay-facing `comparison_views`.
3. Deterministic tests and retained fixture evidence proving the first effective-display, formatting-context, and comparison-view export slice.
4. Updated downstream guidance and outbound OxFml handoff responses for DNA OneCalc and OxReplay.

## Gate Model
### Entry gate
- `W042` has already established the first returned-value split and the remaining work is implementation-facing consumer uptake.
- `W046` has already frozen the first host replay-capture packet and projection path.
- `W054` has already landed the runtime and replay facade surfaces that DNA OneCalc should consume.

### Exit gate
- A typed verification publication/export packet exists and is reachable from the landed runtime and replay facades.
- Effective display text is no longer limited to plain visible value summary when the current admitted locale/number-format slice can be rendered honestly.
- Style, format, and conditional-formatting context required for XML-backed verification are carried in one stable comparison-friendly structure.
- Replay projection publishes the admitted `comparison_views` family set directly from OxFml-owned publication facts for runtime-result and first-host-capture projections.
- Best-effort conditional-formatting applicability and effective style/display consequences are computed when the current rule facts are sufficient.
- Remaining unsupported display/conditional-formatting breadth is explicitly listed in docs and handoff notes.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - a first `VerificationPublicationContext` and `VerificationPublicationSurface` now exist in code and are carried through `RuntimeFormulaResult`, the first-host replay capture packet, and `ReplayProjectionResult`
  - the current effective-display slice now covers the earlier currency bridge plus a broader best-effort local lane for grouped/fixed numeric codes, percent formats, sectioned negative formats, and date-token patterns such as `m/d/yyyy`
  - the current first conditional-formatting slice now computes applicability and effective font/fill/display consequences for operator-driven rules and simple current-cell expression rules such as `=A1>0`, including bounded `AND(...)` / `OR(...)` forms
  - replay projection now publishes `comparison_views` for `visible_value`, `effective_display_text`, `formatting_view`, and `conditional_formatting_view` when the verification publication surface is present; retained local fixture evidence lives at `crates/oxfml_core/tests/fixtures/xml_verification_comparison_views_projection.json`
  - full display-code breadth beyond the current admitted and heuristically rendered slice remains outside this workset
  - broad conditional-formatting semantics, priority chains, and full formula-evaluation closure remain outside this workset
  - broader retained-witness or pack-grade replay promotion of this family remains downstream of this workset
- claim_confidence: provisional
- reviewed_inbound_observations: OxFunc current note reviewed with no direct packet conflict; OxCalc current note reviewed and its semantic-display-boundary pressure remains aligned with this workset
