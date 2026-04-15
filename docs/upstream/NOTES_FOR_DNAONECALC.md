# Notes for DNAOneCalc

Status: `active`
Owner lane: `OxFml`
Relationship: outbound consumer-architecture and migration note from OxFml for the next DNA OneCalc integration round

## 1. Purpose
Record the current OxFml-side plan for migrating DNA OneCalc onto the new consumer-facing OxFml architecture without reopening the frozen OxFml <-> OxFunc seam.

This note is an OxFml-owned design and migration input.
It is not a bilateral freeze packet.

## 2. Core Message
OxFml has now turned the earlier facade direction into one canonical consumer
interface packet:
1. `docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`

Navigation packet for DNA OneCalc:
1. start with `docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
2. then read `docs/spec/OXFML_DNA_ONECALC_DOWNSTREAM_CONSUMER_CONTRACT.md`
3. then read `docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
4. for editor-specific behavior, use `docs/spec/formula-language/OXFML_EDITOR_LANGUAGE_SERVICE_AND_HOST_INTEGRATION_PLAN.md`

Ordinary `OxFml_V1` entry surface for DNA OneCalc:
1. `oxfml_core::consumer::runtime`
2. `oxfml_core::consumer::editor`
3. `oxfml_core::consumer::replay`

OxFml intends to move DNA OneCalc toward a cleaner consumer-facing integration model:
1. runtime facade for execution,
2. editor facade for edit/diagnostic/completion/help interactions,
3. replay facade for replay-aware capture and projection.

Current OxFml reading:
1. DNA OneCalc should plan against the new facade contract now rather than
   deepening long-lived dependence on historical OxFml host, language-service,
   and replay entry surfaces,
2. DNA OneCalc should not build a competing wrapper vocabulary around frozen OxFml/OxFunc packet families,
3. DNA OneCalc should migrate onto the landed facades as the preferred integration shape,
4. the new canonical packet deliberately weights DNA OneCalc's direct consumer
   experience highest, especially around runtime assembly burden, editor
   stitching burden, and function-help ownership at the consumer boundary,
5. OxFml now treats the consumer contract as the landed `OxFml_V1` surface for
   downstream implementation.

Current OxFml refinement after the latest OneCalc input:
1. `RuntimeEnvironment` is now explicitly provider-plus-pin and reusable rather
   than just a thin entry wrapper,
2. `RuntimeFormulaRequest` now explicitly carries the home for typed query
   bundle and volatile inputs,
3. `EditorDocument` is now explicitly snapshot-oriented,
4. the editor contract now names canonical function-help results at the OxFml
   consumer boundary,
5. carrier-validation placement is now treated as an intentional part of the
   consumer contract rather than unresolved spillover,
6. `RuntimeFormulaRequest` now also carries optional `VerificationPublicationContext`
   for XML-backed or other comparison-heavy verification lanes,
7. `RuntimeFormulaResult`, the first-host replay capture packet, and
   `ReplayProjectionResult` now carry `VerificationPublicationSurface` as the
   current comparison-friendly export packet for visible value, effective
   display, format/style context, and host-supplied conditional-formatting
   context.
8. `RuntimeFormulaResult` now also carries additive OxFml-owned
   `comparison_views` for the admitted comparison family set when
   `verification_publication_surface` facts are present; replay projection
   carries the same family set for retained or replay-facing comparison.

## 3. Frozen Seam Constraint
The consumer-facing redesign does not reopen the OxFml <-> OxFunc seam.

DNA OneCalc should assume the following remain frozen while the migration happens:
1. typed context/query bundle families already frozen with OxFunc,
2. returned-value surface split already frozen with OxFunc,
3. runtime library-context provider-plus-pin model already frozen with OxFunc,
4. registered-external direct packet set for worksheet `CALL` / `REGISTER.ID` already frozen with OxFunc,
5. minimum callable carrier already frozen with OxFunc.

Working rule:
1. DNA OneCalc may change how it enters OxFml,
2. DNA OneCalc must not fork, rename, or reinterpret the frozen OxFml/OxFunc packet and carrier vocabulary.

## 4. Target Surface Versus Current Implementation Reality
Target surface after `W054` is:
1. runtime facade,
2. editor facade,
3. replay facade.

Current implementation reality:
1. the facade modules now exist in OxFml code,
2. ordinary downstream integration should use those facade modules,
3. historical low-level surfaces that still exist are implementation substrate
   or explicit `test_support`, not the intended downstream contract.

## 5. Detailed Migration Plan For DNA OneCalc

### Phase A: Vocabulary And Boundary Alignment
DNA OneCalc should align its local vocabulary to the existing OxFml packet names first.

Required discipline:
1. use upstream names where OxFml already has stable packet and field names,
2. keep OneCalc-local classification names only as local organizational labels,
3. avoid introducing a second runtime/editor/replay naming layer that would have to be unwound later.

Immediate DNA OneCalc-side expectation:
1. continue using `FormulaSourceRecord`, `TypedContextQueryBundle`, `LibraryContextProvider`, `LibraryContextSnapshotRef`, `ReturnedValueSurface`, `AcceptedCandidateResult`, `CommitBundle`, `RejectRecord`, `FormulaEditRequest`, `FormulaEditResult`, and related current upstream names directly,
2. for XML-backed verification, also use `VerificationPublicationContext` and `VerificationPublicationSurface` directly rather than introducing a OneCalc-local comparison wrapper over raw runtime/replay artifacts.

### Phase B: Runtime Facade Uptake
When the first runtime facade lands in OxFml, DNA OneCalc should migrate execution entrypoints in this order:
1. ordinary direct-host execution,
2. explicit-input execution,
3. reference/table/registered-external probe packets,
4. session-oriented execution where still relevant locally.

Runtime migration goals:
1. move OneCalc host code away from manual assembly of parse/bind/plan/evaluate/commit calls,
2. keep provider-plus-pin runtime catalog selection explicit,
3. preserve current host-specific policy above the OxFml runtime environment rather than inside it,
4. eliminate routine manual assembly of `SingleFormulaHost`,
   `InMemoryLibraryContextProvider`, and `TypedContextQueryBundle` in ordinary
   consumer flows,
5. feed XML-backed verification through `VerificationPublicationContext` on the
   runtime request rather than treating formatting/style comparison as a
   downstream-only reconstruction problem.

Expected OxFml runtime facade shape:
1. `RuntimeEnvironment`
2. `RuntimeFormulaRequest`
3. `RuntimeFormulaResult`
4. `RuntimeSessionFacade`

### Phase C: Editor Facade Uptake
When the first editor facade lands in OxFml, DNA OneCalc should migrate editor integration in this order:
1. immutable edit application and diagnostics,
2. completion,
3. signature help and function-help lookup request building,
4. intelligent-completion context consumption.

Editor migration goals:
1. stop manually stitching parse/bind/plan/editor packet steps together in DNA OneCalc,
2. keep canonical diagnostics and completion validity in OxFml,
3. move the consumer boundary toward canonical OxFml function-help results
   rather than local OxFml request-building plus OxFunc metadata stitching,
4. keep OxFunc-owned help payload truth out of DNA OneCalc-local semantic code.

Expected OxFml editor facade shape:
1. `EditorEnvironment`
2. `EditorDocument`
3. `EditorEditService`
4. `EditorInteractionResult`

### Phase D: Replay Facade Uptake
When the first replay facade lands in OxFml, DNA OneCalc should migrate replay-aware capture in this order:
1. ordinary host replay capture,
2. retained local scenario evidence,
3. later richer sidecar/projection use only where genuinely needed.

Replay migration goals:
1. move away from direct dependence on broad internal artifact families as the default host integration shape,
2. preserve the same semantic artifact truth under a narrower replay projection service,
3. keep replay packaging additive rather than semantic,
4. make replay-meaningful provenance such as source-case aliases, snapshot pins,
   and registry metadata explicit in machine-readable projection results,
5. consume `verification_publication_surface` from replay projection instead of
   inferring effective display or formatting view from raw visible-value output.
6. for ordinary direct-host XML verification, prefer
   `RuntimeFormulaResult.comparison_views` when the downstream comparison path
   wants the admitted family-oriented envelope rather than the richer
   `VerificationPublicationSurface`.

Expected OxFml replay facade shape:
1. `ReplayProjectionRequest`
2. `ReplayProjectionService`
3. `ReplayProjectionResult`

## 6. Questions DNA OneCalc Should Answer Back
The next useful DNA OneCalc reply should answer:
1. which current OneCalc integration points are most painful under the flat OxFml root surface,
2. which runtime/environment objects DNA OneCalc wants first from a runtime facade,
3. whether the planned editor facade shape matches the current OneCalc editor architecture,
4. whether replay projection should first target retained local evidence, ordinary host capture, or both,
5. whether any current OneCalc-local wrapper vocabulary should be retired once OxFml facades land.

## 7. Short Response To DNA OneCalc's Current Note
OxFml's current answer to the DNA OneCalc note is:
1. yes, the runtime facade is the first implementation priority,
2. yes, provider-plus-pin and volatile-input injection remain explicit and
   first-class,
3. yes, the editor facade is intended to absorb real composition burden rather
   than merely rename edit packets,
4. yes, the replay facade is intended to narrow consumer dependence on broad
   internal artifact families while preserving replay identity and provenance
   truth,
5. yes, the new canonical packet now treats immutable environment and document
   objects as the preferred end-state rather than optional style,
6. yes, the current packet now explicitly responds to the main OneCalc asks
   around provider-pin explicitness, per-request volatile/query inputs,
   incremental edit truth, canonical function help, and replay-projection
   provenance,
7. yes, the current first `W056` slice now gives OneCalc an OxFml-owned
   `VerificationPublicationSurface` through both runtime and replay projection
   for the admitted locale/number-format/style-context lane, while leaving broad
   display-code and conditional-formatting closure explicitly open.

## 8. Minimum Invariants
The following invariants should stay true through the migration:
1. OneCalc does not become a second owner of parser, binder, semantic-plan, or evaluator semantics,
2. OneCalc does not fork frozen OxFml/OxFunc packet families,
3. OneCalc host policy remains above OxFml semantic meaning,
4. OneCalc UI and persistence concerns remain host-owned, not pushed back into OxFml,
5. the migration remains staged and reversible until facade uptake is real.

## 9. Current Public Surface Update
After the latest Wave 4 packaging cut, OxFml's current public consumer read is:
1. ordinary downstream use should target:
   - `consumer::runtime`
   - `consumer::editor`
   - `consumer::replay`
2. public `substrate::...` access is gone from the library surface,
3. any remaining host/session/adapter reach that still exists is explicit
   `test_support` support surface for bounded internal or integration-test use,
   not ordinary downstream integration contract.
