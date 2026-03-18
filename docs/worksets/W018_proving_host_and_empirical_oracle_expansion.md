# W018: Proving Host and Empirical Oracle Expansion

## Purpose
Broaden the proving-host and empirical-oracle floor so OxFml semantics are exercised over a richer single-node host surface with better replay and oracle linkage.

## Position and Dependencies
- **Depends on**: `W014`, `W015`, `W017`
- **Blocks**: later broader host and pack-facing validation claims
- **Cross-repo**: DNA OneCalc remains a downstream host vehicle and must consume OxFml semantics rather than redefine them

## Scope
### In scope
1. Broaden the single-formula proving host beyond the current narrow floor.
2. Add richer empirical-oracle scenario families for high-risk formula and host-query lanes.
3. Improve replay/oracle linkage for proving-host scenarios.
4. Tighten host-facing planning docs around what still separates OxFml from broader DNA OneCalc proving.

### Out of scope
1. OxCalc multi-node proving.
2. Full pack-grade replay promotion.
3. Host policy surfaces owned outside OxFml.

## Deliverables
1. Broader proving-host fixture families and tests.
2. Broader empirical-oracle scenario coverage tied to replay evidence.
3. Updated host/oracle planning docs reflecting the widened single-node proving floor.

## Gate Model
### Entry gate
- `W017` has established a stronger replay-promotion floor to carry widened proving-host evidence cleanly.

### Exit gate
- The proving-host and oracle floor is materially broader than the current narrow slice.
- Host/oracle scenarios remain tied to replay evidence instead of separate ad hoc cases.
- The remaining gap to broader DNA OneCalc proving is explicit in the live planning docs.

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
- open_lanes: broader DNA OneCalc host policy and pack-grade empirical promotion remain outside this workset scope
- claim_confidence: validated
