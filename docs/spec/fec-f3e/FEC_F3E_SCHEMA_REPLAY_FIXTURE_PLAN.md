# FEC/F3E Schema Replay Fixture Plan

## 1. Purpose
This document defines the first replay and fixture plan for the minimum seam schema layer.

Its scope is narrower than general FEC/F3E testing.
It focuses on proving that the typed payload schemas are sufficient for deterministic interpretation and replay.

Read together with:
1. `FEC_F3E_DESIGN_SPEC.md`
2. `FEC_F3E_TESTING_AND_REPLAY.md`
3. `../OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
4. `../OXFML_MINIMUM_SEAM_SCHEMAS.md`
5. `../OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`

## 2. Fixture Families
The first fixture families should be:
1. accepted candidate result fixture,
2. published commit bundle fixture,
3. typed reject-context fixture set,
4. spill-event fixture set,
5. trace-event correlation fixture set,
6. host-query capability-view fixture set,
7. managed session lifecycle fixture set.

## 3. Required Scenarios
### 3.1 Candidate accepted and published
Fixture must prove:
1. candidate-result payload carries enough data for atomic publication,
2. commit bundle preserves the required publishable fields,
3. trace correlation links candidate and commit.

### 3.2 Candidate built then fence-rejected
Fixture must prove:
1. candidate construction is visible,
2. publication does not occur,
3. `FenceMismatchContext` is sufficient for deterministic replay,
4. trace correlation distinguishes candidate existence from no-publish outcome.

### 3.3 Execute-time capability denial
Fixture must prove:
1. typed denial context exists,
2. no partial publication occurs,
3. capability-denial trace path is replayable.

### 3.4 Spill blocked and spill recovery
Fixture must prove:
1. `ShapeDelta` and `SpillEvent` payloads carry anchor and extent information,
2. blocking loci are explicit,
3. later recovery can be expressed without ad hoc interpretation.

### 3.5 Dynamic-reference rediscovery
Fixture must prove:
1. `TopologyDelta` can express additions, removals, or reclassifications,
2. dynamic-reference facts are coordinator-consumable,
3. the same scenario is replayable from the typed payload surface.

### 3.6 Host-query capability path
Fixture must prove:
1. a typed `HostQueryCapabilityView` can express available query classes,
2. denial policy is explicit when a host-query lane is unavailable,
3. no raw host object leakage is required for function dispatch.

### 3.7 Managed session lifecycle path
Fixture must prove:
1. open, capability-view establishment, execute, and commit phases remain distinct,
2. abort and expiry produce typed no-publish outcomes,
3. trace-event ordering remains deterministic across accepted and rejected session paths.

## 4. Fixture Shape
Each fixture bundle should contain:
1. scenario id,
2. input artifact summary,
3. typed payload object under test,
4. expected interpretation summary,
5. trace correlation expectations,
6. replay acceptance criteria.

Current first local artifact locations:
1. `crates/oxfml_core/tests/fixtures/parse_bind_cases.json`
2. `crates/oxfml_core/tests/fixtures/semantic_plan_replay_cases.json`
3. `crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json`
4. `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
5. `crates/oxfml_core/tests/fixtures/execution_contract_replay_cases.json`
6. `crates/oxfml_core/tests/fixtures/session_lifecycle_replay_cases.json`
7. `crates/oxfml_core/tests/fixtures/single_formula_host_replay_cases.json`
   Current exercised lanes: reuse-sensitive recalc, scalarization, helper forms, spill publication, formatting, and host-query
8. `crates/oxfml_core/tests/fixtures/empirical_oracle_scenarios.json`
   Current exercised lanes: formatting, host-query, scalarization, helper-form invocation, spill publication, and seam-significant `format_delta` / `display_delta`
9. `crates/oxfml_core/tests/fixtures/library_context_snapshot_cases.json`
10. `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/promotion_candidate_families.json`
11. `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/promotion_readiness_index.json`

Current rule:
1. these are local deterministic witness artifacts for the implementation-start slice,
2. they are not yet the full pack-grade replay corpus,
3. future fixture families should reuse the same scenario-id discipline instead of introducing ad hoc naming.
4. when host or seam artifacts gain new coordinator-relevant facts, the affected host and replay fixture families should expand in the same wave rather than lagging behind the schema change.

## 5. Witness Distillation Planning
### 5.1 Reduction Units
The current OxFml local-only reduction-unit anchors are:
1. `oxfml.local.reduction_unit.fixture_case`
2. `oxfml.local.reduction_unit.lifecycle_block`
3. `oxfml.local.reduction_unit.candidate_attempt`
4. `oxfml.local.reduction_unit.commit_attempt`
5. `oxfml.local.reduction_unit.reject_context_slice`
6. `oxfml.local.reduction_unit.effect_slice`
7. `oxfml.local.reduction_unit.artifact_sidecar`

### 5.2 Preservation Predicates
The current additive predicate families for planning are:
1. `pred.reject.family_present`
   - preserve one typed reject family and context class
2. `pred.publication.not_published_reason`
   - preserve one non-publication reason with candidate-versus-no-publish boundary intact
3. `pred.publication.accepted_payload_present`
   - preserve one accepted publication payload class and candidate-to-publication lineage
4. `pred.execution.restriction_present`
   - preserve one execution-restriction family and replay-sensitivity classification
5. `pred.diff.mismatch_present`
   - preserve one commit, reject, effect, or topology mismatch class
6. `pred.invariant.failed`
   - preserve one resource or invariant failure class where applicable
7. `pred.topology.dependency_consequence_present`
   - preserve one typed dependency-consequence evidence class with its causal lane
8. `pred.publication.format_display_present`
   - preserve one seam-significant format or display publication consequence

### 5.3 Closure Rules
Current closure rules are:
1. retaining a candidate or reject keeps `formula_stable_id`, `formula_token`, `snapshot_epoch`, `bind_hash`, and relevant `session_id`,
2. retaining a commit outcome keeps the producing candidate lineage and `commit_attempt_id`,
3. retaining a reject-context slice keeps the surrounding reject family and triggering lifecycle boundary,
4. retaining an effect slice keeps the effect family id, relevant causal lifecycle phase, and typed correlation ids,
5. retaining a sidecar-backed artifact keeps its content fingerprint and source schema identity.
6. retaining a host-facing witness that depends on concrete cell resolution keeps any direct cell bindings needed to replay that semantic path faithfully.

### 5.4 Allowed Transforms
The currently allowed transform families are:
1. subset transforms
   - drop unused fixture cases, lifecycle blocks, sidecar blocks, or optional effect slices
2. projection transforms
   - replace large payload bodies with sidecar refs, hashes, or explicit missing/opaque markers where replay closure remains intact

### 5.5 Explicitly Prohibited Rewrites In This Pass
The current replay rollout does not declare these rewrite families replay-safe:
1. formula-text rewrites,
2. bind rewrites,
3. fence-tuple rewrites,
4. capability-view rewrites.

Working rule:
1. these rewrites remain prohibited unless a later OxFml workset declares them replay-safe and locally evidenced.

### 5.6 Current Authored Local Reduction Artifacts
The first authored local reduction artifact set is:
1. `crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_reduction_manifest.json`
2. `crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_witness_bundle.json`
3. `crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_lifecycle.json`
4. `crates/oxfml_core/tests/fixtures/witness_distillation/fec_accept_publication_reduction_manifest.json`
5. `crates/oxfml_core/tests/fixtures/witness_distillation/fec_accept_publication_witness_bundle.json`
6. `crates/oxfml_core/tests/fixtures/witness_distillation/fec_accept_publication_lifecycle.json`
7. `crates/oxfml_core/tests/fixtures/witness_distillation/session_capability_denied_reduction_manifest.json`
8. `crates/oxfml_core/tests/fixtures/witness_distillation/session_capability_denied_witness_bundle.json`
9. `crates/oxfml_core/tests/fixtures/witness_distillation/session_capability_denied_lifecycle.json`
10. `crates/oxfml_core/tests/fixtures/witness_distillation/execution_contract_host_query_reduction_manifest.json`
11. `crates/oxfml_core/tests/fixtures/witness_distillation/execution_contract_host_query_witness_bundle.json`
12. `crates/oxfml_core/tests/fixtures/witness_distillation/execution_contract_host_query_lifecycle.json`
13. `crates/oxfml_core/tests/fixtures/witness_distillation/single_formula_host_scalarization_reduction_manifest.json`
14. `crates/oxfml_core/tests/fixtures/witness_distillation/single_formula_host_scalarization_witness_bundle.json`
15. `crates/oxfml_core/tests/fixtures/witness_distillation/single_formula_host_scalarization_lifecycle.json`
16. `crates/oxfml_core/tests/fixtures/witness_distillation/empirical_oracle_host_query_reference_reduction_manifest.json`
17. `crates/oxfml_core/tests/fixtures/witness_distillation/empirical_oracle_host_query_reference_witness_bundle.json`
18. `crates/oxfml_core/tests/fixtures/witness_distillation/empirical_oracle_host_query_reference_lifecycle.json`
19. `crates/oxfml_core/tests/fixtures/witness_distillation/retained_witness_set_index.json`

Current rule:
1. this artifact set proves the intended shape of retained local witness reduction over the existing FEC commit/reject fixture family,
2. the local floor is now broadened across accepted publication, session rejection, execution-restriction, single-formula host, and empirical-oracle families,
3. retained-local breadth is now indexable across more than one source family,
4. broader replay-valid reduction claims remain open until additional witness families are exercised.

### 5.7 Local Normalized Bundle Evidence
Current local normalized bundle evidence is:
1. `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/fec_commit_pack_candidate_bundle.json`
2. `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/session_lifecycle_pack_candidate_bundle.json`
3. `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/pack_candidate_index.json`

Current rule:
1. these normalized bundles remain local-only rehearsal artifacts,
2. they must preserve source schema ids, witness refs, and non-pack-eligible state,
3. they do not authorize pack-grade promotion.

## 6. Initial Pack Mapping
The first pack-family alignment is:
1. `PACK.fec.minimum_payload_schemas`
2. `PACK.fec.transaction_boundary`
3. `PACK.fec.reject_detail_replay`
4. `PACK.fec.overlay_lifecycle`
5. `PACK.oxfml.oxfunc.prepared_contract`

## 7. Open Decisions
The following remain open:
1. exact fixture file formats,
2. whether fixtures are JSON-style canonical payload dumps, DSL scenarios, or both,
3. whether trace correlation ids are explicit test literals or harness-generated but normalized,
4. when OxFml should promote any current local witness family to retained shared status,
5. what local evidence will be required before any replay-safe rewrite family is declared.

## 8. Working Rule
No minimum-schema clause should be treated as mature until at least one corresponding fixture family exists.
