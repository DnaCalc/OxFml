# OxFml Formal Artifact Register

## 1. Purpose
This document maps the most important OxFml clause families to their first intended formal and evidence artifacts.

It is the planning register for:
1. Lean-oriented ADTs and invariants,
2. TLA+-oriented state machines and safety properties,
3. replay/schema witness families,
4. conformance-pack alignment.

It is primarily a planning register, but the first checked local formal artifacts now exist at:
1. `../../formal/lean/OxFmlSessionLifecycle.lean`
2. `../../formal/lean/OxFmlExternalReferenceDeferred.lean`
3. `../../formal/tla/FecSessionLifecycle.tla`
4. `../../formal/tla/FecSessionLifecycle.cfg`
5. `../../formal/tla/FecExternalCapabilityGate.tla`
6. `../../formal/tla/FecExternalCapabilityGate.cfg`
7. `../../formal/run_formal.ps1`

Read together with:
1. `OXFML_FORMALIZATION_AND_VERIFICATION.md`
2. `fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
3. `fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`
4. `OXFML_MINIMUM_SEAM_SCHEMAS.md`

## 2. Register Table
| clause_family | first Lean artifact target | first TLA+ artifact target | first replay/fixture target | first pack family |
|---|---|---|---|---|
| Green-tree fidelity | syntax ADTs for full-fidelity tokens, trivia, and recovery nodes | none initially | parse round-trip fixtures | `PACK.oxfml.parse.full_fidelity` |
| Red reconstruction | reconstruction invariants from green tree plus context | none initially | contextual projection fixtures | `PACK.oxfml.parse.full_fidelity` |
| Bind normalization | normalized reference ADTs and unresolved-reference classifications | none initially | bind snapshot fixtures | `PACK.oxfml.bind.reference_normalization` |
| Prepared-call structure | prepared-arg/result ADTs and invariants | none initially | prepared-call contract fixtures | `PACK.oxfml.oxfunc.prepared_contract` |
| Host-query boundary | capability-view and host-query ADTs | capability-denial paths in evaluator lifecycle model | host-query schema fixtures | `PACK.oxfml.oxfunc.prepared_contract` |
| Candidate/publication split | accepted-candidate and commit-bundle relation invariants | publish/no-publish state split | candidate-vs-commit replay fixtures | `PACK.fec.transaction_boundary` |
| Reject taxonomy | typed reject-context ADTs and no-publish theorem surface | reject transition safety | reject-context schema fixtures | `PACK.fec.reject_detail_replay` |
| Minimum payload schemas | payload-field sufficiency invariants | payload sufficiency for commit/reject transitions | schema-validation fixtures | `PACK.fec.minimum_payload_schemas` |
| Session lifecycle | session-state ADTs | lifecycle state machine | phase trace fixtures | `PACK.fec.transaction_boundary` |
| Fence soundness | fence tuple and mismatch classifications | stale/incompatible publish exclusion | fence-mismatch replay fixtures | `PACK.fec.reject_detail_replay` |
| Spill semantics | spill-event payload typing | spill interactions under retries and contention | spill event fixtures | `PACK.fec.overlay_lifecycle` |
| Overlay reuse and eviction | overlay token/fact ADTs | epoch-safe overlay retention and eviction | overlay replay fixtures | `PACK.fec.overlay_lifecycle` |

## 3. First Lean Priorities
The first Lean-owned shapes should be:
1. syntax tree ADTs,
2. normalized reference ADTs,
3. prepared-call/result ADTs,
4. accepted candidate, commit bundle, and reject context ADTs.

The first Lean-oriented invariant families should be:
1. green-tree full-fidelity preservation,
2. bind output classification soundness,
3. no-publish-on-reject,
4. required field-family preservation for minimum seam schemas.

## 4. First TLA+ Priorities
The first TLA+-owned models should be:
1. sequential session lifecycle,
2. candidate/publication split,
3. fence mismatch rejection,
4. session expiry or abort,
5. later extension to concurrent session contention.

## 5. Current Open Planning Gaps
The following planning gaps remain explicit:
1. exact replay artifact promotion path from local crate fixtures to pack-grade corpus,
2. whether some OxFml formal artifacts live locally versus in shared Green-owned repos,
3. when the current local checked Lean/TLA+ artifacts should be promoted into Green-owned proof/model locations.

## 5C. Current Executed Ownership Floor
The current executed ownership floor is:
1. `W016` for the first checked local Lean/TLA+ execution path,
2. `W022` for external-capability and broader clause-family checked artifacts,
3. `W023` for replay-promotion evidence that the formal lane can point at honestly.

## 5B. Current Local Formal Artifact Paths
The current local formal artifact floor is:
1. Lean session lifecycle and no-publish artifact: `formal/lean/OxFmlSessionLifecycle.lean`
2. Lean external-reference deferment artifact: `formal/lean/OxFmlExternalReferenceDeferred.lean`
3. TLA+ session lifecycle and publish-safety model: `formal/tla/FecSessionLifecycle.tla`
4. TLA+ model configuration: `formal/tla/FecSessionLifecycle.cfg`
5. TLA+ external capability gate model: `formal/tla/FecExternalCapabilityGate.tla`
6. TLA+ external capability gate configuration: `formal/tla/FecExternalCapabilityGate.cfg`
7. Local formal runner: `formal/run_formal.ps1`

These are checked local artifacts.
They are not yet Green-owned canonical formal artifacts, and broader clause families still remain open.

## 5A. Current Authored Local Witness Paths
The current local witness floor is:
1. parse/bind fixture path: `crates/oxfml_core/tests/fixtures/parse_bind_cases.json`
2. semantic-plan replay path: `crates/oxfml_core/tests/fixtures/semantic_plan_replay_cases.json`
3. FEC commit/reject replay path: `crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json`
4. prepared-call/result replay path: `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
5. execution-contract replay path: `crates/oxfml_core/tests/fixtures/execution_contract_replay_cases.json`
6. session lifecycle replay path: `crates/oxfml_core/tests/fixtures/session_lifecycle_replay_cases.json`
7. single-formula host replay path: `crates/oxfml_core/tests/fixtures/single_formula_host_replay_cases.json`
8. empirical-oracle scenario path: `crates/oxfml_core/tests/fixtures/empirical_oracle_scenarios.json`

These are implementation-local witnesses, not formal artifacts and not yet full pack-grade replay bundles.
The current retained-local witness index is:
1. `crates/oxfml_core/tests/fixtures/witness_distillation/retained_witness_set_index.json`

The current planning-only host and empirical-pack artifacts are:
1. `crates/oxfml_core/tests/fixtures/empirical_pack_planning/dna_onecalc_host_policy_profiles.json`
2. `crates/oxfml_core/tests/fixtures/empirical_pack_planning/empirical_pack_candidate_groups.json`

## 6. Working Rule
When a new important OxFml clause is promoted:
1. add it to this register,
2. identify at least one first formal or replay witness family,
3. keep the gap explicit until the witness exists.
