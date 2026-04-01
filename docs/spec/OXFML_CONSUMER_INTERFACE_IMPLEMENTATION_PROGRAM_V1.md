# OxFml Consumer Interface Implementation Program V1

## 1. Purpose
This document turns the provisionally frozen consumer contract into a concrete
implementation program for `W054`.

It exists to answer:
1. what code structure should change,
2. in what order,
3. how the implementation should move decisively toward the new consumer
   contract,
4. how OxCalc, DNA OneCalc, and OxReplay should coordinate their uptake,
5. what evidence is required before each implementation wave is considered
   strong enough for downstream use.

This program must be read together with:
1. `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
2. `OXFML_CONSUMER_INTERFACE_REARCHITECTURE_PLAN.md`
3. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
4. `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
5. `OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
6. `formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`

## 2. Hard Constraints
The implementation program is allowed to change OxFml-owned consumer packaging.
It is not allowed to drift the frozen OxFml <-> OxFunc seam.

Hard rules:
1. do not rename or reinterpret frozen OxFml/OxFunc packet families,
2. do not shift built-in catalog truth, returned-value surface truth,
   registered-external catalog truth, or callable carrier truth away from their
   current owners,
3. do not hide provider-plus-pin runtime truth behind ambient mutable state,
4. do not let new consumer facades invent a second packet vocabulary when the
   canonical packet names already exist,
5. if a temporary transition helper is introduced, keep it narrow, internal in
   intent, and remove it once the facade-native path is strong enough.

## 3. Current Code Reality
The current user-visible Rust surface is now intentionally split between:
1. top-level consumer facades
2. canonical packet/capability substrate
3. a narrower advanced substrate escape hatch under `substrate::...`

Current packaging rule:
1. ordinary consumer code should use explicit `consumer::runtime`,
   `consumer::editor`, and `consumer::replay` module paths,
2. facade types are no longer intended to be discovered through flat crate-root
   re-export scanning,
3. `language_service` is now private editor substrate rather than a reachable
   public transition path,
4. remaining `substrate::...` access is temporary advanced transition surface
   and is hidden from ordinary documentation.

That surface already contains important building blocks for the rearchitecture:
1. provider-plus-pin runtime truth in `interface` and `host`
2. typed `LibraryContextSnapshotRef` carriage in `semantics`, `session`, and
   `language_service`
3. stable typed query bundle and returned-value surface packets in `interface`
4. host/runtime result truth in `host` and `seam`
5. replay-projection substrate through host replay capture and replay-oriented
   artifact families

The implementation program should therefore:
1. refactor around those realities,
2. avoid creating parallel abstractions that restate the same runtime truth,
3. lift the existing substrate into consumer-facing facades rather than letting
   the substrate continue to define the user model.

## 4. Target Module Architecture
The preferred long-term package tree is:
1. `consumer::runtime`
2. `consumer::editor`
3. `consumer::replay`
4. existing low-level substrate modules retained as advanced/internal entry
   points

### 4.1 New facade module intent
`consumer::runtime`
1. stable environment, request, result, and session facade
2. preferred entry for OxCalc and DNA OneCalc runtime execution

`consumer::editor`
1. stable editor environment, document, service, and interaction results
2. preferred entry for DNA OneCalc editing and later host/editor integrations

`consumer::replay`
1. stable replay projection request, service, and result
2. preferred entry for OxReplay and replay-aware hosts

### 4.2 Existing modules after the reset
These modules stay real, but change role:
1. `substrate::host`
   becomes runtime substrate, not the preferred consumer entry
2. `substrate::session`
   remains the lifecycle substrate backing the runtime facade
3. `language_service`
   remains private editor substrate backing the editor facade
4. `interface`
   remains the packet and provider substrate used by all three facades
5. `test_support::oxfunc_adapter`
   remains explicit integration-test support, not a mainstream consumer entry
6. `seam`, `semantics`, `binding`, `eval`, `syntax`
   remain canonical substrate and advanced-entry modules

## 5. Implementation Waves

### Wave 0: Refactor Stabilization Baseline
Purpose:
1. start from the already-landed `W041` and active `W043` substrate,
2. prevent the facade work from building on stale runtime assumptions.

Required baseline before Wave 1 is treated as strong:
1. provider-plus-pin is the only preferred runtime execution model in new code,
2. typed query bundle is the only preferred query-capability packet in new code,
3. historical root exports are documented as implementation substrate rather
   than endorsed consumer contract.

Expected local work:
1. finish current `W043` propagation where needed for runtime consumers,
2. identify all public entrypoints that currently assemble runtime context from
    loose fields rather than canonical packets,
3. classify those entrypoints as:
    - facade target,
    - temporary internal transition helper,
    - retained advanced surface.

### Wave 1: Runtime Facade
Purpose:
1. deliver the first consumer-facing facade used by both OxCalc and DNA
   OneCalc,
2. remove the largest current assembly burden without changing the frozen
   OxFml/OxFunc seam.

New boundary objects:
1. `RuntimeEnvironment`
2. `RuntimeFormulaRequest`
3. `RuntimeFormulaResult`
4. `RuntimeSessionFacade`

Primary code moves:
1. add `crates/oxfml_core/src/consumer/runtime/mod.rs`
2. define environment/request/result/session packets there
3. delegate execution through existing `host`, `session`, `interface`, and
   `seam` substrate
4. make `SingleFormulaHost` and session helpers call into the runtime facade
   where practical instead of carrying first-class consumer responsibility
5. make runtime environment construction builder-driven rather than exposing
   raw public library-context carrier types

Required invariants:
1. provider-plus-pin selection lives on `RuntimeEnvironment`
2. per-request volatile inputs live on `RuntimeFormulaRequest`
3. returned-value surface, candidate/commit/reject truth, and replay handles
   stay explicit on `RuntimeFormulaResult`
4. no runtime path silently falls back from pinned snapshot to ambient current
   snapshot

Internal transition work:
1. refactor current host entrypoints toward the runtime facade where that
   reduces duplicate assembly logic
2. do not let those entrypoints define the public architecture
3. document old-to-target mapping in both code docs and downstream notes

Evidence required:
1. deterministic tests proving one-shot runtime execution through the facade
2. deterministic tests proving repeated/session execution through the facade
3. pinned-vs-current snapshot evidence through the facade
4. typed query bundle and returned-value surface evidence through the facade
5. candidate/commit/reject correlation evidence through the facade
6. managed-session lifecycle evidence through the facade:
   - open,
   - execute,
   - commit,
   - abort/expire,
   - and one-shot execute-to-commit

Downstream coordination checkpoint:
1. OxCalc validates runtime/coordinator-facing result sufficiency
2. DNA OneCalc validates runtime entry ergonomics and migration from current
   direct host usage

### Wave 2: Editor Facade
Purpose:
1. unify OxFml editing, diagnostics, completion, signature help, and function
   help into one user-facing service family,
2. remove manual stitching pressure visible in DNA OneCalc.

New boundary objects:
1. `EditorEnvironment`
2. `EditorDocument`
3. `EditorEditService`
4. `EditorInteractionResult`

Primary code moves:
1. add `crates/oxfml_core/src/consumer/editor/mod.rs`
2. wrap current `language_service` substrate
3. make function-help payload lookup and completion request construction part of
   one coherent service model
4. keep carrier validation explicit and OxFml-owned where already in scope
5. make editor environment construction builder-driven rather than exposing
   raw provider/pin assembly helpers as public contract

Required invariants:
1. editor environment uses the same provider-plus-pin rule as runtime
2. editor documents are immutable or snapshot-oriented
3. intelligent completion remains non-canonical until normal edit-path re-entry
4. editor consumers do not have to assemble parse/bind/plan/help calls by hand

Internal transition work:
1. refactor current `language_service` packet assembly toward the editor facade
   where that reduces duplicate logic
2. document `language_service` as editor substrate, not as the intended consumer
   contract

Evidence required:
1. deterministic edit + diagnostics + completion + signature-help tests through
   the facade
2. function-help payload evidence at the facade boundary
3. pinned snapshot behavior through editor interactions
4. intelligent-completion context evidence at the facade boundary without
   consumer-side `language_service` packet assembly

Downstream coordination checkpoint:
1. DNA OneCalc validates edit/help/completion ergonomics
2. OxCalc confirms no coordinator-facing policy drift is implied by editor
   packaging

### Wave 3: Replay Facade
Purpose:
1. give OxReplay and replay-aware hosts a narrow projection surface,
2. stop treating broad artifact drilling as the normal replay integration path.

New boundary objects:
1. `ReplayProjectionRequest`
2. `ReplayProjectionService`
3. `ReplayProjectionResult`

Primary code moves:
1. add `crates/oxfml_core/src/consumer/replay/mod.rs`
2. wrap existing replay-oriented host/session/artifact projection helpers
3. standardize alias, pin, fence, registry, and lifecycle metadata publication
4. prefer named replay request constructors over raw field-by-field projection
   packet assembly

Required invariants:
1. replay projection is additive over OxFml meaning
2. alias and lifecycle metadata remain machine-readable
3. replay consumers do not need to infer pinning or alias information locally

Internal transition work:
1. refactor existing replay/helper projection assembly toward the replay facade
   where that reduces duplicate logic
2. document those helpers as replay substrate, not as the intended consumer
   contract

Evidence required:
1. deterministic replay projection tests from runtime results
2. deterministic replay projection tests from session results
3. metadata-preservation tests for alias, pin, fence, registry, and lifecycle
   fields
4. managed runtime-session replay projection evidence for open, execute,
   commit, and termination result families

Downstream coordination checkpoint:
1. OxReplay validates replay metadata sufficiency
2. DNA OneCalc validates retained-evidence projection path

### Wave 4: Public Packaging Tightening
Purpose:
1. make the facade-first architecture visible at the crate surface,
2. reduce future consumer technical debt without letting historical packaging
   remain the de facto public model.

Primary code moves:
1. make facade entrypoints the clearly documented crate entry surface
2. relocate historical direct-consumer modules under an explicit
   `substrate::...` namespace
3. remove flat crate-root facade discovery as the intended user model
4. demote historical root exports in docs and comments so they no longer read as
   the supported contract
5. remove temporary transition helpers once they are no longer materially
   useful to the refactor

Required invariants:
1. facade modules define the intended public model
2. docs distinguish:
    - preferred facade surface
    - advanced substrate surface
3. no behavior fork between facade entry and any temporary transition path

Evidence required:
1. crate-root smoke tests proving facade entrypoints are reachable
2. docs and examples point ordinary consumers at explicit `consumer::...`
   module paths rather than flat crate-root facade discovery
3. tests proving facade entrypoints carry the canonical runtime/editor/replay
   packet truth

Downstream coordination checkpoint:
1. OxCalc, DNA OneCalc, and OxReplay each have a documented first uptake path
2. downstream notes are updated after each landed wave

## 6. Refactoring Rules
The best end-state is not achieved by thin wrappers alone.
The refactor should intentionally simplify internal ownership boundaries.

Preferred internal rules:
1. stable semantic environment lives in environment objects, not in scattered
   helper arguments
2. volatile request inputs stay per-request
3. facade result objects carry canonical result truth rather than forcing
   consumers to combine multiple lower-level packets
4. repeated provider lookup logic should converge behind shared helpers or
   pinned runtime views instead of reappearing in each facade
5. low-level substrate remains testable directly
6. facade layers should be thin in semantics and thick in packaging clarity
7. runtime/editor facades should share one OxFml-owned provider-plus-pin
   environment model rather than each carrying raw provider/pin/snapshot triples
8. editor/language-service requests should prefer a single pinned-view object
   where lookup is the real concern instead of repeating low-level binding fields

## 7. Transition and Cleanup Policy
Temporary transition support is allowed only where it helps the refactor land
cleanly.

Rules:
1. no historical surface should be described as the intended consumer contract
2. when a current entrypoint has a facade replacement, its docs should point to
   the replacement
3. temporary transition helpers should stay narrow and should not gain a second
   public vocabulary
4. future removals or demotions should happen only after:
    - at least one real downstream uptake,
    - the facade-native path is stable,
    - behavior parity is tested

## 8. Test and Evidence Ladder For W054
Each wave must prove:
1. facade path works,
2. frozen OxFml/OxFunc seam semantics are preserved,
3. at least one downstream-relevant migration scenario is exercised locally.

Required evidence families:
1. direct unit/integration tests for each facade
2. downstream-shaped fixture tests where the consumer burden was the original
   design driver
3. docs and migration tables updated with the newly landed wave

## 9. Downstream Migration Program

### 9.1 DNA OneCalc
First uptake order:
1. runtime facade
2. editor facade
3. replay facade

Migration goals:
1. remove manual host/provider/query-bundle assembly
2. remove manual edit/help/completion stitching
3. move retained evidence projection onto replay service

### 9.2 OxCalc
First uptake order:
1. runtime facade
2. session facade usage through the runtime family
3. replay projection only where coordinator-facing capture truly needs it

Migration goals:
1. keep coordinator policy outside OxFml
2. preserve candidate/commit/reject and surfaced-fact truth
3. reduce direct proving-host-shaped integration pressure

### 9.3 OxReplay
First uptake order:
1. replay facade
2. metadata and alias publication through replay result packets

Migration goals:
1. replace local alias mapping and broad helper dependence
2. preserve pin, fence, registry, and lifecycle metadata explicitly

## 10. Complete Migration Inventory And Demotion Map
This section is the full implementation inventory for the refactor.

It answers:
1. which substrate surfaces still define consumer-relevant behavior,
2. which facade must absorb each one,
3. which surfaces are transitional only,
4. and which public substrate accesses should disappear after parity is reached.

### 10.1 Runtime Facade: Full Subsumption Plan
The runtime facade must fully own ordinary consumer behavior for:
1. one-shot formula execution,
2. repeated execution with reuse,
3. managed session open/execute/commit/abort/expire,
4. one-step execute-to-commit,
5. provider-plus-pin library-context selection,
6. typed query bundle carriage,
7. returned-value surface and candidate/commit/reject truth,
8. registered-external runtime mutation and execution,
9. replay-facing runtime result publication,
10. bounded runtime inspection needed by OxCalc and DNA OneCalc.

Current substrate session lifecycle surfaces and their target fate:
1. `SessionService::new`
   -> internal runtime service construction only
   -> final state: `pub(crate)` or private substrate helper
2. `SessionService::prepare`
   -> subsumed by `RuntimeSessionFacade::open_managed_session`
   -> final state: substrate-internal
3. `SessionService::open_session`
   -> subsumed by `RuntimeSessionFacade::open_managed_session`
   -> final state: substrate-internal
4. `SessionService::establish_capability_view`
   -> subsumed by runtime managed execution orchestration
   -> final state: substrate-internal
5. `SessionService::execute`
   -> subsumed by `RuntimeSessionFacade::execute_managed`
   -> final state: substrate-internal
6. `SessionService::commit`
   -> subsumed by `RuntimeSessionFacade::commit_managed`
      and `RuntimeSessionFacade::execute_and_commit_managed`
   -> final state: substrate-internal
7. `SessionService::abort_session`
   -> subsumed by `RuntimeSessionFacade::abort_managed`
   -> final state: substrate-internal
8. `SessionService::expire_session`
   -> subsumed by `RuntimeSessionFacade::expire_managed`
   -> final state: substrate-internal
9. `SessionService::session`
   -> subsumed by `RuntimeSessionFacade::managed_session_snapshot`
      and any later runtime inspection packet
   -> final state: substrate-internal
10. `SessionService::overlay_entries`
    -> if downstream consumers need this, expose a runtime inspection packet;
       otherwise keep it purely internal
    -> final state: substrate-internal
11. `SessionService::active_locus_claim_owner`
    -> if downstream consumers need this, expose a runtime contention/inspection
       packet; otherwise keep it internal
    -> final state: substrate-internal

Current public substrate session types and their target fate:
1. `PrepareRequest`
   -> internal orchestration packet only
   -> final state: substrate-internal
2. `PreparedSession`
   -> internal orchestration packet only
   -> final state: substrate-internal
3. `OpenSessionResult`
   -> replaced by `RuntimeManagedOpenResult`
   -> final state: substrate-internal
4. `ExecuteRequest`
   -> internal orchestration packet only
   -> final state: substrate-internal
5. `CapabilityViewSpec`
   -> internal runtime capability/orchestration packet
   -> final state: substrate-internal
6. `CapabilityView`
   -> internal runtime capability/orchestration packet
   -> final state: substrate-internal
7. `SessionRecord`
   -> replaced for ordinary consumers by `RuntimeManagedSessionSnapshot`
   -> retained only as internal runtime/replay substrate until replay parity
   -> final state: substrate-internal
8. `OverlayEntry`
   -> internal runtime/session bookkeeping
   -> final state: substrate-internal
9. `SessionPhase`
   -> ordinary consumers should rely on `RuntimeManagedSessionPhase`
   -> final state: substrate-internal or test-only
10. `SessionService`
    -> implementation substrate behind `RuntimeSessionFacade`
    -> final state: substrate-internal

Current substrate host surfaces and their target fate:
1. `SingleFormulaHost::new`
   -> subsumed by `RuntimeEnvironment` and `RuntimeSessionFacade`
   -> final state: substrate-internal
2. `SingleFormulaHost::set_formula_text`
   -> subsumed by `RuntimeFormulaRequest`
   -> final state: substrate-internal
3. `SingleFormulaHost::set_formula_source`
   -> subsumed by `RuntimeFormulaRequest`
   -> final state: substrate-internal
4. `SingleFormulaHost::set_formula_channel_kind`
   -> subsumed by `RuntimeFormulaRequest` source record
   -> final state: substrate-internal
5. `SingleFormulaHost::set_defined_name_value`
   -> subsumed by `RuntimeEnvironment::with_defined_names`
   -> final state: substrate-internal
6. `SingleFormulaHost::set_defined_name_reference`
   -> subsumed by `RuntimeEnvironment::with_defined_names`
   -> final state: substrate-internal
7. `SingleFormulaHost::set_defined_name_callable`
   -> subsumed by `RuntimeEnvironment::with_defined_names`
   -> final state: substrate-internal
8. `SingleFormulaHost::set_cell_value`
   -> subsumed by `RuntimeEnvironment::with_cell_values`
   -> final state: substrate-internal
9. `SingleFormulaHost::set_table_catalog`
   -> subsumed by `RuntimeEnvironment::with_table_context`
   -> final state: substrate-internal
10. `SingleFormulaHost::set_enclosing_table_ref`
    -> subsumed by `RuntimeEnvironment::with_table_context`
    -> final state: substrate-internal
11. `SingleFormulaHost::set_caller_table_region`
    -> subsumed by `RuntimeEnvironment::with_table_context`
    -> final state: substrate-internal
12. `SingleFormulaHost::recalc`
    -> subsumed by `RuntimeEnvironment::execute`
    -> final state: substrate-internal
13. `SingleFormulaHost::recalc_with_backend`
    -> subsumed by `RuntimeEnvironment::execute`
    -> final state: substrate-internal
14. `SingleFormulaHost::recalc_with_interfaces`
    -> subsumed by `RuntimeEnvironment::execute`
    -> final state: substrate-internal
15. `SingleFormulaHost::recalc_with_interfaces_and_snapshot_ref`
    -> subsumed by `RuntimeEnvironment::execute`
    -> final state: substrate-internal
16. `SingleFormulaHost::recalc_with_library_context_view`
    -> subsumed by `RuntimeEnvironment::execute`
    -> final state: substrate-internal
17. `SingleFormulaHost::recalc_with_observed_fence_override`
    -> retain only as internal test/probe helper until runtime replay/testing
       no longer needs it
    -> final state: internal test support
18. `SingleFormulaHost::run_empirical_oracle_scenario`
    -> retain only as internal test/probe helper
    -> final state: internal test support
19. `SingleFormulaHost::recalc_with_rtd_provider`
    -> subsumed by typed query bundle on `RuntimeFormulaRequest`
    -> final state: substrate-internal
20. `SingleFormulaHost::recalc_with_registered_external_provider`
    -> subsumed by typed query bundle on `RuntimeFormulaRequest`
    -> final state: substrate-internal
21. `SingleFormulaHost::apply_registered_external_catalog_mutation`
    -> move behind a runtime-owned external catalog mutation surface
    -> final state: substrate-internal after runtime mutation parity

Current public substrate host packets and their target fate:
1. `HostRecalcOutput`
   -> ordinary consumers should use `RuntimeFormulaResult`
   -> retained only as internal host/runtime substrate until replay and tests
      stop depending on it
   -> final state: substrate-internal
2. `FirstHostReplayCapturePacket`
   -> ordinary consumers should use `ReplayProjectionResult`
   -> retained only as replay substrate while first-host capture tests migrate
   -> final state: substrate-internal
3. `ArtifactReuseReport`
   -> may stay embedded on `RuntimeFormulaResult`, but ordinary consumers should
      not import it through `substrate::host`
   -> final state: re-home under runtime-facing result metadata or internalize
4. `EmpiricalOracleScenario`
   -> test/support only
   -> final state: internal test support

Current runtime parity floor:
1. `consumer::runtime` now owns ordinary one-shot execution, repeated execution
   with reuse, managed open/execute/commit/abort/expire, one-step
   execute-to-commit, provider-plus-pin library-context selection, typed query
   bundle carriage, returned-value surface and candidate/commit/reject truth,
   registered-external runtime mutation, registered-external runtime execution,
   `RTD` runtime execution, and bounded managed-session diagnostics.
2. ordinary consumer-facing evidence now exists through
   `crates/oxfml_core/tests/runtime_consumer_facade_tests.rs` for:
   - pinned and inline library-context execution,
   - unresolved snapshot rejection,
   - repeated-execution reuse,
   - managed open/execute/commit/abort and execute-to-commit,
   - caller-context carriage,
   - `RTD` execution through typed query bundle,
   - registered-external execution through typed query bundle,
   - runtime-owned registration/catalog mutation,
   - bounded managed diagnostics for overlay and locus-claim state.
3. direct `host` and `session` tests still remain for advanced substrate,
   replay, seam, and proving-host detail, but they no longer define ordinary
   consumer-facing runtime parity.

Remaining runtime work after `R1`:
1. keep `host` and `session` available only as advanced substrate until the
   later public-packaging and replay closure work lands
2. complete the later Wave 4 removal/demotion cut once replay parity and final
   substrate cleanup are strong enough

### 10.2 Editor Facade: Full Subsumption Plan
The editor facade must fully own ordinary consumer behavior for:
1. document opening,
2. incremental edit application,
3. diagnostics,
4. completion proposal production,
5. completion validation and application,
6. signature help,
7. function-help lookup and payload publication,
8. intelligent completion context,
9. pinned library-context lookup for all editor interactions.

Current substrate language-service payload types and their target fate:
1. `EditorTriviaKind`
   -> keep only as editor payload if still needed at the consumer boundary
   -> demote direct `substrate::language_service` access
2. `EditorTrivia`
   -> same rule as `EditorTriviaKind`
3. `EditorToken`
   -> same rule as `EditorTriviaKind`
4. `EditorSyntaxSnapshot`
   -> keep as `consumer::editor` payload, not a substrate import
5. `LiveDiagnosticSeverity`
   -> keep as `consumer::editor` payload, not a substrate import
6. `LiveDiagnosticStage`
   -> keep as `consumer::editor` payload, not a substrate import
7. `LiveDiagnostic`
   -> keep as `consumer::editor` payload, not a substrate import
8. `LiveDiagnosticSnapshot`
   -> keep as `consumer::editor` payload, not a substrate import
9. `FormulaTextChangeRange`
   -> keep only if the editor facade still exposes edit-delta detail directly;
      otherwise internalize
10. `CompletionProposalKind`
    -> keep as `consumer::editor` payload, not a substrate import
11. `CompletionProposal`
    -> keep as `consumer::editor` payload, not a substrate import
12. `CompletionResult`
    -> keep as `consumer::editor` payload, not a substrate import
13. `SignatureHelpContext`
    -> keep as `consumer::editor` payload, not a substrate import
14. `FunctionHelpSignatureForm`
    -> keep as `consumer::editor` payload, not a substrate import
15. `FunctionHelpPacket`
    -> keep as `consumer::editor` payload, not a substrate import
16. `IntelligentCompletionContext`
    -> keep as `consumer::editor` payload, not a substrate import

Current substrate language-service orchestration types and their target fate:
1. `EditorPlanOptions`
   -> keep as facade-owned editor plan/config options
   -> final state: substrate-internal
2. `EditorAnalysisStage`
   -> keep as facade-owned editor analysis depth or interaction policy
   -> final state: substrate-internal
3. `FormulaEditRequest`
   -> substrate-internal
4. `FormulaEditResult`
   -> substrate-internal
5. `FormulaEditReuseSummary`
   -> substrate-internal or re-homed under editor debug metadata
6. `CompletionValidationRequest`
   -> substrate-internal
7. `CompletionValidationResult`
   -> substrate-internal
8. `CompletionRequest`
   -> substrate-internal
9. deterministic function-help lookup packet
   -> substrate-internal; ordinary consumers should receive `FunctionHelpPacket`

Current substrate language-service functions and their target fate:
1. `build_function_help_lookup_request`
   -> subsumed by `EditorEditService`
   -> final state: substrate-internal
2. `build_editor_syntax_snapshot`
   -> subsumed by `EditorEditService`
   -> final state: substrate-internal
3. `apply_formula_edit`
   -> subsumed by `EditorEditService::apply_edit`
   -> final state: substrate-internal
4. `apply_completion_proposal`
   -> subsumed by `EditorEditService::apply_completion_proposal`
   -> final state: substrate-internal
5. `validate_completion_candidate`
   -> subsumed by `EditorEditService::validate_completion`
   -> final state: substrate-internal
6. `build_live_diagnostics`
   -> subsumed by `EditorEditService`
   -> final state: substrate-internal
7. `collect_completion_proposals`
   -> subsumed by `EditorEditService::completion_at_cursor`
   -> final state: substrate-internal
8. `build_intelligent_completion_context`
   -> subsumed by `EditorEditService::intelligent_completion_context_at_cursor`
   -> final state: substrate-internal
9. `signature_help_context_at_cursor`
   -> subsumed by `EditorEditService::signature_help_at_cursor`
   -> final state: substrate-internal

Additional editor work still required before full subsumption:
1. replace remaining substrate-owned configuration packets with facade-owned
   editor options
2. decide which editor payload types remain visible as canonical facade result
   payloads and re-home them under `consumer::editor`
3. migrate ordinary consumer-shaped tests away from `substrate::language_service`
4. once parity is reached, keep `language_service` as private implementation
   substrate only

### 10.3 Replay Facade: Full Subsumption Plan
The replay facade must fully own ordinary consumer behavior for:
1. projection from one-shot runtime results,
2. projection from managed runtime-session lifecycle results,
3. fixture-family projection,
4. retained-witness projection,
5. alias, pin, fence, registry, and lifecycle metadata publication,
6. bounded replay-facing capture for downstream hosts.

Current replay parity floor:
1. ordinary replay consumers now project:
   - `RuntimeFormulaResult`
   - managed runtime-session open / execute / commit / termination packets
   - managed runtime-session snapshots
   - bounded first-host capture packets
   - fixture-family metadata
   - retained-witness metadata
2. `ReplayProjectionRequest::host_result` and
   `ReplayProjectionRequest::session_record` are no longer part of the public
   replay contract.
3. bounded host capture now flows through `ReplayFirstHostCaptureSource`
   instead of raw `HostRecalcOutput`.

Current substrate replay/adapter surfaces and their target fate:
1. `HostRecalcOutput::to_first_host_replay_capture_packet`
   -> replay-internal packet builder only
   -> final state: substrate-internal
2. `SessionRecord`-based replay projection
   -> no longer part of the public replay contract
   -> final state: substrate-internal
3. `OxFuncAdapterRequest`
   -> not part of the consumer contract
   -> final state: explicit `test_support` module only
4. `OxFuncPreparationArtifact`
   -> not part of the consumer contract
   -> final state: explicit `test_support` module only
5. `OxFuncEvaluationArtifact`
   -> not part of the consumer contract
   -> final state: explicit `test_support` module only
6. `OxFuncMismatchOwnerGuess`
   -> not part of the consumer contract
   -> final state: explicit `test_support` module only
7. `OxFuncMismatchArtifact`
   -> not part of the consumer contract
   -> final state: explicit `test_support` module only
8. `OxFuncAdapterRun`
   -> not part of the consumer contract
   -> final state: explicit `test_support` module only
9. `run_oxfunc_preparation_adapter`
   -> not part of the consumer contract
   -> final state: explicit `test_support` module only

Remaining replay work after `P1`:
1. keep first-host packet construction and other replay substrate helpers as
   advanced internal/test-support implementation while wider replay-family
   breadth lands
2. complete broader registry/lifecycle/promotion breadth beyond the current
   replay floor
3. finish the later Wave 4 cleanup once replay parity and final substrate
   removal are strong enough

### 10.4 Full Public Substrate Demotion Map
Final public-packaging target:
1. ordinary consumers use only:
   - `consumer::runtime`
   - `consumer::editor`
   - `consumer::replay`
2. canonical frozen seam packets remain available through their existing
   canonical modules and root exports where appropriate
3. `substrate::...` disappears from the endorsed public contract and is
   reduced to internal or test-support-only use

Every currently reachable substrate family to demote:
1. `substrate::host::*`
   -> demote fully after runtime parity
2. `substrate::session::*`
   -> demote fully after runtime parity
3. private `language_service` implementation helpers
   -> keep private after editor parity
4. `test_support::oxfunc_adapter::*`
   -> keep only as explicit test/support surface after replay/test parity

Demotion classes:
1. `internalize`
   -> move to `pub(crate)` or private implementation
2. `re-home`
   -> keep payload meaning, but expose it through the facade module instead of
      `substrate::...`
3. `test_support_only`
   -> retain only in test/support code, not in the public library contract

### 10.5 Complete Execution Order
Phase A: Runtime parity and session/host collapse
1. finish runtime-owned session lifecycle coverage
2. add runtime-owned external catalog mutation surface
3. replace ordinary host/session-oriented consumer tests with runtime facade use
4. internalize `host` and `session` once parity is reached

Phase B: Editor parity and language-service collapse
1. replace substrate-owned editor configuration packets with facade-owned ones
2. re-home editor payload types under `consumer::editor`
3. internalize `language_service` orchestration functions and request/result
   packets
4. keep `language_service` private once parity is reached

Phase C: Replay parity and adapter collapse
1. keep ordinary consumer replay projection on runtime-first and bounded
   first-host-capture sources only
2. keep OxFunc adapter artifacts in explicit test-support
3. internalize replay substrate helpers further once broader replay parity is
   reached

Phase D: Final packaging cut
1. remove public `substrate::...` access entirely
2. keep only `consumer::runtime`, `consumer::editor`, `consumer::replay` as
   ordinary public entry families
3. update all downstream notes and docs to match that final public surface

## 11. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - first runtime, editor, and replay facade modules now exist, but each wave
    is still only partial relative to the full W054 target
  - some temporary transition helpers still exist in code, but they should not
    define the public architecture and still need later cleanup/removal
  - `W043` provider-plus-pin normalization still needs to stay aligned with
    the remaining runtime, editor, and replay facade work
  - Epic E1 editor parity is now at its planned floor:
    editor payload vocabulary now lives under `consumer::editor`, facade-owned
    `EditorPlanOptions` and `EditorAnalysisStage` replace the last
    substrate-shaped public editor config, ordinary editor-facing tests now
    exercise the facade contract, public `substrate::language_service` access
    has been removed, and `language_service` now acts as private editor
    substrate
  - replay projection now carries retained-witness, fixture-family, bounded
    first-host-capture, and managed-session lifecycle metadata, while raw
    host-result and session-record constructors are no longer part of the
    public replay contract
  - full-suite validation is green again after the E1 pass
