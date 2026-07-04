# W077: Strict Excel Grid BindProfile And R1C1 Identity

## Purpose

Process `HANDOFF-DNATREECALC-001` into an OxFml-owned formula-language plan for the `strict-excel-grid` profile: typed `BindProfile`, symbolic relative references, A1 `$` fidelity, grid bounds, caller-independent bind identity, compiled-plan caching, and translation/rebind APIs.

**State:** the core seam is built and consumer-consumed. The frozen public shape is
recorded below; the GridBounds ask has been ratified as consumer-side (superseding
handoff item 6). See the "Status" and "Frozen public shape" sections.

## Depends on

- W037 R1C1 formula-channel floor.
- W074 name/call and host-context identity lessons.
- W075 compiled-plan optimization floor.
- OxCalc W061 grid model and reference-machine planning.

## Scope

1. Define the public `BindProfile` shape and default-preserving migration rule.
2. ~~Define `GridBounds` and the out-of-bounds to `#REF!` contract.~~ **Superseded** —
   bounds semantics stay consumer-side in OxCalc (see "GridBounds ratification" below).
   Retained only as a pointer to the OxCalc-owned `ExcelGridBounds` / `#REF!` contract.
3. Define symbolic reference ADTs for relative R1C1 and caller-relative A1 references.
4. Remove caller anchor from bind identity only when the profile proves caller-independent symbolic refs are active.
5. Define compiled-plan cache keys and per-cell caller-coordinate instantiation.
6. Define translation/rebind APIs for fill, paste, region stamping, and insert/delete shifting.
7. Fix non-default host-reference syntax reuse penalties before `strict-excel-grid` makes non-default syntax common.

## Non-goals

- OxCalc grid storage or dependency graph implementation.
- OxFunc function semantics.
- Spill placement/blockage arbitration; OxFml only preserves syntax, capability facts, and pass-through shape facts.
- Any default-behavior drift for existing TreeCalc or ordinary worksheet channels.

## Closure condition

W077 closes only when default-profile behavior is regression-proven unchanged, `strict-excel-grid` bind fixtures prove caller-independent identity and `$` fidelity, grid-bound translations produce deterministic `#REF!`, plan-cache identity is explicit, and OxCalc has acknowledged the public packet shape for W061.

## Initial lanes

1. Spec update and type-shape freeze.
2. Default behavior guardrails and fixture inventory.
3. Symbolic R1C1 and caller-relative A1 bind records.
4. Bind fingerprint/cache-key migration.
5. Translation/rebind API design and focused fixtures.
6. OxCalc acknowledgement packet.

## Status

- execution_state: in_progress (materially built, consumer-consumed; residual items enumerated below)
- scope_completeness: scope_complete
- target_completeness: target_partial (core seam built and green; residual legacy-A1 `$` fidelity + trait-level transform coverage tracked as open lanes)
- integration_completeness: integrated (OxCalc `StrictExcelGridReferenceProfile` consumes the seam end-to-end)
- open_lanes: none as of fml-7t6.1-.4 (all closed); historical residuals resolved below
- claim_confidence: validated for the frozen shape and consumer acknowledgement; provisional for full translation-API breadth

The public seam is built in `crates/oxfml_core/src/binding/profile.rs` and consumed by
`bind_context_fingerprint_for` in `crates/oxfml_core/src/binding/mod.rs`
(honors `ExcludeCallerAnchorForTemplate`). The acceptance and guardrail tests pass:
`cargo test -p oxfml_core --test reference_profile_api_tests` (13/13), including
`symbolic_profile_template_identity_is_caller_independent` and
`default_binding_keeps_caller_anchor_in_bind_fingerprint`. OxCalc's
`StrictExcelGridReferenceProfile` (sibling repo, `crates/.../grid/reference_engine.rs`)
already consumes the seam end-to-end.

## OxCalc seam acknowledgement (closure clause)

The closure condition requires that OxCalc has acknowledged and consumed the public
packet shape. As of OxCalc HEAD, `StrictExcelGridReferenceProfile` implements the
`ReferenceBindProfile` seam **end-to-end**:

- Implementation: `OxCalc/src/oxcalc-core/src/grid/reference_engine.rs`
  (`impl ReferenceBindProfile for StrictExcelGridReferenceProfile`), with the transform
  arm in `OxCalc/src/oxcalc-core/src/grid/reference_engine/transform.rs`.
- It consumes the frozen shape directly: `reference_policy()` returns `ProfileSymbolic`,
  `bind_atom`/`bind_range` emit `ProfileReferenceRecord`s, and `transform_reference`
  returns the OxFml `ReferenceTransformOutcome` variants (`Shifted`, `Expanded`, `Shrunk`,
  `FullyInvalid`, etc.).
- **Its caller-independence mechanism.** OxCalc's `fingerprint_policy()` returns
  `IncludeCallerAnchor` (verified: `reference_engine.rs` ~line 622) — it does **not** use
  `ExcludeCallerAnchorForTemplate`. Caller-independent template identity is achieved on the
  **R1C1** channel, where the relative offsets are already caller-independent in the normal
  form: two placements of the same R1C1 formula share `formula_template_identity` while
  refreshing `placed_formula_identity` (witnessed by
  `strict_profile_r1c1_template_identity_is_caller_independent`). On the **A1** channel the
  profile deliberately rebinds per placement, so A1 template identity differs across cells
  (witnessed by `strict_profile_a1_incremental_bind_rebinds_when_caller_anchor_changes`).
  The `ExcludeCallerAnchorForTemplate` policy remains an available OxFml primitive (and is
  exercised by the OxFml-side `FakeSymbolicProfile` tests); the strict-excel-grid profile
  simply chose the R1C1-normal-form route to the same caller-independence goal. Note that
  OxCalc DOES ship profiles that consume `ExcludeCallerAnchorForTemplate` directly —
  `TreeCalcReferenceBindProfile` and `TreeCalcContextReferenceBindProfile`
  (`OxCalc/src/oxcalc-core/src/tree_reference_system.rs` ~lines 144, 232) — so the policy
  is used by shipped profiles, not only test fakes.
- OxCalc owns the grid-semantic side of the seam (its `ExcelGridBounds` and the
  out-of-bounds `#REF!` contract) — see the GridBounds ratification below.

OxCalc test coverage witnessing the seam (in `grid/reference_engine.rs`):
`strict_profile_binds_a1_cells_with_dollar_fidelity`,
`strict_profile_r1c1_template_identity_is_caller_independent`,
`strict_profile_a1_incremental_bind_rebinds_when_caller_anchor_changes`,
`strict_profile_rejects_absolute_a1_out_of_bounds`,
`strict_profile_structural_insert_expands_area_reference`,
`strict_profile_structural_delete_shrinks_area_reference` (asserts the `Shrunk` outcome),
`strict_profile_structural_delete_turns_deleted_point_into_ref_error` (asserts the
`FullyInvalid` outcome).

At the OxFml seam, the trait-level transform outcomes (`Shifted`, `PartiallyInvalid`,
`FullyInvalid`) and the plan-reuse regression for `HANDOFF-DNATREECALC-001` item 8 are
covered by `tests/reference_profile_api_tests.rs`
(`profile_transform_reference_*`, `non_default_syntax_profile_reuses_plan_across_placements`,
`default_syntax_pays_plan_reuse_penalty_across_placements`).

## Frozen public shape (exact as-built names)

The closure condition requires the exact public type/API names to be recorded before
closing. These are the names as built in `crates/oxfml_core/src/binding/profile.rs`
(re-exported from `crates/oxfml_core/src/lib.rs`). This section is the freeze of record.

### `BindProfile` fields

```rust
pub struct BindProfile {
    pub profile_id: String,
    pub profile_version: ProfileVersion,           // ProfileVersion(pub String), ::v1() -> "v1"
    pub reference_policy: ReferencePolicy,
    pub fingerprint_policy: ReferenceFingerprintPolicy,
    pub syntax_capabilities: ReferenceSyntaxCapabilities,
}
```

Constructor of record: `BindProfile::legacy_compatibility()` (default-preserving profile).

### `ReferencePolicy` variants

`FormulaOnly`, `LegacyCompatibility`, `ProfileSymbolic`, `HostExtended`.

### `ReferenceFingerprintPolicy` variants

`IncludeCallerAnchor` (default), `ExcludeCallerAnchorForTemplate` (caller-independent
template identity — the load-bearing variant for grid template sharing).

### `ReferenceSyntaxCapabilities` fields

`a1_references`, `r1c1_references`, `host_references`, `structured_references`,
`spill_references` (all `bool`); const ctor `ReferenceSyntaxCapabilities::worksheet_legacy()`.

### `ReferenceBindProfile` trait methods

```rust
pub trait ReferenceBindProfile {
    fn profile_id(&self) -> &str;
    fn profile_version(&self) -> ProfileVersion;                    // default v1
    fn reference_policy(&self) -> ReferencePolicy;                  // default LegacyCompatibility
    fn fingerprint_policy(&self) -> ReferenceFingerprintPolicy;     // default IncludeCallerAnchor
    fn fingerprint(&self, context: &ReferenceProfileFingerprintContext) -> ReferenceProfileFingerprint;
    fn operator_capabilities(&self) -> ReferenceOperatorCapabilities;
    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult;
    fn bind_name(&self, request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult;
    fn selector_syntax(&self) -> Vec<ReferenceSelectorSyntax>;
    fn bind_selector(&self, request: &ReferenceSelectorBindRequest) -> ReferenceAtomBindResult;
    fn bind_structured_reference(&self, request: &ReferenceStructuredBindRequest) -> ReferenceAtomBindResult;
    fn bind_range(&self, request: &ReferenceRangeBindRequest) -> ReferenceRangeBindResult;
    fn normal_form_key(&self, reference: &ProfileReferenceRecord, context: &ReferenceProfileFingerprintContext) -> ReferenceNormalFormKey;
    fn dependency_hints(&self, reference: &ProfileReferenceRecord, context: &ReferenceProfileFingerprintContext) -> ReferenceDependencyEnvelope;
    fn instantiate_reference(&self, request: &ReferenceInstantiationRequest) -> InstantiatedReference;
    fn transform_reference(&self, request: &ReferenceTransformRequest) -> ReferenceTransformResult;
    fn render_reference(&self, request: &ReferenceRenderRequest) -> ReferenceRenderResult;
    fn reference_completion_proposals(&self, request: &ReferenceCompletionRequest) -> ReferenceCompletionResult;
}
```

### Transform request/result/outcome types

- `ReferenceTransformRequest { reference: ProfileReferenceRecord, transform_kind: ReferenceTransformKind, payload: Option<ProfilePayload> }`
- `ReferenceTransformKind`: `StructuralEdit`, `RenderModeChange`, `HostSpecific(String)`
- `ReferenceTransformResult { outcome: ReferenceTransformOutcome, reference: Option<ProfileReferenceRecord>, diagnostics: Vec<String> }`
- `ReferenceTransformOutcome` (exact, full set): `Unchanged`, `Shifted`, `Expanded`, `Shrunk`, `Split`, `PartiallyInvalid`, `FullyInvalid`, `DynamicOrHostSensitive`, `Unsupported`, `GeometryCoupledOpaqueConflict`

### Instantiate request/result types

- `ReferenceInstantiationRequest { bound_reference: ProfileReferenceRecord, runtime_host_formula_context: RuntimeHostFormulaContext, purpose: ReferenceInstantiationPurpose }`
- `ReferenceInstantiationPurpose`: `StaticDependencyExtraction`, `RuntimeEvaluation`, `Rendering`
- `RuntimeHostFormulaContext { profile_id: String, context_payload: Option<ProfilePayload> }`
- `InstantiatedReference`: `ReferenceLike { profile_id, identity_key }`, `StaticDependencyEnvelope(ReferenceDependencyEnvelope)`, `DynamicDependencyRequest(ReferenceDependencyEnvelope)`, `RefError`, `Unsupported { reason }`

### The three identity types (plus runtime dependency identity)

- `FormulaSourceIdentity { key: String }`
- `FormulaTemplateIdentity { key: String }`  (caller-independent under `ExcludeCallerAnchorForTemplate`)
- `PlacedFormulaIdentity { key: String }`  (caller-dependent)
- `RuntimeDependencyIdentity { key: String }`

Each is a newtype over `key: String`; they are `pub` re-exported from the crate root.

## GridBounds ratification (owner-level)

GridBounds / bounds semantics stay **CONSUMER-SIDE**. OxCalc owns `ExcelGridBounds`
and the out-of-bounds `#REF!` contract. OxFml owns grammar and the bind lifecycle and
only preserves syntax facts, capability facts, and pass-through shape facts; grid
semantics (including bounds and `#REF!` materialization on out-of-bounds translation)
stay in OxCalc.

This **supersedes** `HANDOFF-DNATREECALC-001` item 6, which requested
`GridBounds { max_row: 1_048_576, max_col: 16_384 }` be defined **in OxFml**. That item
is withdrawn per the seam guardrail: OxFml does not carry the grid dimension constants or
the out-of-bounds `#REF!` rule. Rationale: keeping bounds consumer-side prevents OxFml
from encoding a specific host's grid geometry into the formula-language core, which would
couple the grammar to one grid dialect and violate the "OxFml preserves shape facts,
not grid semantics" boundary. Scope item 2 ("Define GridBounds ... to `#REF!` contract")
is retained only as a documentation pointer to the OxCalc-owned contract, not as OxFml
work.