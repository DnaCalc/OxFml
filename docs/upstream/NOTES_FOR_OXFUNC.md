# Notes for OxFunc

Status: `active`
Owner lane: `OxFml`
Relationship: current outbound seam-context note from OxFml to OxFunc

## 1. Purpose

Provide the current OxFml-side seam floor that OxFunc should assume now.

This is a current-state message, not a historical ledger.
It keeps only the distinctions, exercised behaviors, and open topics that still matter at the OxFml/OxFunc boundary.

## 2. Core Message

OxFml currently treats the first adapter wave as real and usable.

The main current points for OxFunc are:
1. helper-form and callable-value lanes now have an exercised local floor rather than note-only preservation,
2. caller-context, scalarization, host-query, and formatting-sensitive lanes are represented in replayable local artifacts,
3. prepared arguments and prepared results preserve semantic distinctions that OxFunc should continue to treat as real seam facts,
4. the current OxFml direction is still toward a runtime library-context interface rather than long-term build-time catalog ingestion,
5. the March 26 unary-negative-literal and blank single-cell issues are no longer open OxFml seam defects.

## 3. Current Evidence In OxFml

### 3.1 Canonical seam docs
1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
2. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
3. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
4. `docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
5. `docs/spec/OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
6. `docs/spec/OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`

### 3.2 Exercised local evidence
1. `crates/oxfml_core/tests/evaluator_tests.rs`
2. `crates/oxfml_core/tests/semantic_plan_tests.rs`
3. `crates/oxfml_core/tests/replay_fixture_tests.rs`
4. `crates/oxfml_core/tests/host_tests.rs`
5. `crates/oxfml_core/tests/fixtures/semantic_plan_replay_cases.json`
6. `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
7. `crates/oxfml_core/tests/fixtures/single_formula_host_replay_cases.json`
8. `crates/oxfml_core/tests/fixtures/empirical_oracle_scenarios.json`
9. `crates/oxfml_core/tests/fixtures/w050_oxfunc_pinned_fixture_corpus.json`

## 4. Current Seam Floor

### 4.1 Helper forms and callable values
OxFml has an exercised local floor for:
1. `LET` sequential helper binding,
2. helper-name shadowing,
3. `LAMBDA` literal formation,
4. immediate invocation,
5. helper-bound invocation,
6. lexical capture-sensitive callable summaries.

Current OxFml reading:
1. helper lambdas must preserve lexical capture rather than dynamic name re-read,
2. callable values are semantically real even where worksheet publication remains narrower,
3. the final downstream shared callable-value carrier is still open.

### 4.2 Scalarization and caller context
OxFml currently exercises:
1. explicit `@`,
2. `_xlfn.SINGLE` / `SINGLE`,
3. caller-context-sensitive evaluation lanes,
4. direct-cell-binding proving-host cases where defined names are insufficient.

OxFunc should continue treating these as real seam distinctions, not draft-only topics.

### 4.3 Host-query and formatting-sensitive lanes
OxFml currently exercises:
1. `TEXT`,
2. `INFO`,
3. `CELL("filename", ...)`,
4. reference-sensitive host/query proving-host cases.

OxFml still treats typed host-query views as the right direction rather than object-handle surfaces.

### 4.4 Prepared argument and result distinctions
Prepared arguments and results currently preserve:
1. source,
2. structure,
3. reference identity,
4. blankness,
5. caller-context distinctions,
6. publication- and formatting-oriented result metadata.

The mandatory OxFml-side invariants remain:
1. direct scalar input is not interchangeable with array-like input,
2. omitted argument, blank cell, empty string, and error remain distinct,
3. reference-returning meaning is not collapsed into unconditional eager dereference,
4. caller-context-dependent scalarization remains explicit and replayable,
5. typed host-query views remain capability-scoped,
6. helper-name shadowing must not change the meaning of an already-created helper lambda,
7. direct cell bindings must be preserved whenever semantic truth depends on concrete cell resolution.

## 5. Current Library-Context Position

OxFml still treats a versioned external library-context snapshot as the right current integration shape.

Current downstream artifact in use:
1. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`

Current OxFml reading:
1. the export is useful now for bounded consumption and test pinning,
2. it is not yet the final runtime ABI,
3. OxFml can already consume canonical ids, registration/source shape, and first-pass interface/profile fields from it,
4. long-term convergence should move toward a formal runtime provider/consumer model rather than continued note-based coordination.

## 6. Current Ownership Split

### 6.1 Not open as OxFml seam defects
OxFml does not currently treat the following as open local seam issues:
1. unary signed literals,
2. blank single-cell stand-in resolution for ordinary worksheet references,
3. the former `PV` / `FV` adapter-evaluation failure caused by unary negative literal handling.

Current local exercised consequences:
1. `=SIGN(-5)` evaluates through the ordinary evaluator path,
2. `=PV(0.05,10,-100)` evaluates through the ordinary evaluator path,
3. `=FV(0.05,10,-100)` evaluates through the ordinary evaluator path,
4. `=ISBLANK(A9)` with no fixture cell evaluates as `TRUE`,
5. blank single-cell stand-ins remain distinct from empty text.

### 6.2 Current OxFunc-local area
OxFml currently understands that the recent low-order residuals in:
1. `ASINH`,
2. `PV`,
3. `FV`,
4. `PMT`

were not open OxFml seam defects.

Current OxFml reading is:
1. OxFunc has taken those rows through local `W053`,
2. the repaired explanation is OxFunc-local integer-`POWER` publication alignment rather than a new OxFml-side ask,
3. OxFml does not currently hold an open action on those rows.

## 7. Current Open Topics Relevant To OxFunc

The topics OxFml still considers open and worth active coordination are:
1. the smallest final provenance vocabulary for `PreparedArgument` and `PreparedResult`,
2. the final placement of explicit `@` semantics in the execution pipeline,
3. the final compatibility and round-trip treatment of `_xlfn.SINGLE(...)`,
4. the first locked execution-profile vocabulary for downstream scheduler consumption,
5. the exact typed carrier shape for broader host-query return families,
6. the final shared carrier for callable helper values beyond the current replayable summary surface,
7. the smallest honest shared runtime library-context shape,
8. the split between library-context availability truth and runtime capability/provider-failure truth,
9. broader grouped-aggregation adapter coverage for `GROUPBY` / `PIVOTBY`,
10. publication-sensitive result-class preservation for `HYPERLINK` / `IMAGE`.

## 8. Current Requests To OxFunc

The next useful OxFunc-side outputs for OxFml are:
1. keep the current library-context export stable enough for bounded OxFml-side consumption until the runtime interface replaces it,
2. identify which callable-value facts OxFunc would need beyond the current helper summary carrier,
3. identify whether any currently expected function traits are still missing from the semantic-plan profile,
4. identify whether the present host-query capability split is already enough for the next `CELL` / `INFO` tightening pass,
5. align the runtime/provider-consumer library-context model before widening note traffic further,
6. confirm the minimum first adapter families OxFunc wants for `GROUPBY` / `PIVOTBY` once the first local `W053` slice exists,
7. confirm the smallest OxFunc-visible returned-value distinctions needed for `HYPERLINK` / `IMAGE`.

## 9. Current Summary

Current OxFml position to OxFunc:
1. the first adapter wave is real and exercised,
2. semantic distinctions around helper forms, callable values, scalarization, blankness, and host-query sensitivity remain intentional and should be preserved,
3. the March 26 unary-negative-literal and blank single-cell issues are closed locally on the OxFml side,
4. the recent `ASINH` / `PV` / `FV` / `PMT` cleanup is understood as OxFunc-local and repaired there,
5. the main remaining joint topics are no longer historical residual cleanup; they are `W052` typed registered-external packet tightening plus the newly-bounded `W053` grouped-aggregation/publication-class adapter expansion.

## 10. Current W052 Plan After OxFunc's Latest Note

OxFml reads the latest OxFunc note as:
1. confirming that `W049` / `W050` should remain mismatch-driven rather than reopening broad seam debate,
2. confirming that the March 26 unary-negative and blank single-cell residuals are closed on the OxFml side,
3. narrowing the live open packet back to worksheet `CALL` / `REGISTER.ID` runtime ownership under `W052`.

### 10.1 Current direct answers to OxFunc's bounded questions
Current OxFml read of OxFunc's three bounded questions is:
1. yes, `RegisteredExternalProvider` should remain separate from `HostInfoProvider`,
2. yes, the first bounded runtime packet should carry direct typed runtime packets rather than only snapshot rows,
3. yes, OxFml still sees worksheet `CALL` runtime as staying above OxFunc except for request normalization, descriptor-driven argument handling, runtime registration truth, and worksheet-visible result projection.

### 10.2 Current ownership split
Current sharpened OxFml ownership split is:
1. built-in function and operator catalog truth remains OxFunc-owned,
2. runtime registered-external catalog truth also remains OxFunc-owned,
3. OxFml should not maintain a competing host-local function catalog,
4. OxFml owns formula parsing, bind classification, typed request normalization, and worksheet-visible consequence classification,
5. OxCalc or a direct host owns higher-level external-library policy, security policy, and source-specific registration initiation,
6. worksheet `REGISTER.ID`, host API registration, and VBA shim registration are distinct initiating channels that should converge on the same OxFunc-owned catalog mutation seam,
7. unregister should use that same bounded seam rather than a host-local side path.

### 10.3 Current registration-channel plan
Current OxFml plan is that registered external functions may enter or leave the OxFunc-owned runtime catalog through three registration channels plus one symmetric removal lane:
1. worksheet `REGISTER.ID`
   - initiated from formula evaluation,
   - normalized by OxFml into a `RegisterIdRequest`,
   - resolved through `RegisteredExternalProvider::resolve_register_id(...)`,
   - yields descriptor truth later used by worksheet `CALL`,
2. host API registration
   - initiated by a host-side API call,
   - normalized by OxFml into `RegisteredExternalCatalogMutationRequest::Register(...)`,
   - may preserve richer host hints such as display/help text or execution profile,
3. VBA shim registration
   - initiated after host-owned VBA project loading,
   - normalized through the same `Register(...)` mutation packet,
   - preserves source-project, source-module, and source-procedure provenance,
4. unregister
   - normalized by OxFml into `RegisteredExternalCatalogMutationRequest::Unregister(...)`,
   - preserves initiating channel plus stable registration identity,
   - leaves resulting catalog truth and any snapshot-generation effects OxFunc-owned.

### 10.4 Current first bounded typed packet
Current OxFml proposal is that the first bounded runtime packet should carry these direct typed lanes:
1. `RegisterIdRequest`
2. `RegisteredExternalDescriptor`
3. `RegisteredExternalCallRequest`
4. `RegisteredExternalProvider`
5. `RegisteredExternalCatalogMutationRequest`
6. `RegisteredExternalCatalogMutationResult`
7. `RegisteredExternalCatalogController`

Current OxFml reading of those packet families is:
1. they are runtime request/result packets, not merely library-context snapshot metadata,
2. they should cross the seam directly wherever the host/runtime path needs them,
3. the runtime library-context snapshot may still carry admission/profile truth about whether worksheet `CALL` / `REGISTER.ID` is admitted or gated in a given environment,
4. the snapshot/provider layer should not be the only place where per-request registration, invocation, and unregister packets can be observed.

### 10.5 Current packet-shape proposal
Current best-effort OxFml packet split is:

#### `RegisterIdRequest`
1. `library_name`
2. `procedure_name`
3. optional `type_text`
4. `caller_anchor`
5. optional `host_execution_profile`

#### `RegisteredExternalDescriptor`
1. `register_id`
2. `library_name`
3. `procedure_name`
4. optional `type_text`
5. `descriptor_state`
6. any registration facts required by OxFunc to decide reference-dereference and general worksheet-to-external type coercion

#### `RegisteredExternalCallRequest`
1. `target_kind`
   - `RegisterId`
   - `DirectLibraryProcedure`
2. optional `register_id`
3. optional `library_name`
4. optional `procedure_name`
5. optional `type_text`
6. `normalized_arguments`
7. `caller_anchor`
8. optional `host_execution_profile`
9. optional `descriptor_ref`

#### `RegisteredExternalProvider`
1. `resolve_register_id`
2. `describe_registration`
3. `invoke_registered`
4. `invoke_direct`

#### `RegisteredExternalCatalogMutationRequest`
1. `Register`
   - `registration_channel`
   - `register_id_request`
   - optional `stable_registration_id_hint`
   - optional `display_name_hint`
   - optional `help_text_hint`
   - optional VBA source provenance
   - optional `host_execution_profile`
2. `Unregister`
   - `registration_channel`
   - `stable_registration_id`
   - optional `host_execution_profile`

#### `RegisteredExternalCatalogMutationResult`
1. `RegisterApplied`
   - `descriptor`
   - optional `host_execution_profile`
2. `UnregisterApplied`
   - `stable_registration_id`
   - optional `host_execution_profile`

#### `RegisteredExternalCatalogController`
1. host-facing OxFml funnel surface that applies a typed mutation packet into OxFunc-owned catalog mutation logic,
2. not a claim that OxFml owns catalog mutation semantics,
3. intended to preserve the initiating channel while OxFunc remains the owner of resulting catalog truth.

### 10.6 Current reference and conversion plan
Current OxFml plan is that worksheet `CALL` should follow the same broad principle already used for built-ins:
1. OxFml should not globally dereference references before call dispatch,
2. OxFml should preserve reference-visible prepared arguments where runtime descriptor truth may require them,
3. OxFunc should be able to consult registration metadata or direct-call metadata to decide:
   - whether a reference must remain reference-visible,
   - whether a reference should be dereferenced before native invocation,
   - which general worksheet-to-external type coercions apply,
4. worksheet `CALL` should therefore not get a special OxFml-only eager-dereference rule.

Current implication:
1. `RegisteredExternalDescriptor` must be rich enough for OxFunc to see argument-policy-relevant registration facts,
2. the bounded runtime packet must let OxFunc obtain that descriptor for register-id targets and direct-call targets,
3. descriptor-driven dereference and general type conversion should stay OxFunc-owned rather than being pre-flattened in OxFml.

### 10.7 Current invalidation and snapshot-generation plan
Current OxFml reading is that not every registration is the same class of change.

The current split is:
1. bind-visible function registration or unregister
   - should be treated like structural change,
   - should produce a new `LibraryContextSnapshot`,
   - should invalidate formulas pinned to the earlier bind-visible function world where affected,
2. defined-name add/remove/rename/reclassification
   - should be treated the same broad way for invalidation,
   - remains structure-context-owned rather than OxFunc-owned,
3. registered-external descriptor mutation used only through worksheet `CALL` / `REGISTER.ID`
   - should normally be a narrower reevaluation lane,
   - should not force broad rebinding unless it also changes the bind-visible function-name world.

Current recommended indexing consequences are:
1. keep explicit usage indexes for function surface names, canonical ids, and unresolved function identifiers,
2. keep explicit usage indexes for defined-name identifiers and unresolved name identifiers,
3. keep explicit usage indexes for worksheet `CALL`, worksheet `REGISTER.ID`, stable registration ids, and direct `{ library, procedure, type_text }` triples,
4. let invalidation follow the world that actually changed rather than collapsing all runtime registration into one universal rebuild rule.

### 10.8 Current local exercised floor
Current local evidence for this narrower lane exists in:
1. `crates/oxfml_core/tests/w052_registered_external_interface_tests.rs`
2. `docs/spec/formula-language/OXFML_REGISTERED_EXTERNAL_PROVIDER_AND_CALL_REGISTER_ID_BOUNDARY.md`
3. `docs/spec/formula-language/OXFML_NAME_WORLD_AND_RUNTIME_REGISTRATION_INVALIDATION.md`

The currently exercised local packet floor includes:
1. worksheet `REGISTER.ID`,
2. worksheet `CALL`,
3. reference-visible `CALL` arguments,
4. host API registration,
5. VBA shim registration,
6. unregister packet carriage.

### 10.9 Current specific requests back to OxFunc
The next useful OxFunc replies for `W052` are:
1. confirm whether the packet split above is the right first shared runtime packet,
2. identify any descriptor fields OxFunc needs beyond the current best-effort `RegisteredExternalDescriptor` sketch to decide dereference and general type coercion,
3. identify whether `RegisterIdRequest` needs any additional normalized fields beyond `{ library_name, procedure_name, type_text, caller_anchor, host_execution_profile }`,
4. state whether OxFunc wants registration-channel provenance preserved exactly as `WorksheetRegisterId`, `HostApiRegistration`, and `VbaProjectShimRegistration` or under a narrower shared vocabulary,
5. state the minimum OxFunc-visible consequences of register/unregister on `LibraryContextSnapshot` generation,
6. identify any exact field names OxFunc wants frozen now rather than left as best-effort placeholders.

### 10.10 Current remaining note-level open topics
Current remaining `W052` note-level open topics are:
1. exact shared field naming,
2. the smallest final shared `RegisteredExternalDescriptor` field set,
3. minimum snapshot-generation consequences of register/unregister,
4. any later coordinator-visible consequences if OxCalc needs the same packet sharpened further.

## 11. Current Read After OxFunc's March 27 Note

OxFml reads the current OxFunc note as:
1. confirming that admitted `@` and the admitted helper family are already real seam facts rather than note-only topics,
2. confirming that `CALL` / `REGISTER.ID` remains a bounded typed registered-external seam under `W052`,
3. adding one new bounded adapter-expansion ask rather than reopening broad callable or provenance theory.

Current OxFml owner mapping for OxFunc's newly sharpened asks is:
1. `GROUPBY` / `PIVOTBY` adapter expansion plus helper bind-time rejection parity now belongs under new local owner `W053`,
2. `HYPERLINK` and `IMAGE` remain primarily return-surface/publication-class work under `W042`, but their next bounded evidence push also sits inside `W053`,
3. `CALL` / `REGISTER.ID` remains under `W052`, not under the new grouped-aggregation lane.

Current OxFml read of the specific March 27 asks is:
1. yes, the next bounded adapter work should be real `GROUPBY` / `PIVOTBY` cases through the live OxFml parser/binder/preparation/evaluation path,
2. yes, bind-time helper rejection cases that Excel rejects before evaluation should stay on the bind/admission side rather than being treated as OxFunc runtime cleanup,
3. yes, `HYPERLINK` should preserve publication intent above plain text,
4. yes, `IMAGE` should preserve a richer result class than ordinary scalar text or a fake placeholder scalar.

Current best-effort local plan for `W053` is:
1. add one or more real `GROUPBY` adapter cases:
   - built-in aggregation callable lane such as `SUM`
   - prepared lambda lane if admitted by the current carrier
   - at least one totals/filter/header/sort-sensitive lane
2. add one or more real `PIVOTBY` adapter cases:
   - default callable-backed pivot lane
   - at least one totals/filter/header-band lane
3. widen helper bind-time rejection adapter coverage for:
   - duplicate `LET` names
   - duplicate `LAMBDA` parameter names
   - malformed helper lambda declarations already pinned locally
4. widen `W042` evidence so:
   - `HYPERLINK` preserves value plus publication intent
   - `IMAGE` preserves a richer result/publication class rather than scalarizing to plain text

Current local non-claims:
1. OxFml is not claiming full `GROUPBY` / `PIVOTBY` option-matrix closure in this next lane,
2. OxFml is not claiming final rich-value model closure for `IMAGE`,
3. OxFml is not claiming a new generic callable ABI round is needed for this slice.

Current next useful OxFunc reply after this note is:
1. confirm whether the `W053` owner split is the right bounded next lane,
2. identify any exact first adapter scenarios OxFunc most wants prioritized inside `GROUPBY` / `PIVOTBY`,
3. identify whether OxFunc needs any extra returned-value fields beyond current OxFml `W042` vocabulary to preserve `HYPERLINK` publication intent and `IMAGE` rich-value classification honestly.

### 11.1 Current local `W053` floor
OxFml now has a first real local `W053` slice.

Current local adapter evidence now exists for:
1. `GROUPBY` default callable lane via inline arrays plus `LAMBDA(x,SUM(x))`,
2. `GROUPBY` sort-sensitive lane,
3. `PIVOTBY` default callable lane,
4. `PIVOTBY` filter-and-zero-totals-sensitive lane.

Current helper bind-time rejection evidence now also includes:
1. duplicate `LAMBDA` parameter names as bind-time `BindMismatch`,
2. malformed `LAMBDA` parameter declaration as bind-time `BindMismatch`,
3. existing duplicate `LET` bind-time `BindMismatch`.

Current return-surface consequences now relevant to `W053` are:
1. `HYPERLINK` preserves `ValueWithPresentation` through evaluator, host, and adapter paths,
2. rich-value packet classification now preserves explicit `rich_value_type_name`,
3. `IMAGE` still lacks equivalent end-to-end evaluator/adapter evidence because no admitted local `IMAGE(...)` lane is exercised yet.
