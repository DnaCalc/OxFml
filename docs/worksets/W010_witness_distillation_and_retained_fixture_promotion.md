# W010: Witness Distillation and Retained Fixture Promotion

## Purpose
Turn the replay-adapter rollout into an OxFml-local witness distillation and retained-fixture promotion policy without overclaiming pack-grade maturity.

## Position and Dependencies
- **Depends on**: `W009`
- **Blocks**: later OxFml replay witness promotion and broader replay appliance maturity claims
- **Cross-repo**: Foundation remains authoritative for predicate, mismatch, lifecycle, quarantine, and capability registries; OxFml remains authoritative for reduction-unit meaning over its artifacts

## Scope
### In scope
1. Define OxFml reduction-unit anchors for commit, reject, effect, and artifact-snapshot witness reduction.
2. Define the first preservation predicate families and closure rules for OxFml replay witnesses.
3. Define subset and projection transforms that OxFml considers admissible in this rollout phase.
4. Define witness lifecycle, quarantine, supersession, and retained-fixture promotion policy for OxFml.
5. Define the first retained-fixture promotion path from local witness evidence to governed replay witness assets.

### Out of scope
1. Claiming replay-safe formula-text rewrites.
2. Claiming replay-safe bind rewrites.
3. Claiming replay-safe fence rewrites.
4. Claiming replay-safe capability-view rewrites.
5. Full pack-grade promotion.
6. Claiming `cap.C5.pack_valid`.

## Deliverables
1. OxFml-local reduction-unit, predicate, and closure planning integrated into replay fixture docs.
2. Witness lifecycle and quarantine rules incorporated into OxFml assurance docs.
3. A retained-fixture promotion policy that keeps explanatory-only and quarantined outputs out of pack-eligible claims.
4. A bounded residual list for what evidence still separates OxFml from `cap.C4.distill_valid` and `cap.C5.pack_valid`.

## Gate Model
### Entry gate
- `W009` has incorporated the replay adapter note, capability manifest, registry pins, and explain-surface binding.
- Current fixture families are explicit enough to define reduction-unit anchors and lifecycle policy without inventing a new scenario DSL.

### Exit gate
- Reduction units, preservation predicates, closure rules, and transform families are explicit in the live spec set.
- Witness lifecycle, quarantine, and retained-fixture promotion policy are explicit in the live OxFml spec set.
- The updated docs do not overclaim full pack-grade promotion or replay-safe rewrites.

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
- open_lanes: retained-fixture promotion remains local-policy only, and pack-grade promotion remains outside this workset
- claim_confidence: validated
