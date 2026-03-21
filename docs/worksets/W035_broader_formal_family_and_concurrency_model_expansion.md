# W035: Broader Formal Family and Concurrency Model Expansion

## Purpose
Broaden the checked local formal floor beyond the current session lifecycle and external capability gate so the next OxFml wave has stronger formal coverage over replay promotion, distributed/runtime consequences, and publication invariants.

## Position and Dependencies
- **Depends on**: `W033`, `W034`
- **Blocks**: later Stage 2 promotion and any stronger formal-completeness claims
- **Cross-repo**: OxFml owns local formalization of its artifact and seam meaning; Foundation doctrine remains authoritative for cross-program assurance posture

## Scope
### In scope
1. Add new checked Lean and/or TLA+ artifacts over the broadened replay and runtime consequence families from `W033` and `W034`.
2. Strengthen clause-to-artifact mapping in the assurance docs for the newly widened families.
3. Extend the canonical local formal runner only where needed to keep the new checked artifacts deterministic and repeatable.
4. Record remaining formal gaps explicitly rather than leaving them implied.

### Out of scope
1. Final proof closure for every OxFml clause family.
2. Pack-grade formal artifact promotion outside this repo.
3. Coordinator-policy proofs owned by OxCalc.
4. Broad formula-language review work better handled by `W031`.

## Deliverables
1. A broader checked local formal floor for OxFml replay/runtime/publication families.
2. Updated assurance mapping and formal register entries for the widened floor.
3. A sharper residual list for the formal families still outside checked local coverage.

## Gate Model
### Entry gate
- `W033` has widened replay-promotion evidence beyond the current retained-local rehearsal floor.
- `W034` has widened runtime consequence families beyond the current local async/provider slice.

### Exit gate
- At least one new checked formal family exists beyond the current session lifecycle and external capability gate floor.
- The assurance map and formal register reflect the widened checked local floor.
- Remaining formal gaps are explicitly recorded rather than implied.

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
  - the checked local floor now extends beyond the session lifecycle and external capability gate with deferred-name-carrier, failure-stage, and external-name-carrier Lean artifacts plus higher-order callable, session-contention, retry-after-release, overlay-cleanup, pinned-epoch overlay, distributed-placement, retry-ordering fairness, and placement-deferral expiry TLA+ models
  - broader concurrency and replay-promotion formal families remain outside the current exercised slice
- claim_confidence: draft
