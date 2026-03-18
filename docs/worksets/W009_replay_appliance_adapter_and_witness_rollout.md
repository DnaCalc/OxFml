# W009: Replay Appliance Adapter and Witness Rollout

## Purpose
Incorporate the Foundation replay appliance handoff into OxFml-owned canonical docs without weakening OxFml semantic authority, and publish the first OxFml replay adapter note plus conservative capability manifest.

## Position and Dependencies
- **Depends on**: `W005`, `W007`, `W008`
- **Blocks**: `W010`
- **Cross-repo**: Foundation replay governance is authoritative for registry ids, capability levels, and witness lifecycle policy; OxFml remains authoritative for formula and seam semantics

## Scope
### In scope
1. Create the OxFml-local replay adapter note and capability manifest.
2. Incorporate additive replay appliance rollout clauses into the canonical OxFml spec set.
3. Pin Foundation replay registry families conservatively for the current OxFml pass.
4. Define the OxFml-side `DNA ReCalc` ingest, normalize, validate, replay, diff, and explain workflow.
5. Bind current OxFml fixture families into the replay rollout plan.
6. Make the adapter capability claim path explicit through `cap.C3.explain_valid`.

### Out of scope
1. Claiming replay-safe formula-text rewrites.
2. Claiming replay-safe bind rewrites.
3. Claiming replay-safe fence rewrites.
4. Claiming replay-safe capability-view rewrites.
5. Claiming `cap.C4.distill_valid`.
6. Claiming `cap.C5.pack_valid`.

## Deliverables
1. `docs/spec/OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md` with OxFml-local authority split, projection rules, identity preservation, lifecycle use, and open alignment items.
2. `docs/spec/OXFML_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json` with conservative capability claims, known limits, conformance refs, and registry pins.
3. Updated canonical replay, schema, taxonomy, and assurance docs reflecting the adapter rollout.
4. A planned workset sequence that makes witness distillation and retained-fixture promotion explicit after `W008`.

## Gate Model
### Entry gate
- `W005`, `W007`, and `W008` have produced a local witness floor strong enough to support conservative replay adapter claims.
- OxFml canonical identity, artifact, schema, taxonomy, and replay docs are already live and can absorb additive replay rollout rules.

### Exit gate
- The adapter note and manifest exist and are referenced from the canonical spec index.
- Updated canonical docs preserve OxFml semantic authority while adopting Foundation replay governance additively.
- The manifest claims no more than `cap.C3.explain_valid`, scaffolds `cap.C4.distill_valid`, and does not claim `cap.C5.pack_valid`.
- The updated docs explicitly exclude replay-safe formula, bind, fence, and capability-view rewrites in this pass.

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
- open_lanes: registry pins still rely on the Foundation handoff package rather than published machine-readable snapshots, and no replay-safe rewrite families are declared in this pass
- claim_confidence: validated
