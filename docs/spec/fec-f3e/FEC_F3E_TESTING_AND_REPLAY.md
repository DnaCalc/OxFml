# FEC/F3E Testing and Replay Strategy

## 1. Purpose
This document defines the initial OxFml testing, replay, and evaluation strategy for the formula engine and FEC/F3E seam.

The goal is to make OxFml testable in isolation, testable with OxFunc, and testable at the OxCalc seam without conflating those layers.

This document should be read together with:
1. `FEC_F3E_DESIGN_SPEC.md`
2. `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
3. `FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`
4. `../OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
5. `../OXFML_MINIMUM_SEAM_SCHEMAS.md`
6. `../OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`
7. `../OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`
8. `../OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
9. `../OXFML_EMPIRICAL_PACK_PLANNING.md`

## 1A. Test Ladder
The canonical OxFml test ladder is defined in:
1. `../OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`

This document applies that ladder to FEC/F3E-specific testing and replay obligations.

## 2. Assurance Layers
OxFml testing is split into six layers:

1. **Syntax fidelity**
   - tokenization,
   - parse acceptance/rejection,
   - full-fidelity round-tripping,
   - entered-text vs stored-text capture.
2. **Bind and normalization**
   - name scope resolution,
   - structured reference binding,
   - relative/absolute address normalization,
   - explicit unresolved-reference classification.
3. **Minimal local bootstrap evaluator**
   - literals and operators,
   - tiny fixture function set or probe/test-only functions,
   - defined-name supplied values,
   - fast OxFml-owned bring-up and benchmark loops.
4. **OxFunc preparation contract**
   - prepared-argument provenance,
   - reference-preserving dispatch,
   - lazy/eager/reference-preserved evaluation modes,
   - format/locale service injection.
5. **FEC/F3E transaction boundary**
   - session lifecycle,
   - snapshot/token/capability fences,
   - candidate-result versus published-bundle distinction,
   - atomic commit bundle shape,
   - minimum delta and reject-context payload schemas,
   - typed reject detail,
   - surfaced runtime-derived effect families.
6. **Replay and integration**
   - dynamic dependency rediscovery,
   - spill takeover/clearance/blocked flows,
   - format dependency invalidation,
   - single-formula proving host,
   - DNA OneCalc single-node proving and OxCalc seam proving,
   - Excel empirical oracle comparison runs.

## 3. Required Artifact Types
Each exercised OxFml behavior should eventually produce one or more of:
1. deterministic scenario definitions,
2. replay bundles,
3. structured trace logs,
4. normalized parse/bind snapshots,
5. conformance-matrix rows with evidence links,
6. schema-validation fixtures for typed seam payload objects.

## 3A. Evidence Tiers
OxFml currently distinguishes two assurance tiers for replay evidence:

1. **Local witness evidence**
   - deterministic fixtures and tests living inside the repo,
   - sufficient for local workset-gate closure where the declared scope is an implementation-start baseline,
   - not sufficient by itself for program-level assurance claims.
2. **Pack-grade evidence**
   - promoted scenario packs with stable identifiers, scenario metadata, and explicit clause mapping,
   - required for stronger program-level assurance, broader promotion claims, and cross-repo conformance narratives.

Working rule:
1. local witness evidence may satisfy a local workset gate when the workset explicitly targets a baseline slice,
2. local witness evidence must not be described as if it were already pack-grade corpus,
3. spec and status docs should state which tier is currently present.

## 3B. DNA ReCalc Workflow For OxFml Fixture Families
The OxFml replay rollout adopts the Foundation `DNA ReCalc` workflow additively.

Current OxFml workflow:
1. ingest
   - import lane-native fixture families and local witness artifacts as OxFml-owned source material
2. normalize
   - emit additive replay bundle envelopes while preserving source schema ids, typed payloads, and sidecar refs
3. validate
   - validate bundle shape, source-schema compatibility, and explicit projection gaps
4. replay
   - rerun supported fixture scenarios deterministically against preserved OxFml semantics
5. diff
   - compare normalized replay outputs using additive mismatch families while preserving OxFml source kinds
6. explain
   - answer why-changed, why-rejected, and why-not-published questions from bundle artifacts and source refs
7. distill
   - planned for future rollout only after replay-valid reduced witnesses are locally evidenced

Workflow rule:
1. OxFml fixture import does not flatten typed artifacts into generic replay prose,
2. normalization is additive transport only,
3. witness distillation is offline and remains outside the current claimed capability level.

## 3C. Adapter Capability Claim Path
The OxFml replay adapter capability path is:
1. publish a conservative capability manifest,
2. bind each claimed level to local witness-tier conformance artifacts,
3. surface known limits explicitly,
4. keep pack-grade promotion separate from current capability claims.

Current target:
1. claim `cap.C0.ingest_valid`
2. claim `cap.C1.replay_valid`
3. claim `cap.C2.diff_valid`
4. claim `cap.C3.explain_valid`
5. scaffold but do not claim `cap.C4.distill_valid`
6. do not claim `cap.C5.pack_valid`

Current rule:
1. the capability manifest is honest only if known gaps stay explicit,
2. local witness-tier evidence is sufficient for the current OxFml local rollout target,
3. local witness-tier evidence is not sufficient to imply pack-grade maturity.

## 4. Initial Pack Map
The baseline OxFml pack map is:

1. `PACK.oxfml.parse.full_fidelity`
2. `PACK.oxfml.bind.reference_normalization`
3. `PACK.oxfml.bootstrap_evaluator.minimal`
4. `PACK.oxfml.oxfunc.prepared_contract`
5. `PACK.oxfml.single_formula_host.recalc`
6. `PACK.oxfml.empirical_formula_oracle`
7. `PACK.fec.commit_atomicity`
8. `PACK.fec.reject_detail_replay`
9. `PACK.fec.overlay_lifecycle`
10. `PACK.fec.format_dependency_tokens`
11. `PACK.format.semantic_vs_display_boundary`
12. `PACK.fec.transaction_boundary`
13. `PACK.fec.minimum_payload_schemas`

## 4A. Witness Lifecycle, Quarantine, and Pack Eligibility
OxFml adopts the Foundation witness lifecycle and quarantine model as rollout governance.

Current rules:
1. explanatory-only witnesses are not pack-eligible,
2. quarantined witnesses are not pack-eligible,
3. reduced witnesses remain local evidence until they carry explicit lifecycle refs and satisfy replay-valid policy,
4. pack eligibility additionally requires the adapter capability surface to meet the pack-required level,
5. current OxFml rollout does not declare formula-text, bind, fence, or capability-view rewrites replay-safe,
6. local replay bundles and normalized fixtures may remain useful for ingest, replay, diff, and explain even when not pack-eligible,
7. local normalized pack-candidate bundles are rehearsal artifacts only and must remain explicitly non-pack-eligible.
8. retained-local witness sets may broaden across host and empirical-oracle families, but retained-local breadth alone does not imply `cap.C4.distill_valid`.

## 5. Local Bootstrap Evaluator Role
OxFml should maintain a minimal local bootstrap evaluator surface for fast OxFml-owned testing.

Its role is:
1. not to replace OxFunc,
2. to exercise parser/binder/evaluator/seam paths quickly,
3. to support local regression and benchmark loops with a tiny fixture function set,
4. to support defined-name-driven single-formula proving before full downstream breadth is available.

This local kernel must remain intentionally small.

## 6. OxFunc-Backed Semantic Role
Beyond the minimal bootstrap kernel, OxFml should use OxFunc outputs for downstream function-semantic testing.

That means:
1. OxFml should test prepared-call/result behavior against OxFunc semantics,
2. OxFml should avoid broad local reimplementation of real function families,
3. the wider function-semantic corpus should come from OxFunc-backed runs.

## 7. Single-Formula Proving Host Role
Before full DNA OneCalc host specification, OxFml should exercise a single-formula proving host with:
1. one formula under test,
2. mutable defined-name inputs,
3. mutable direct cell bindings where a reference-sensitive formula needs concrete resolution,
4. full update and full recalc semantics,
5. no multi-formula dependency graph,
6. candidate/commit/reject/trace output capture.

Current exercised local floor:
1. reuse-sensitive recalc over changed host inputs,
2. scalarization-sensitive host runs for `@` and `_xlfn.SINGLE`,
3. helper-form host runs for `LET` and callable `LAMBDA`,
4. spill-sensitive host runs for `SEQUENCE`,
5. formatting-sensitive host runs for `TEXT`,
6. host-query-sensitive host runs for `INFO` and `CELL("filename", ...)`.

## 8. Empirical Oracle Role
OxFml should maintain formula-oriented empirical validation scaffolding that uses Excel behavior as oracle.

The scaffolding should make it easy to verify:
1. stored-form vs entered-form behavior,
2. single-formula evaluation behavior,
3. defined-name input update behavior,
4. high-risk lanes such as `@`, `#`, `SINGLE`, `LET`, `LAMBDA`, spill publication, formatting-sensitive semantics, and host-query semantics.

Current exercised local floor:
1. formatting oracle scenarios via `TEXT`,
2. host-query oracle scenarios via `INFO("directory")` and `CELL("filename", ref)`,
3. scalarization oracle scenarios via `@` and `_xlfn.SINGLE`,
4. helper-form oracle scenarios via `LET` and callable `LAMBDA`,
5. spill-shaped oracle scenarios via `SEQUENCE(2)`.

## 9. Formal and Model-Checking Obligations
OxFml testing is coupled to formal assurance from the start.

The initial formal obligations are:
1. Lean-friendly type definitions for syntax, bind outputs, prepared-call contracts, commit bundles, and reject taxonomy,
2. TLA+ models for session lifecycle, commit atomicity, snapshot/token fences, session expiry, and concurrent contention behavior,
3. explicit mapping from each pack family to the prose contract sections it validates,
4. replay artifacts that witness the same clauses concretely.

The first TLA+ priority is the FEC/F3E transaction and concurrency surface.
The first Lean priority is the typed contract surface and structural invariants.

The clause-to-witness mapping used for seam planning is defined in:
1. `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`

## 10. DNA OneCalc Evaluation Role
DNA OneCalc is the preferred early proving host for:
1. parser correctness,
2. binder/reference normalization,
3. OxFunc integration,
4. single-node evaluation semantics,
5. reduced-profile FEC/F3E transaction exercises.

DNA OneCalc is not allowed to redefine OxFml semantics.
Its role is to exercise the OxFml/OxFunc contracts without OxCalc multi-node coordination.

The current host-policy baseline and empirical-pack planning docs are:
1. `../OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
2. `../OXFML_EMPIRICAL_PACK_PLANNING.md`

## 11. OxCalc Integration Role
OxCalc integration testing should validate:
1. atomic bundle publication,
2. policy-boundary discipline,
3. replay-stable rejects,
4. coordinator consumption of topology evidence without seam policy leakage,
5. schema-sufficient candidate, commit, spill, and reject payloads for deterministic accept/reject handling.

## 12. Current Open Assurance Lanes
The following remain explicitly open:
1. contention replay for multi-session commit conflicts,
2. canonical unified trace schema versus subsystem schema merge strategy,
3. proof obligations for fast-path soundness,
4. proof obligations for parallel reduction determinism,
5. full cross-build empirical refresh of legacy Excel-compat evidence,
6. pack-grade promotion of the currently local witness corpus,
7. TLA+ and Lean artifact authoring for the now-exercised execution-profile and proving-host slices,
8. local evidence for `cap.C4.distill_valid`,
9. all policy surfaces needed for `cap.C5.pack_valid`.

## 13. Current Local Witness Floor
The current local witness floor for the exercised implementation-start slice is:
1. parse/bind fixtures: `crates/oxfml_core/tests/fixtures/parse_bind_cases.json`
2. semantic-plan fixtures: `crates/oxfml_core/tests/fixtures/semantic_plan_replay_cases.json`
3. prepared-call/result fixtures: `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
4. FEC commit/reject fixtures: `crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json`
5. execution-contract fixtures: `crates/oxfml_core/tests/fixtures/execution_contract_replay_cases.json`
6. session lifecycle fixtures: `crates/oxfml_core/tests/fixtures/session_lifecycle_replay_cases.json`
   Current exercised lanes: fence rejection, contention, async-coupled external-provider execution, dependency consequence facts, and overlay-family expectations
7. single-formula host fixtures: `crates/oxfml_core/tests/fixtures/single_formula_host_replay_cases.json`
   Current exercised lanes: reuse-sensitive recalc, scalarization, helper forms, spill, formatting, host-query, and seam-significant publication-surface deltas
8. empirical-oracle scenario fixtures: `crates/oxfml_core/tests/fixtures/empirical_oracle_scenarios.json`
   Current exercised lanes: formatting, host-query, scalarization, helper-form invocation, spill publication, and seam-significant `format_delta` / `display_delta`
9. replay-adapter manifest and conformance checks: `docs/spec/OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`, `docs/spec/OXFML_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json`, and `crates/oxfml_core/tests/replay_adapter_and_witness_tests.rs`
10. first local reduced-witness artifacts: `crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_reduction_manifest.json`, `crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_witness_bundle.json`, and `crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_lifecycle.json`
11. broadened reduced-witness artifacts: `crates/oxfml_core/tests/fixtures/witness_distillation/fec_accept_publication_reduction_manifest.json`, `crates/oxfml_core/tests/fixtures/witness_distillation/session_capability_denied_reduction_manifest.json`, `crates/oxfml_core/tests/fixtures/witness_distillation/execution_contract_host_query_reduction_manifest.json`, and `crates/oxfml_core/tests/replay_adapter_and_witness_tests.rs`
12. local normalized pack-candidate bundles: `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/fec_commit_pack_candidate_bundle.json`, `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/session_lifecycle_pack_candidate_bundle.json`, and `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/pack_candidate_index.json`
13. promotion-readiness planning artifacts: `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/promotion_candidate_families.json` and `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/promotion_readiness_index.json`

These are local witness artifacts, not yet promoted pack-grade corpus.
