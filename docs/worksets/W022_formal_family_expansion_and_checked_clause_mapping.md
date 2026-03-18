# W022: Formal Family Expansion and Checked Clause Mapping

## Purpose
Extend the checked local formal floor beyond the first session-lifecycle artifact pair so more exercised OxFml clause families are mapped to checked Lean and TLA+ artifacts.

## Position and Dependencies
- **Depends on**: `W019`, `W020`, `W016`
- **Blocks**: `W023`
- **Cross-repo**: Foundation doctrine remains authoritative for proof/model posture; this workset stays local to OxFml-owned clauses and evidence

## Scope
### In scope
1. Extend checked local formal artifacts beyond the first session-lifecycle pair into additional exercised clause families.
2. Tighten clause-to-formal-artifact mapping for replay, runtime, and proving-host surfaces already exercised locally.
3. Add or refine deterministic harness surfaces for local proof/model execution where practical.
4. Update formal planning docs so checked versus skeleton-only status stays explicit.

### Out of scope
1. Full proof closure for all OxFml semantics.
2. Green-owned formal promotion outside this repo.
3. Pack-grade replay promotion.

## Deliverables
1. Additional checked local formal artifacts or model runs for exercised clause families.
2. Sharper clause-to-formal mapping across the local replay/runtime/proving-host floor.
3. Updated formal planning docs with explicit residuals.

## Gate Model
### Entry gate
- `W020` has stabilized a broader local semantic and replay floor for the formal lane to target.

Sequencing rule:
1. `W022` may start before `W021` reaches its gate where the formal work targets already exercised parser, semantic, replay, or proving-host clauses,
2. `W021` outputs should be absorbed into `W022` incrementally rather than forcing the whole formal lane to wait.

### Exit gate
- More than the initial session-lifecycle clause family has checked local formal coverage.
- Clause-to-formal mapping is explicit for the broadened local floor.
- Residual unproven lanes remain explicit rather than implied closed.

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
- open_lanes: broader formal families beyond the current checked local floor and Green-owned formal promotion remain outside this workset scope
- claim_confidence: validated
