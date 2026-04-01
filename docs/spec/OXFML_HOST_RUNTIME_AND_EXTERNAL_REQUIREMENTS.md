# OxFml Host Runtime and External Requirements

## 1. Purpose and Status
This document defines the current OxFml-owned host/runtime and external-interface requirements for implementation of a host that drives the OxFml and OxFunc combination.

It exists to unify three surfaces that were previously documented separately:
1. the code-facing transform and service sketch,
2. the reduced-profile direct-host baseline,
3. the coordinator-facing seam with OxCalc.

Status:
1. canonical OxFml draft for host/runtime requirements,
2. implementation-facing for the currently covered local scope,
3. reviewed by OxCalc as sufficient for first implementation planning on the current covered slice, with the current host/runtime note round now converged on that first-slice reading,
4. not yet promoted as shared seam-freeze text,
5. intended as the bounded coordination packet for the current OxCalc round and any later mismatch-driven refinement.

Read together with:
1. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
2. `OXFML_CONSUMER_INTERFACE_REARCHITECTURE_PLAN.md`
3. `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
4. `OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
5. `OXFML_CANONICAL_ARTIFACT_SHAPES.md`
6. `OXFML_MINIMUM_SEAM_SCHEMAS.md`
7. `OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`
8. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`
9. `formula-language/OXFML_OXFUNC_LIBRARY_CONTEXT_RUNTIME_INTERFACE.md`
10. `formula-language/OXFML_R1C1_FORMULA_CHANNEL.md`
11. `formula-language/OXFML_CF_DV_RESTRICTED_SUBLANGUAGES.md`
12. `formula-language/OXFML_STRUCTURED_REFERENCE_AND_TABLE_BOUNDARY.md`
13. `formula-language/OXFML_NAME_WORLD_AND_RUNTIME_REGISTRATION_INVALIDATION.md`

## 2. Authority Boundary
OxFml remains authoritative for:
1. formula grammar, parse, bind, and semantic-plan meaning,
2. evaluator-owned candidate, commit, reject, trace, and replay-safe artifact meaning,
3. typed capability, fence, effect, and reject semantics,
4. runtime library-context snapshot correlation where evaluator semantics depend on catalog truth.

OxFunc remains authoritative for:
1. built-in function semantic truth,
2. value-type universe and worksheet-error payload meaning,
3. runtime library-context catalog truth and snapshot generation.

Hosts consuming the OxFml and OxFunc combination may own:
1. host-supplied bindings and provider implementations,
2. recalc trigger policy,
3. scheduler and publication policy where that is outside OxFml evaluator meaning,
4. process and packaging concerns.

Hosts must not:
1. redefine candidate, commit, reject, fence, or capability semantics,
2. collapse accepted candidate into committed publication,
3. replace typed evaluator/runtime outcomes with generic host failures,
4. hide snapshot drift or host/provider truth behind opaque mutable globals.

Current consumer-surface direction:
1. hosts should use the runtime facade family defined in `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`,
2. this document remains the semantic host/runtime contract,
3. the current ordinary Rust-facing host surface is the landed `consumer::runtime` family,
4. consumer-surface packaging work must compose over the frozen OxFml/OxFunc shared interface families rather than reopen them.

## 3. Host Modes
Two host modes are in scope for the current OxFml contract.

### 3.1 Direct Host Mode
Direct host mode is a proving or single-formula application host with no OxCalc coordinator involved.

Current direct-host scope:
1. one formula or one narrow local recalc surface,
2. mutable defined-name and direct-cell bindings,
3. typed host-query providers,
4. deterministic candidate, commit, reject, and trace production through OxFml-owned artifacts.

Direct host mode does not imply:
1. graph-wide dependency coordination,
2. multi-session publication arbitration,
3. distributed placement policy,
4. OxCalc-equivalent coordinator semantics.

### 3.2 OxCalc-Integrated Host Mode
OxCalc-integrated mode is the coordinator-facing host path where OxCalc drives scheduling, intake, and publication policy across a broader workbook or graph.

Current OxCalc-integrated host contract requires:
1. explicit candidate versus commit separation,
2. typed reject and runtime-effect carriage,
3. stable artifact and correlation identities,
4. no host-side reinterpretation of OxFml artifact meaning.

## 4. Required Inputs
Every conforming host must supply the following explicitly for the currently covered scope.

### 4.1 Formula and Structure Inputs
1. `FormulaSourceRecord`
2. formula-stable identity
3. structure-context identity or version
4. caller anchor and address-mode context where relative or host-sensitive meaning depends on it
5. direct cell bindings where semantic truth depends on concrete resolution
6. defined-name bindings

### 4.1A Host-Owned Table Context Inputs
When a formula channel permits structured references, the host must also supply explicit table context rather than expecting OxFml to recover it from workbook globals.

Required first semantic packet:
1. `table_catalog`
2. `enclosing_table_ref`
3. `caller_table_region`

Required first packet meaning:
1. `table_catalog` carries stable table identity, range, column map, and header/totals presence,
2. `enclosing_table_ref` identifies the effective table for omitted-table-name forms such as `[@Amount]`,
3. `caller_table_region` carries row/region-sensitive meaning needed for `#This Row`, header, data, or totals-sensitive bind.

Working rule:
1. direct hosts and OxCalc-integrated hosts should supply the same semantic packet even if their surrounding transport differs,
2. host ownership of tables matches the broader rule that workbook objects remain host/coordinator-owned,
3. OxFml owns grammar, bind, and evaluator consequences once the packet is supplied.

### 4.2 Runtime Catalog Inputs
1. `LibraryContextProvider`
2. immutable `LibraryContextSnapshot`
3. explicit snapshot identity and generation
4. runtime lookup over that pinned snapshot during bind or semantic-plan work
5. for runtime catalog mutation lanes, a host-facing OxFunc-owned registered-external mutation/controller surface rather than host-local catalog mutation

Working rule:
1. runtime catalog truth is a runtime interface, not a build-time-only ingestion step,
2. registration or removal must yield a new snapshot generation,
3. a host must not mutate a pinned snapshot in place and still claim stable replay or bind truth,
4. built-in catalog population remains OxFunc-owned from the start,
5. runtime registration and unregister of external functions should be funneled through OxFml packet normalization into OxFunc-owned catalog mutation rather than implemented as host-local side tables.

Current invalidation rule:
1. if runtime registration/removal creates or removes an ordinary formula-callable surface by name, treat that as a bind-visible name-world change,
2. if host-owned defined names are added, removed, renamed, or reclassified, treat that as the same broad invalidation class,
3. if a change only affects worksheet `CALL` / `REGISTER.ID` descriptor/runtime truth, treat it as a narrower reevaluation lane unless it also changes a bind-visible function-name world.

### 4.3 Typed Context and Query Inputs
For the currently covered local scope, a host must be able to supply the first typed context/query bundle families:
1. `ReferenceResolver`
2. `HostInfoProvider`
   - `query_cell_info(...)`
   - `query_info(...)`
   - `query_formula_text(reference)`
   - `query_sheet_index(CurrentSheet | Reference | SheetNameText)`
   - `query_sheet_count(Workbook | Reference)`
   - `query_aggregate_reference_context(reference)`
   - `query_width_conversion_mode(function)`
3. `RtdProvider`
   - `RtdRequest { prog_id, server_name, topic_strings }`
   - `RtdProviderResult::{ Value, NoValueYet, CapabilityDenied, ConnectionFailed, ProviderError }`
4. `RegisteredExternalProvider`
   - `resolve_register_id(RegisterIdRequest)`
   - `lookup_registered_external(register_id)`
   - `invoke_registered_external(descriptor, args)`
5. for host-initiated runtime registration and unregister:
   - `RegisteredExternalCatalogMutationRequest`
   - `RegisteredExternalCatalogController`
6. scalar context inputs:
   - `now_serial`
   - `random_value`
   - `LocaleFormatContext`
   - date-system context

Deferred-provider rule:
1. provider families explicitly deferred in OxFml or OxFunc remain outside this host contract,
2. hosts must not infer support for a deferred provider family from the existence of a generic typed-provider slot,
3. worksheet `CALL` / `REGISTER.ID` and host/VBA registration channels should share the same OxFunc-owned registered-external catalog truth rather than independent side channels.

### 4.4 Capability and Fence Inputs
Where the session path is used, a host must also supply:
1. capability view or capability profile identity,
2. snapshot and token fence basis,
3. commit-attempt identity where publication is attempted.

## 5. Required Operations
The minimum implementation-facing operation chain is:
1. `parse`
2. `project_red_view`
3. `bind`
4. `compile_semantic_plan`
5. `evaluate`
6. `commit`

Optional operational layers may exist:
1. repository services,
2. session services,
3. trace capture services,
4. proving-host helpers.

Working rule:
1. service layers are allowed,
2. canonical transform meaning remains normative,
3. a host implementation must be explainable in terms of explicit transform inputs and outputs even when caches or services are used internally.

## 6. Required Outputs
For the currently covered scope, a host implementation must preserve the following output families.

### 6.1 Artifact Outputs
1. `ParseResult`
2. `BindResult`
3. `CompileSemanticPlanResult`
4. `EvaluationOutput`
5. `AcceptedCandidateResult | RejectRecord`
6. `CommitBundle | RejectRecord`

### 6.2 Return-Surface Outputs
The host-visible return surface must preserve the first three-way split:
1. ordinary value,
2. `ValueWithPresentation`,
3. typed host/provider outcome projection.

### 6.3 Coordinator-Relevant Outputs
An OxCalc-integrated host must preserve:
1. `candidate_result_id`
2. `commit_attempt_id` where present
3. `reject_record_id`
4. optional `fence_snapshot_ref`
5. typed effect, reject, and topology-sensitive consequence surfaces
6. trace and replay correlation sufficient for deterministic diagnosis

## 7. Candidate, Commit, and Reject Rules
The host contract must preserve the following distinctions.

### 7.1 Edit Rejection Versus Accepted-Unresolved
1. a host may reject an edit before canonical artifact adoption when the formula cannot honestly enter the parse/bind/plan ladder,
2. a host may also accept formula text into canonical artifact state while preserving unresolved-name or bind-diagnostic facts,
3. accepted-unresolved is not the same thing as edit rejection.

### 7.2 Evaluation Versus Publication
1. `evaluate` yields an accepted candidate or a typed reject,
2. accepted candidate is not committed publication,
3. `commit` yields a published bundle or a typed no-publish reject.

### 7.3 Host Failure Projection Rule
Where an exercised host-query or provider lane projects through OxFml today:
1. the host must preserve the typed outcome family,
2. the host must not replace it with a generic exception or opaque transport error,
3. the host may add local diagnostics only if canonical typed meaning remains preserved.

## 8. Direct-Binding and Host-Sensitive Truth
Hosts must preserve direct cell bindings for lanes where semantic truth depends on concrete resolution.

Current explicit families include:
1. `@` scalarization,
2. `_xlfn.SINGLE`,
3. reference-sensitive `CELL(...)`,
4. other host-sensitive or spill-sensitive lanes where the canonical artifact still depends on direct cell identity.

Defined names alone are insufficient for these families.

## 9. Runtime Library-Context Requirements
The normative runtime seam to OxFunc is:
1. `LibraryContextProvider`
2. immutable `LibraryContextSnapshot`
3. explicit snapshot identity and generation
4. runtime-consumable surface lookup from OxFml

The CSV or other exported catalog artifact is:
1. useful for pinning,
2. useful for mismatch reporting,
3. useful for generated tests,
4. not the normative runtime interface by itself.

Implementation rule:
1. a host may ingest exported artifacts for testing or cold-start preparation,
2. runtime semantic truth must still be representable through the provider/snapshot interface.

## 10. Currently Covered Implementation Scope
For the currently covered local floor, a host implementation is expected to be sufficient for:
1. direct-host execution of the proving-host slice,
2. OxCalc consumption of candidate, commit, reject, trace, and runtime-effect families already carried canonically,
3. formula-entry channels already exercised locally:
   - ordinary worksheet A1 formulas
   - `WorksheetR1C1` for the current translated cell-and-area floor
4. typed host-query/provider lanes already exercised locally:
   - `INFO`
   - `CELL`
   - `RTD`
5. current OxFml/OxFunc higher-order callable floor already exercised locally:
   - `LET`
   - `LAMBDA`
   - `MAP`
   - `REDUCE`
   - `SCAN`
   - `BYROW`
   - `BYCOL`
   - `MAKEARRAY`

This section is intentionally narrower than “full Excel and all functions”.
Broader language and function coverage remains driven by the open worksets and exercised-evidence floor.

## 11. Explicit Deferrals
The following are not authorized by this host/runtime requirements draft:
1. full workbook-graph scheduler policy,
2. pack-grade replay claims,
3. deferred external/provider families,
4. final cross-process ABI,
5. full distributed placement policy,
6. full UI or rendering policy,
7. unexercised built-in or sublanguage families beyond the current local evidence floor.

## 12. OxCalc Coordination Questions
The next OxCalc coordination round should answer:
1. whether this direct-host versus OxCalc-integrated split is sufficient for implementation planning,
2. whether the required input families are enough for the first coordinator-host implementation slice,
3. whether the current required output families are sufficient for coordinator-controlled publication,
4. whether any currently covered host-query or effect family is still too narrow to implement a host honestly,
5. whether any narrower handoff is required now or whether note-level convergence is enough for this packet.

Current OxCalc intake after the first review pass is:
1. yes for the first direct-host and coordinator-host implementation slice,
2. no for broader shared seam-freeze promotion yet,
3. remaining residuals stay concentrated in:
   - caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset,
   - execution-restriction transport shape beyond the current semantic minimum,
   - publication and topology breadth beyond the current local exercised floor,
   - provider-failure and callable-publication only if they later become coordinator-visible.

Current OxCalc intake after the latest confirmation pass is:
1. the first host/runtime packet is settled enough for first implementation planning,
2. caller-anchor and address-mode carriage remains in the `W026` note lane,
3. provider-failure and callable-publication remain watch lanes only,
4. no new formal handoff is warranted from the current host/runtime packet alone.

## 13. Working Rule
Use this document as the current canonical OxFml draft for host/runtime and external requirements.

Do not over-read it as:
1. OxCalc agreement,
2. full product-host specification,
3. full language or built-in-function closure,
4. permission to bypass the canonical OxFml artifact and seam docs.

## 14. First Host Implementation Workflow
For a first direct single-cell host implementation, the expected implementation workflow is:

1. bootstrap runtime catalog truth
   - create or obtain a `LibraryContextProvider`,
   - pin one immutable `LibraryContextSnapshot`,
   - keep the chosen snapshot identity visible to parse, bind, and semantic-plan work,
2. create host state
   - construct the host formula source and stable identity,
   - configure caller location and structure-context identity,
   - load direct-cell bindings and defined-name bindings,
3. attach typed providers and scalar context
   - attach `HostInfoProvider` if the formula may use host-query functions,
   - attach `RtdProvider` if `RTD` is in scope,
   - attach locale/date-system context,
   - attach `now_serial` and `random_value` if the selected scope needs them,
4. run the canonical transform chain
   - `parse`
   - `project_red_view`
   - `bind`
   - `compile_semantic_plan`
   - `evaluate`
   - `commit`,
5. consume the host-facing output packet
   - `SemanticPlan`
   - `ExecutionContract`
   - frozen `TypedContextQueryBundleSpec`
   - `EvaluationOutput`
   - `ReturnedValueSurface`
   - `AcceptedCandidateResult`
   - `CommitBundle` or `RejectRecord`
   - `trace_events`,
6. decide local host action
   - for a direct host, render or report the ordinary value or typed host/provider outcome,
   - for an OxCalc-integrated host, hand candidate, commit, reject, and trace families onward without redefining their meaning.

For the current local floor, the direct host path is concretely represented by the single-formula host API, including:
1. `recalc(...)`
2. `recalc_with_backend(...)`
3. `recalc_with_interfaces(...)`
4. `recalc_with_rtd_provider(...)`

## 15. Current Implementation Readiness Assessment
For a first single-cell host implementation, the current local floor is:

### 15.1 In place now
1. a direct host can already execute a single formula through the current host path,
2. runtime library-context provider consumption exists locally,
3. grouped typed-query evidence exists for the currently exercised host-query/provider slice:
   - `INFO`
   - `CELL`
   - `RTD`,
4. the first host-visible return-surface split exists and is exercised:
   - ordinary value,
   - `ValueWithPresentation`,
   - typed host/provider outcome projection,
5. candidate, commit, reject, and trace outputs already exist in the direct-host path,
6. current higher-order callable execution exists locally for the currently exercised slice:
   - `LET`
   - `LAMBDA`
   - `MAP`
   - `REDUCE`
   - `SCAN`
   - `BYROW`
   - `BYCOL`
   - `MAKEARRAY`.

### 15.2 Not in place now
1. this is not yet a full Excel cell-formula implementation,
2. this is not yet coverage for nearly all built-in functions,
3. broader language families remain open, including work still owned by:
   - structured references and table formulas,
   - broader name and external-name host-boundary work,
4. restricted conditional-formatting and data-validation carriers are now specified, but they are not part of the first ordinary single-cell host packet,
5. broader host-query/provider families beyond the current `INFO` / `CELL` / `RTD` slice remain outside the current exercised floor,
6. broader runtime/distributed host policy remains outside the first direct-host slice,
7. pack-grade replay promotion is not in place.

Working rule:
1. the current host packet is sufficient for a first single-cell implementation over the currently covered slice,
2. it is not yet honest to describe the whole OxFml + OxFunc combination as full Excel formula coverage.

## 16. Replay Appliance Integration Path
The replay appliance can already be integrated into a first host through an explicit first-host capture packet.

For the current local floor, a host should retain and project:
1. formula source and stable identity,
2. pinned library-context snapshot identity,
3. frozen `TypedContextQueryBundleSpec`,
4. `SemanticPlan` identity and execution contract summary,
5. `EvaluationOutput`,
6. `ReturnedValueSurface`,
7. `AcceptedCandidateResult`,
8. `CommitBundle` or `RejectRecord`,
9. `trace_events`.

The direct-host packet already exposes the needed raw surfaces through `HostRecalcOutput`.
For the current local floor, hosts may project that output through the first helper packet:
1. `HostRecalcOutput::to_first_host_replay_capture_packet()`

Current replay-integration rule:
1. the host may project `HostRecalcOutput` into the replay appliance using the existing adapter and canonical artifact families,
2. the host must preserve candidate-versus-commit distinction and typed reject meaning when doing so,
3. the host must preserve direct-binding-sensitive and host-query-sensitive truth where the replay witness depends on it,
4. the host must not treat replay projection as permission to rewrite formula text, bind payloads, fence tuples, or capability views.

Current limitation:
1. the helper packet is a first-host capture projection, not a pack-grade replay bundle builder,
2. hosts still need to map that packet into the wider replay appliance import families,
3. replay evidence remains local-witness-tier rather than pack-grade.

## 17. Relationship To Fixture Hosts And Stand-In Coordinator Packets
The current host/runtime contract is also the source packet family for deterministic fixture hosts used in integration artifacts such as the OxFunc adapter wave.

Working rule:
1. fixture hosts may stand in for host/coordinator-owned truths locally,
2. they should still reuse the same semantic packet families described in this document,
3. that reuse must not be over-read as production OxCalc coordinator API freeze.

Current first stand-in packet direction is tracked in:
1. `docs/spec/OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md`

Current intended reuse:
1. OxFunc-facing adapter fixtures under `W049` and `W050`,
2. later direct-host integration tests,
3. later OxCalc-integrated test packets where the coordinator wraps or reuses the same semantic host inputs.
