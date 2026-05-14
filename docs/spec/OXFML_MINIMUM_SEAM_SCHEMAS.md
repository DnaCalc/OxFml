# OxFml Minimum Seam Schemas

## 1. Purpose
This document defines the current minimum schema objects for the most important seam payload families.

It is narrower than the artifact-shapes document and more concrete than the taxonomy document.
Its job is to say what the minimum typed payloads must carry without prematurely freezing final wire encodings or implementation structs.

This document should be read together with:
1. `OXFML_CANONICAL_ARTIFACT_SHAPES.md`
2. `OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`
3. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`
4. `formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`

## 2. Working Rule
For the schema objects below:
1. field families are canonical,
2. exact concrete type names remain open,
3. optional implementation-only debug fields do not belong in the canonical minimum,
4. typed variant payloads are preferred over generic maps or free-form strings.

## 3. Delta Schema Objects
### 3.1 `ValueDelta`
`ValueDelta` carries worksheet-visible value publication consequences.

Minimum fields:
1. `delta_family`
2. `formula_stable_id`
3. `primary_locus`
4. `affected_value_loci`
5. `published_value_class`
6. `published_payload`
7. optional `blankness_transition`
8. optional `result_extent`
9. optional correlation to candidate result or commit attempt

Minimum rules:
1. `published_value_class` must distinguish scalar, error, array-anchor payload, and explicit blank-like publication,
2. if the value consequence depends on an array extent, `result_extent` must be capturable,
3. `ValueDelta` must not carry dependency-only or policy-only information.

### 3.2 `ShapeDelta`
`ShapeDelta` carries occupancy and shape publication consequences.

Minimum fields:
1. `delta_family`
2. `formula_stable_id`
3. `anchor_locus`
4. `intended_extent`
5. optional `published_extent`
6. optional `blocked_loci`
7. `shape_outcome_class`
8. optional correlation to candidate result or commit attempt

Minimum rules:
1. `shape_outcome_class` must distinguish at least established, reconfigured, cleared, and blocked shape outcomes,
2. blocked outcomes must carry explicit blocking loci when capturable,
3. shape publication must remain distinct from value publication even when both arise from one result.

### 3.3 `TopologyDelta`
`TopologyDelta` carries coordinator-consumable evaluator facts and dependency consequences.

Minimum fields:
1. `delta_family`
2. `formula_stable_id`
3. optional `dependency_additions`
4. optional `dependency_removals`
5. optional `dependency_reclassifications`
6. optional `dependency_consequence_fact_refs`
7. optional `dynamic_reference_fact_refs`
8. optional `spill_fact_refs`
9. optional `format_dependency_tokens`
10. optional `capability_effect_refs`
11. optional correlation to candidate result or commit attempt

Minimum rules:
1. topology facts must be typed and machine-comparable,
2. topology payloads must not contain scheduler or fairness policy,
3. if a surfaced evaluator fact is coordinator-relevant but carried outside `TopologyDelta`, this delta must still make the publication consequence derivable,
4. dependency consequence facts are additive evidence and do not replace explicit removals or reclassifications where those are already contractual.

### 3.4 `FormatDelta`
`FormatDelta` carries semantic formatting consequences that must cross the seam.

Minimum fields:
1. `delta_family`
2. `formula_stable_id`
3. `target_loci`
4. `format_effect_class`
5. `format_effect_payload`
6. optional `dependency_token_refs`

Working rule:
1. `FormatDelta` may be derived from prepared-result `format_hint` when the hint crosses the seam as a publication obligation,
2. a local prepared-result hint alone does not imply a seam-significant `FormatDelta`.

### 3.5 `DisplayDelta`
`DisplayDelta` is optional and exists only when a publication-surface consequence is a seam obligation.

Minimum fields:
1. `delta_family`
2. `formula_stable_id`
3. `target_loci`
4. `display_effect_class`
5. `display_effect_payload`

Working rule:
1. `DisplayDelta` may be derived from prepared-result `publication_hint` when the publication surface itself is seam-significant,
2. renderer-only display changes remain out of scope.

## 4. Evaluator Fact and Event Schema Objects
### 4.1 `DynamicReferenceFact`
Minimum fields:
1. `fact_kind`
2. `formula_stable_id`
3. `discovery_site`
4. optional `reference_identity`
5. optional `target_extent`
6. optional `resolution_failure_class`
7. optional prior-versus-current comparison marker

### 4.2 `SpillFact`
Minimum fields:
1. `fact_kind`
2. `formula_stable_id`
3. `anchor_locus`
4. `intended_extent`
5. optional `published_extent`
6. optional `blocked_loci`
7. optional `blocked_reason_class`

### 4.3 `FormatDependencyFact`
Minimum fields:
1. `fact_kind`
2. `formula_stable_id`
3. `dependency_token`
4. `dependency_class`
5. optional locale/date-system/format-service scope

### 4.4 `CapabilityEffectFact`
Minimum fields:
1. `fact_kind`
2. `formula_stable_id`
3. `capability_kind`
4. `phase_kind`
5. `effect_class`
6. optional `fallback_class`

Current local exercised families additionally include:
1. `async_coupling`
2. `serial_scheduler_lane`
3. `single_flight`
4. `thread_affinity`

### 4.5 `DependencyConsequenceFact`
Minimum fields:
1. `fact_kind`
2. `formula_stable_id`
3. `dependency_identity`
4. `consequence_kind`
5. `evidence_class`
6. `projection_state`

### 4.6 `SpillEvent`
Minimum fields:
1. `spill_event_kind`
2. `formula_stable_id`
3. `anchor_locus`
4. `intended_extent`
5. optional `affected_extent`
6. optional `blocking_loci`
7. optional `blocking_reason_class`
8. correlation to candidate result or commit attempt

### 4.7 `PreparedFormulaIdentity`
Minimum fields:
1. `prepared_formula_key`
2. `formula_stable_id`
3. `formula_text_version`
4. source token identity or source payload fingerprint
5. `library_context_snapshot_ref`
6. `structure_context_version`
7. optional caller/locus context key
8. `plan_template_key`
9. `hole_binding_fingerprint`
10. optional `formal_reference_set_ref`
11. optional `semantic_kernel_metadata_version`
12. optional `arg_admission_metadata_version`

Minimum rules:
1. this schema is the OxFml-owned replacement target for OxCalc W050
   compatibility fingerprints.
2. exact public type names remain open, but the identity categories must stay
   distinct.
3. `semantic_kernel_metadata_version` and `arg_admission_metadata_version` are
   OxFunc-owned invalidation bridges when present; they are not OxFml-owned
   kernel or admission semantics.

### 4.8 `PlanTemplateIdentity`
Minimum fields:
1. `shape_key`
2. `dispatch_skeleton_key`
3. `plan_template_key`
4. optional folded-plan identity or folding classification when OxFml admits
   identity-affecting folding
5. ordered `TemplateHole` entries

Minimum `TemplateHole` fields:
1. `hole_id`
2. `ordinal`
3. optional path within the semantic plan
4. `hole_kind`
5. stable `hole_kind_key`

Allowed initial `hole_kind` families:
1. `ValueHole`
2. `RefOrValueHole`
3. `CallableHole`
4. `ShapeSensitiveHole`
5. `SparseRangeHole`
6. `RichValueHole`

### 4.9 `HoleBindingSet`
Minimum fields:
1. `hole_binding_fingerprint`
2. per-hole binding records keyed by `hole_id`
3. binding payload category
4. optional formal-reference or invocation-input link

Minimum rules:
1. literal values, concrete references, omitted arguments, and helper names are
   binding payloads by default.
2. future narrower producers must add stable-keyed template identity rather than
   silently changing this default.

### 4.10 `FormalReference`
Minimum fields:
1. `reference_handle`
2. canonical reference descriptor or normalized reference text
3. reference family
4. optional caller-anchor/address-mode dependence
5. optional host-mappable identity payload
6. optional reference-to-hole or reference-to-input binding identity

Minimum reference families:
1. direct
2. relative or caller-sensitive
3. unresolved
4. host-sensitive
5. dynamic-potential
6. capability-sensitive
7. shape/topology-sensitive

### 4.11 `CorrectnessFloorContext`
Minimum fields:
1. `profile_version`
2. `numerical_reduction_policy`
3. `error_algebra`
4. optional `semantic_kernel_metadata_version`

Minimum rules:
1. selector values are semantic context and replay identity.
2. selector mismatch is replay-invalid unless a migration proof is attached.
3. OxFml carries the context; OxFunc owns kernel enforcement and metadata for
   affected functions.
4. `semantic_kernel_metadata_version` is the prepared-package invalidation
   bridge when OxFunc selector behavior, affected-function classification, or
   reduction/error-collapse metadata changes.
5. runtime and replay fields for selector values and the metadata version are
   reserved until OxFml emits them from a real profile-context source.

### 4.12 `RichValueCapabilityColumns`
Minimum fields:
1. `required_capability_set_keys`
2. optional `producer_capability_set_keys`
3. optional `exercised_capability_keys`
4. optional producer carrier identity or returned rich/sparse carrier identity
5. optional `arg_admission_metadata_version`

Minimum rules:
1. required capability-set keys are template/replay identity for
   `RichValueHole`.
2. producer and exercised keys remain absent or empty until concrete producer
   and kernel activation exists.
3. empty current output means reserved/no-producer evidence, not rich-value
   capability support.
4. producer capability publication is typed metadata on the producer or returned
   rich/sparse carrier, not a function-name-specific diagnostic string.
5. `arg_admission_metadata_version` is the prepared-package invalidation bridge
   when OxFunc argument-preparation/admission metadata changes.
6. `IMAGE` / `_webimage` is the preferred first rich producer activation lane;
   sparse range readers remain deferred.

## 5. Typed Reject-Context Schemas
### 5.1 `FenceMismatchContext`
Minimum fields:
1. `mismatch_member_kind`
2. `expected_value`
3. `observed_value`
4. `mismatch_class`

### 5.2 `CapabilityDenialContext`
Minimum fields:
1. `capability_kind`
2. `phase_kind`
3. `denial_class`
4. `fallback_available`

### 5.3 `SessionTerminationContext`
Minimum fields:
1. `termination_class`
2. `session_id`
3. `candidate_already_built`
4. optional `termination_cause`

### 5.4 `BindMismatchContext`
Minimum fields:
1. `bind_hash`
2. optional `bound_formula_id`
3. `mismatch_class`
4. `discovery_phase`

### 5.5 `StructuralConflictContext`
Minimum fields:
1. `conflict_kind`
2. `conflicting_loci`
3. optional `conflicting_extent`
4. `retry_admissibility`

### 5.6 `DynamicReferenceFailureContext`
Minimum fields:
1. `dynamic_reference_family`
2. `failure_class`
3. optional `partial_reference_identity`
4. optional `discovery_site`

### 5.7 `ResourceInvariantContext`
Minimum fields:
1. `failure_family`
2. `machine_detail_code`
3. optional `resource_class`
4. optional implementation-only debug detail kept outside the canonical minimum

### 5.8 `CapabilitySetMismatchContext`
Minimum fields:
1. `required_capability_set_keys`
2. optional `producer_capability_set_keys`
3. mismatch phase
4. mismatch class
5. optional template hole identity

Minimum rules:
1. if producer capability facts are known during bind or plan admission,
   mismatch may surface as a bind or semantic-plan diagnostic.
2. if producer facts are only known during execution, mismatch must surface as a
   typed evaluation reject or diagnostic.
3. producer-superset admission must not rewrite the required-set identity
   recorded in the template.
4. mismatch timing depends on when OxFml can observe OxFunc producer metadata;
   OxFml must not invent producer capabilities when OxFunc has not emitted them.

## 6. Host-Query Capability Schema
### 6.1 `HostQueryCapabilityView`
This schema supports functions like `CELL` and `INFO`.

Minimum fields:
1. `capability_view_key`
2. `profile_version`
3. `available_cell_query_kinds`
4. `available_workbook_fact_kinds`
5. `available_environment_fact_kinds`
6. optional `selection_context_support_class`
7. `denial_policy_class`

Boundary rule:
1. this view exposes typed fact families,
2. it must not expose raw workbook object handles,
3. when omitted-reference host-query lanes depend on active selection, the view must be able to report whether active-selection context is available,
4. it may be absent in call paths that do not admit host-query semantics.

## 7. Trace Event Schema
### 7.1 `TraceEvent`
Minimum fields:
1. `trace_schema_id`
2. `event_kind`
3. `formula_stable_id`
4. optional `session_id`
5. optional `candidate_result_id`
6. optional `commit_attempt_id`
7. optional `reject_record_id`
8. optional `fence_snapshot_ref`
9. `event_order_key`
10. typed `event_payload`

Minimum rules:
1. trace events must distinguish candidate construction from publication,
2. reject events must be correlatable to typed reject contexts,
3. surfaced evaluator effects must be representable either directly in `event_payload` or by stable typed references.
4. prepared-package, plan-template, hole-binding, formal-reference,
   correctness-floor, and rich-capability columns should be structured fields or
   typed payload members when present, not diagnostic strings.

## 8. Replay Adapter Additive Schema Fields
The replay rollout adds optional projection-facing fields without changing OxFml seam meaning.

### 8.1 `ReplayBundleEnvelopeRef`
Optional additive fields:
1. `bundle_id`
2. `run_id`
3. `scenario_id`
4. `source_schema_id`
5. `bundle_schema_id`
6. `bundle_schema_version`

Working rule:
1. this object links a local seam payload to a normalized replay envelope,
2. it does not replace OxFml-local identity or fence fields.

### 8.2 `ReplayRegistryBinding`
Optional additive fields:
1. `registry_family`
2. `registry_version`
3. `entry_id`

Working rule:
1. registry bindings reference Foundation replay governance families,
2. registry bindings may annotate a seam payload or fixture scenario,
3. registry bindings do not redefine the local typed payload they annotate.

### 8.3 `CapabilityManifestRef`
Optional additive fields:
1. `adapter_id`
2. `adapter_version`
3. `lane_id`
4. `manifest_ref`

Working rule:
1. this ref links a replay-normalized payload to the current adapter capability manifest,
2. capability claims remain rollout governance, not semantic truth.

### 8.4 `WitnessLifecycleRef`
Optional additive fields:
1. `witness_id`
2. `lifecycle_state`
3. optional `retention_policy_id`
4. optional `quarantine_reason`
5. optional `reduction_manifest_ref`

Working rule:
1. lifecycle refs may annotate replay witnesses or reduced witness bundles,
2. lifecycle refs must not change the semantic meaning of candidate, commit, reject, or effect payloads.

### 8.5 `ProjectionFieldStatus`
Optional additive fields:
1. `field_path`
2. `status_class`
3. optional `reason_class`
4. optional `source_sidecar_ref`

Allowed `status_class` values in this pass:
1. `present`
2. `missing_explicit`
3. `opaque_preserved`

Working rule:
1. these markers exist to prevent invented defaults during normalization,
2. they are additive replay metadata only.

Additive replay rule:
1. any schema object in this document may carry optional `bundle_envelope_ref`, `registry_bindings`, `capability_manifest_ref`, `witness_lifecycle_ref`, or `projection_field_status` members,
2. these members are optional and additive,
3. no existing OxFml seam field is redefined to fit them.

## 9. Open Decisions
The following remain open:
1. whether `ValueDelta` and `ShapeDelta` are best represented as one-per-family object or as containers over entry lists,
2. exact payload typing for `published_payload` and `format_effect_payload`,
3. whether some fact refs are embedded objects versus stable ids,
4. whether Stage 2 concurrency needs finer-grained reject contexts for retry versus terminal failure,
5. exact encoding of `event_order_key` for cross-engine replay portability.

## 10. Working Rule
Until implementation begins:
1. use these schemas as the minimum typed surface,
2. prefer additive refinement over collapsing fields,
3. keep coordinator-visible consequences derivable without ad hoc interpretation.
