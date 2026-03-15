# OxFml FEC/F3E Spec Set

This directory is the canonical OxFml-owned spec set for the evaluator seam between OxFml and OxCalc.

## Bootstrap Reading Order
1. `FEC_F3E_DESIGN_SPEC.md`
2. `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
3. `FEC_F3E_TESTING_AND_REPLAY.md`
4. `FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`

## Canonical documents
- `FEC_F3E_DESIGN_SPEC.md`
  OxFml-owned seam specification and boundary contract.
- `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
  canonical map from seam clauses to replay/formal obligations and conformance evidence ids.
- `FEC_F3E_TESTING_AND_REPLAY.md`
  testing, replay, and pack strategy for OxFml.
- `FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`
  first replay and fixture plan for the minimum seam schema layer.
- `FEC_F3E_PROTOCOL_CONFORMANCE_MATRIX.csv`
  seam requirement register.

## Archive
Historical transition material is kept under `archive/` and is not part of the required startup read set.

Notable archive contents:
- `archive/FEC_F3E_REDESIGN_OBSERVATIONS.md`
- `archive/FEC_F3E_REDESIGN_SYNTHESIS.md`
- `archive/FEC_F3E_FOUNDATION_UPDATED_SPEC_POINTERS_PROMPT.md`

## Working rule
These documents are written as OxFml source-of-truth artifacts.
They must not describe themselves as imported host or pathfinder documents.

Foundation may keep read-only mirrors.
OxCalc may consume and hand back coordinator-facing requirements through the handoff process.
