# OxFml Implementation Baseline

## 1. Purpose
This document narrows the implementation-shape discussion to the current working baseline for starting code.

It does not freeze the final packaging or all API details.
It does freeze the current implementation direction strongly enough that parser, binder, semantic-plan, and evaluator code can start against a coherent model.

Read together with:
1. `OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`
2. `OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
3. `OXFML_CANONICAL_ARTIFACT_SHAPES.md`
4. `formula-language/OXFML_PARSER_AND_BINDER_REALIZATION.md`
5. `formula-language/OXFML_NORMALIZED_REFERENCE_ADTS.md`
6. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
7. `OXFML_HIGH_RISK_AND_EARLY_ATTENTION_AREAS.md`
8. `OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`
9. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`

## 2. Baseline Decision
The current OxFml implementation baseline is:
1. Option C hybrid canonical-core plus runtime services,
2. with the canonical-core surface treated as normative,
3. and runtime services treated as optional operational accelerators.

This is stronger than a preference and weaker than a final public API freeze.

## 3. Canonical-Core Rule
The baseline implementation must expose a coherent canonical core made of explicit transforms:
1. parse formula source -> `GreenTreeRoot`,
2. project context -> red view,
3. bind syntax plus context -> `BoundFormula`,
4. compile bind artifact plus OxFunc metadata -> `SemanticPlan`,
5. evaluate plan plus fenced context -> accepted candidate result or typed reject,
6. commit candidate under FEC/F3E -> published bundle or typed no-publish reject.

Every runtime service must be observationally reducible to that core.

## 4. Runtime-Service Rule
Runtime services may exist for:
1. syntax interning,
2. bind and semantic-plan caching,
3. evaluator session tracking,
4. overlay retention and reuse,
5. replay trace capture.

They must not become the only place where canonical meaning exists.

## 5. Public Surface Direction
The current public surface direction should separate:
1. artifact constructors and transforms,
2. optional repository or cache services,
3. evaluator session services,
4. seam/publication results.

The current code-facing sketch for that surface is:
1. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`

Current intended baseline:
1. parse/bind/plan should be usable without a long-lived runtime,
2. evaluation may use either direct stateless invocation or a managed session runtime,
3. commit and reject consequences remain typed seam outputs, not host callbacks.

## 6. Storage and Ownership Direction
The current expected storage direction is:
1. durable formula/document artifacts live above evaluator sessions,
2. OxCalc-integrated mode will likely retain durable green/bind/plan artifacts in host or coordinator-owned immutable structures,
3. OxFml may still provide repositories for convenience and caching,
4. DNA OneCalc may use lighter direct ownership without changing semantics.

## 7. Baseline Implementation Order
The intended code-start order is:
1. canonical artifact ADTs,
2. parser and green-tree realization,
3. binder and normalized reference ADTs,
4. semantic-plan compilation against OxFunc metadata,
5. minimal local bootstrap evaluator path over immutable inputs,
6. OxFunc-backed semantic execution path,
7. single-formula recalc host path,
8. runtime repositories and caches,
9. managed FEC/F3E session services.

This order is deliberate:
1. canonical semantics first,
2. runtime convenience second.

## 8. Mandatory API Constraints
The first implementation surface must preserve:
1. explicit formula identity and version keys,
2. explicit structure/profile/caller context,
3. explicit accepted-candidate vs published-commit distinction,
4. explicit typed reject outcomes,
5. explicit replay correlation handles,
6. no dependence on raw workbook object leakage into OxFunc,
7. explicit execution and scheduling profile surfaces where function or formula evaluation is not generically concurrent-safe.

The current normalized reference ADT baseline for those API constraints is:
1. `formula-language/OXFML_NORMALIZED_REFERENCE_ADTS.md`

## 9. Concurrency and Scheduling Baseline
The implementation baseline must leave room for high-performance concurrent and async calculation from the start.

This does not mean Stage 2 concurrency is implemented immediately.
It does mean:
1. semantic plans must be able to carry execution-profile metadata,
2. OxFml must be able to expose scheduler-relevant formula or call profiles without leaking scheduler policy into the seam,
3. host-query, thread-affine, async, or otherwise restricted function lanes must be modelable explicitly,
4. replay and formal artifacts must be able to observe those restrictions.

## 10. Deferred Decisions
The following are intentionally deferred:
1. final package/module boundaries,
2. wire/serialization encodings,
3. whether repositories are trait-based, object-based, or free-function based,
4. exact lifetime and residency policy for runtime caches,
5. final public handle types for repository-backed mode.

## 11. Working Rule
Implementation work should proceed as if:
1. the canonical-core surface is the semantic contract,
2. runtime services are allowed but non-authoritative,
3. any optimization path must be testable against the direct canonical-core path.
