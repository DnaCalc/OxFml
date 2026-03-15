# W001: Spec Reset for Formula Engine and FEC/F3E Baseline

## Purpose
Reset the OxFml spec set so it is written as the canonical specification for OxFml itself, not as an imported DnaVisiCalc/pathfinder artifact.

This workset establishes a fresh baseline for:
1. formula parsing/binding/evaluation architecture,
2. the FEC/F3E evaluator seam,
3. OxFunc/OxCalc boundary rules,
4. testing and replay expectations for OxFml.

## Position and Dependencies
- **Depends on**: none
- **Blocks**: worksets that implement or refine OxFml evaluator contracts
- **Cross-repo**: OxFunc upstream observations reviewed; OxCalc seam handoffs remain recorded for ad hoc future coordination where coordinator-facing clauses mature further

## Scope
### In scope
1. Rewrite OxFml canonical spec docs to remove DnaVisiCalc-specific ownership and implementation framing.
2. Add a fresh formula-engine architecture document with Roslyn-like green/red parse-tree design.
3. Rewrite FEC/F3E spec documents around OxFml/OxFunc/OxCalc boundaries.
4. Add initial OxFml testing and replay strategy text.
5. Update local spec indexes and pointers.
6. Separate canonical bootstrap docs from archived empirical planning/evidence material.
7. Make the FEC/F3E formal and assurance surface explicit in the live spec set.
8. Make the implementation-shape and state-ownership options explicit without locking the repo to one runtime model.
9. Make the canonical artifact identity and version vocabulary explicit for later implementation work.
10. Make the canonical artifact field surfaces explicit for bind, semantic, prepared-call, commit, and reject artifacts.
11. Make the seam taxonomy layer explicit for deltas, evaluator facts, reject contexts, and trace events.
12. Make the minimum typed schema objects explicit for coordinator-visible seam payload families and OxFunc host-query capability views.
13. Add implementation-start planning docs for parser/binder realization, baseline API direction, replay fixture planning, and formal artifact mapping.
14. Add normalized reference ADT and public-surface/API sketch docs so implementation-start boundaries are explicit before code work begins.

### Out of scope
1. Implementation code.
2. Full empirical matrix refresh across all legacy evidence ids.
3. Cross-repo handoff closure with OxCalc.
4. DNA OneCalc host spec.

## Deliverables
1. A fresh OxFml formula-engine architecture spec.
2. Rewritten FEC/F3E canonical docs without pathfinder-import wording.
3. Initial testing/replay spec for OxFml.
4. Updated local spec index/pointer docs.
5. A canonical bootstrap path that does not require archived empirical planning documents.
6. A canonical FEC/F3E assurance map with readable conformance-matrix evidence identifiers.
7. A canonical implementation-options note covering stateless, stateful, and hybrid surfaces.
8. A canonical identity/version vocabulary for artifacts, fingerprints, and runtime handles.
9. A canonical artifact-shapes note for the main formula/evaluation/seam artifacts.
10. A canonical taxonomy note for deltas, evaluator facts, reject contexts, and trace-event families.
11. A canonical minimum-schema note for delta payloads, spill events, typed reject contexts, trace payloads, and host-query capability views.
12. A parser/binder realization note, implementation baseline note, schema replay fixture plan, and formal artifact register.
13. A canonical normalized reference ADT note and a first public API/runtime-service sketch.

## Gate Model
### Entry gate
- Required startup docs and inbound observations reviewed.
- Rewrite scope limited to OxFml-local canonical docs.

### Exit gate
- Canonical OxFml spec docs no longer describe themselves as imported DnaVisiCalc artifacts.
- Formula parse/bind/eval architecture is specified with green/red tree framing.
- FEC/F3E evaluator-side contract is specified with OxFunc/OxCalc boundaries.
- Testing/replay expectations are explicitly documented.
- Local indexes point to the new canonical structure.
- Archived empirical planning/evidence docs are separated from required bootstrap reading.
- FEC/F3E assurance obligations are explicit in the live bootstrap set.
- Implementation-shape options and state-ownership constraints are explicit in the live bootstrap set.
- Artifact identity, version-key, fingerprint, and handle vocabulary is explicit in the live bootstrap set.
- Canonical field surfaces for the main OxFml artifacts are explicit in the live bootstrap set.
- Canonical taxonomy families for deltas, evaluator facts, reject contexts, and trace events are explicit in the live bootstrap set.
- Canonical minimum schemas for coordinator-visible seam payload families are explicit in the live bootstrap set.
- Implementation-start planning docs for parser/binder realization, baseline API direction, replay fixture planning, and formal artifact mapping are explicit in the live bootstrap set.
- Normalized reference ADT and public-surface/API sketch docs are explicit in the live bootstrap set.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | no |
| 6 | No known semantic gaps remain in declared scope? | no |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes: OxCalc upstream observation ledger missing; recorded OxCalc seam handoffs remain open for ad hoc future coordination; no replay evidence refreshed against the rewritten spec set
- claim_confidence: draft
- reviewed inbound observations: OxFunc ledger reviewed; OxCalc ledger missing at expected path
