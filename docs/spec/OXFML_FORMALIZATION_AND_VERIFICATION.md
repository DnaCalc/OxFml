# OxFml Formalization and Verification

## 1. Purpose
This document defines the near-formal verification posture for OxFml.

OxFml is not just a parser and evaluator implementation target. It is part of the DNA Calc core semantics stack and must be specified so that important contracts can be modeled, checked, and replayed.

The current clause-to-artifact planning register is:
1. `OXFML_FORMAL_ARTIFACT_REGISTER.md`

The current local checked-formal floor is:
1. `../../formal/lean/OxFmlSessionLifecycle.lean`
2. `../../formal/lean/OxFmlExternalReferenceDeferred.lean`
3. `../../formal/lean/OxFmlNameCarrierDeferred.lean`
4. `../../formal/lean/OxFmlFailureStageSplit.lean`
5. `../../formal/lean/OxFmlExternalNameCarrier.lean`
6. `../../formal/tla/FecSessionLifecycle.tla`
7. `../../formal/tla/FecSessionLifecycle.cfg`
8. `../../formal/tla/FecExternalCapabilityGate.tla`
9. `../../formal/tla/FecExternalCapabilityGate.cfg`
10. `../../formal/tla/FecHigherOrderCallableBoundary.tla`
11. `../../formal/tla/FecHigherOrderCallableBoundary.cfg`
12. `../../formal/tla/FecSessionContentionBoundary.tla`
13. `../../formal/tla/FecSessionContentionBoundary.cfg`
14. `../../formal/tla/FecRetryAfterReleaseBoundary.tla`
15. `../../formal/tla/FecRetryAfterReleaseBoundary.cfg`
16. `../../formal/tla/FecOverlayCleanupBoundary.tla`
17. `../../formal/tla/FecOverlayCleanupBoundary.cfg`
18. `../../formal/tla/FecPinnedEpochOverlayBoundary.tla`
19. `../../formal/tla/FecPinnedEpochOverlayBoundary.cfg`
20. `../../formal/tla/FecDistributedPlacementBoundary.tla`
21. `../../formal/tla/FecDistributedPlacementBoundary.cfg`
22. `../../formal/tla/FecRetryOrderingBoundary.tla`
23. `../../formal/tla/FecRetryOrderingBoundary.cfg`
24. `../../formal/tla/FecPlacementDeferralExpiryBoundary.tla`
25. `../../formal/tla/FecPlacementDeferralExpiryBoundary.cfg`
26. `../../formal/run_formal.ps1`

This continues the broader DNA Calc verification posture already present across sibling lanes:
1. Lean-oriented semantic and type formalization in Foundation and OxFunc,
2. TLA+-oriented concurrency and async verification in the broader engine/tooling stack, including OxVba-hosted model-checking practice,
3. replay-backed conformance evidence as the concrete witness layer.

## 2. Verification Stack
OxFml uses a coupled assurance stack:
1. normative prose specs,
2. typed executable data structures,
3. deterministic replay artifacts,
4. Lean formalization for structural and semantic invariants,
5. TLA+ models for concurrent and async seam behavior,
6. conformance packs and scenario suites.

No critical semantic clause should exist only as prose when a typed or model-checked form is practical.

Runner rule:
1. a local formal artifact is operationally stronger when the repo also contains the deterministic command surface that checks it,
2. where practical, checked local formal artifacts should ship with a canonical runner or command path in the repo,
3. docs should distinguish clearly between authored-only artifacts and checked artifacts with a known local execution path.

## 3. Lean-Oriented Surfaces
The following OxFml surfaces should be designed for Lean-friendly formalization:

1. syntax-tree ADTs
   - green node shape,
   - token/span fidelity,
   - recovery node admissibility.
2. bind outputs
   - normalized reference forms,
   - unresolved-reference classification,
   - bind-context dependence.
3. prepared-call contracts
   - argument structure classes,
   - evaluation modes,
   - result classes,
   - reference preservation.
4. reject taxonomy
   - typed reject families,
   - no-publish-on-reject rule,
   - fence mismatch classification.
5. commit bundle types
   - atomic bundle shape,
   - required delta families,
   - optional feature-gated delta families.
6. canonical artifact shapes
   - `BoundFormula`,
   - `SemanticPlan`,
   - `PreparedCall` / `PreparedResult`,
   - `CommitBundle` / `RejectRecord`.
7. seam delta and reject taxonomies
   - delta family typing,
   - evaluator-fact families,
   - trace-event families,
   - reject-context families.

## 4. TLA+-Oriented Surfaces
The following OxFml surfaces should be designed for TLA+ modeling:

1. evaluator session lifecycle
   - `prepare -> open_session -> capability_view -> execute -> commit`
2. session registry and expiry rules
3. snapshot and token fence enforcement
4. commit atomicity and publish/no-publish split
5. multi-session contention and retry behavior
6. overlay visibility and epoch-safe eviction
7. staged concurrency promotion from sequential coordinator to partitioned parallel evaluators

TLA+ is especially important for Stage 2 and later seam behavior, where concurrency and async execution introduce correctness risks that replay alone cannot exhaust.

## 5. Replay as the Concrete Witness Layer
Replay artifacts are the concrete bridge between prose and formal claims.

For OxFml, replay artifacts should witness:
1. parse and normalization outcomes,
2. bind-context-dependent reference resolution,
3. prepared OxFunc call boundaries,
4. dynamic dependency rediscovery,
5. spill event sequences,
6. formatting-sensitive invalidation behavior,
7. commit accept/reject boundaries,
8. concurrency-sensitive contention scenarios when Stage 2 work begins.

## 6. Required Coupling Rule
Every important OxFml contract should map to at least one of:
1. Lean theorem or typed formal artifact,
2. TLA+ property or model-check scenario,
3. deterministic replay artifact,
4. conformance pack row.

If a contract currently lacks one of these, the gap must be recorded explicitly.

## 7. Near-Term Formal Priorities
The first formal priorities for OxFml are:
1. full-fidelity green/red syntax invariants,
2. bind normalization soundness and explicit unresolved-reference handling,
3. prepared-call structure preservation for OxFunc-facing semantics,
4. no-publish-on-reject and fence-soundness rules for FEC/F3E,
5. session lifecycle and commit atomicity in sequential coordinator mode,
6. TLA+ modeling of concurrent session conflicts before Stage 2 promotion.

The current checked local floor now covers:
1. a Lean-checked session lifecycle and no-publish artifact,
2. a Lean-checked external-reference deferment and capability-admissibility artifact,
3. a Lean-checked deferred-name-carrier classification artifact for formula-bearing name lanes,
4. a Lean-checked failure-stage split artifact for edit rejection versus accepted-unresolved versus semantic-plan/runtime/provider outcomes,
5. a Lean-checked external-name carrier artifact for same-external-book restriction plus provider-stage runtime typing,
6. a Lean-checked async-capability consequence artifact for the external-provider lane,
7. a TLC-checked TLA+ session lifecycle and publish-safety model,
8. a TLC-checked TLA+ external capability gate model including async-consequence invariants,
9. a TLC-checked TLA+ higher-order callable boundary model for catalog admission versus callable-invoker rejection,
10. a TLC-checked TLA+ session-contention boundary model for busy-locus rejection versus publishable execution,
11. a TLC-checked TLA+ retry-after-release boundary model for coordinator-visible retry admissibility after busy-locus rejection,
12. a TLC-checked TLA+ overlay-cleanup boundary model for session-local, epoch-scoped overlays under commit, abort, and expiry,
13. a TLC-checked TLA+ pinned-epoch overlay boundary model for exact-match retained reuse and unpinned deterministic eviction,
14. a TLC-checked TLA+ distributed-placement boundary model for local admission versus remote-placement deferral and no-publish until locally admitted commit,
15. a TLC-checked TLA+ retry-ordering fairness boundary model for non-overtaking among already surfaced retry-admissible sessions,
16. a TLC-checked TLA+ placement-deferral expiry boundary model for deferred remote placement that expires or rejects without local claim or publication,
17. a canonical local execution path through `formal/run_formal.ps1`.

## 8. Open Lanes
The following formal lanes remain open and must be tracked as such:
1. the precise Lean boundary between OxFml semantic plans and OxFunc function definitions,
2. the richer TLA+ model shape for overlay retention beyond the current pinned-epoch reuse and eviction boundary,
3. proof obligations for fast-path soundness,
4. proof obligations for deterministic parallel reduction,
5. successful higher-order callable execution beyond the current checked callable-invoker rejection boundary,
6. richer overlay reuse and pinned-epoch eviction models beyond the current retained-reuse boundary,
7. richer fairness and distributed placement models beyond the current retry-ordering, local-admission, and deferred-placement-expiry boundaries,
8. the final trace-schema split between subsystem traces and unified engine replay.
