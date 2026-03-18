# W015: Stage 2 Runtime Contention and Concurrency Hardening

## Purpose
Broaden the FEC/F3E runtime from the current sequential managed-session floor into a stronger Stage 2 contention and concurrency-hardening baseline without leaking OxCalc-owned global scheduler policy.

## Position and Dependencies
- **Depends on**: `W014`, `W004`, `W007`
- **Blocks**: `W016`, `W017`, `W018`
- **Cross-repo**: coordinator-facing clauses remain OxFml-owned at the seam and are assessed for ad hoc OxCalc handoff when they mature materially

## Scope
### In scope
1. Multi-session contention and stale-commit behavior within the OxFml-managed runtime boundary.
2. Stronger reject, trace, and effect handling for concurrent or retry-sensitive paths.
3. Overlay cleanup, timeout, abort, and retry-sensitive runtime closure needed before Stage 2 promotion.
4. Deterministic local replay fixtures for contention-sensitive runtime cases.

### Out of scope
1. OxCalc-owned global scheduling policy.
2. Full async/distributed runtime.
3. Pack-grade replay promotion.

## Deliverables
1. A stronger exercised contention-aware managed runtime baseline.
2. Deterministic runtime replay artifacts for concurrent or retry-sensitive reject/effect paths.
3. Updated seam/runtime docs reflecting the Stage 2 pre-promotion floor.

## Gate Model
### Entry gate
- `W014` has broadened the semantic surface enough to make Stage 2 runtime cases representative.

### Exit gate
- Contention-sensitive runtime paths are exercised locally with typed rejects and traces.
- Runtime cleanup and no-publish rules remain explicit under contention-sensitive paths.
- The live seam docs reflect the stronger Stage 2 pre-promotion baseline.

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
- open_lanes: broader async or distributed runtime policy and OxCalc-owned scheduler integration remain outside this workset scope
- claim_confidence: validated
