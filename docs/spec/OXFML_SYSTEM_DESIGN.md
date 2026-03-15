# OxFml System Design

## 1. Purpose
This document is the top-level OxFml system design for the formula-processing and single-node evaluation lane.

It defines:
1. the canonical internal structure of the OxFml library and repository,
2. the relationship between formula parsing, binding, semantic planning, evaluation, and FEC/F3E publication,
3. the boundaries with OxFunc and OxCalc,
4. the formal-model and verification posture that OxFml must carry from the start.

## 2. Architectural Role in DNA Calc
OxFml is the permanent lane owner for:
1. Excel-compatible formula language processing,
2. full-fidelity syntax and versioned formula views,
3. bind/reference normalization and evaluator-side dependency evidence,
4. single-node evaluation semantics,
5. the evaluator side of the FEC/F3E seam,
6. evaluator-side reject and trace structures.

OxFml is not:
1. the owner of global scheduling policy,
2. the owner of workbook-wide dependency closure strategy,
3. the owner of function-kernel semantics,
4. a host-specific pathfinder implementation repo.

## 3. Canonical Bootstrap Reading Order
When bootstrapping OxFml design work, read this local spec set in this order:
1. `docs/spec/OXFML_SYSTEM_DESIGN.md`
2. `docs/spec/OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`
3. `docs/spec/OXFML_IMPLEMENTATION_BASELINE.md`
4. `docs/spec/OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
5. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
6. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
7. `docs/spec/OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`
8. `docs/spec/OXFML_HIGH_RISK_AND_EARLY_ATTENTION_AREAS.md`
9. `docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
10. `docs/spec/OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`
11. `docs/spec/OXFML_FORMALIZATION_AND_VERIFICATION.md`
12. `docs/spec/OXFML_FORMAL_ARTIFACT_REGISTER.md`
13. `docs/spec/formula-language/README.md`
14. `docs/spec/formula-language/OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
15. `docs/spec/formula-language/OXFML_PARSER_AND_BINDER_REALIZATION.md`
16. `docs/spec/formula-language/OXFML_NORMALIZED_REFERENCE_ADTS.md`
17. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
18. `docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md`
19. `docs/spec/fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
20. `docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`
21. `docs/spec/fec-f3e/FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`
22. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
23. `docs/spec/formatting/EXCEL_FORMATTING_HIERARCHY_AND_VISIBILITY_MODEL.md`

Historical transition material lives under archive paths and is not part of bootstrap reading.

## 4. Internal Library Structure
OxFml should be organized as a coherent library with these major subsystems:

1. `syntax`
   - tokenization,
   - green syntax trees,
   - red contextual views,
   - full-fidelity round-tripping.
2. `binding`
   - name and table resolution,
   - address-mode and caller-context application,
   - normalized references,
   - dependency seed extraction.
3. `semantics`
   - operator and function dispatch planning,
   - OxFunc catalog integration,
   - evaluation-mode classification,
   - fast-path classification.
4. `evaluation`
   - prepared-argument/result construction,
   - reference-preserving execution,
   - dynamic dependency discovery,
   - spill and formatting overlay discovery.
5. `fec_f3e`
   - evaluator session lifecycle,
   - capability view,
   - commit bundle construction,
   - reject taxonomy,
   - seam trace emission.
6. `replay_and_conformance`
   - scenario definitions,
   - replay bundles,
   - conformance-matrix integration,
   - deterministic trace validation.
7. `formal`
   - Lean-oriented ADT/spec surfaces,
   - TLA+-oriented session/concurrency models,
   - proof/pack obligation mapping.

This decomposition is conceptual first and implementation-oriented second. Exact source-tree layout may evolve, but the separation of concerns must remain.

## 5. Layer Relationships
The canonical OxFml flow is:
1. formula text enters `syntax`,
2. contextual views flow into `binding`,
3. bound formulas compile into `semantics`,
4. execution uses `evaluation` plus OxFunc metadata,
5. publication uses `fec_f3e`,
6. replay and proofs consume emitted artifacts from all prior layers.

No layer is allowed to erase distinctions that a downstream layer depends on semantically.

## 6. Ownership and Stateful-vs-Stateless Posture
The implementation plan for API shape and runtime ownership is still intentionally open.

The canonical option analysis for this topic is:
1. `docs/spec/OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`

The canonical vocabulary for identities, version keys, fingerprints, and runtime handles is:
1. `docs/spec/OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`

The canonical field surfaces for the main artifact families are:
1. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`

The canonical minimum schema objects for seam payload families are:
1. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`

The canonical taxonomy layer for deltas, evaluator facts, reject contexts, and trace events is:
1. `docs/spec/OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`

The current implementation-start baseline is:
1. `docs/spec/OXFML_IMPLEMENTATION_BASELINE.md`

The current early-risk register is:
1. `docs/spec/OXFML_HIGH_RISK_AND_EARLY_ATTENTION_AREAS.md`

The current public API and runtime-service baseline is:
1. `docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`

The current test ladder and proving-host model are:
1. `docs/spec/OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`

The current formal planning register is:
1. `docs/spec/OXFML_FORMAL_ARTIFACT_REGISTER.md`

What is already fixed:
1. parse trees and other core semantic artifacts must admit immutable versioned representations,
2. those artifacts must be suitable for inclusion in larger immutable workbook/document structures above OxFml,
3. formula meaning must be explainable from explicit artifacts plus explicit context, not from hidden mutation.

What remains open:
1. whether the public implementation surface is mostly stateless or exposed as long-lived services,
2. whether parse-tree and bind-artifact storage is primarily host-owned, OxCalc-owned, or packaged behind OxFml repositories,
3. how much execution-state residency is retained between evaluations.

Constraint:
1. even if implementations maintain caches, indexes, or evaluator session registries, those must be optimization state,
2. canonical semantic truth must still be representable in a stateless, replayable form,
3. persistent workbook/document ownership above the evaluator belongs to the enclosing host or coordinator, not to ephemeral execution sessions.

## 7. Boundary with OxFunc
OxFunc is the downstream semantic companion library.

OxFml depends on OxFunc for:
1. function identifiers,
2. function profiles and traits,
3. argument evaluation rules,
4. coercion rules,
5. may-return-reference behavior,
6. locale and format service needs,
7. reduction-order constraints where deterministic execution depends on them,
8. query classification and result-shaping policy for typed host-query functions such as `CELL` and `INFO`.

OxFml must not force OxFunc to recover distinctions that OxFml erased.

The canonical OxFml-local statement of this boundary is:
1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`

That document is the primary local promotion of the active OxFunc upstream `NOTES_FOR_OXFML` requirements.

## 8. Boundary with OxCalc
OxCalc is the upstream coordinator and multi-node engine owner.

OxFml provides OxCalc with:
1. typed commit bundles,
2. overlay-derived topology facts,
3. typed rejects,
4. replay-stable traces,
5. evaluator-side capability and bind evidence.

OxCalc retains ownership of:
1. dirty closure,
2. scheduling and publication policy,
3. fairness and visibility policy,
4. multi-session contention policy.

OxCalc is also the likely owner of higher immutable workbook/document structures in the integrated mode, though the precise storage surface remains open.

## 9. Host Modes
OxFml must support two major consumption modes:

1. **OxCalc-integrated mode**
   - full evaluator seam usage against coordinator-owned snapshot and policy state.
2. **DNA OneCalc mode**
   - single-node proving host using OxFml and OxFunc without OxCalc dependency closure or scheduler policy.

The pre-DNA-OneCalc proving-host ladder is defined in:
1. `docs/spec/OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`

DNA OneCalc proves the OxFml/OxFunc lane. It does not define the lane.

## 10. Formal and Assurance Posture
OxFml is part of DNA Calc's near-formal core.

From the start, OxFml specs must be written so they can support:
1. Lean ADTs and invariants for syntax, bind outputs, prepared-call contracts, and reject structures,
2. TLA+ models for concurrent evaluator sessions and commit/publish rules,
3. deterministic replay bundles for semantic and concurrency-sensitive scenarios,
4. explicit mapping from each important contract clause to a proof, model check, or conformance pack.

## 11. Working Rule
Canonical OxFml design docs must describe:
1. the intended baseline architecture,
2. the intended formal and replay obligations,
3. the declared open lanes.

They must not present legacy pathfinder implementation text as the current OxFml baseline.
