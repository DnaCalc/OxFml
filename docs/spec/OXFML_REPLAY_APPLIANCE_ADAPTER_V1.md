# OxFml Replay Appliance Adapter V1

## 1. Purpose
This document defines the OxFml-local adapter contract for the Foundation Replay appliance rollout.

It adapts the Foundation `DNA ReCalc` replay governance model into the OxFml canonical spec set without transferring OxFml semantic ownership to Foundation or to generic replay tooling.

Read together with:
1. `OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
2. `OXFML_CANONICAL_ARTIFACT_SHAPES.md`
3. `OXFML_MINIMUM_SEAM_SCHEMAS.md`
4. `OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`
5. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`
6. `fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`
7. `fec-f3e/FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`
8. `fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`

## 2. Scope and Non-Goals
### 2.1 Scope
This adapter specification covers:
1. projection from OxFml artifact families into replay bundle objects,
2. replay-preserved identity and fence rules,
3. fixture-family import into `DNA ReCalc` bundle workflows,
4. normalized event-family mapping,
5. adapter capability claims and limits,
6. registry version pins and witness lifecycle usage,
7. rollout rules for local witness evidence versus future retained and promoted witnesses.

### 2.2 Non-goals
This pass does not:
1. replace OxFml-owned formula semantics, evaluator facts, reject meanings, or trace kinds,
2. authorize formula-text rewrites,
3. authorize bind-payload rewrites,
4. authorize fence-tuple rewrites,
5. authorize capability-view rewrites,
6. claim `cap.C4.distill_valid`,
7. claim `cap.C5.pack_valid`,
8. define a new OxFml-local scenario DSL.

## 3. Authority Split and Explicit Conflict Handling
The authority split is:
1. OxFml owns formula-language semantics, evaluator and seam artifact meanings, canonical identity categories, fence rules, typed reject semantics, and typed effect semantics.
2. Foundation owns replay rollout governance for normalized bundle transport, registry ids, capability-level governance, witness lifecycle states, and cross-lane replay tooling policy.
3. `DNA ReCalc` may normalize transport, correlation, comparison, and lifecycle metadata, but it may not redefine OxFml artifact meaning.

Explicit conflict rule:
1. if Foundation generic replay wording would flatten a typed OxFml artifact into a generic event label, OxFml semantics win and the replay adapter must preserve the OxFml source kind plus a normalized family mapping,
2. if Foundation distillation policy would permit generic rewrites, OxFml currently constrains replay-safe transforms to subset and projection transforms only,
3. if Foundation bundle policy implies one generic id family, OxFml preserved identity categories remain distinct inside the normalized bundle.

## 4. Bundle Projection Rules For The OxFml Artifact Ladder
The adapter projects the OxFml artifact ladder as follows:

1. `FormulaSourceRecord`
   - projected as scenario input identity and source-artifact references,
   - large textual bodies may remain sidecar-backed.
2. `GreenTreeRoot`
   - projected by source-artifact reference or sidecar,
   - never required inline at every lifecycle event.
3. `BoundFormula`
   - projected by source-artifact reference, bind identity fields, and optional sidecar.
4. `SemanticPlan`
   - projected by plan identity, execution-profile summary, helper-environment profile, and optional sidecar.
5. `PreparedCall`
   - projected as event payload or sidecar-backed call packet at prepare/execute boundaries.
6. `PreparedResult`
   - projected as candidate-facing result payload or sidecar-backed call-result packet.
7. evaluator facts
   - projected either inline in normalized event payloads or by typed fact refs,
   - never flattened into prose-only diagnostics.
8. `AcceptedCandidateResult`
   - projected to normalized `candidate.*` families and candidate view material.
9. `CommitBundle`
   - projected to normalized `publication.*` families and published-view material.
10. `RejectRecord`
   - projected to normalized `reject.*` families and reject-set material.

Projection rules:
1. the normalized replay model is additive transport, not replacement semantics,
2. source schema ids and source artifact refs remain mandatory,
3. if a payload body is too large for inline replay transport, the adapter must preserve a sidecar ref plus content fingerprint,
4. explicit missing or opaque markers must be used where a normalized field cannot be populated honestly.

## 5. Preserved Identity Categories And Fence-Related Keys
The adapter must preserve the OxFml identity categories from `OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`:
1. stable ids,
2. version keys,
3. content fingerprints,
4. runtime handles,
5. fence tuple members.

The minimum replay-preserved identity and fence set is:
1. `formula_stable_id`,
2. `formula_text_version` where the source fixture or host surface distinguishes it,
3. `formula_token`,
4. `green_tree_key` and/or `green_tree_fingerprint` where syntax bodies are projected,
5. `structure_context_version`,
6. `bind_hash`,
7. `semantic_plan_key` and optional `semantic_plan_fingerprint`,
8. `snapshot_epoch`,
9. `profile_version`,
10. `capability_view_key` where present,
11. `session_id`,
12. `commit_attempt_id`,
13. `commit_bundle_fingerprint` where present,
14. `reject_record_fingerprint` where present.

Replay rule:
1. runtime handles may appear only as auxiliary correlation metadata,
2. runtime handles may not become the only replay identity,
3. configuration and capture-mode context may be additive replay metadata,
4. additive replay metadata may not rewrite or substitute for OxFml fence meaning.

## 6. Fixture-Family Import Rules
The adapter imports current OxFml fixture families as first-class replay sources.

Current import mapping:
1. `parse_bind_cases.json`
   - source family: parse/bind witness
   - replay role: source artifact and schema witness import
2. `semantic_plan_replay_cases.json`
   - source family: semantic-plan witness
   - replay role: semantic plan, helper profile, and execution-profile import
3. `prepared_call_replay_cases.json`
   - source family: prepared-call/result witness
   - replay role: prepare/execute call packet import
4. `fec_commit_replay_cases.json`
   - source family: candidate/commit/reject witness
   - replay role: transaction-boundary scenario import
5. `execution_contract_replay_cases.json`
   - source family: execution-profile witness
   - replay role: scheduler-facing effect and restriction import
6. `session_lifecycle_replay_cases.json`
   - source family: session lifecycle witness
   - replay role: lifecycle phase and reject-path import
7. `single_formula_host_replay_cases.json`
   - source family: proving-host witness
   - replay role: host-level scenario import
8. `empirical_oracle_scenarios.json`
   - source family: empirical-oracle witness
   - replay role: oracle comparison and explain import

Import rules:
1. source fixture family names remain preserved,
2. scenario ids must remain stable across repeated normalization of the same fixture case,
3. import may add bundle envelope, registry bindings, and lifecycle metadata,
4. import may not rewrite formula text, bind payloads, fence tuples, or capability views in this pass.

## 7. Normalized Event-Family Mapping
The normalized family mapping for OxFml is:

1. session boundaries
   - `PrepareStarted`, `PrepareRejected`, `SessionOpened`, `CapabilityViewResolved`, `ExecuteStarted`, `ExecuteCompleted`
   - normalized families: `session.*`
2. candidate boundaries
   - `AcceptedCandidateResultBuilt`
   - normalized families: `candidate.*`
3. commit/publication boundaries
   - `CommitStarted`, `CommitAccepted`
   - normalized families: `publication.*`
4. reject boundaries
   - `RejectIssued`, `FenceMismatchRejected`, `CapabilityDeniedRejected`, `SessionExpiredRejected`
   - normalized families: `reject.*`
5. effect boundaries
   - `DynamicReferenceDiscovered`, `SpillEventObserved`, `FormatDependencyObserved`, `OverlayRegistered`, `OverlayEvicted`
   - normalized families: `dependency.*`, `spill.*`, `host_query.*`, or `overlay.*` as appropriate

Mapping rules:
1. OxFml source event kinds remain authoritative and must be preserved in bundle payloads,
2. normalized families are used for cross-lane diff and explain indexing only,
3. candidate-versus-publication distinction is mandatory and may not be collapsed into one result family,
4. reject-is-no-publish semantics remain OxFml-owned and must survive normalization.

## 8. Adapter Capability Target And Known Limits
The OxFml target for this rollout is:
1. claim `cap.C0.ingest_valid`,
2. claim `cap.C1.replay_valid`,
3. claim `cap.C2.diff_valid`,
4. claim `cap.C3.explain_valid`,
5. scaffold but do not claim `cap.C4.distill_valid`,
6. do not claim `cap.C5.pack_valid`.

Known limits for this pass:
1. no replay-safe formula-text rewrite family is declared,
2. no replay-safe bind rewrite family is declared,
3. no replay-safe fence rewrite family is declared,
4. no replay-safe capability-view rewrite family is declared,
5. subsystem schema merge strategy versus one unified replay trace schema remains open,
6. current source schema ids for adapter import are still OxFml-local identifiers rather than published machine-readable schema ids,
7. witness distillation is now locally evidenced for a narrow retained-local floor, but not yet at pack-grade breadth,
8. normalized pack-candidate bundle evidence is local-only and non-pack-eligible in this pass.

## 9. Registry Version Pins
Until Foundation publishes machine-readable registry snapshots, OxFml pins the replay governance families to the authoritative Foundation handoff package:
1. local pin name: `oxfml.local.registry_pin.foundation_handoff_20260315_pass01`
2. source root: `..\\Foundation\\research\\runs\\20260315-215019-replay-appliance-authoritative-pass-01\\outputs`

Pinned registry families for this pass:
1. `predicate_kind`
2. `mismatch_kind`
3. `severity_class`
4. `reduction_status`
5. `witness_lifecycle_state`
6. `capability_level`

Pinning rule:
1. registry entry ids come from the Foundation handoff vocabulary,
2. OxFml may add local-only auxiliary ids for reduction-unit anchors or source schema ids,
3. any local-only auxiliary id must carry the `oxfml.local.*` prefix and must not be confused with Foundation registry ids.

## 10. Witness Lifecycle And Quarantine Usage Rules
OxFml adopts the Foundation witness lifecycle and quarantine families as rollout governance, not as semantic truth.

Rules:
1. local replay bundles and replay fixtures may be normalized and replayed without immediately becoming retained witnesses,
2. explanatory-only and quarantined witnesses are not pack-eligible,
3. quarantined witnesses remain indexable and explain-addressable,
4. retained or promoted witness claims require explicit lifecycle refs,
5. witness lifecycle state never changes the meaning of OxFml candidate, commit, or reject artifacts,
6. lifecycle state only governs retention, promotion, quarantine, and GC policy.

Current expected lifecycle use in OxFml:
1. current local fixtures are local witness evidence,
2. future reduced witnesses from `W010` begin at `wit.generated_local`,
3. explanatory-only reductions should use `wit.explanatory_only`,
4. quarantine reasons should use Foundation families such as `oracle_unstable`, `capture_insufficient`, `schema_incompatible`, or `replay_invalid`.

Current local extension:
1. reduced witnesses broadened after `W010` may move directly to `wit.retained_local` when replay-valid closure is exercised and no quarantine reason applies,
2. normalized pack-candidate bundles remain local-only evidence and are not themselves witness lifecycle promotions,
3. the current retained-local floor now spans FEC commit/reject, session rejection, execution contract, single-formula host, and empirical-oracle families.

## 11. Open Alignment Items
The current OxFml replay rollout still carries these alignment items:
1. `capability_view_key` is checked today but remains open as a first-class fence tuple member,
2. subsystem schema merge strategy versus one unified replay trace schema remains open,
3. some code and minimum-schema surfaces still need closure around fields such as `reject_record_id` and `fence_snapshot_ref`,
4. `BindMismatchContext` still needs tighter exercised closure between prose, code, and replay normalization,
5. helper-form and scalarization provenance continue to narrow with the OxFunc boundary,
6. current reduced-witness breadth is still narrow and local,
7. normalized pack-candidate bundle evidence exists only as local rehearsal and remains intentionally non-pack-eligible,
8. current retained-witness set breadth is stronger than the first rehearsal floor but still not broad enough for a `cap.C4.distill_valid` claim,
9. DNA OneCalc host-policy and empirical-pack planning are now explicit, but remain planning-only and non-pack-grade.

## 12. Working Rule
Use this adapter document as the OxFml-local replay rollout authority for:
1. bundle projection over typed OxFml artifacts,
2. conservative capability claims,
3. registry and lifecycle pinning for current replay governance,
4. witness rollout planning into `W009` and `W010`.

Do not use this document to weaken OxFml-owned semantic meaning or to authorize generic replay rewrites that OxFml has not declared replay-safe.
