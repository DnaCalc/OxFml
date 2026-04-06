# W057: Formatting And Conditional-Formatting Excel Parity

## Purpose
Drive the remaining OxFml-owned formatting and conditional-formatting semantics from the current bounded `W056` export-and-best-effort floor toward full honest Excel parity for the single-node evaluator and replay-facing publication surfaces.

This workset exists because `W056` now proves useful XML-verification export, broader best-effort display rendering, and best-effort evaluable conditional-formatting consequences, but it still leaves large Excel-semantic gaps open. Those gaps need an explicit owner packet rather than drifting as note debt.

## Position and Dependencies
- **Depends on**: `W030`, `W031`, `W039`, `W042`, `W056`
- **Blocks**: any future claim of full Excel formatting parity or full conditional-formatting parity on the OxFml lane; stronger downstream replay and OneCalc parity claims for formatting-complete XML verification
- **Cross-repo**: `DnaOneCalc` and `OxReplay` consume the replay-facing output; `OxFunc` may need bounded format-code engine widening where shared formatter support is the right owner; no downstream repo is authorized to infer the missing semantics locally

## Scope
### In scope
1. Define the remaining OxFml-owned semantics needed for full worksheet-visible number/date/text display parity at the single-node publication boundary.
2. Classify and realize the remaining number-format-code families beyond the current best-effort lane, including section selection, literals, fill/spacing tokens, scaling, date/time token interactions, elapsed-time families, fractional/scientific families, and text-placeholder behavior.
3. Define and realize the remaining conditional-formatting evaluation semantics that OxFml can and should own for replay-facing publication and XML verification.
4. Make conditional-formatting priority, applicability, effective-style, and effective-display consequences explicit where Excel semantics require them and the single-node evaluator can state them honestly.
5. Record owner splits between OxFml-local semantics, host-supplied context, and any OxFunc formatter widening required for parity.
6. Produce deterministic replay and retained-fixture evidence for the admitted parity slices as they land.

### Out of scope
1. UI-only rendering polish that does not change worksheet-visible semantic display truth.
2. Multi-node recalc ordering or coordinator policy interactions beyond the evaluator/publication seam.
3. Downstream host UX changes in `DnaOneCalc` or diff/explain consumption changes in `OxReplay`.
4. Claims of parity without exercised deterministic evidence for each admitted family.

## Outstanding Semantic Backlog
### 1. Number-format and display semantics
1. Sectioned format-code selection beyond the current positive/negative/zero fallback heuristics, including explicit conditions and text sections.
2. Placeholder semantics for `#`, `0`, `?`, text placeholder `@`, escaped literals, quoted literals, fill `*`, spacing `_`, and locale-sensitive separators.
3. Numeric scaling semantics including trailing commas, percent scaling, thousands/millions display conventions, and negative-zero treatment.
4. Scientific and engineering notation families.
5. Fraction display families and denominator-width behavior.
6. Date/time token parity including:
   - month versus minute disambiguation,
   - day/month/year token families,
   - named month/day forms,
   - elapsed-time forms such as `[h]`, `[m]`, `[s]`,
   - date-system interaction (`1900` / `1904`),
   - invalid date serial handling.
7. Text-format sections and mixed text/number display patterns.
8. Color tokens and bracketed modifiers where they change semantic display or effective-style output.

### 2. Conditional-formatting semantics
1. Rule-priority ordering and effective-rule selection when multiple rules apply.
2. Stop-if-true and rule short-circuit behavior where applicable to the carrier family.
3. Broader expression-formula evaluation beyond the current simple current-cell binary and bounded `AND(...)` / `OR(...)` subset.
4. Full operator taxonomy, threshold typing, blank/error/text handling, and cross-type comparison behavior.
5. Effective-style merge semantics across base style, direct style, and conditional overlays.
6. Conditional number-format consequences when a rule changes displayed format rather than only color/style.
7. Broader carrier families such as icon sets, data bars, and color scales, including the rule for which parts are semantic enough for replay publication versus UI-only.
8. Exact owner split for host-supplied versus OxFml-evaluated rule facts when the host extracts XML carrier truth.

### 3. Replay and publication consequences
1. Promote the parity-relevant formatting and conditional-formatting facts into the canonical replay-facing projection without flattening them into convenience strings.
2. Keep `comparison_views` and `verification_publication_surface` aligned as the richer parity facts grow.
3. Retain deterministic witness families that prove not only payload carriage but actual Excel-visible outcome parity for admitted rows.

## Deliverables
1. A canonical spec packet for remaining format-code semantics and remaining conditional-formatting semantics needed for Excel parity.
2. Bounded implementation slices in OxFml code for the admitted next-wave parity families.
3. Deterministic tests covering each newly admitted family with explicit Excel-facing expected display or effective-style outcomes.
4. Retained fixture evidence for XML-style cases and replay-facing comparison views proving the admitted parity slices.
5. Updated downstream guidance naming exactly what has become real and what remains partial.

## Gate Model
### Entry gate
- `W030` established the semantic-format versus display boundary.
- `W031` classified the `MS-OE376` formatting-adjacent backlog.
- `W039` established the first honest restricted-carrier floor for CF/DV formulas.
- `W056` established the first useful verification/export and replay-facing comparison-view packet plus best-effort rendering/evaluation.

### Exit gate
- Remaining formatting and conditional-formatting families are split into:
  - exercised parity-realized,
  - explicitly deferred with rationale,
  - or blocked with named owner/action.
- No admitted parity family remains only note-described without deterministic tests and replay-facing evidence.
- Replay/publication surfaces expose the parity-relevant facts without downstream reinterpretation.
- Documentation explicitly distinguishes full parity-realized families from still-partial families.

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
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the full Excel-parity family set is intentionally larger than the currently admitted `W056` best-effort slice
  - exact owner split between OxFml-local logic and any required OxFunc format-engine widening remains to be narrowed family by family
  - downstream retained replay and host uptake will remain partial until later evidence waves land
- claim_confidence: draft
- reviewed_inbound_observations: current OxFunc and OxCalc ledgers plus the OxReplay XML comparison-view request remain relevant; `W056` outbound notes should be treated as the immediate predecessor state for this workset
