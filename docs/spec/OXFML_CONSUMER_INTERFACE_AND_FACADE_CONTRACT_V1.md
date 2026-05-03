# OxFml Consumer Interface And Facade Contract V1

## 1. Purpose And Status
This document defines the OxFml-owned consumer-facing interface contract for the
next architecture phase.

It exists to turn the current broad redesign direction into one concrete packet
that downstream users can compare against and implement toward.

Status:
1. canonical OxFml consumer-facing interface contract for `OxFml_V1`,
2. the implemented and documented consumer-facing architecture packet for `W054`,
3. intended to replace ad hoc consumer-specific interpretations of the facade
   direction,
4. compatible with the frozen OxFml <-> OxFunc seam,
5. frozen `OxFml_V1` consumer-seam packet from the OxFml side for downstream
   consumer migration,
6. the named facade set has landed in code:
   - runtime facade is implemented,
   - editor facade is implemented,
   - replay facade is implemented,
   - editor function-help has a canonical packet at the facade boundary,
   - replay projection carries fixture-family and retained-witness metadata,
   - ordinary public consumer entry is `consumer::runtime`,
     `consumer::editor`, and `consumer::replay`.

Read together with:
1. `OXFML_CONSUMER_INTERFACE_REARCHITECTURE_PLAN.md`
2. `OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`
3. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
4. `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
5. `OXFML_DNA_ONECALC_DOWNSTREAM_CONSUMER_CONTRACT.md`
6. `OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
7. `formula-language/OXFML_EDITOR_LANGUAGE_SERVICE_AND_HOST_INTEGRATION_PLAN.md`
8. `formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`

## 2. Design Inputs And Weighting
The current consumer contract is synthesized from three downstream consumers:
1. `DNA OneCalc`
2. `OxCalc`
3. `OxReplay`

Current weighting rule:
1. `DNA OneCalc` is weighted highest for present-day consumer experience because
   it is the most advanced direct user of OxFml,
2. `OxCalc` is weighted next for runtime and coordinator-facing durability
   because it is expected to become the most important downstream runtime
   consumer,
3. `OxReplay` is weighted as the replay-packaging and metadata-preservation
   consumer, not as the primary runtime driver.

Working interpretation:
1. runtime and editor packaging should primarily solve the real assembly burden
   visible in `DNA OneCalc`,
2. runtime results and environment structure must still preserve the stronger
   runtime/result truth `OxCalc` needs,
3. replay projection must preserve the metadata and identity truth `OxReplay`
   needs without becoming the dominant design driver for runtime entry.

Current consumer-side disposition:
1. `OxReplay` is treated as aligned on the replay-facing direction and no longer
   the main source of shape pressure,
2. `OxCalc` accepts this packet as the implementation-driving consumer packet
   and current runtime-facing seam packet,
3. `DNA OneCalc` remains the primary experiential driver and its current note is
   treated as the most important acknowledgment of the target shape for runtime,
   editor, and replay packaging.

## 2A. Freeze Rule
This packet is now frozen as the `OxFml_V1` consumer-facing seam target for:
1. `DNA OneCalc`
2. `OxCalc`
3. `OxReplay`

Working meaning:
1. downstream implementation and migration work should target this packet,
2. the broad architecture direction is no longer open for `W054`,
3. any later issue belongs to a new bounded follow-on workset rather than to
   `W054`,
4. any such future follow-on must preserve the frozen OxFml <-> OxFunc seam.

Explicit reopen rule:
1. do not reopen `W054`,
2. prefer later bounded follow-on worksets over reopening the consumer
   architecture packet,
3. do not use implementation churn alone as a reason to widen semantic scope,
4. any future issue must be recorded outside `W054`.

## 3. Hard Boundary: Frozen OxFml/OxFunc Seam
This contract does not reopen the frozen OxFml/OxFunc seam.

Hard rules:
1. do not rename, merge, or reinterpret frozen OxFml/OxFunc packet families,
2. do not move built-in function truth, runtime library-context truth, returned
   value class truth, callable carrier truth, or registered-external catalog
   truth out of their current owners,
3. do not create a competing consumer vocabulary over frozen shared packet
   names when those names already exist,
4. do not hide provider-plus-pin or typed query family truth behind opaque
   mutable host state.

Allowed changes:
1. consumer-facing request/result/environment packaging,
2. public module packaging and facade layering,
3. migration away from flat crate-root integration,
4. temporary internal transition helpers only where they materially reduce
   refactor risk,
5. stronger OxFml-owned canonical consumer outputs where OxFml already owns the
   semantic meaning.

## 4. Architecture Layers
The intended OxFml public architecture now has two explicit consumer-facing
layers plus a non-contract implementation substrate.

### 4.1 Canonical substrate
This remains the semantic substrate and advanced entry surface:
1. parse/bind/semantic-plan/evaluate/commit transforms,
2. canonical artifact families,
3. typed query bundle and runtime provider model,
4. typed returned-value surface,
5. replay-safe identity and fence-bearing artifacts.

### 4.2 Consumer facades
These become the preferred long-term user entry surfaces:
1. runtime facade,
2. editor facade,
3. replay facade.

### 4.3 Internal transition helpers
Temporary transition helpers may exist inside OxFml while the refactor is in
flight, but they are not part of the intended consumer contract.

Rules:
1. they are internal refactoring aids, not endorsed downstream entry surfaces,
2. they must not define the public architecture,
3. they should be removed once the corresponding facade-native path is strong
   enough.

Current packaging rule:
1. ordinary consumers should use explicit `consumer::runtime`,
   `consumer::editor`, and `consumer::replay` entry paths,
2. crate-root facade flattening is no longer the intended discovery model,
3. historical direct-consumer substrate is hidden from ordinary documentation
   and should be treated as transitional implementation substrate only.

## 5. Primary Consumer Contract
The first stable consumer contract is:
1. explicit stable environment objects,
2. explicit immutable or effectively immutable document/request objects,
3. explicit canonical result objects,
4. explicit provider-plus-pin library-context selection,
5. explicit typed query bundle carriage,
6. explicit replay and provenance handles,
7. no requirement for ordinary consumers to manually assemble low-level OxFml
   runtime chains.

## 6. Runtime Facade Contract
The runtime facade is the first implementation priority.

### 6.1 RuntimeEnvironment
`RuntimeEnvironment` is a stable reusable semantic environment object.

It should own:
1. `LibraryContextProvider`
2. pinned `LibraryContextSnapshotRef` or explicit current-snapshot selection
3. stable structure-context identity
4. stable direct-binding, defined-name, and table-context inputs that are not
   per-request
5. stable registered-external provider/controller inputs where the host lane
   requires them
6. stable execution-family or backend selection where OxFml already exposes it
7. stable replay/capture policy only where that policy is semantic or packet
   shaping, not UI policy

Current implementation note:
1. the first landed runtime/editor slices now centralize this ownership through
   facade-native environment builders over provider-plus-pin state
2. consumer-facing code should prefer those builders or a derived
   `PinnedLibraryContextView`, not repeated raw provider/pin/snapshot triples

It must not:
1. make provider or pin selection implicit,
2. silently execute against provider current snapshot when a pin was selected,
3. absorb host UI or persistence policy,
4. redefine frozen OxFml/OxFunc packet families.

### 6.2 RuntimeFormulaRequest
`RuntimeFormulaRequest` is the per-execution request object.

It should carry:
1. `FormulaSourceRecord`
2. formula channel kind
3. optional caller-context packet for the first admitted caller-sensitive lane:
   - `caller_anchor`
   - formula-channel and address-mode-sensitive context
   - structure-context identity
4. explicit execution mode or trigger kind
5. per-request `TypedContextQueryBundle`
6. per-request volatile inputs such as `now_serial` and `random_value`
7. explicit-input overrides or recalc-cause fields where they affect semantic
   behavior
8. optional per-request direct-cell or probe-only context where
   the selected lane requires it
9. requested artifact depth or replay-projection policy where admitted

Working rule:
1. provider-plus-pin selection stays on `RuntimeEnvironment`,
2. volatile and query-bundle inputs stay explicit on the request,
3. caller-context carriage for the first TreeCalc-relative subset remains
   explicit rather than hidden behind ambient host state.

### 6.3 RuntimeFormulaResult
`RuntimeFormulaResult` is the canonical consumer-facing runtime result.

It should return:
1. canonical diagnostics
2. semantic-plan identity and execution-contract summary
3. `ReturnedValueSurface`
4. evaluation output summary
5. candidate / commit / reject truth
6. explicit stable correlation subset where present:
   - `candidate_result_id`
   - `commit_attempt_id`
   - `reject_record_id`
   - optional fence snapshot references
7. trace and replay-correlation handles where admitted
8. runtime-effect and capability-sensitive facts in consumer-oriented but still
   canonical form
9. surfaced first-slice coordinator-relevant fact families remain reachable:
   - execution-restriction-sensitive surfaced facts
   - capability-sensitive surfaced facts
   - topology/effect fact refs where present
   - dependency-sensitive surfaced facts where publication or invalidation
     meaning depends on them
10. additive OxFml-owned `comparison_views` when `verification_publication_surface`
    is present and OxFml can state the admitted comparison families directly
    from its own publication facts without downstream reinterpretation
11. explicit caller-context dependence signal for the first admitted
    caller-sensitive subset where OxFml can determine that dependence honestly
12. enough stable identity to support retained evidence, compare, and replay

It must preserve:
1. candidate versus commit separation,
2. reject-is-no-publish semantics,
3. `value_delta`, `shape_delta`, `topology_delta`, and optional
   `format_delta` / `display_delta`,
4. typed host/provider outcome distinctions,
5. pinned library-context identity,
6. the current consume-now packet truth without implying closure of still
   narrower `W026` residuals beyond the admitted slice.

### 6.4 RuntimeSessionFacade
`RuntimeSessionFacade` is the stable repeated-execution surface.

It should support:
1. one-shot direct execution,
2. driven single-formula or repeated execution over a stable environment,
3. session prepare/open/execute/commit lifecycle where needed,
4. replay capture projection from the same runtime/session result family.

Working rule:
1. this facade is the preferred long-term replacement for consumers manually
   managing `SingleFormulaHost` and adjacent packet choreography,
2. session and one-shot execution must still use the same canonical runtime
   packet truth,
3. some implementation steps may temporarily use a mixed facade-plus-lower-level
   session phase while the refactor lands, but the endpoint is that direct
   session-lifecycle consumers migrate onto this facade.

## 7. Editor Facade Contract
The editor facade is the second implementation priority.

### 7.1 EditorEnvironment
`EditorEnvironment` is the stable semantic editing environment.

It should own:
1. `BindContext`
2. structure-context identity
3. `LibraryContextProvider`
4. pinned `LibraryContextSnapshotRef`
5. visible-name and table-context inputs where needed
6. any carrier-restriction mode that affects formula-language legality

Current implementation note:
1. the first landed editor slice now uses the same facade-native
   provider-plus-pin environment model as runtime
2. completion/help request construction derives a `PinnedLibraryContextView`
   from that environment rather than rebuilding pinning inputs at each callsite

### 7.2 EditorDocument
`EditorDocument` is the immutable or snapshot-oriented editing object.

It should carry:
1. formula source identity and text
2. immutable current syntax/bind/plan state where available
3. document/version identity
4. cursor or selection context only where the interaction requires it

### 7.3 EditorEditService
`EditorEditService` should provide:
1. apply edit
2. derive diagnostics
3. derive completion
4. derive signature help
5. derive canonical function-help result at the OxFml consumer boundary,
   including:
   - lookup key
   - display name
   - display signature
   - active argument index
   - validity and availability classification
   - provisionality where relevant
6. derive carrier-specific validation where OxFml intends those validations to
   be part of the consumer-facing editor surface

Current floor note:
1. the first editor-facade slice now returns a canonical `FunctionHelpPacket`
   directly on interaction results rather than only exposing a lookup request.

Current carrier-validation rule:
1. conditional-format and other specialized carrier validations should not be
   left ambiguous,
2. for the current consumer contract, they remain OxFml-owned validations,
3. they may live either as specialized editor-service operations or as clearly
   adjacent low-level semantic operations, but must be documented as intentional
   rather than temporary spillover.

### 7.4 EditorInteractionResult
`EditorInteractionResult` should return:
1. updated document state
2. diagnostics
3. text-change and incremental reuse facts
4. completion and signature-help results
5. canonical function-help result where available
6. stable context needed to reissue the next editor interaction honestly

### 7.4A Live Diagnostic Packet Precision
Live diagnostics are the editor-facing projection of syntax, bind,
semantic-plan, and later runtime facts. Each diagnostic packet should carry:
1. a stable diagnostic identifier for this formula/version projection,
2. severity,
3. stage,
4. human-readable message,
5. stable diagnostic code,
6. primary source span,
7. optional related source spans,
8. optional worksheet-visible error class when the formula-language consequence
   is already known.

Current stable code floor:
1. `syntax` for generic parse diagnostics pending narrower parse taxonomy,
2. `unknown_name` for unresolved bare identifiers/names,
3. `unknown_function` for syntactically valid function-call surfaces without
   registered metadata or admitted function identity,
4. `known_symbol_not_callable` for a symbol that resolves but is not invocable
   in the active binding context,
5. `function_arity_mismatch` for catalog-known calls rejected by the authoring
   arity boundary,
6. `function_gated_or_unavailable` for catalog/context-known function surfaces
   preserved as unavailable or gated rather than unknown,
7. `structured_reference_unresolved` for table/structured-reference lookup
   failures,
8. `reference_invalid_or_deferred` for reference-validity cases where the host
   or workbook profile must provide final truth.

Unknown function calls are not parse failures. They preserve call shape, may
still evaluate to worksheet `#NAME?`, and should point their primary span at the
callee token rather than the whole formula. Unresolved bare identifiers remain
bind-stage facts, but their diagnostic packet shape should align with unknown
function diagnostics so editor hosts can render both without host-side symbol
inference.

Working rule:
1. editor consumers should not need to stitch parse/bind/plan/completion/help
   calls together by hand,
2. completion validity, diagnostics, and help truth remain OxFml-owned at the
   consumer boundary,
3. `EditorDocument` should be treated as immutable or snapshot-oriented by
   design,
4. intelligent completion remains non-canonical until re-entry through the
   ordinary edit path.

## 8. Replay Facade Contract
The replay facade is the third implementation priority.

### 8.1 ReplayProjectionRequest
`ReplayProjectionRequest` should carry:
1. runtime result, host result, or session result
2. requested projection family
3. sidecar or retention policy
4. requested metadata breadth where the source packet already supports it

### 8.2 ReplayProjectionService
`ReplayProjectionService` should provide:
1. runtime-result to replay projection
2. session-result to replay projection
3. deterministic fixture-family to replay projection where admitted
4. projection of source-case-id to shared-scenario alias metadata where that
   metadata exists

### 8.3 ReplayProjectionResult
`ReplayProjectionResult` should preserve:
1. source case id
2. shared scenario alias when present
3. source schema and source artifact family
4. pinned `LibraryContextSnapshotRef` when present
5. replay-relevant fence members when present
6. registry bindings and capability floor
7. lifecycle metadata when applicable
8. canonical replay envelope refs and sidecar refs
9. adapter-declared `comparison_views` when OxFml can state family-specific replay comparison facts without downstream reinterpretation

Current floor note:
1. the first replay-facade slice now explicitly carries:
   - source fixture family
   - source schema id
   - source case ids
   - registry pin
   - retained-witness id
   - witness lifecycle state
   - retention policy id
   - reduction/source refs
   for fixture-family and retained-witness projection.
2. the active `W056` follow-on now additionally allows runtime-result and first-host-capture projection to publish machine-readable `comparison_views` for the current XML verification lane.
3. the currently admitted view families are:
   - `comparison_value`
   - `visible_value_text`
   - `effective_display_text`
   - `formatting_view`
   - `conditional_formatting_view`
4. these families are additive replay-facing views over OxFml-owned publication truth; they do not replace `verification_publication_surface` and they do not widen replay capability claims by themselves.
5. the admitted SpreadsheetML XML verification lane may publish comparison-oriented `formatting_view` and `conditional_formatting_view` envelopes that are narrower than the underlying `VerificationPublicationSurface`, provided OxFml can state those envelopes directly from its own publication facts.

Working rule:
1. replay packaging is additive over OxFml meaning,
2. replay consumers should not need broad internal artifact drilling for
   ordinary replay integration,
3. replay projection should publish machine-readable metadata where downstream
   replay hosts currently maintain private mappings,
4. the current preferred alias-publication rule is to embed shared-scenario
   alias data directly in projection results, with optional dedicated sidecars
   later only if batching or corpus tooling needs them,
5. `comparison_views` must remain adapter-declared facts rather than downstream convenience strings,
6. comparison-oriented family envelopes may be profile-specific where that keeps cross-lane comparison honest without widening semantic claims,
7. the first post-FEC family targeted for preferred replay-facade projection
   should be session lifecycle.

## 9. Current-Surface To Target-Surface Migration

### 9.1 DNA OneCalc
DNA OneCalc should migrate in this order:
1. runtime execution
2. driven/session execution
3. edit and diagnostics
4. completion and function help
5. replay-aware capture and retained evidence projection

Current pain to remove:
1. manual `SingleFormulaHost` assembly
2. manual `InMemoryLibraryContextProvider` and `TypedContextQueryBundle`
   choreography
3. manual edit/completion/help stitching
4. replay capture through broad helper or artifact dependence

Current-to-target mapping:
1. current direct runtime entry:
   - `SingleFormulaHost`
   - `recalc_with_interfaces(...)`
   - local `TypedContextQueryBundle` assembly
   -> target:
   - `RuntimeEnvironment`
   - `RuntimeFormulaRequest`
   - `RuntimeFormulaResult`
2. current driven host and retained-run entry:
   - local driven-host wrapper over `SingleFormulaHost`
   -> target:
   - `RuntimeSessionFacade`
3. current editor entry:
   - `FormulaEditRequest` / `FormulaEditResult`
   - completion helpers
   - function-help packet building and publication
   -> target:
   - `EditorEnvironment`
   - `EditorDocument`
   - `EditorEditService`
   - `EditorInteractionResult`
4. current replay capture:
   - host output and replay helper projection
   -> target:
   - `ReplayProjectionRequest`
   - `ReplayProjectionService`
   - `ReplayProjectionResult`

### 9.2 OxCalc
OxCalc should migrate runtime entry first.

Primary migration goal:
1. replace direct consumption of proving-host-flavored entrypoints with the
   runtime facade while preserving current host/runtime packet truth.

Primary constraints:
1. keep coordinator policy above OxFml,
2. keep candidate / commit / reject / trace truth visible,
3. keep provider-plus-pin selection explicit,
4. do not let packaging imply closure of still narrower `W026` residuals.

Current-to-target mapping:
1. current host/coordinator entry:
   - `SingleFormulaHost`
   - `HostRecalcOutput`
   - direct session/provider packet composition
   -> target:
   - `RuntimeEnvironment`
   - `RuntimeFormulaRequest`
   - `RuntimeFormulaResult`
2. current repeated/runtime lifecycle use:
   - direct session-lifecycle and host packet usage
   -> target:
   - `RuntimeSessionFacade`
3. current replay-capture projection:
   - `FirstHostReplayCapturePacket`
   - host-output-driven replay projection
   -> target:
   - replay projection derived directly from runtime/session result families
     through `ReplayProjectionService`

### 9.3 OxReplay
OxReplay should migrate to replay projection service usage.

Primary migration goal:
1. stop depending on private fixture alias maps and broad helper assumptions,
2. consume machine-readable replay projection metadata and stable projection
   request/result shapes.

Current-to-target mapping:
1. current intake:
   - fixture-family assumptions
   - local alias mapping
   - broad helper-path projection
   -> target:
   - `ReplayProjectionRequest`
   - `ReplayProjectionService`
   - `ReplayProjectionResult`
2. current metadata recovery:
   - source-case mapping and pin metadata inferred locally
   -> target:
   - machine-readable alias, pin, fence, registry, and lifecycle fields emitted
     directly by OxFml projection results
3. current XML verification comparison:
   - local view-family recovery or partial host-local formatting reinterpretation
   -> target:
   - `ReplayProjectionResult.comparison_views` emitted by OxFml when `verification_publication_surface` facts are present

## 10. Current Implementation Reality
Historical root exports and older host/editor/replay helper surfaces still
exist in the codebase today, but they are no longer the intended consumer
contract.

Working rule:
1. downstream planning should target the facade contract now,
2. existing historical surfaces should be treated as implementation substrate
   while the refactor continues,
3. no consumer should build a second long-lived wrapper vocabulary over frozen
   OxFml/OxFunc packets while waiting for fuller facade coverage,
4. the ordinary discovery path is now explicit `consumer::runtime`,
   `consumer::editor`, and `consumer::replay`, not flat crate-root facade
   discovery.

## 11. Implementation Contract For W054
`W054` should now be read as an executed redesign/refactor program:
1. runtime facade implemented,
2. editor facade implemented,
3. replay facade implemented,
4. public consumer packaging reset to facade-first,
5. downstream coordination notes updated to the landed public surface,
6. concrete execution history remains recorded in
   `OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`.

Current landed code note:
1. `crates/oxfml_core/src/consumer/runtime` exposes:
   - `RuntimeEnvironment`
   - `RuntimeFormulaRequest`
   - `RuntimeFormulaResult`
   - `RuntimeSessionFacade`
2. `crates/oxfml_core/src/consumer/editor` exposes:
   - `EditorEnvironment`
   - `EditorDocument`
   - `EditorEditService`
   - `EditorInteractionResult`
3. `crates/oxfml_core/src/consumer/replay` exposes:
   - `ReplayProjectionRequest`
   - `ReplayProjectionService`
   - `ReplayProjectionResult`
4. public `substrate::...` access is gone from the library surface,
5. remaining advanced access is explicit `test_support` only.

The key implementation test is:
1. ordinary consumers should be able to use OxFml through stable environment,
   request, result, and service objects,
2. without manually composing low-level semantic packet flows,
3. while still preserving pinned runtime truth, canonical result truth, and
   replay/provenance truth.

## 12. W054 Scope Boundary
`W054` includes:
1. the consumer-facing architecture redesign,
2. the facade module implementation,
3. the facade-first public packaging reset,
4. the consumer-facing documentation and notes sync for that architecture.

`W054` does not include:
1. later replay breadth expansion beyond the landed `ReplayProjection*` floor,
2. further optional narrowing of explicit `test_support` helpers,
3. new downstream uptake work inside other repos,
4. any future refinement that would require a new workset name.

## 13. Working Rule
Use this document as the current OxFml-owned consumer-interface packet for:
1. public packaging redesign,
2. downstream migration planning,
3. facade implementation sequencing,
4. consumer-note synchronization.

Do not use it to:
1. reopen the frozen OxFml/OxFunc seam,
2. redefine OxCalc coordinator policy,
3. redefine OxReplay adapter semantics,
4. claim that the facade modules already exist today.

Current freeze rule:
1. this packet is now the `OxFml_V1` consumer contract,
2. downstream repos should implement against it,
3. `W054` itself is not a lane for further broadening,
4. any future change must be explicit, bounded, and outside `W054`.
