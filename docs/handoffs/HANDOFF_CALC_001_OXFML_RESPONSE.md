# HANDOFF-CALC-001 OxFml Review and Acknowledgment

## Purpose
Record the OxFml-side review of `../OxCalc/docs/handoffs/HANDOFF_CALC_001_OXFML_COORDINATOR_SEAM_HARDENING.md` against the current canonical OxFml seam docs.

This document is an OxFml working acknowledgment and response.
It is not a closure artifact.

## Findings
### F1. Candidate-result versus publication was under-specified in the live seam text
Severity: high

Current OxFml seam text already required atomic commit bundles and typed rejects, but it moved too quickly from evaluator execution to published bundle language.

Relevant OxFml references:
1. `docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md`
2. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`

Decision:
1. adapt and promote now.

Action taken:
1. introduced explicit `AcceptedCandidateResult` language and artifact shape,
2. clarified that evaluator success is not itself publication,
3. clarified that accepted commit promotes candidate result to published bundle.

### F2. Reject-is-no-publish was present, but fence/capability consequences needed sharper seam wording
Severity: medium

Current OxFml text already required typed, replay-stable, non-publishing rejects.
However, the coordinator-facing consequence for stale or incompatible candidate work was not stated sharply enough in the seam text itself.

Decision:
1. adapt and promote now.

Action taken:
1. clarified that stale or incompatible candidate work is rejected rather than partially published,
2. required typed reject detail for fence and capability incompatibility.

### F3. Runtime-derived effect reporting existed in fragments but not as one explicit coordinator-facing rule
Severity: medium

OxFml already had evaluator facts, spill events, topology deltas, and overlay rules.
What was missing was a single sentence binding them to coordinator correctness.

Decision:
1. adapt and promote now at the general-rule level,
2. defer exhaustive runtime-effect taxonomy details pending exercised evidence.

Action taken:
1. added explicit coordinator-relevant runtime-derived effect surfacing language in the seam text,
2. linked this to candidate-result payload structure rather than scheduler policy.

## Clause Decisions
1. Accepted-result payload structure: `adapt`
2. Structured reject detail: `adapt`
3. Fence consequences: `adapt`
4. Runtime-derived effect reporting: `adapt` for the general obligation; exhaustive taxonomy remains open

## OxFml-Side Drafted Changes
The following canonical OxFml docs were updated:
1. `docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md`
2. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
3. `docs/spec/fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
4. `docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`
5. `docs/spec/fec-f3e/FEC_F3E_PROTOCOL_CONFORMANCE_MATRIX.csv`

## Trace and Replay Implications
1. trace/event correlation must distinguish candidate-result construction from commit acceptance,
2. reject traces must capture fence/capability mismatch detail as typed no-publish outcomes,
3. replay packs will need candidate-vs-publication boundary cases in addition to commit/reject cases,
4. exhaustive effect-reporting schema details remain open pending exercised evidence.

## Handback Needed to OxCalc
1. OxCalc can now align to the new candidate-result versus publication wording in the OxFml canonical seam,
2. OxCalc should not assume the exhaustive runtime-derived effect taxonomy is closed yet,
3. if OxCalc needs stronger coordinator-local derivation rules beyond the current surfaced families, send a narrower follow-on handoff tied to specific replay-sensitive scenarios.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - no replay artifacts yet proving the new candidate-result/publication boundary
  - exhaustive runtime-derived effect taxonomy still open
  - no receiving-repo acknowledgment yet
