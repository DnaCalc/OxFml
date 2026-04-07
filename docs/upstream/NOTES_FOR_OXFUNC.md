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
5. the March 26 unary-negative-literal and blank single-cell issues are no longer open OxFml seam defects,
6. OxFunc's `docs/function-lane/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` plus `docs/handoffs/HANDOFF_SHARED_INTERFACE_FREEZE_PROMOTION_TO_OXFML_V1.md` are now the current shared freeze floor from the OxFml side for the narrowed seam families.

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
3. the minimum shared callable carrier is now narrowed candidate rather than wholly open:
   - opaque callable identity or token
   - `origin_kind`
   - `capture_mode`
   - `arity_shape`
   - `invocation_contract_ref`
4. parameter-name, capture-name, and body-kind detail may remain provenance/replay detail rather than minimum shared transport fields for the current phase.

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
9. publication-sensitive result-class preservation for `HYPERLINK` / `IMAGE`,
10. the remaining shared packet and invalidation questions under `W052`,
11. the ordinary-operator execution seam, especially array-lifted operator transport beyond the current scalar binary arithmetic slice.

### 7.1 Current operator-seam read
Current OxFml read:
1. OxFml now treats ordinary operator semantics as OxFunc-owned rather than as a valid long-term OxFml-local evaluator lane,
2. OxFml has filed a formal outbound packet:
   - `docs/handoffs/HANDOFF_OXFUNC_001_OPERATOR_VALUE_SURFACE_AND_ARRAY_LIFT_EXPANSION.md`
3. OxFml wants the broader ordinary-operator family treated coherently rather than one operator row at a time:
   - unary arithmetic
   - postfix percent
   - concat
   - comparisons
   - reference operators where the shared seam admits them.

### 7.2 Current multi-area reference read
Current OxFml read:
1. simple lexical ranges such as `A1:B2` do not need to survive as runtime `cell : cell` operator traces,
2. it is honest to normalize those as a single range reference object,
3. same-sheet multi-area references are different from 3D sheet spans,
4. OxFunc now exposes a real same-sheet `ReferenceKind::MultiArea` seam shape plus normalization/splitting helpers,
5. same-sheet multi-area value-required materialization now travels through OxFunc-owned resolver-driven combination semantics rather than an OxFml-local aggregation helper.

Current OxFml-side evidence and pressure points:
1. `crates/oxfml_core/src/eval/mod.rs`
2. `crates/oxfml_core/tests/evaluator_tests.rs`
3. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
4. `docs/worksets/W059_operator_semantic_dispatch_boundary_correction.md`

Current OxFunc-side references OxFml is reading:
1. `../OxFunc/crates/oxfunc_core/src/value.rs`
2. `../OxFunc/crates/oxfunc_core/src/resolver.rs`
3. `../OxFunc/crates/oxfunc_core/src/functions/operator_reference_family.rs`
4. `../OxFunc/docs/function-lane/FUNCTION_SLICE_OPERATOR_REFERENCE_FAMILY_CONTRACT_PRELIM.md`
5. `../OxFunc/docs/function-lane/FUNCTION_SLICE_INDEX_CONTRACT_PRELIM.md`
6. `../OxFunc/docs/function-lane/FUNCTION_SLICE_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_CONTRACT_PRELIM.md`

Current shared-seam direction from OxFml:
1. `ReferenceLike` now has a first-class same-sheet `MultiArea` representation,
2. `OP_UNION_REF` should continue to publish `ReferenceKind::MultiArea`, not regress to overloading `ReferenceKind::Area` with a parenthesized multi-target string,
3. multi-area structure should remain consumable by:
   - `AREAS`
   - `INDEX(..., area_num)`
   - resolver normalization and capability checks
   - OxFml local/runtime reference transport
4. mixed-sheet multi-area should be classified distinctly from same-sheet multi-area and should not be silently flattened into one reference string,
5. 3D sheet-span references must remain a separate construct from same-sheet multi-area.

### 7.3 Requested OxFunc type shape
OxFml wants the OxFunc seam to carry the following conceptual distinction:
1. single-area A1-like reference,
2. normalized rectangular/whole-row/whole-column area,
3. same-sheet multi-area,
4. 3D sheet span,
5. structured reference,
6. spill anchor.

The current OxFml-preferred shape is:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    A1,
    Area,
    MultiArea,
    ThreeD,
    Structured,
    SpillAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceLike {
    pub kind: ReferenceKind,
    pub target: String,
}
```

OxFml also wants helper APIs at this seam so downstream consumers do not have to reinterpret raw `target` strings ad hoc:

```rust
impl ReferenceLike {
    pub fn new(kind: ReferenceKind, target: impl Into<String>) -> Self { ... }

    pub fn multi_area(targets: Vec<String>) -> Option<Self> { ... }

    pub fn normalized(self) -> Self { ... }

    pub fn multi_area_targets(&self) -> Option<Vec<String>> { ... }

    pub fn area_count(&self) -> usize { ... }
}
```

The intended behavior of those helpers is:
1. `multi_area(...)` rejects empty inputs and single-target pseudo-multi-area construction,
2. `normalized()` trims and canonicalizes while preserving `MultiArea` identity,
3. `multi_area_targets()` returns the parsed member targets only for `ReferenceKind::MultiArea`,
4. `area_count()` returns the member count for multi-area and `1` otherwise.

### 7.4 Target semantics OxFml wants OxFunc to enforce
OxFml wants the following semantic split made explicit:
1. same-sheet multi-area example:
   - `(A1:A2,G1:G2)`
   - same prefix / same sheet identity
   - valid `ReferenceKind::MultiArea`
2. mixed-sheet multi-area example:
   - `(Sheet1!A1:A2,Sheet2!G1:G2)`
   - not the same thing as same-sheet multi-area
   - should not be silently normalized into one same-sheet carrier
   - current OxFml expectation is rejection as unsupported source for local materialization lanes
3. 3D span example:
   - `Sheet1:Sheet3!A1`
   - separate `ReferenceKind::ThreeD`
   - consumed by sheet-topology-sensitive functions such as `SHEETS`
   - not interchangeable with multi-area membership.

### 7.5 Current OxFunc implementation points OxFml depends on
OxFml now depends on OxFunc continuing to:
1. keep the type-level `ReferenceKind::MultiArea` distinction real end-to-end,
2. preserve and pass through multi-area references in `normalize_reference(...)` and `resolve_eval_value(...)`,
3. keep `eval_op_union_ref_surface(...)` returning `ReferenceKind::MultiArea`,
4. keep adapter mappings and reference-sensitive functions `MultiArea`-aware rather than treating union as only a raw parenthesized string convention,
5. keep mixed-sheet multi-area rejection explicit where the slice only admits same-sheet multi-area,
6. keep 3D sheet spans distinct from multi-area.

Concrete OxFunc files OxFml expects to matter:
1. `../OxFunc/crates/oxfunc_core/src/value.rs`
2. `../OxFunc/crates/oxfunc_core/src/resolver.rs`
3. `../OxFunc/crates/oxfunc_core/src/functions/operator_reference_family.rs`
4. `../OxFunc/crates/oxfunc_core/src/functions/adapters.rs`
5. `../OxFunc/crates/oxfunc_core/src/functions/index.rs`
6. `../OxFunc/crates/oxfunc_core/src/functions/reference_metadata_family.rs`

### 7.6 Suggested implementation sketch
OxFml is not prescribing the exact final code, but the minimum intended behavior is approximately:

```rust
pub fn eval_op_union_ref_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, OperatorReferenceError> {
    if args.len() != 2 {
        return Err(OperatorReferenceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }

    let lhs = reference_arg(&args[0])?;
    let rhs = reference_arg(&args[1])?;

    let targets = match (&lhs.kind, &rhs.kind) {
        (ReferenceKind::MultiArea, ReferenceKind::MultiArea) => {
            let mut parts = lhs.multi_area_targets().ok_or(
                OperatorReferenceError::UnsupportedReferenceSource("invalid_multi_area_reference"),
            )?;
            parts.extend(
                rhs.multi_area_targets().ok_or(
                    OperatorReferenceError::UnsupportedReferenceSource("invalid_multi_area_reference"),
                )?,
            );
            parts
        }
        (ReferenceKind::MultiArea, _) => {
            let mut parts = lhs.multi_area_targets().ok_or(
                OperatorReferenceError::UnsupportedReferenceSource("invalid_multi_area_reference"),
            )?;
            parts.push(rhs.target.trim().to_string());
            parts
        }
        (_, ReferenceKind::MultiArea) => {
            let mut parts = vec![lhs.target.trim().to_string()];
            parts.extend(
                rhs.multi_area_targets().ok_or(
                    OperatorReferenceError::UnsupportedReferenceSource("invalid_multi_area_reference"),
                )?,
            );
            parts
        }
        _ => vec![lhs.target.trim().to_string(), rhs.target.trim().to_string()],
    };

    let multi = ReferenceLike::multi_area(targets).ok_or(
        OperatorReferenceError::UnsupportedReferenceSource("invalid_multi_area_reference"),
    )?;

    Ok(EvalValue::Reference(multi))
}
```

OxFml expects any final OxFunc implementation to preserve the same core outcomes even if the code shape differs.

### 7.7 Active follow-on handoff
The current follow-on handoff for this lane is:
1. `docs/handoffs/HANDOFF_OXFUNC_002_MULTIAREA_VALUE_MATERIALIZATION_STYLE_A.md`

That handoff narrowed the next step further:
1. keep `ReferenceKind::MultiArea` as the shared reference carrier,
2. move same-sheet multi-area value-materialization semantics into OxFunc,
3. use the existing `ReferenceResolver` in a Style A design where OxFunc resolves member targets one by one and owns the combination rule,
4. let OxFml remove the remaining local same-sheet multi-area aggregation helper after OxFunc lands the change.

Current state:
1. OxFunc has now acknowledged and landed that Style A seam in `OxFunc/W076`,
2. OxFml has now switched its non-reference-preserved lanes over to the OxFunc-owned path and removed the superseded local aggregation helper.

### 7.7 Why OxFml still calls this out
Current OxFml reason:
1. ordinary operator dispatch is now being pushed toward the proper OxFml/OxFunc boundary,
2. union/reference operators cannot stay honest if `MultiArea` regresses back into only a string convention,
3. `AREAS`, `INDEX`, and related reference-sensitive lanes already treat same-sheet multi-area as a real semantic shape,
4. OxFml wants the shared seam contract to stay explicit now that value-required lanes also travel through OxFunc-owned materialization,
5. OxFml wants to keep reducing local reinterpretation pressure rather than deepen it.

## 8. Current Requests To OxFunc

The next useful OxFunc-side outputs for OxFml are now:
1. keep the current library-context export stable enough for bounded OxFml-side consumption until the runtime interface replaces it,
2. keep the multi-area `ReferenceLike` distinction explicit and consumable across reference operators, `INDEX`, `AREAS`, and resolver normalization,
3. keep same-sheet multi-area distinct from 3D sheet spans and from mixed-sheet unsupported-source cases,
4. flag only concrete wording corrections against the mirrored freeze packet or `HO-FN-004`,
5. flag only concrete packet or carrier mismatches that block promotion,
6. otherwise treat the mirrored packet plus `HO-FN-004` as the shared freeze floor for current owner-packet promotion.

## 9. Current Summary

Current OxFml position to OxFunc:
1. the first adapter wave is real and exercised,
2. semantic distinctions around helper forms, callable values, scalarization, blankness, and host-query sensitivity remain intentional and should be preserved,
3. the March 26 unary-negative-literal and blank single-cell issues are closed locally on the OxFml side,
4. the recent `ASINH` / `PV` / `FV` / `PMT` cleanup is understood as OxFunc-local and repaired there,
5. the main remaining joint topics are no longer historical residual cleanup; they are `W052` typed registered-external packet freeze plus the now-concrete `IMAGE` runtime/publication integration lane under `W042`/`W053`.

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
Current OxFml local freeze is that the first bounded runtime packet should carry these direct typed lanes:
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
3. OxFml now adopts the OxFunc-owned request/result packet types directly rather than sketching a parallel wrapper vocabulary for those three shared packet families,
4. normalized worksheet `REGISTER.ID` and `CALL` packets are now exposed in OxFml trace/adapter artifacts through `PreparedCall`,
5. the runtime library-context snapshot may still carry admission/profile truth about whether worksheet `CALL` / `REGISTER.ID` is admitted or gated in a given environment,
6. the snapshot/provider layer should not be the only place where per-request registration, invocation, and unregister packets can be observed.

### 10.5 Current OxFml-side packet freeze decisions
Current OxFml-side packet freeze is:

#### `RegisterIdRequest`
1. adopt the OxFunc-owned packet directly:
   - `library_name`
   - `procedure`
   - `declared_type_text`
2. `caller_anchor` and `host_execution_profile` are adjacent OxFml host/adaptor facts, not reasons to fork the shared request type

#### `RegisteredExternalDescriptor`
1. adopt the OxFunc-owned packet directly:
   - `stable_registration_id`
   - `register_id`
   - `origin_kind`
   - `display_name`
   - `library_name`
   - `procedure`
   - `declared_type_text`
2. if more argument-policy facts are needed, OxFunc should extend this shared descriptor upstream and OxFml should adopt that extension directly rather than inventing a sibling wrapper

#### `RegisteredExternalCallRequest`
1. adopt the OxFunc-owned packet directly:
   - `target`
   - `invocation_args`
2. `target` remains:
   - `RegisterId(f64)`
   - `Direct(RegisterIdRequest)`
3. `caller_anchor` and `host_execution_profile` remain adjacent OxFml host/adaptor facts

#### `RegisteredExternalProvider`
1. `resolve_register_id`
2. `lookup_registered_external`
3. `invoke_registered_external`

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
3. these mutation packets remain OxFml-owned host/coordinator funnel packets over OxFunc-owned catalog truth unless OxFunc explicitly wants to adopt them as shared runtime packet families too

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

Current OxFml-side freeze for `W052` is:
1. bind-visible function registration or unregister must produce a new `LibraryContextSnapshot` generation and bind invalidation,
2. descriptor mutation used only through worksheet `CALL` / `REGISTER.ID` should default to targeted reevaluation rather than broad rebinding,
3. the shared seam still needs OxFunc acknowledgment of that split, but OxFml is no longer treating it as undecided locally.

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
6. unregister packet carriage,
7. direct adoption of OxFunc-owned `RegisterIdRequest`, `RegisteredExternalCallRequest`, and `RegisteredExternalDescriptor` packet types on the OxFml side,
8. normalized worksheet `REGISTER.ID` and `CALL` packet exposure through `PreparedCall`,
9. both direct-target `CALL(...)` and register-id-target `CALL(...)` lanes.

### 10.9 Current specific requests back to OxFunc
The next useful OxFunc replies for `W052` are:
1. confirm the local OxFml-side freeze above as the right first shared runtime packet split,
2. identify any descriptor fields OxFunc needs beyond the current shared `RegisteredExternalDescriptor` to decide dereference and general type coercion,
3. state whether OxFunc wants registration-channel provenance preserved exactly as `WorksheetRegisterId`, `HostApiRegistration`, and `VbaProjectShimRegistration` or under a narrower shared vocabulary,
4. state whether `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` should remain OxFml-owned funnel packets or become OxFunc-owned shared runtime packet families,
5. confirm the minimum shared snapshot-generation consequences of register/unregister.

### 10.10 Current remaining note-level open topics
Current remaining `W052` note-level open topics are:
1. exact shared field naming,
2. the smallest final shared `RegisteredExternalDescriptor` field set,
3. whether mutation/controller packets stay OxFml-owned wrappers or become shared OxFunc-owned runtime packet families,
4. minimum snapshot-generation consequences of register/unregister,
5. any later coordinator-visible consequences if OxCalc needs the same packet sharpened further.

## 11. Current Read After OxFunc's March 27 Note

OxFml reads the current OxFunc note as:
1. confirming that admitted `@` and the admitted helper family are already real seam facts rather than note-only topics,
2. confirming that `CALL` / `REGISTER.ID` remains a bounded typed registered-external seam under `W052`,
3. treating the landed `W053` grouped-aggregation corpus as real seam evidence rather than a still-open first adapter ask,
4. leaving `IMAGE` integration and `W052` packet tightening as the live bounded lanes.

Current OxFml owner mapping for OxFunc's newly sharpened asks is:
1. the landed grouped-aggregation adapter corpus remains owned locally under `W053` as the current callable-heavy regression floor,
2. `HYPERLINK` remains return-surface/publication-hint work under `W042`, while `IMAGE` is now a concrete host-query-plus-rich-value lane under `W042` / `W053`,
3. `CALL` / `REGISTER.ID` remains under `W052`, not under grouped aggregation.

Current OxFml read of the current bounded asks is:
1. yes, the landed `GROUPBY` / `PIVOTBY` corpus should now be treated as a real seam regression floor rather than an open first-proof request,
2. yes, bind-time helper rejection cases that Excel rejects before evaluation stay on the bind/admission side rather than being treated as OxFunc runtime cleanup,
3. yes, `HYPERLINK` should preserve publication intent above plain text,
4. yes, `IMAGE` should preserve the locked `_webimage` rich-value carrier while keeping published fallback separate from the semantic return carrier.

Current local non-claims:
1. OxFml is not claiming full `GROUPBY` / `PIVOTBY` option-matrix closure in this next lane,
2. OxFml is not claiming that the richer return-surface field naming is jointly frozen yet even though the local `IMAGE(...)` evaluator/host/adapter lane is now exercised,
3. OxFml is not claiming a new generic callable ABI round is needed for this slice.

Current next useful OxFunc reply after this note is:
1. identify whether any extra returned-value fields beyond current OxFml `W042` vocabulary are still needed for `IMAGE`,
2. confirm whether `TypedContextQueryFamily::Image` is the right first freeze name for the host-query family,
3. continue narrowing exact `W052` field names, mutation-family ownership, and minimum snapshot-generation consequences,
4. flag only concrete grouped-aggregation mismatches if the existing `W053` regression floor proves insufficient later.

### 11.1 Current local `W053` floor
OxFml now has a first real local `W053` slice.

Current local adapter evidence now exists for:
1. `GROUPBY` default callable lane via inline arrays plus `LAMBDA(x,SUM(x))`,
2. `GROUPBY` built-in aggregation callable lane via bare `SUM`,
3. `GROUPBY` visible-header lane via bare built-in aggregation callable carriage,
4. `GROUPBY` hierarchical-subtotal lane via bare built-in aggregation callable carriage,
5. `GROUPBY` sort-sensitive lane for both inline `LAMBDA(...)` and bare built-in aggregation callable carriage,
6. `GROUPBY` filtered descending-value sort lane via bare built-in aggregation callable carriage,
7. `GROUPBY` tabular-subtotal runtime rejection lane as an explicit adapter-visible evaluation failure,
8. `PIVOTBY` default callable lane,
9. `PIVOTBY` built-in aggregation callable lane via bare `SUM`,
10. `PIVOTBY` visible-header band lane via bare built-in aggregation callable carriage,
11. `PIVOTBY` row/column-total sort lane via bare built-in aggregation callable carriage,
12. `PIVOTBY` filter-and-zero-totals-sensitive lane for both inline `LAMBDA(...)` and bare built-in aggregation callable carriage.

Current helper bind-time rejection evidence now also includes:
1. duplicate `LAMBDA` parameter names as bind-time `BindMismatch`,
2. malformed `LAMBDA` parameter declaration as bind-time `BindMismatch`,
3. duplicate `LET` bind-time `BindMismatch`.

Current deterministic local fixture corpus now also includes:
1. `crates/oxfml_core/tests/fixtures/w053_grouped_aggregation_cases.json`,
2. grouped-aggregation success lanes,
3. grouped-aggregation runtime-rejection lane,
4. helper bind-time rejection lanes,
5. `HYPERLINK` publication-intent lane,
6. `IMAGE` rich-value lane.

Current OxFml-side seam reading from that added evidence is:
1. grouped-aggregation semantics remain OxFunc-owned,
2. the OxFml obligation is the adapter seam only:
   - parse the grouped-aggregation formula correctly,
   - preserve callable-slot carriage,
   - distinguish bare built-in aggregation callable tokens from unresolved ordinary names,
   - reject malformed helper declarations on the bind side before grouped-aggregation runtime dispatch,
3. bare built-in aggregation callable carriage for grouped aggregation is now a real local seam fact rather than a note-only expectation.

Current local mismatch read:
1. the widened `W053` corpus has not exposed a new concrete OxFunc mismatch on the OxFml side,
2. the remaining next useful OxFunc action is acknowledgment/integration of the widened local floor rather than another broad seam redesign.

Current return-surface consequences now relevant to `W053` are:
1. `HYPERLINK` preserves `ValueWithPresentation` through evaluator, host, and adapter paths,
2. OxFml now preserves OxFunc extended top-level return surfaces generically rather than via a `HYPERLINK`-only branch, with `TODAY` now also exercised as `ValueWithPresentation`,
3. explicit `_webimage` rich-value packet evidence now preserves `ReturnedValueSurfaceKind::RichValue` plus `rich_value_type_name`, host-side publication no longer drops non-ordinary return classes merely because the published worksheet fallback differs, and commit-bundle carriage now preserves the same non-ordinary rich-value class,
4. `IMAGE(...)` is now exercised locally through evaluator, host, and adapter paths with typed `HostInfoProvider::query_image(...)` normalization, preserved published fallback, and `TypedContextQueryFamily::Image`.

## 12. Current Closure Packet From OxFml

This section is the cleaned-up closure packet for the currently open OxFml <-> OxFunc lanes. It is intended to supersede the more narrative parts of the earlier note when deciding what still needs acknowledgment.

### 12.1 Current local OxFml position

OxFml now considers the following local positions stable:
1. `W042`
   - the first shared returned-value split should be:
     - `OrdinaryValue`
     - `ValueWithPresentation`
     - `RichValue`
     - `TypedHostProviderOutcome`
   - `HYPERLINK` is exercised locally as `ValueWithPresentation`
   - `IMAGE` is exercised locally as `RichValue` carrying `_webimage`
   - published worksheet fallback remains separate from the semantic return carrier
   - `TypedContextQueryFamily::Image` is the proposed first freeze name for the `IMAGE` host-query lane
2. `W052`
   - OxFml adopts the OxFunc-owned shared packet types directly for:
     - `RegisterIdRequest`
     - `RegisteredExternalDescriptor`
     - `RegisteredExternalCallRequest`
   - descriptor-driven dereference and general type coercion remain OxFunc-owned
   - `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` remain OxFml-owned funnel packets unless OxFunc explicitly wants them promoted into the shared runtime packet family
   - invalidation split should be:
     - bind-visible registration/unregister => new `LibraryContextSnapshot` generation
     - `CALL` / `REGISTER.ID`-only descriptor mutation => targeted reevaluation by default
3. `W053`
   - grouped-aggregation semantics remain OxFunc-owned
   - OxFml’s responsibility is only the adapter seam:
     - parse correctly
     - bind helper forms correctly
     - preserve callable-slot carriage
     - reject malformed helper forms before runtime dispatch
   - the widened local `GROUPBY` / `PIVOTBY` / helper-rejection / `HYPERLINK` / `IMAGE` corpus has not exposed a new concrete OxFunc mismatch

### 12.2 Specific confirmations OxFml now needs from OxFunc

OxFunc’s latest note now gives OxFml a convergent current-phase answer on `W052`:
1. direct shared packet set:
   - `RegisterIdRequest`
   - `RegisteredExternalDescriptor`
   - `RegisteredExternalCallRequest`
2. minimum shared `RegisteredExternalDescriptor` field set:
   - keep the current seven-field descriptor
3. mutation/controller family ownership:
   - keep `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` OxFml-owned for the current phase
4. minimum shared snapshot-generation consequences:
   - bind-visible registration or unregister => new `LibraryContextSnapshot` generation plus bind invalidation where the visible function or name world changes
   - `CALL` / `REGISTER.ID`-only descriptor mutation => targeted reevaluation by default

Current sharper wording:
1. OxFml is no longer asking whether `RegisteredExternalProvider` stays separate from `HostInfoProvider`; OxFml treats that as already settled
2. OxFml is no longer asking whether a parallel OxFml wrapper vocabulary is needed for:
   - `RegisterIdRequest`
   - `RegisteredExternalDescriptor`
   - `RegisteredExternalCallRequest`
   OxFml treats direct adoption of the OxFunc packet types as the settled direction
3. the only live shared decisions are now:
   - exact field naming
   - minimum `RegisteredExternalDescriptor` field set
   - mutation/controller family ownership
   - snapshot-generation consequences

Current OxFml read after that latest OxFunc reply is:
1. those four decisions are now converged with OxFunc and acceptable from the OxFml side as current shared freeze wording for the narrowed `W052` packet family
2. the remaining live work is coordinator-facing propagation and canonical promotion, not further OxFml/OxFunc model redesign

### 12.3 Open lanes remaining after OxFml local work

OxFunc’s current note now confirms:
1. the four-way `W042` return-surface split is sufficient for the current-phase shared freeze
2. no extra `IMAGE` fields are needed beyond the present `W042` vocabulary
3. `TypedContextQueryFamily::Image` is the right first freeze name
4. the widened `W053` grouped-aggregation corpus is sufficient as the current regression floor, with future reopening only on concrete mismatch

So the remaining OxFml-side note lanes now reduce to:
1. `W042`: broader packet promotion and consumer uptake, not return-surface model disagreement
2. `W052`: coordinator-facing propagation and canonical promotion, not packet-shape disagreement with OxFunc
3. `W053`: mismatch-driven reopening only

### 12.4 Non-asks

OxFml is not asking OxFunc for:
1. a new broad callable ABI redesign
2. full `GROUPBY` / `PIVOTBY` option-matrix closure
3. a broad new adapter round outside concrete mismatch
4. a full rich-value object-model redesign beyond the first shared return-surface freeze

### 12.5 Current Freeze-Candidate Read

Current OxFml read of OxFunc's newest note is:
1. the OxFunc-side `OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` is the right current anchor for promoting the shared interface into explicit freeze text
2. the minimum shared callable carrier is now converged enough to state explicitly as:
   - opaque callable identity or token
   - `origin_kind`
   - `capture_mode`
   - `arity_shape`
   - `invocation_contract_ref`
3. parameter-name, capture-name, and body-kind detail may remain provenance/replay detail rather than minimum shared transport fields for the current phase
4. no additional explicit invocation-model field is currently needed beyond the opaque callable identity plus `invocation_contract_ref`
5. OxFml has now mirrored that current-phase packet set locally in:
   - `docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`
6. the remaining OxFml/OxFunc work is now promotion and wording freeze rather than another broad callable-boundary redesign

Current reply direction from OxFml:
1. OxFml now reads `docs/handoffs/HANDOFF_SHARED_INTERFACE_FREEZE_PROMOTION_TO_OXFML_V1.md` as acceptable current shared freeze wording for the narrowed seam families
2. the mirrored packet at `docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md` plus `HO-FN-004` is now the shared freeze floor OxFml wants used for owner-packet promotion and propagation
3. later widening should now be mismatch-driven rather than another broad seam re-derivation
