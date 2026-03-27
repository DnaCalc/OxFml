# W051: OxCalc Fixture Host and Stand-In Packet

## Purpose
Freeze the first bounded stand-in host/coordinator packet that OxFml can use in deterministic integration artifacts while keeping production OxCalc coordinator semantics out of scope.

## Position and Dependencies
- **Depends on**: `W045`, `W049`, `W050`
- **Blocks**: none
- **Cross-repo**: bounded OxCalc seam round keyed to the reuse of host/coordinator-owned truths in OxFunc-facing adapter and fixture artifacts

## Scope
### In scope
1. Define the first deterministic stand-in host/coordinator packet for integration artifacts.
2. Make host/coordinator-owned versus OxFml-owned truth explicit in that packet.
3. Reuse current host/runtime packet families rather than inventing a separate ad hoc mock interface.
4. Initiate a bounded OxCalc note round on that packet.
5. Keep `CALL` / `REGISTER.ID` and broader production coordinator policy explicitly out of the first stand-in packet wave.

### Out of scope
1. Production OxCalc coordinator API freeze.
2. Full graph scheduler policy.
3. Full distributed/runtime ownership.
4. Deferred registered-external runtime beyond current first-wave fixture needs.

## Deliverables
1. A canonical first stand-in host/coordinator packet draft.
2. A bounded OxCalc note round keyed to that draft.
3. An explicit list of non-assumptions so fixture-host reuse does not get mistaken for coordinator API freeze.

## Gate Model
### Entry gate
- `W045` has converged enough that host/runtime packet families are known.
- `W049` / `W050` have made it clear that some OxFunc-facing fixture inputs are actually stand-ins for OxCalc-owned host truths.

### Exit gate
- One canonical OxFml draft stand-in packet exists.
- OxCalc coordination has been initiated against that packet.
- Host/coordinator-owned truths versus OxFml-owned truths are explicit and non-collapsed.

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
  - a canonical stand-in host/coordinator packet draft now exists and OxCalc has reviewed it as settled enough for deterministic fixture-host and first TreeCalc-facing integration reuse, but the packet is not yet frozen beyond the current narrow first wave
  - the accepted identity refinements are now part of the packet direction:
    - `fixture_input_id`
    - structure-context identity
    - optional `formula_slot_id`
  - candidate / commit / reject capture remains intentionally separate from the stand-in input packet and that boundary is converged at note level, not yet shared seam-freeze text
  - broader coordinator API freeze and later packet reuse across wider slot families remain outside the current narrow stand-in packet
- claim_confidence: provisional
