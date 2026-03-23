# W037: R1C1 Formula Channels and Reference Translation

## Purpose
Introduce R1C1 as a first-class OxFml formula channel, including grammar, bind translation, caller-position sensitivity, and replay/proving coverage, instead of leaving it as an undocumented gap beside the A1-first baseline.

## Position and Dependencies
- **Depends on**: `W031`
- **Blocks**: later broader formula-mode and multi-carrier work
- **Cross-repo**: OxFml owns formula-channel parsing, translation, and local semantic meaning; downstream consumers use the resulting normalized reference and runtime artifacts

## Scope
### In scope
1. Define the canonical OxFml treatment of R1C1 as a separate formula channel.
2. Add parser and binder support for the first R1C1 reference and expression families.
3. Define caller-position-sensitive translation into normalized reference meaning.
4. Add deterministic replay/proving artifacts for the exercised local R1C1 floor.

### Out of scope
1. Full parity with every Excel formula carrier.
2. Broad name/external-name carrier work.
3. Conditional-formatting or data-validation sublanguage work.

## Deliverables
1. Canonical R1C1 channel rules and artifact treatment.
2. Exercised parser/binder/evaluator coverage for the first R1C1 baseline.
3. Explicit residual list for remaining R1C1 grammar/translation families.

## Gate Model
### Entry gate
- `W031` has classified R1C1 as a missing first-class lane.

### Exit gate
- R1C1 is represented canonically as a formula channel in OxFml docs.
- The local parser/binder floor includes nontrivial R1C1 reference translation with deterministic evidence.
- Remaining R1C1 gaps are explicit rather than implicit.

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
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated
