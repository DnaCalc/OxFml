# OxFml DNA OneCalc Downstream Consumer Contract

## 1. Purpose And Status
This document is the current OxFml-owned downstream-consumer clarification for DNA OneCalc first integration.

It exists to give DNA OneCalc one readable surface that answers:
1. which host/runtime fields are mandatory for default H0/H1 execution,
2. which fields are only required for bounded seam-sensitive probe packets,
3. which fields are coordinator or TreeCalc reference material, not part of the initial OneCalc host claim,
4. which editor/language-service packet surfaces are integration-ready and which remain local-only evidence,
5. what the host must do with each returned-value class,
6. what OxFml does not currently authorize OneCalc to claim.

Status:
1. canonical OxFml downstream-consumer clarification note,
2. read together with the documents listed in Section 1A,
3. does not supersede the broader host/runtime contract or the canonical spec set,
4. sequence-bound to the landed `OxFml_V1` consumer surface and the Foundation `DNA_ONECALC_SCOPE_AND_SPEC.md` as of its writing.

### 1A. Read Together With
Start here for DNA OneCalc integration:
1. `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` — top-level `OxFml_V1` consumer seam
2. `OXFML_DNA_ONECALC_DOWNSTREAM_CONSUMER_CONTRACT.md` — this document
3. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md` — code-facing surface map

Then use these narrower companion specs:
1. `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` — primary host/runtime coordination packet
2. `OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md` — reduced-profile OneCalc companion
3. `OXFML_CONSUMER_INTERFACE_REARCHITECTURE_PLAN.md` — design background
4. `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` — canonical consumer-facing contract packet
6. `formula-language/OXFML_EDITOR_LANGUAGE_SERVICE_AND_HOST_INTEGRATION_PLAN.md` — editor/language-service plan
7. `OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md` — deterministic fixture-host packet
8. `OXFML_CANONICAL_ARTIFACT_SHAPES.md` — canonical artifact ladder
9. `OXFML_MINIMUM_SEAM_SCHEMAS.md` — minimum candidate/commit/reject/trace fields
10. `OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md` — reject and effect taxonomy
11. `formula-language/OXFML_OXFUNC_LIBRARY_CONTEXT_RUNTIME_INTERFACE.md` — runtime library-context model
12. `formula-language/OXFML_REGISTERED_EXTERNAL_PROVIDER_AND_CALL_REGISTER_ID_BOUNDARY.md` — registered-external boundary

---

## 2. Host/Runtime Subset For DNA OneCalc
Ordinary `OxFml_V1` entry rule for DNA OneCalc:
1. use `oxfml_core::consumer::runtime` for execution-facing integration,
2. use `oxfml_core::consumer::editor` for edit/help/completion integration,
3. use `oxfml_core::consumer::replay` for replay-aware projection,
4. do not treat any explicit `test_support::...` access as ordinary downstream contract.

### 2.1 OC-H0 Mandatory Fields (Literal And Function Core)
The narrowest honest H0 host path requires these fields explicitly:

| # | Field or family | Source | Notes |
|---|----------------|--------|-------|
| 1 | `FormulaSourceRecord` | host | formula text plus formula-stable identity |
| 2 | `formula_channel_kind` | host | currently `WorksheetA1` for default H0 |
| 3 | `structure_context_version` | host | may be a fixed sentinel for single-formula H0 |
| 4 | `LibraryContextProvider` | host | must supply at least one immutable snapshot |
| 5 | immutable `LibraryContextSnapshot` | OxFunc-backed | pinned snapshot identity visible to parse/bind/plan |
| 6 | `LocaleFormatContext` | host | locale and date-system context |
| 7 | `now_serial` | host | required if any volatile-time function is in scope |
| 8 | `random_value` | host | required if `RAND` or `RANDBETWEEN` is in scope |

H0 does not require:
1. caller-anchor or address-mode context,
2. direct cell bindings,
3. defined-name bindings,
4. table metadata,
5. host-query providers (`HostInfoProvider`, `RtdProvider`),
6. registered-external providers,
7. capability view or fence basis beyond the default single-formula path.

### 2.2 OC-H1 Mandatory Fields (Explicit-Input Host)
OC-H1 adds these fields on top of H0:

| # | Field or family | Source | Notes |
|---|----------------|--------|-------|
| 1 | `defined_name_bindings` | host | explicit host-bound input slots |
| 2 | `HostInfoProvider` | host | when the formula may use `INFO`, `CELL`, or other host-query functions |
| 3 | `RtdProvider` | host | when `RTD` is in scope |
| 4 | display context | host | base formatting state for effective-display projection |
| 5 | explicit recalc trigger | host | deterministic recalc request |

OC-H1 still does not require:
1. caller-anchor or direct cell bindings (those belong to probe packets),
2. table metadata (belongs to `StructuredReferenceProbePacket`),
3. registered-external providers (belongs to `RegisteredExternalProbePacket`),
4. multi-formula dependency graph or scheduler policy.

### 2.3 Probe-Only Fields
These fields are only required when a bounded seam-sensitive probe packet is admitted.

| # | Field or family | Admitted by | Notes |
|---|----------------|-------------|-------|
| 1 | `caller_anchor` | `ReferenceProbePacket` | caller location for `@`, `_xlfn.SINGLE`, or reference-sensitive `CELL(...)` |
| 2 | `active_selection_anchor` | `ReferenceProbePacket` | only where `_xlfn.SINGLE` selection-sensitive truth needs it |
| 3 | `cell_fixture` / direct cell bindings | `ReferenceProbePacket` | concrete cell state for reference-sensitive evaluation |
| 4 | `table_catalog` | `StructuredReferenceProbePacket` | stable table identity, range, column map, header/totals presence |
| 5 | `enclosing_table_ref` | `StructuredReferenceProbePacket` | effective table for omitted-table-name forms |
| 6 | `caller_table_region` | `StructuredReferenceProbePacket` | row/region-sensitive meaning for `#This Row` |
| 7 | `RegisteredExternalProvider` | `RegisteredExternalProbePacket` | `CALL` / `REGISTER.ID` / extension-sensitive invocation |
| 8 | `RegisteredExternalCatalogMutationRequest` | `RegisteredExternalProbePacket` | host-initiated registration/unregister |
| 9 | `RegisteredExternalCatalogController` | `RegisteredExternalProbePacket` | mutation controller for catalog changes |

### 2.4 Coordinator Or TreeCalc Reference Material — Not Part Of Initial OneCalc Host Claim
The following are documented in the broader host/runtime contract but are not part of the first DNA OneCalc host claim:

1. `candidate_result_id` / `commit_attempt_id` for multi-session coordinator publication arbitration,
2. `fence_snapshot_ref` for multi-session publish arbitration,
3. topology-sensitive consequence surfaces for graph-wide dependency coordination,
4. distributed placement policy,
5. multi-session publish arbitration and contention semantics beyond single-formula proving,
6. scheduler-policy meaning or execution-restriction transport for coordinator scheduling,
7. caller-anchor and address-mode carriage for the TreeCalc relative-reference subset (currently in `W026` note lane),
8. execution-restriction transport shape beyond the current semantic minimum.

DNA OneCalc may still produce candidate, commit, reject, and trace artifacts for its own replay and evidence purposes, but it does not claim coordinator-grade publication arbitration.

---

## 3. Packet Taxonomy For DNA OneCalc

These are OneCalc-local classification names as defined in `DNA_ONECALC_SCOPE_AND_SPEC.md` Section 7.2.1. They must be reconciled against upstream OxFml packet families rather than silently replacing them.

### 3.1 ExplicitInputPacket

**Role**: default OC-H1 packet kind for the public DNA OneCalc host model.

**Allowed fields**:
1. `FormulaSourceRecord` (formula text, formula-stable identity),
2. `formula_channel_kind`,
3. `structure_context_version`,
4. `defined_name_bindings`,
5. `LibraryContextProvider` / immutable `LibraryContextSnapshot`,
6. `LocaleFormatContext` (locale, date-system),
7. `now_serial`, `random_value`,
8. `HostInfoProvider` (for `INFO`, `CELL`, host-query lanes),
9. `RtdProvider` (for `RTD` lanes),
10. display context,
11. explicit recalc trigger.

**Forbidden fields**:
1. generic worksheet cell maps,
2. workbook-style name managers,
3. open reference environments,
4. caller-anchor or direct cell bindings,
5. table metadata,
6. registered-external providers,
7. scheduler-policy inputs.

**Currently exercised semantic lanes**:
1. literals, operators, and built-in functions requiring no external provider or workbook state,
2. defined-name-driven explicit inputs,
3. `INFO` and `CELL` host-query lanes where the host supplies answers,
4. `RTD` lanes where the host supplies a provider,
5. `LET`, `LAMBDA`, and the current higher-order callable floor (`MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`),
6. adopted defined-name callable transport,
7. semantic formatting through `TEXT`, `VALUE`, `NOW`, `TODAY`, `CELL`, `INFO`,
8. `HYPERLINK` publication-intent preservation.

### 3.2 ReferenceProbePacket

**Role**: bounded probe-only packet admitted only when a real upstream semantic lane requires reference-sensitive truth.

**Allowed extra fields** (on top of `ExplicitInputPacket`):
1. `caller_anchor`,
2. `active_selection_anchor`,
3. `cell_fixture` / direct cell bindings,
4. `probe_reason` (mandatory, must identify the semantic lane requiring this probe).

**Forbidden fields**:
1. generic worksheet environment,
2. workbook-style name managers,
3. multi-formula dependency graph or scheduler policy,
4. table metadata (use `StructuredReferenceProbePacket` instead).

**Currently exercised semantic lanes**:
1. `@` scalarization,
2. `_xlfn.SINGLE`,
3. reference-sensitive `CELL(...)` lanes,
4. any lane where evaluation truth depends on concrete cell identity.

**UI and artifact rule**: must remain visibly exceptional in UI and retained artifacts, with explicit `probe_reason` visible.

### 3.3 StructuredReferenceProbePacket

**Role**: probe-only packet for table or structured-reference semantics.

**Allowed extra fields** (on top of `ExplicitInputPacket`):
1. `table_catalog`,
2. `enclosing_table_ref`,
3. `caller_table_region`,
4. `probe_reason` (mandatory).

May also carry `ReferenceProbePacket` fields when the lane requires both table context and direct-cell reference truth.

**Forbidden fields**:
1. generic worksheet environment,
2. hidden table engine or workbook-graph table semantics.

**Currently exercised semantic lanes**:
1. `Table1[Amount]`, `[@Amount]`,
2. section-only selectors: `Table1[#Headers]`, `Table1[#Totals]`,
3. multi-column section-qualified selectors: `Table1[[#All],[Amount]:[Tax]]`, `Table1[[#Data],[Amount]:[Tax]]`,
4. current-row-sensitive structured-reference evaluation,
5. defined-name collision disambiguation within table scope.

### 3.4 RegisteredExternalProbePacket

**Role**: packet kind for registered-external or extension-sensitive scenarios.

**Allowed extra fields** (on top of `ExplicitInputPacket`):
1. `RegisteredExternalProvider`,
2. `RegisteredExternalCatalogMutationRequest` and `RegisteredExternalCatalogController` where host-initiated mutation is exercised,
3. extension profile or provider state,
4. `probe_reason` (mandatory).

**Forbidden fields**:
1. generic worksheet environment,
2. untyped extension side channels.

**Currently exercised semantic lanes**:
1. worksheet `REGISTER.ID` and worksheet `CALL`,
2. reference-visible `CALL` arguments,
3. host API registration and VBA shim registration,
4. unregister packet carriage,
5. `GROUPBY` and `PIVOTBY` grouped-aggregation adapter carriage through both inline `LAMBDA(...)` and bare built-in aggregation callables.

**Upstream ownership rule**: built-in catalog truth and runtime register/unregister semantics are OxFunc-owned; OxFml preserves channel-specific host/VBA provenance and worksheet-visible consequence typing.

---

## 4. Editor And Language-Service Integration Readiness

### 4.1 Integration-Ready Packet Surfaces
The following OxFml language-service packet surfaces are currently good enough for DNA OneCalc host integration. DNA OneCalc should consume these rather than inventing local equivalents.

| # | Packet surface | OxFml source | Integration-ready? | Notes |
|---|---------------|-------------|-------------------|-------|
| 1 | `FormulaEditRequest` / `FormulaEditResult` | `language_service/mod.rs` | **yes** | immutable edit request/result with text-change ranges, incremental parse/red/bind reuse, optional semantic-plan follow-on |
| 2 | `LiveDiagnosticSnapshot` | `language_service/mod.rs` | **yes** | unified syntax/bind/semantic-plan diagnostics for squiggle/list use |
| 3 | Deterministic completion proposals | `language_service/mod.rs` | **yes** | function names, defined names, table names, table columns, structured selectors, R1C1 syntax assists |
| 4 | Completion-candidate validation and proposal application | `language_service/mod.rs` | **yes** | re-enters normal parse/bind pipeline |
| 5 | `SignatureHelpContext` | `language_service/mod.rs` | **yes** | cursor-sensitive call/argument context |
| 6 | `FunctionHelpPacket` with pinned snapshot context | `consumer/editor` facade | **yes** | canonical help payload and pinned snapshot context at the facade boundary |
| 7 | `IntelligentCompletionContext` | `language_service/mod.rs` | **yes** | normalized context packet for external non-canonical completion |
| 8 | `EditorSyntaxSnapshot` | `language_service/mod.rs` | **yes** | owned-trivia token view for editor rendering |

### 4.2 Local-Only Evidence Surfaces (Not Yet Integration-Ready)
The following surfaces exist as deterministic local evidence but are not yet frozen for host integration:

| # | Surface | Why not yet integration-ready | Depends on |
|---|---------|------------------------------|------------|
| 1 | OxFunc-backed help/signature payload retrieval | OxFunc has not yet frozen a help/signature provider contract | OxFunc help-metadata freeze |
| 2 | Shared host/OxCalc immutable formula-edit packet | packet shape is proposed but not yet frozen as a shared seam contract | OxCalc immutable-edit seam round |
| 3 | Shared host-facing validated intelligent-completion result packet | packet shape exists locally but not yet frozen for host consumption | OxCalc validated-completion seam round |
| 4 | Editor packet replay-appliance projection | evidence is deterministic local evidence, not replay-appliance graded | replay adapter promotion |

### 4.3 Current Host Integration Working Rules
1. DNA OneCalc should consume the integration-ready packet surfaces in Section 4.1 rather than inventing a second parser/binder/editor truth locally.
2. For OxFunc-backed function help and signature help, DNA OneCalc should start from the current library-context snapshot export and its metadata fields, while keeping the host ready for the later provider-backed snapshot model.
3. Diagnostics should remain OxFml-derived wherever the canonical meaning lives in OxFml.
4. Intelligent completion remains host-owned and non-canonical until it re-enters OxFml through the ordinary edit path.
5. DNA OneCalc may add presentation, interaction, and command affordances (cursor/selection state, IME state, popup state, command routing) but must not locally own canonical parse, bind, diagnostic, completion validity, or function/signature help payload truth.

---

## 5. Returned Value Surface — Host Obligations

The first returned-value split is defined in `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md` Section 8B and `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` Section 6.2.

DNA OneCalc must handle each value class as follows.

### 5.1 Ordinary Value
**What it is**: a scalar or array evaluation result with no additional presentation or provider-outcome semantics.

**Host obligations**:
1. render the value using the effective display projection from locale, date-system, and any applicable host format state,
2. persist the value in retained scenario artifacts,
3. replay-project the value through the current replay-capture path.

### 5.2 Value With Presentation
**What it is**: an evaluation result that carries evaluator-returned presentation hints, such as date/time format hints from `NOW`/`TODAY`, or publication-intent hints from `HYPERLINK`.

**Host obligations**:
1. render the value using the evaluator-returned presentation hints as a first source of effective-display truth, composed with host style state,
2. distinguish evaluator-returned presentation hints from persisted host style state in the rendering pipeline,
3. persist both the value and the presentation hints in retained scenario artifacts,
4. replay-project both the value and the effective presentation through the current replay-capture path,
5. do not silently collapse presentation hints into host style state or discard them.

### 5.3 Typed Host/Provider Outcome
**What it is**: an evaluation result where the returned value surface carries a typed host-query or provider outcome projection, such as `RTD` value/capability-denied/connection-failed outcomes, `INFO` unsupported-query outcomes, or `CELL` provider-failure outcomes.

**Host obligations**:
1. render the outcome using its typed classification, not as a generic error,
2. preserve the typed outcome family in retained artifacts,
3. replay-project the typed outcome through the current replay-capture path,
4. do not replace typed evaluator/runtime outcomes with generic host exceptions or opaque transport errors,
5. surface capability-denied or provider-unavailable outcomes explicitly in the UI.

### 5.4 Rich Value Or Non-Ordinary Value
**What it is**: a value class that is richer than ordinary scalar/array, such as callable-value carriers, `HYPERLINK` publication-intent, or `IMAGE` rich-value.

**Current host obligations**:
1. the current first-freeze candidate treats rich values as an extension of `ValueWithPresentation` plus typed host/provider outcome projection,
2. DNA OneCalc should render rich-value consequences (such as `HYPERLINK` display text and target, or `IMAGE` display intent) where the evaluator surfaces them through presentation hints or publication-intent fields,
3. DNA OneCalc should persist and replay-project the rich-value surface as far as the current evaluator carriers support,
4. callable values that remain in the return surface (e.g., `LAMBDA` results) should be displayed as typed callable descriptions rather than silently discarded or shown as errors,
5. the richer value carrier beyond the current first-pass split remains a deferred decision.

---

## 6. Not-Authorized List

The current OxFml draft does not authorize DNA OneCalc to claim the following:

### 6.1 Evaluator And Seam Scope
1. full Excel formula-language closure — the current exercised grammar floor is broader than basic but still narrower than full Excel,
2. full built-in function coverage — the admitted function surface is always filtered through the OxFunc `W050` deferred overlay and `W051` in-scope-not-complete overlay,
3. pack-grade replay evidence — the current replay floor is `C3.explain_valid`; `C4.distill_valid` and `C5.pack_valid` are not authorized,
4. coordinator-grade publication arbitration — DNA OneCalc is a direct single-formula host, not an OxCalc-equivalent coordinator,
5. full distributed or async runtime behavior beyond the current local external-provider and contention model.

### 6.2 Language-Service Scope
1. OxFunc-backed function-help or signature-help payload retrieval as a frozen contract,
2. shared host/OxCalc immutable formula-edit packet as a frozen seam,
3. shared validated intelligent-completion result packet as a frozen seam,
4. editor packet replay-appliance projection.

### 6.3 Formatting And Display Scope
1. full `MS-OE376` formatting parity,
2. broad conditional-formatting rule families beyond the current restricted-carrier floor,
3. broad data-validation rule families beyond the current restricted-carrier floor,
4. full semantic-formatting family coverage beyond the current exercised subset.

### 6.4 Extension And Provider Scope
1. broader worksheet `CALL` / `REGISTER.ID` host/coordinator behavior beyond the current frozen OxFml/OxFunc packet floor,
2. deferred external/provider families not already exercised in the current floor,
3. final cross-process ABI for extensions.

### 6.5 Host And Integration Scope
1. full product-host specification — the current contract is a bounded first-slice packet,
2. full OxCalc equivalence — DNA OneCalc must remain narrower than OxCalc,
3. pack-grade scenario promotion — the current evidence is local-only and non-pack-eligible,
4. workbook-graph semantics — DNA OneCalc remains an isolated single-node host,
5. stable `SpreadsheetML 2003` isolated-instance persistence contract from `Ox*` repos — that mapping is a `DnaOneCalc`-local responsibility.

---

## 7. Reconciliation Rules

### 7.1 Packet Name Reconciliation
The OneCalc-local packet taxonomy names (`ExplicitInputPacket`, `ReferenceProbePacket`, `StructuredReferenceProbePacket`, `RegisteredExternalProbePacket`) are design-governance classification names. They must be reconciled against the upstream OxFml host/runtime packet families documented in `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` and `OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md` rather than silently replacing upstream field names or packet families.

### 7.2 Field Name Stability
Where the canonical OxFml docs already use specific field names, DNA OneCalc should use the same names in its host implementation. In particular:
1. `FormulaSourceRecord`, `formula_channel_kind`, `structure_context_version`,
2. `LibraryContextProvider`, `LibraryContextSnapshot`,
3. `HostInfoProvider`, `RtdProvider`, `RegisteredExternalProvider`,
4. `LocaleFormatContext`,
5. `AcceptedCandidateResult`, `CommitBundle`, `RejectRecord`,
6. `FormulaEditRequest`, `FormulaEditResult`,
7. `LiveDiagnosticSnapshot`, `SignatureHelpContext`, `FunctionHelpPacket`,
8. `IntelligentCompletionContext`,
9. `ReturnedValueSurface`.

### 7.3 Ownership Preservation
1. OxFml owns parser, binder, semantic-plan, evaluator, editor/language-service substrate, and formula-semantic formatting meaning.
2. OxFunc owns function semantics, value-type universe, library-context catalog truth, and function-help payload truth.
3. DNA OneCalc owns host shell, host policy, UI, persistence, scenario orchestration, and upstream handoff production.
4. DNA OneCalc must not locally redefine any OxFml- or OxFunc-owned semantic surface.

### 7.4 Consumer-Facing Packaging Direction
DNA OneCalc should treat the current flat OxFml crate surface as transitional.

Current support rule:
1. until `W054` lands real facade modules, DNA OneCalc should integrate through the currently supported host, language-service, replay-projection, and packet surfaces exposed by OxFml,
2. it should not wait for facade packaging before integrating with the current exercised OxFml floor.

Preferred migration target:
1. consume the runtime facade family for execution and host integration,
2. consume the editor facade family for edit/diagnostic/completion/help interactions,
3. consume the replay facade family for replay-aware capture and projection,
4. use low-level transform or seam artifacts directly only for advanced harness or schema-provenance work,
5. do not introduce a competing wrapper vocabulary over frozen OxFml/OxFunc shared packet families while doing so.

Current contract rule:
1. `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` is now the canonical
   OxFml-owned packet for the runtime/editor/replay consumer model,
2. this document remains the OneCalc-specific narrowing and host-profile
   companion for that broader contract.

---

## 8. Working Rule
Use this document as the current single downstream-consumer clarification for DNA OneCalc first integration with OxFml.

Do not over-read it as:
1. a full product specification for DNA OneCalc,
2. permission to bypass the broader canonical OxFml spec set,
3. a frozen shared seam contract — it is an OxFml-owned clarification note, not a bilateral freeze,
4. authorization for any surface listed in Section 6.
