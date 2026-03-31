# W045: Host Runtime and External Requirements Freeze

## Purpose
Unify the current OxFml host/runtime contract into one implementation-facing seam packet that is sufficient for a direct host and narrow OxCalc-integrated host implementation over the currently covered scope.

## Position and Dependencies
- **Depends on**: `W041`, `W042`, `W043`, `W024`
- **Blocks**: none
- **Cross-repo**: bounded OxCalc seam-coordination packet; OxFml owns canonical draft text, but no closure claim is valid until OxCalc acknowledges and integrates the shared seam reading

## Scope
### In scope
1. Create one canonical OxFml host/runtime and external requirements draft.
2. Unify direct-host and OxCalc-integrated host requirements without collapsing them into one mode.
3. Make required host inputs, services, outputs, and authority boundaries explicit for the currently covered scope.
4. Initiate a bounded OxCalc coordination round keyed to that draft.
5. Record any explicit non-assumptions and deferred lanes needed for honest implementation planning.

### Out of scope
1. Full workbook coordinator policy.
2. Full product-host specification for DNA OneCalc.
3. Final cross-process ABI or deployment packaging.
4. Broader language or built-in-function closure beyond the exercised local floor.

## Deliverables
1. A canonical OxFml host/runtime and external requirements draft.
2. Explicit direct-host versus OxCalc-integrated host mode requirements.
3. A bounded OxCalc note round keyed to that draft.
4. An explicit list of open lanes that still prevent seam-freeze closure.

## Gate Model
### Entry gate
- `W041`, `W042`, and `W043` have produced enough packet-level host/query, return-surface, and runtime-provider truth that a unifying host contract can be drafted honestly.

### Exit gate
- One canonical host/runtime requirements doc exists and is implementation-facing for the currently covered scope.
- OxCalc coordination has been initiated against that draft.
- Direct-host and coordinator-host responsibilities are explicit and non-collapsed.
- Remaining non-frozen lanes are explicitly listed.

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
  - a canonical OxFml draft host/runtime requirements doc now exists and the current OxCalc note round is converged on first-slice implementation sufficiency, but it is not yet promoted as shared seam-freeze text
  - broader host/query, return-surface, and runtime-provider packet execution in `W042` and `W043` remains partial, so the unified host packet is still anchored to a partial local floor
  - caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset remains in the packetized `W026` residual note lane rather than a frozen host-runtime clause
  - execution-restriction transport shape and publication/topology breadth remain narrower than final shared closure and are now explicitly narrowed as the remaining `W026` residual sequences
  - provider-failure and callable-publication remain watch lanes only unless they become coordinator-visible in exercised evidence
  - full product-host policy, broader distributed/runtime ownership, and broader deferred-provider families remain intentionally outside scope
- claim_confidence: provisional
