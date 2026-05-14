# HANDOFF-CALC-004 OxFml Receiving Review

## Purpose
Record the OxFml-side receiving review of
`../OxCalc/docs/handoffs/HANDOFF_CALC_004_OXFML_CAPABILITY_SET_HOLE_ADMISSION.md`.

This is a receiving acknowledgement and integration-dependency update. It does
not end the OxCalc integration dependency.

## Decision Summary
Decision: `accept_identity_reservation_defer_activation`.

Accepted into OxFml canonical direction:
1. default template-hole taxonomy should include `ValueHole`,
   `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`, `SparseRangeHole`,
   and `RichValueHole`,
2. hole identity is part of plan-template identity,
3. wide-by-default mapping from current OxFunc `ArgPreparationProfile` values is
   the correct first rule,
4. `RichValueHole` identity is the required capability set, not producer class,
5. required capability-set keys are replay/template identity,
6. producer and exercised capability columns are reserved but empty until real
   producers and kernels exist,
7. capability mismatch must be typed and replay-visible when producer facts are
   known.

## Clause Disposition Matrix
| Packet clause | Disposition | Owner | Notes |
|---|---|---|---|
| default hole taxonomy | accepted | OxFml | Canonical docs now list all six default families. |
| `ValueHole(value_class_bound)` | accepted | OxFml | Identity accepted. Runtime emission deferred until hole skeleton exists. |
| `RefOrValueHole(ref_observability)` | accepted | OxFml | Identity accepted. |
| `CallableHole(callable_signature)` | accepted | OxFml | Identity accepted; concrete callable carrier remains existing callable seam. |
| `ShapeSensitiveHole(extent_class)` | accepted | OxFml | Identity accepted. |
| `SparseRangeHole(extent_class, cardinality_class)` | adapted | OxFml identity / OxFunc activation | Identity accepted; runtime sparse reader activation deferred. |
| `RichValueHole(required_capability_set)` | adapted | OxFml identity / OxFunc activation | Required-set identity accepted; producer/kernel activation deferred. |
| hole kind is part of `PlanTemplate` identity | accepted | OxFml | Canonical rule accepted. Runtime/replay field emission remains deferred until backed by actual emitted facts. |
| `hole_id`, `ordinal`, `path`, stable serialization replay-visible | deferred_with_successor_owner | OxFml successor | Owner: template-hole emission successor. Blocker: canonical hole skeleton construction. |
| literals/references/omitted/helper names remain binding payloads by default | accepted | OxFml | Recorded in canonical docs. |
| current OxCalc projections retire after OxFml fields exist | accepted | OxFml/OxCalc | Current code starts that path with prepared identity; full retirement awaits hole skeleton/input transport. |
| `ValuesOnlyPreAdapter` -> `ValueHole(AnyValue)` | adapted | OxFml/OxFunc | Accepted as mapping rule; emission needs OxFunc metadata integration. |
| `RefsVisibleInAdapter` -> `RefOrValueHole(ReferenceIdentityVisible)` | adapted | OxFml/OxFunc | Same. |
| invocation callees -> `CallableHole(AnyCallable)` | accepted | OxFml | Identity accepted; concrete payload remains binding payload. |
| shape-sensitive calls -> `ShapeSensitiveHole` | accepted | OxFml | Identity accepted when shape participates in semantics. |
| bind-visible arg-preparation profile change invalidates prepared callables | routed_to_oxfunc | OxFunc metadata / OxFml invalidation consumption | Requires OxFunc version signal. |
| sparse reader protocol shape | deferred_with_successor_owner | OxFunc successor with OxFml transport | Owner: sparse-reader activation successor. |
| `Defined` includes assigned empty-string text | routed_to_oxfunc | OxFunc | Kernel/value-reader semantics. |
| `Blank` covers never-assigned and assigned-then-cleared values | routed_to_oxfunc | OxFunc/OxCalc structure input | Needs value-reader plus structure-state boundary. |
| sheet-structural state remains host/coordinator-owned | accepted | OxFml/OxCalc | Recorded as ownership rule. |
| `Indexable` capability selector | accepted | OxFml identity | Producer/kernel semantics deferred to OxFunc. |
| `Enumerable` capability selector | accepted | OxFml identity | Same. |
| `Shaped` capability selector | accepted | OxFml identity | Same. |
| `Materialisable` capability selector | accepted | OxFml identity | Same. |
| capability stable-key sort/dedup | accepted | OxFml | Canonical identity rule accepted. |
| producer admission as stable-key superset | adapted | OxFml identity / OxFunc producer | OxFml records rule; producers belong to OxFunc/successor work. |
| `RichValueHole` identity is required set, not producer class | accepted | OxFml | Canonical rule accepted. |
| capability replay columns | adapted | OxFml | Canonical docs define `RichValueCapabilityColumns`; runtime emission deferred except no-producer status. |
| capability mismatch deterministic path | adapted | OxFml | Canonical `CapabilitySetMismatchContext` accepted; timing depends on when producer facts are known. |
| replay invalid under missing/different required-set key | accepted | OxFml | Canonical replay rule accepted. |
| producer-superset must not rewrite required-set identity | accepted | OxFml | Canonical rule accepted. |
| `ArgPreparationProfile::RichArgAccepted(capability_set)` | routed_to_oxfunc | OxFunc | OxFml will consume equivalent metadata once acknowledged. |
| `SparseIteratorOk` or equivalent profile | routed_to_oxfunc | OxFunc | Successor metadata. |

## Name Adaptation
OxFml accepts the packet names as stable intent. Final public names may be
wrapped under `TemplateHoleKind::*` or equivalent OxFml-owned enum names.

Preferred field families:
1. `TemplateHole`
2. `hole_kind_key`
3. `RichValueCapabilityColumns`
4. `required_capability_set_keys`
5. `producer_capability_set_keys`
6. `exercised_capability_keys`
7. `CapabilitySetMismatchContext`

## OxFunc-Owned Pieces
1. `ArgPreparationProfile::RichArgAccepted(capability_set)` or equivalent,
2. future sparse-reader admission metadata,
3. rich/sparse kernel activation,
4. producer capability publication,
5. bind-visible profile versioning for rich/sparse admission changes.

## OxFunc Receiving Response Consumed
OxFunc has acknowledged this split in
`../OxFunc/docs/handoffs/HANDOFF-CALC-004_OXFUNC_RECEIPT.md` and the canonical
OxFunc seed contract
`../OxFunc/docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`.

OxFml accepts:
1. a metadata/profile shape equivalent to
   `RichArgAccepted(required_capability_set)`,
2. future sparse-reader admission metadata equivalent to
   `SparseRangeAccepted(extent_class, cardinality_class)`, with exact Rust
   naming deferred,
3. the reserved invalidation bridge `arg_admission_metadata_version`,
4. producer capability publication as typed metadata on the producer or returned
   rich/sparse carrier.

OxFml interpretation:
1. `arg_admission_metadata_version` invalidates prepared packages when
   argument-preparation/admission metadata changes,
2. `producer_capability_set_keys` and `exercised_capability_keys` remain absent
   or empty until OxFunc emits producer or exercised capability facts,
3. `IMAGE` / `_webimage` producer capability publication is the preferred first
   rich activation lane,
4. sparse range readers remain deferred.

## Deferred Pieces
1. concrete sparse reader runtime API,
2. concrete rich-value producer support,
3. rich-kernel execution,
4. final first activation workset assignment,
5. migration of OxCalc's reserved local columns to canonical emitted fields.

## OxFml Counterpart Changes Landed In This Pass
1. Canonical docs define template-hole identity families, rich capability
   columns, and capability mismatch context.
2. Runtime/replay field emission remains deferred unless backed by actual
   emitted OxFml/OxFunc facts; absence of producer capability facts must not be
   interpreted as rich/sparse support.

## Replacement Plan
This packet should be superseded by a narrower identity-first plan:
1. OxFml owns template-hole identity, capability-set identity, mismatch surface,
   and replay columns.
2. OxFunc owns producer/kernel metadata and activation.
3. OxCalc keeps current empty/reserved evidence as compatibility evidence until
   OxFml/OxFunc emit producer and exercised capability facts.

## Canonical Docs Updated
1. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
2. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
3. `docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
4. `docs/upstream/NOTES_FOR_OXCALC.md`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFunc Rust metadata model for rich/sparse admission
  - sparse/rich producer activation successor work
  - OxFml consumption of `arg_admission_metadata_version` in runtime/replay
    artifacts
  - first `IMAGE` / `_webimage` producer capability publication evidence
