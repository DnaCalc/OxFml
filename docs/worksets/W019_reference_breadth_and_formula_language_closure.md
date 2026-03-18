# W019: Reference Breadth and Formula-Language Closure

## Purpose
Broaden the parser and binder beyond the current widened local floor so more of the remaining Excel-compatible reference surface is exercised with deterministic local evidence.

## Position and Dependencies
- **Depends on**: `W013`
- **Blocks**: `W020`, `W022`, `W024`
- **Cross-repo**: OxFunc remains an inbound semantic pressure for reference-preserving output shape, but this workset is OxFml-owned formula-language closure work

## Scope
### In scope
1. Broaden reference parsing and binding for whole-row, whole-column, quoted-sheet, and richer qualified reference cases.
2. Add structured or external reference classification where parser/binder ownership is already clear even if full execution is still deferred.
3. Tighten incremental reuse behavior over the broadened reference surface.
4. Extend deterministic parse/bind fixtures and normalized-reference assertions for the widened formula-language floor.

### Out of scope
1. Full Excel grammar closure.
2. Full execution semantics for every newly classified reference family.
3. Pack-grade replay promotion.

Working boundary rule:
1. any newly accepted but not yet executable reference family must remain explicitly classified at parse/bind level,
2. the downstream evaluator contract for typed execution refusal or deferment belongs to `W020`, not this workset.

## Deliverables
1. Broader exercised parser/binder coverage over remaining high-value reference lanes.
2. Deterministic fixtures for the widened reference and reuse surface.
3. Updated formula-language planning docs reflecting the broader local floor.

## Gate Model
### Entry gate
- `W013` has established the first widened parser/binder and incremental reuse baseline.

### Exit gate
- The local parser/binder floor covers materially more of the remaining high-value reference surface.
- Incremental reuse remains exercised across that broader reference set.
- The widened reference floor is reflected in live planning/spec docs.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | yes |
| 6 | No known semantic gaps remain in declared scope? | yes |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes: fuller Excel grammar closure, broader structured-reference breadth, and pack-grade replay remain outside this workset scope
- claim_confidence: validated
