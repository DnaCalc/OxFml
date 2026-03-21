# W034: Distributed Runtime and Coordinator Consequence Boundary

## Purpose
Push the current OxFml-managed async/runtime floor beyond the local contention and external-provider baseline so distributed or multi-session evaluator consequences are typed, replayable, and clearly separated from OxCalc-owned coordinator policy.

## Position and Dependencies
- **Depends on**: `W029`, `W032`
- **Blocks**: `W035`
- **Cross-repo**: OxFml owns evaluator/runtime artifact meaning and typed consequence surfaces; OxCalc owns coordinator graph, placement, retry, and fairness policy

## Scope
### In scope
1. Broaden multi-session and distributed-style runtime consequence families beyond the current local contention and async-provider floor.
2. Tighten which runtime consequences must surface as candidate, reject, topology, overlay, or trace facts for downstream coordinator correctness.
3. Clarify coordinator-visible versus evaluator-local consequences in canonical seam docs.
4. Add deterministic replay and local formal artifacts for at least one broader distributed-style runtime family.

### Out of scope
1. Full OxCalc graph scheduler policy.
2. Full multi-node product deployment semantics.
3. Pack-grade replay or formal promotion.
4. OxFunc semantic transport narrowing beyond what this runtime boundary directly needs.

## Deliverables
1. A stronger distributed/multi-session runtime consequence baseline for OxFml.
2. Narrower coordinator-facing artifact surfaces for those consequences.
3. Replay and formal evidence for at least one broader distributed-style runtime lane.

## Gate Model
### Entry gate
- `W029` established async-coupled external-provider runtime and formal baselines.
- `W032` narrowed the library-context and provider taxonomy enough that distributed/runtime consequence typing can stay semantically honest.

### Exit gate
- Distributed or broader multi-session runtime consequence classes are explicit and exercised beyond the current local floor.
- Coordinator-visible versus evaluator-local runtime consequences are narrower than today.
- Replay and local formal artifacts cover at least one broader distributed-style consequence family.

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
  - the checked local formal floor now includes session-contention, retry-after-release, overlay-cleanup, pinned-epoch overlay, distributed-placement, retry-ordering fairness, and placement-deferral expiry boundary models for busy-locus rejection, publishable execution, session-local overlay release, exact-match retained reuse, unpinned deterministic eviction, local-admission versus remote-deferral placement outcomes, non-overtaking surfaced retries, and deferred placement expiry-without-claim, but richer fairness and placement-policy semantics remain open
  - full coordinator scheduling policy and product-level distributed semantics remain outside this workset scope
- claim_confidence: draft
