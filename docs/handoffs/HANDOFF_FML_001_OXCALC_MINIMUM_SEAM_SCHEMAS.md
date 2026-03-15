# HANDOFF-FML-001: Minimum Seam Schema Tightening for Candidate, Commit, Reject, and Trace Payloads

## 1. Purpose
Record the OxFml-side promotion of minimum typed schema objects for coordinator-visible seam payloads and request OxCalc review of coordinator-facing consequences.

This handoff does not treat OxCalc mirror text as canonical.
OxFml remains the owner of the shared evaluator-facing seam specification.

## 2. Trigger
OxFml has tightened the canonical seam docs from taxonomy-only wording to minimum schema objects for:
1. candidate and commit payload families,
2. spill-event payloads,
3. typed reject-context objects,
4. trace payload correlation objects.

OxFml also made the host-query capability boundary explicit for `CELL` / `INFO`-style lanes consumed through OxFunc.

## 3. Canonical OxFml Sources
The promoted OxFml-side text now lives in:
1. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
2. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
3. `docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md`
4. `docs/spec/fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
5. `docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`
6. `docs/spec/fec-f3e/FEC_F3E_PROTOCOL_CONFORMANCE_MATRIX.csv`

## 4. Coordinator-Facing Impact
### 4.1 Accepted candidate and commit interpretation
OxCalc should now assume that accepted candidate results and published commit bundles are not only typed by family, but also expected to expose minimum fields sufficient to:
1. identify the relevant formula and publication loci,
2. interpret spill/blocking consequences without ad hoc inference,
3. distinguish dependency additions/removals/reclassifications from policy decisions,
4. correlate commit and reject outcomes against candidate construction.

### 4.2 Reject interpretation
OxCalc should now assume typed reject contexts rather than generic structured blobs.

The canonical reject-context families currently tightened are:
1. `FenceMismatchContext`
2. `CapabilityDenialContext`
3. `SessionTerminationContext`
4. `BindMismatchContext`
5. `StructuralConflictContext`
6. `DynamicReferenceFailureContext`
7. `ResourceInvariantContext`

### 4.3 Trace and replay consequences
Replay-sensitive coordinator flows should now preserve correlation to:
1. `candidate_result_id`
2. `commit_attempt_id`
3. `reject_record_id`
4. optional fence snapshot references

## 5. Requested OxCalc Review
Please review and respond on:
1. whether the minimum schema field sets are sufficient for coordinator-controlled accept/reject/publication decisions,
2. whether any additional coordinator-required field family is still missing,
3. whether any proposed field family leaks coordinator policy back into the evaluator seam,
4. whether the current trace correlation keys are sufficient for deterministic coordinator replay.

## 6. Current OxFml Position
### Promote directly
1. minimum typed schema sufficiency as a seam rule,
2. typed reject-context objects instead of generic reject maps,
3. typed host-query capability view for host-observing OxFunc lanes.

### Still open
1. exact wire/object encoding,
2. whether some fact refs are embedded objects or stable ids,
3. Stage 2 concurrency-specific refinement of retry-vs-terminal reject payloads,
4. replay artifacts proving the new schema surfaces.

## 7. Requested Response Shape
OxCalc response should:
1. reference `HANDOFF-FML-001`,
2. identify any missing coordinator-critical field families narrowly,
3. avoid treating OxCalc-local mirror text as canonical,
4. distinguish required additions from optional implementation conveniences.
