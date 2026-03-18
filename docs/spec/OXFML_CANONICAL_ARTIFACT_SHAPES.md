# OxFml Canonical Artifact Shapes

## 1. Purpose
This document defines the current canonical field surfaces for the main OxFml artifacts.

The goal is to stabilize:
1. what information each artifact family must carry,
2. which distinctions are canonical versus optional,
3. where later implementation work may add internal detail without changing semantics.

This document does not freeze exact language-level types or wire encodings.
It defines the semantic shape that implementations and later formal artifacts should preserve.

This document should be read together with:
1. `OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
2. `OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`
3. `formula-language/OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
4. `formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
5. `OXFML_MINIMUM_SEAM_SCHEMAS.md`
6. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`
7. `OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`

## 2. Shape Rule
Each canonical artifact shape should separate:
1. identity/version metadata,
2. semantic payload,
3. diagnostics or evidence metadata,
4. optional implementation-only residency data.

Only the first three belong in the canonical semantic surface.

## 3. Formula Source Record
The formula source record captures the worksheet-surface formula input state before and after normalization.

Minimum fields:
1. `formula_stable_id`
2. `formula_text_version`
3. `entered_formula_text`
4. optional `stored_formula_text`
5. source span or host-locus metadata
6. parse-time diagnostics set

Working rule:
1. entered and stored text must remain distinguishable when the host surface distinguishes them,
2. this record is the textual entry point for parse and replay.

## 4. Green Tree Root
The canonical green-tree root should carry at least:
1. `green_tree_key`
2. optional `green_tree_fingerprint`
3. root syntax kind
4. full-fidelity token/trivia tree
5. recovery/error nodes where present
6. parse diagnostics

Canonical property:
1. no workbook-specific or caller-specific context belongs in the green tree.

## 5. BoundFormula
`BoundFormula` is the canonical binding output for one formula under one structure/profile context.

Minimum fields:
1. `formula_stable_id`
2. syntax identity reference
   - at least the relevant green-tree/root identity
3. `structure_context_version`
4. `profile_version`
5. optional `bound_formula_id`
6. `bind_hash`
7. bound expression root
8. normalized reference set
9. dependency seed set
10. unresolved-reference records
11. capability requirements
12. bind diagnostics

Bound expression root should preserve:
1. operator/function structure,
2. normalized names and references,
3. caller-context-dependent bindings where they are already resolved at bind stage,
4. explicit unresolved nodes where binding cannot finish honestly.

What does not belong in `BoundFormula`:
1. evaluator session state,
2. mutable overlay state,
3. commit/publication decisions.

## 6. SemanticPlan
`SemanticPlan` is the evaluator-facing artifact compiled from `BoundFormula` plus OxFunc metadata.

Minimum fields:
1. semantic-plan identity
   - `semantic_plan_key`
   - optional `semantic_plan_fingerprint`
2. `formula_stable_id`
3. `bind_hash`
4. relevant library-context snapshot identity
5. relevant OxFunc catalog/profile identity
6. operator/function dispatch graph
7. evaluation-mode requirements
8. reduction policy requirements
9. reference-preservation requirements
10. helper-environment profile
   - at minimum whether `LET`, `LAMBDA`, and helper invocation are present
   - and whether lexical helper capture is required by the formula shape
11. overlay participation flags
12. locale/format service requirements
13. execution and scheduling profile requirements
14. availability/gating summary where formula admission or runtime capability depends on catalog/profile/provider state
15. fast-path classification
16. semantic diagnostics or unsupported-lane markers

Current local floor:
1. `library_context_snapshot_ref` records the consumed external library-context snapshot identity when present,
2. `availability_summaries` preserve parse/bind, semantic-plan, runtime-capability, and post-dispatch/provider states per surfaced function lane.

Canonical property:
1. `SemanticPlan` explains how evaluation should proceed,
2. it does not itself contain runtime session state,
3. it may carry library-context and availability truth needed to preserve semantic admission distinctions without owning mutable registry state.

## 7. PreparedArgument
Prepared arguments are the canonical OxFml-to-OxFunc call-shape units.

Minimum fields:
1. argument ordinal or named-position identity
2. `structure_class`
3. `source_class`
4. `value_view`
5. optional `reference_identity`
6. `evaluation_mode`
7. `blankness_class`
8. `caller_context`
9. optional provenance metadata
10. preparation diagnostics if the argument is representable but degraded

Canonical property:
1. prepared arguments must carry enough structure for OxFunc to apply Excel-compatible function semantics without reconstructing lost provenance.

## 8. PreparedCall
`PreparedCall` is the canonical evaluator-to-function dispatch package.

Minimum fields:
1. function identity
2. function profile/trait reference from OxFunc
3. prepared argument list
4. call-site caller context
5. locale/date-system/format service context
6. optional host-query capability view
7. evaluation mode summary for the call
8. optional replay correlation metadata

Working rule:
1. `PreparedCall` may be materialized eagerly or lazily,
2. its semantic content must remain reconstructible for replay.

## 9. PreparedResult
Prepared results are the canonical function-to-evaluator result units.

Minimum fields:
1. `result_class`
2. `structure_class`
3. `payload`
4. optional `reference_identity`
5. optional `format_hint`
6. optional `publication_hint`
7. optional callable-value profile and structured callable detail
8. optional provenance/derivation marker
9. result diagnostics if the result carries degraded or version-scoped semantics

Canonical property:
1. prepared results must distinguish scalar, array, reference, and error outcomes without collapsing them prematurely.
2. callable helper values may remain semantically first-class even when publication carriers remain narrower than the full callable transport problem.

## 10. Evaluator Facts
Evaluator facts are the intermediate execution facts that later feed the seam.

Minimum field families:
1. dynamic reference discoveries
2. spill discoveries and conflicts
3. format dependency discoveries
4. capability-sensitive execution observations
5. host-query execution observations where host facts or denials affect replay or publication
6. trace correlation metadata

Working rule:
1. evaluator facts are inputs to commit-bundle construction,
2. they are not themselves scheduler policy,
3. they may still surface scheduler-relevant execution facts where coordinator correctness depends on them.

## 11. AcceptedCandidateResult
`AcceptedCandidateResult` is the canonical non-published accepted evaluator outcome.

It is the candidate payload presented for coordinator-controlled commit acceptance.

Minimum fields:
1. identity/correlation
   - `formula_stable_id`
   - optional `session_id`
   - optional candidate-result fingerprint
   - fence tuple snapshot
   - optional `capability_view_key`
2. candidate value/shape/topology payloads
   - `value_delta`
   - `shape_delta`
   - `topology_delta`
3. optional `format_delta`
4. optional `display_delta`
5. optional spill-event set
6. surfaced evaluator facts needed for coordinator correctness where not already derivable from the deltas
7. trace fragment or trace correlation metadata

Canonical property:
1. `AcceptedCandidateResult` is structured evaluator output, not publication,
2. it must be rich enough for one coherent atomic publication if accepted,
3. it must carry enough compatibility basis for deterministic accept-versus-reject decisions.

## 12. CommitBundle
`CommitBundle` is the canonical published seam artifact produced when an `AcceptedCandidateResult` is accepted for publication.

Minimum fields:
1. identity/correlation
   - `formula_stable_id`
   - `commit_attempt_id`
   - fence tuple snapshot
   - optional `commit_bundle_fingerprint`
2. `value_delta`
3. `shape_delta`
4. `topology_delta`
5. optional `format_delta`
6. optional `display_delta`
7. optional spill-event set
8. trace fragment or trace correlation metadata

`value_delta` should contain:
1. the publishable value payload changes,
2. error payload changes where the worksheet-visible value is an error.

`shape_delta` should contain:
1. spill or shape-visible occupancy changes,
2. array shape changes that affect worksheet visibility.

`topology_delta` should contain:
1. dependency and invalidation-relevant evaluator facts,
2. dynamic-reference facts,
3. typed dependency consequence facts for additions, removals, or reclassifications,
4. other coordinator-consumable facts that are not scheduler policy.

What does not belong in `CommitBundle`:
1. scheduler decisions,
2. opaque host-only side effects,
3. untyped free-form error strings as the sole explanation of behavior.

## 13. RejectRecord
`RejectRecord` is the canonical non-publishing seam artifact on rejected evaluation or commit.

Minimum fields:
1. identity/correlation
   - `formula_stable_id`
   - optional `session_id`
   - optional `commit_attempt_id`
   - optional `reject_record_fingerprint`
2. `reject_code`
3. typed reject context
4. fence snapshot or mismatch detail where relevant
5. trace correlation metadata
6. optional diagnostics/supporting evidence fields

Canonical property:
1. `RejectRecord` must be machine-typed and replay-stable,
2. it must not be only a message string,
3. fence or capability incompatibility must be expressible without ambiguity.

## 14. Trace Event Shape
The seam trace event shape should minimally carry:
1. schema/version id
2. event kind
3. formula/session/attempt correlation ids
4. relevant fence members or references to them
5. event payload
6. event ordering metadata

Canonical property:
1. trace events must be sufficient to correlate execution with either a `CommitBundle` or a `RejectRecord`.

## 15. Shape Relationship Summary
The canonical artifact flow is:
1. `FormulaSourceRecord`
2. `GreenTreeRoot`
3. `BoundFormula`
4. `SemanticPlan`
5. `PreparedCall`
6. `PreparedResult`
7. evaluator facts
8. `AcceptedCandidateResult`
9. `CommitBundle` or `RejectRecord`

Not every implementation needs to persist every artifact.
Every implementation must preserve the semantic distinctions these shapes require.

## 16. Replay Bundle Projection
The Replay appliance projects OxFml artifacts into bundle objects without replacing their meaning.

Projection guidance:
1. `FormulaSourceRecord`
   - scenario input record plus source-artifact refs
2. `GreenTreeRoot`
   - source-artifact ref or sidecar-backed syntax payload
3. `BoundFormula`
   - source-artifact ref plus bind identity/fingerprint fields
4. `SemanticPlan`
   - source-artifact ref plus execution-profile and helper-profile summary
5. `PreparedCall`
   - normalized replay event payload or sidecar-backed prepared-call packet
6. `PreparedResult`
   - candidate-facing event payload or sidecar-backed prepared-result packet
7. evaluator facts
   - normalized effect events, fact refs, or topology delta payload
8. `AcceptedCandidateResult`
   - normalized `candidate.*` events plus candidate-view material
9. `CommitBundle`
   - normalized `publication.*` events plus published-view material
10. `RejectRecord`
   - normalized `reject.*` events plus reject-set material
11. promotion-readiness and retained-witness material
   - replay-family refs, lifecycle refs, and reduction-manifest lineage remain additive sidecars rather than replacements for OxFml artifact meaning

Projection rule:
1. source schema ids remain preserved,
2. source artifact refs remain auditable,
3. normalized replay family mapping is additive only.

## 17. Sidecar And Large Artifact Rules
Large OxFml artifact bodies may be sidecar-backed in replay bundles.

Initial sidecar-capable families:
1. full-fidelity green trees,
2. full `BoundFormula` bodies,
3. full `SemanticPlan` bodies,
4. large `PreparedCall` or `PreparedResult` packets,
5. candidate or reject payload bodies that exceed inline replay practicality.

Sidecar rules:
1. sidecars must preserve content fingerprint and source schema id,
2. replay bundles must remain able to distinguish inline, sidecar-backed, missing-explicit, and opaque-preserved payload states,
3. sidecar externalization may not erase replay-causal keys needed for candidate, commit, reject, or effect interpretation.

## 18. Witness Reduction-Unit Anchors
The replay rollout requires stable anchor points for future witness distillation.

Current OxFml local-only reduction-unit anchors are:
1. `oxfml.local.reduction_unit.fixture_case`
2. `oxfml.local.reduction_unit.lifecycle_block`
3. `oxfml.local.reduction_unit.candidate_attempt`
4. `oxfml.local.reduction_unit.commit_attempt`
5. `oxfml.local.reduction_unit.reject_context_slice`
6. `oxfml.local.reduction_unit.effect_slice`
7. `oxfml.local.reduction_unit.artifact_sidecar`

Anchor rules:
1. these are OxFml-local planning ids, not Foundation registry ids,
2. candidate and commit anchors must preserve candidate-versus-publication lineage,
3. reject-context and effect-slice anchors must preserve typed family identity and causal lifecycle phase,
4. sidecar reduction may prune large artifact bodies only where replay closure remains intact.

## 19. Open Decisions
The following remain open:
1. exact field naming once implementation starts,
2. whether some identity/fingerprint fields are carried directly or via nested metadata objects,
3. the minimum provenance metadata for prepared arguments/results,
4. exact delta substructure inside `value_delta`, `shape_delta`, `topology_delta`, `format_delta`, and `display_delta`,
5. whether some evaluator facts are persisted separately from `CommitBundle`.

## 20. Working Rule
Until implementation begins:
1. use these shapes as the semantic baseline,
2. add fields only when they preserve rather than blur distinctions,
3. do not remove a field category without updating the boundary and replay rationale.

Exact delta/effect/reject/trace family taxonomies are defined in:
1. `OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`

The current minimum schema objects for those families are defined in:
1. `OXFML_MINIMUM_SEAM_SCHEMAS.md`
