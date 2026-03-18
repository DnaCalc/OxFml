# W020: Semantic Catalog and Callable-Value Breadth

## Purpose
Broaden the OxFml semantic-plan and evaluator floor beyond the current helper/scalarization baseline so more of the OxFunc-facing boundary is exercised with richer provenance and callable-value behavior.

## Position and Dependencies
- **Depends on**: `W019`, `W014`
- **Blocks**: `W021`, `W022`, `W023`, `W024`
- **Cross-repo**: OxFunc inbound observations remain required semantic input, but OxFml still owns semantic-plan structure, provenance carriers, and evaluator-facing boundary artifacts

## Scope
### In scope
1. Broaden semantic-plan and evaluator coverage over more OxFunc-backed function families with higher semantic risk.
2. Tighten prepared-call/result provenance for richer callable values, helper environments, and reference-preserving lanes.
3. Improve local callable-value carriers beyond the current summary-only floor where OxFml semantics need stronger replay truth.
4. Define evaluator-facing behavior for newly classified but not yet fully executable reference families, including typed execution refusal or deferment where needed.
5. Extend replay-backed evidence for the widened semantic boundary.

### Out of scope
1. Full OxFunc catalog closure.
2. Full helper-family expansion such as `MAP` or `SCAN`.
3. Pack-grade replay promotion.

## Deliverables
1. Broader semantic-plan and evaluator coverage over higher-risk OxFunc boundary lanes.
2. Richer prepared-call/result provenance and callable-value evidence.
3. A typed evaluator contract for newly classified but not yet fully executable reference families.
4. Updated boundary docs that reflect the broadened semantic floor.

## Gate Model
### Entry gate
- `W019` has broadened the remaining parser/binder reference floor enough to support richer semantic coverage.

### Exit gate
- More of the OxFunc-facing semantic boundary is exercised locally with typed provenance.
- Callable-value and helper-environment handling are stronger than the current narrow floor.
- Newly classified non-executable reference families have explicit typed evaluator behavior rather than silent semantic gaps.
- Updated docs and replay artifacts reflect the broadened semantic surface.

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
- open_lanes: fuller OxFunc catalog closure, richer callable-value carriers beyond the current exercised floor, and pack-grade replay remain outside this workset scope
- claim_confidence: validated
