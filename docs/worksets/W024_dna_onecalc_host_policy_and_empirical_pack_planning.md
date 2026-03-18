# W024: DNA OneCalc Host Policy and Empirical Pack Planning

## Purpose
Turn the broadened single-formula proving-host floor into a clearer DNA OneCalc-facing host policy baseline and empirical-pack planning surface without claiming full host-product or pack-grade maturity.

## Position and Dependencies
- **Depends on**: `W020`, `W023`, `W018`
- **Blocks**: later broader host and empirical-pack promotion work
- **Cross-repo**: DNA OneCalc remains a downstream host vehicle and must consume OxFml semantics rather than redefine them

## Scope
### In scope
1. Clarify the host-policy boundary between OxFml’s proving host and the later DNA OneCalc host specification.
2. Broaden proving-host and empirical-oracle planning over the stronger semantic/runtime/replay floor.
3. Define empirical-pack planning and scenario grouping rules without claiming pack-grade promotion.
4. Tighten live docs so the remaining gap between the current proving host and broader DNA OneCalc proving is explicit.

### Out of scope
1. Multi-node OxCalc proving.
2. Full DNA OneCalc product specification.
3. Pack-grade empirical promotion.

## Deliverables
1. A clearer host-policy baseline for future DNA OneCalc consumption of OxFml.
2. Broader empirical-pack planning over the stronger local proving floor.
3. Updated proving-host and planning docs that keep the remaining host gap explicit.

## Gate Model
### Entry gate
- `W023` has established a stronger local replay-promotion floor to carry the next host and empirical planning wave cleanly.

Sequencing rule:
1. `W024` is intentionally follow-on rather than critical-path work,
2. it may refine host-policy and empirical-pack planning in parallel with later runtime broadening, so long as it does not overclaim runtime maturity that OxFml has not yet exercised.

### Exit gate
- The proving-host-to-DNA-OneCalc boundary is clearer than the current single-node local floor.
- Empirical-pack planning exists without overclaiming pack-grade maturity.
- The remaining gap to broader DNA OneCalc proving is explicit in live planning docs.

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
- open_lanes: full DNA OneCalc product specification and pack-grade empirical promotion remain outside this workset scope
- claim_confidence: validated
