# HANDOFF-CALC-002 OxFml Receiving Review

## Purpose
Record the OxFml-side receiving review of
`../OxCalc/docs/handoffs/HANDOFF_CALC_002_OXFML_RECALC_SESSION_AND_PLAN_TEMPLATES.md`.

This is a receiving acknowledgement and integration-dependency update. It does
not end the OxCalc integration dependency.

## Decision Summary
Decision: `adapt_and_promote_narrower_successor_plan`.

Accepted into OxFml canonical direction:
1. public runtime/session facade is the ordinary OxCalc surface,
2. OxFml should expose prepared formula identity without private binder access,
3. prepared identity should split package identity, plan-template identity, and
   binding identity,
4. formal reference/input transport should replace synthetic A1 and defined-name
   compatibility inputs,
5. managed-session result paths should expose or link to the same
   coordinator-relevant truth as one-shot execution,
6. candidate, commit, reject, trace, returned-value, template, and hole facts
   should be structured fields rather than diagnostic strings,
7. bind-visible metadata changes need invalidation signals,
8. compile-time folding and template reuse should be OxFml-owned trace/identity
   facts if OxFml admits them.

## Clause Disposition Matrix
| Packet clause | Disposition | Owner | Notes |
|---|---|---|---|
| Prepared callable surface | adapted | OxFml | Use OxFml-owned prepared formula package terminology. Current code now exposes `RuntimePreparedFormulaIdentity` from direct runtime, managed open/execute/session, and replay projections. |
| `prepared_callable_key` | adapted | OxFml | Current field name is `prepared_formula_key`. |
| `formula_stable_id` | accepted | OxFml | Exposed in `RuntimePreparedFormulaIdentity`. |
| source formula text version and source token identity | accepted | OxFml | Exposed as `formula_text_version` and `formula_token`. |
| `library_context_snapshot_ref` | accepted | OxFml | Exposed in prepared identity. |
| structure-context identity | accepted | OxFml | Exposed as `structure_context_version`. |
| caller/locus context affecting bind/reference meaning | adapted | OxFml | Current field is `caller_context_key`, derived from public locus context. Broader caller/address-mode closure remains W026-family successor work. |
| `PlanTemplate` identity or handle | adapted | OxFml | Current code exposes `RuntimePlanTemplateIdentity.plan_template_key`; full shape abstraction remains successor work. |
| `HoleBindings` identity or handle | adapted | OxFml | Current code exposes `RuntimeHoleBindingIdentity.hole_binding_fingerprint`; canonical hole skeleton remains successor work. |
| canonical formal-reference set | adapted | OxFml | Current code exposes `formal_references` from public bind references/unresolved references; final reference-to-hole transport remains successor work. |
| bind diagnostics and syntax diagnostics | accepted | OxFml | Already exposed on runtime and managed open results. |
| one-shot and managed identity parity | accepted | OxFml | Same derived identity logic is used for direct and managed paths. |
| immutable prepared callable rule | adapted | OxFml | Accepted as identity rule; exact cache/lifetime semantics are deferred to runtime implementation successor work. |
| no private internals for consumer reuse | accepted | OxFml | Public runtime/replay fields now carry first current-floor identity. |
| `shape_key` | deferred_with_successor_owner | OxFml successor | Explicitly left `None` in current code because canonical literal/reference abstraction is not yet emitted. Owner: prepared-template successor lane. Blocker: canonical shape abstraction and replay evidence. |
| `dispatch_skeleton_key` | adapted | OxFml | Current key derives from public function bindings, availability, catalog identity, and snapshot ref. |
| `plan_template_key` | accepted | OxFml | Current key uses `semantic_plan_key` as the current-floor template identity. |
| ordered template holes and stable `hole_id` | deferred_with_successor_owner | OxFml successor | Owner: prepared-template/hole skeleton successor. Blocker: canonical hole producer rules beyond identity reservation. |
| canonical hole kind | deferred_with_successor_owner | OxFml successor | Accepted in spec, not emitted in runtime code except empty current-floor skeleton. |
| `hole_binding_fingerprint` | adapted | OxFml | Current fingerprint derives from public bind references, unresolved references, helper profile, and capability requirements. |
| per-hole binding payload category | deferred_with_successor_owner | OxFml successor | Owner: hole-binding payload successor. Blocker: emitted hole skeleton. |
| template reuse/cache trace identity | deferred_with_successor_owner | OxFml successor | Owner: runtime cache/reuse successor. Current artifact reuse report remains separate. |
| formal reference/input transport fields | adapted | OxFml | Current `RuntimeFormalReference` covers handle, descriptor, family, caller dependence, and host-mappable identity where available. |
| per-invocation value/reference/callable input bindings | deferred_with_successor_owner | OxFml successor | Owner: invocation-input transport successor. Blocker: reference-to-hole/input binding model. |
| full managed result parity | adapted | OxFml | Current managed session/replay now carries prepared identity; full `RuntimeFormulaResult` parity remains successor work. |
| candidate/reject/returned-value/execution outcome surfaces | accepted | OxFml | Existing managed/result surfaces carry these categories; parity gaps remain recorded. |
| comparison/publication surfaces in managed commit | deferred_with_successor_owner | OxFml successor | Owner: managed-result parity successor. Blocker: commit result does not yet carry full verification publication surface. |
| trace/replay/correlation columns | adapted | OxFml | Current replay projection carries prepared identity and existing candidate/session/trace fields. |
| parent/child prepared-call invocation structure | deferred_with_successor_owner | OxFml successor | Ordered prepared-call records remain current public trace granularity. Owner: trace hierarchy successor if evidence requires it. |
| kernel-returned value per prepared call | accepted | OxFml | Existing prepared-call trace records `returned_value` when trace mode is enabled. |
| bind-visible `ArgPreparationProfile` metadata version | routed_to_oxfunc | OxFunc | OxFml will consume once OxFunc provides canonical versioning. |
| affected callable/function identity set | routed_to_oxfunc | OxFunc | Needed for targeted invalidation; OxCalc may conservatively rebind until available. |
| canonical profile name/stable serialization | routed_to_oxfunc | OxFunc | OxFml consumes as function metadata. |
| relationship to `StructureContextVersion` | adapted | OxFml/OxFunc | OxFml keeps workbook structure and function metadata versioning distinct; final linked invalidation model needs OxFunc version signal. |
| folded-plan identity/stable folding trace | deferred_with_successor_owner | OxFml successor with OxFunc input where semantic equivalence is function-owned | Blocker: no canonical folded plan form yet. |
| folding reason/classification | deferred_with_successor_owner | OxFml successor | Same blocker as folded-plan identity. |
| template reuse/cache counters | deferred_with_successor_owner | OxFml successor | Current artifact reuse report is not the canonical template cache counter. |
| collision/compat diagnostics for cache keys | deferred_with_successor_owner | OxFml successor | Owner: template cache successor. |

## Name Adaptation
OxCalc compatibility names are accepted as intent, not frozen API spelling.

Preferred OxFml terminology:
1. `PreparedFormulaPackage` or equivalent for OxCalc `PreparedCallable`,
2. `PlanTemplate` for the reusable plan-template identity,
3. `HoleBindingSet` for OxCalc `HoleBindings`,
4. `FormalReference` / `FormalReferenceSet` for canonical reference/input
   transport.

## OxFunc-Owned Pieces
1. `ArgPreparationProfile` source metadata,
2. affected-function or affected-callable sets for narrower invalidation,
3. any function-metadata version that makes argument-preparation changes
   bind-visible,
4. any folding metadata that depends on function-semantic equivalence rather
   than OxFml-only plan normalization.

## Deferred Pieces
1. exact public Rust struct names,
2. full parent/child prepared-call invocation nesting beyond the current ordered
   trace floor,
3. canonical cache lifetime policy,
4. exact folding equivalence classes,
5. implementation of the new fields in the runtime facade.

## OxFml Counterpart Changes Landed In This Pass
1. Added `RuntimePreparedFormulaIdentity`, `RuntimePlanTemplateIdentity`,
   `RuntimeHoleBindingIdentity`, and `RuntimeFormalReference` to the public
   runtime facade.
2. Added prepared identity fields to `RuntimeFormulaResult`,
   `RuntimeManagedOpenResult`, `RuntimeManagedExecutionResult`, and
   `RuntimeManagedSessionSnapshot`.
3. Added prepared identity projection to `ReplayProjectionResult` for runtime
   result and managed-session families.
4. Added focused runtime/replay assertions for direct execution, managed
   session identity parity, formal-reference projection, and replay projection.

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
  - implement canonical prepared package, plan template, hole binding, and
    formal-reference fields
  - OxFunc metadata/version cooperation for argument-preparation invalidation
  - OxCalc migration away from W050 compatibility fingerprints after fields land
