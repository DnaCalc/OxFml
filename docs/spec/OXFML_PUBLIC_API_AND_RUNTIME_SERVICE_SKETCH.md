# OxFml Public API and Runtime Service Sketch

## 1. Purpose
This document defines the first code-facing OxFml public surface sketch.

It is not a language-level signature freeze.
It is the current API-shape baseline that implementation work should target unless a later workset narrows it further.

Read together with:
1. `OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`
2. `OXFML_IMPLEMENTATION_BASELINE.md`
3. `OXFML_CANONICAL_ARTIFACT_SHAPES.md`
4. `OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`
5. `formula-language/OXFML_PARSER_AND_BINDER_REALIZATION.md`
6. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`

## 2. Surface Rule
The public OxFml surface should separate:
1. canonical artifact transforms,
2. optional repositories and runtime services,
3. evaluator-session and commit operations,
4. proving-host helpers.

The canonical artifact transforms are normative.
Repositories and services are optional operational layers over the same semantics.

## 3. Canonical Transform Surface
The current canonical transform chain is:
1. `parse`
2. `project_red_view`
3. `bind`
4. `compile_semantic_plan`
5. `evaluate`
6. `commit`

Each step must accept explicit inputs and return explicit typed outputs.

## 4. Current Request and Result Shapes
### 4.1 `ParseRequest` -> `ParseResult`
Minimum request fields:
1. `FormulaSourceRecord`
2. parse profile or compatibility context

Minimum result fields:
1. `GreenTreeRoot`
2. parse diagnostics
3. optional token-stream or trivia projection

### 4.2 `RedProjectionRequest` -> `RedProjection`
Minimum request fields:
1. `GreenTreeRoot`
2. `formula_stable_id`
3. source-span and caller/workbook context as needed

Minimum result fields:
1. red root view
2. span/parent-position helpers
3. contextual diagnostic helpers

### 4.3 `BindRequest` -> `BindResult`
Minimum request fields:
1. `GreenTreeRoot` and/or red projection
2. `formula_stable_id`
3. `formula_token`
4. `structure_context_version`
5. scope and table metadata
6. caller anchor and address-mode context
7. library-context snapshot or function/operator lookup surface
8. profile and capability context

Minimum result fields:
1. `BoundFormula`
2. bind diagnostics
3. unresolved-reference records

Result rule:
1. `bind` may reject a formula edit when the submitted text cannot honestly enter the bound-artifact world,
2. `bind` may also accept the formula text and produce a `BoundFormula` with unresolved-reference or bind-diagnostic records,
3. accepting the formula into bound-artifact state is not the same thing as claiming later evaluation success.

### 4.4 `CompileSemanticPlanRequest` -> `CompileSemanticPlanResult`
Minimum request fields:
1. `BoundFormula`
2. library-context snapshot identity or handle
3. OxFunc catalog or trait surface identity
4. locale, date-system, and format-service context
5. per-surface availability identity sufficient to explain:
   - stable surface identity
   - name-resolution table reference
   - semantic trait/profile reference
   - gating/profile reference

Minimum result fields:
1. `SemanticPlan`
2. semantic diagnostics and unsupported-lane markers
3. execution-profile summary
4. helper-environment profile summary
5. availability/gating summary where formula admission or runtime capability depends on catalog/profile/provider state
6. typed callable-carrier summary where semantically callable results must remain recoverable in replay or later dispatch, including callable values preserved through adopted defined-name context in the current local floor

Result rule:
1. `compile_semantic_plan` must preserve the difference between:
   - edit rejection before canonical artifact adoption,
   - accepted formula text with bind-time unresolved-name or unsupported-lane diagnostics,
   - runtime capability/provider outcomes that only become knowable later.
2. when a formula is accepted into the canonical artifact ladder but still has unresolved-name meaning, OxFml preserves that classification and OxFunc remains authoritative for the eventual `#NAME?` value payload and related value-universe behavior.

### 4.5 `EvaluateRequest` -> `AcceptedCandidateResult | RejectRecord`
Minimum request fields:
1. `SemanticPlan`
2. explicit evaluation context
3. host-query capability view where required
4. snapshot, token, and capability fence members
5. replay-correlation ids

Minimum result rule:
1. evaluation returns an accepted candidate or a typed reject,
2. evaluation does not publish.
3. evaluation is not the place where edit rejection is decided; edit rejection belongs to earlier parse/bind/plan acceptance rules.

### 4.6 `CommitRequest` -> `CommitBundle | RejectRecord`
Minimum request fields:
1. `AcceptedCandidateResult`
2. commit-attempt identity
3. accept-or-reject fence basis

Minimum result rule:
1. commit returns a published bundle or a typed no-publish reject,
2. commit consequences remain distinct from evaluator success.

## 5. Optional Repository and Runtime Surfaces
The first implementation may also expose optional services such as:
1. `SyntaxRepository`
2. `BindRepository`
3. `SemanticPlanRepository`
4. `EvaluationSessionService`
5. `TraceCaptureService`
6. `LibraryContextProvider`
7. host capability providers such as:
   - `HostInfoProvider`
   - `RtdProvider`

Working rule:
1. these services may own caches, indexes, or runtime handles,
2. they must not be the only explanation of semantic truth,
3. all service-backed results must remain reproducible through the canonical transform surface.

## 6. Current Handle Vocabulary
If handle-based services exist, the first handle families should remain narrow:
1. syntax artifact handle
2. bound-formula handle
3. semantic-plan handle
4. session handle
5. trace or replay handle

Working rule:
1. handles are operational conveniences,
2. canonical artifacts remain the semantic baseline,
3. handles must be mappable back to artifact identities and version keys.

## 7. Single-Formula Proving Host Surface
OxFml should also expose a small proving-host helper surface for the ladder defined in `OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`.

The current intended operations are:
1. create or refresh single-formula host state
2. update defined-name inputs
3. trigger full recalc
4. retrieve candidate, commit, reject, and trace outputs
5. run an empirical-oracle scenario through the same proving-host surface

Working rule:
1. this surface is a proving host, not a second scheduler,
2. it should exercise the same canonical transform and seam outputs,
3. it should not require OxCalc multi-node infrastructure.

## 8. Execution-Profile and Concurrency Surface
The public surface must leave room for scheduler-relevant execution metadata from the start.

The first exposed shape should allow:
1. formula-level execution profile summary from `SemanticPlan`
2. helper-environment profile summary from `SemanticPlan`
3. call-site or operator-level restrictions where needed
4. explicit flags for host-query, thread-affine, async, single-flight, or serial-only lanes

Working rule:
1. OxFml surfaces execution restrictions,
2. OxCalc or a host consumes them for scheduling,
3. OxFml also surfaces helper-environment shape where downstream semantic coordination depends on it,
4. OxFml does not become the scheduler-policy owner.

## 8A. First Shared Typed Context/Query Bundle
For the current covered OxFunc scope, OxFml should be able to consume a first shared typed context/query bundle without reopening broad seam theory.

Current first-pass families are:
1. `ReferenceResolver`
2. `HostInfoProvider`
   - `query_cell_info(...)`
   - `query_info(...)`
   - `query_formula_text(reference)`
   - `query_sheet_index(CurrentSheet | Reference | SheetNameText)`
   - `query_sheet_count(Workbook | Reference)`
   - `query_aggregate_reference_context(reference)`
   - `query_width_conversion_mode(function)`
   - `query_translate(request)`
3. `RtdProvider`
   - `RtdRequest { prog_id, server_name, topic_strings }`
   - `RtdProviderResult::{ Value, NoValueYet, CapabilityDenied, ConnectionFailed, ProviderError }`
4. host-supplied scalar context providers:
   - `now_serial`
   - `random_value`
   - `LocaleFormatContext`

Working rule:
1. OxFml prefers capability-scoped typed providers over raw host objects,
2. the current OxFunc query names and result partitioning are acceptable as the first freeze candidate,
3. exact names may still be merged or split later if a concrete consumer mismatch appears,
4. any such merge/split must preserve the same semantic families,
5. the remaining clarification is now implementation-facing rather than semantic: whether actual OxFml consumer modeling exposes a concrete need to merge or split any first-pass family.

## 8B. First Shared Returned Value Surface
For the current covered scope, the first returned-value split should remain explicit.

Current first-freeze candidate:
1. ordinary value
2. `ValueWithPresentation`
3. typed host/provider outcome projection

Working rule:
1. OxFml currently accepts that explicit split as the first shared freeze candidate,
2. richer publication-facing or display-facing factoring should not be invented until a concrete mismatch appears,
3. publication-aware value hints remain distinct from typed host/provider outcome projection,
4. the remaining clarification is now implementation-facing rather than semantic: whether actual return-carrier freezing exposes a concrete need to refactor the current first-pass split.

## 8C. First Runtime Library-Context Consumer Model
For the current covered OxFunc scope, OxFml should also model a real runtime consumer for built-in catalog truth rather than rely only on export-file pinning.

Current first-pass direction:
1. `LibraryContextProvider`
   - `current_snapshot()`
   - `snapshot_by_identity(snapshot_ref)`
   - `lookup_surface(snapshot_ref, surface_key)`
2. immutable `LibraryContextSnapshot`
3. runtime-consumable `LibraryContextEntry`
4. explicit snapshot identity and generation on parse, bind, and semantic-plan artifacts

Working rule:
1. OxFml prefers a cleaner runtime-only consumer shape plus an explicit CSV/export mapping layer,
2. the committed `W044` export remains the immediate pinning and mismatch artifact,
3. runtime registration or removal must yield explicit new snapshot generations rather than mutate a pinned snapshot in place,
4. snapshot drift must not be hidden inside evaluation or session execution,
5. the remaining clarification is now implementation-facing rather than semantic: whether actual OxFml consumer modeling exposes any runtime-only versus export-mapping mismatch that forces a narrower shape.

## 9. Current Preferred Packaging Shape
The current preferred packaging shape is:
1. a stateless canonical-core module set,
2. optional repository/service modules,
3. an FEC/F3E session service layer,
4. an optional proving-host helper layer.

This is the API reflection of the hybrid implementation baseline.

## 10. Deferred Decisions
The following remain open:
1. exact trait/interface/function names,
2. whether the direct transform surface is free-function based or service-object based,
3. whether red projection is publicly exposed or kept as an internal helper surface,
4. whether proving-host helpers live in the main library or a sibling support package,
5. exact error/result carrier style for language bindings,
6. the smallest honest library-context snapshot shape beyond the current local minimum field floor,
7. the final callable-value carrier beyond the current typed minimum plus replay-summary floor.
8. whether the first typed context/query bundle needs a narrower capability-family merge or split after initial consumer modeling.
9. whether the runtime `LibraryContextProvider` model should mirror the CSV artifact closely or use a cleaner runtime-only shape plus explicit mapping layer.
10. exact shared field names for the first frozen typed context/query bundle and returned-value split.

## 11. Workset Implications
Current expected primary owners:
1. `W002`: parser, red-projection, bind, and artifact-surface narrowing
2. `W003`: semantic-plan, evaluation, and execution-profile surface narrowing
3. `W004`: session and commit service surface narrowing
4. `W008`: single-formula proving-host helper surface narrowing
5. `W041`: typed context and query bundle freeze
6. `W042`: return surface and publication-hint freeze
7. `W043`: runtime library-context provider consumer model

## 12. Working Rule
Implementation should treat this document as the current public-surface baseline:
1. direct transforms first,
2. optional services second,
3. publication and reject consequences typed,
4. proving-host helpers narrow and explicit.
