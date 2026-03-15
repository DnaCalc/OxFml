# FEC/F3E Design Specification

## 1. Purpose
This document is the canonical OxFml-owned specification for the evaluator seam between OxFml and OxCalc.

The seam exists to let OxFml evaluate one formula instance against a versioned workbook snapshot and produce either:
1. an accepted candidate result payload suitable for coordinator-controlled atomic publication, or
2. a typed reject outcome with no accepted-state publication.

Baseline transaction lane:
1. `prepare`
2. `open_session`
3. `capability_view`
4. `execute`
5. `commit`

Canonical posture:
1. this document defines the live seam contract,
2. `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md` defines how the contract is supposed to be witnessed,
3. archive material and later run packs are support inputs, not bootstrap authority.

## 2. Scope and Ownership
OxFml owns:
1. evaluator session lifecycle,
2. evaluator-side capability requirements,
3. commit bundle shape,
4. evaluator-side trace and reject-detail schema,
5. overlay participation rules for dynamic references, spill, and format-sensitive evaluation.

OxCalc owns:
1. coordinator scheduling policy,
2. publication fencing policy beyond the evaluator contract,
3. dirty-closure and global recalc policy,
4. contention handling policy for concurrent evaluators.

OxFunc owns:
1. function semantic definitions,
2. coercion and evaluation traits exposed through the OxFunc catalog,
3. function-family reduction and evaluation rules consumed by OxFml semantic planning.

The prepared semantic boundary consumed by OxFunc is defined in:
1. `../formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`

The canonical identity/version vocabulary used by this seam is defined in:
1. `../OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`

The canonical field surfaces for accepted candidate results, commit bundles, and reject records are defined in:
1. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`

The canonical minimum schema objects for deltas, spill events, reject contexts, and trace payloads are defined in:
1. `../OXFML_MINIMUM_SEAM_SCHEMAS.md`

The canonical taxonomy layer for delta families, evaluator facts, reject contexts, and trace events is defined in:
1. `../OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`

## 3. Evidence and Assurance Posture
This seam spec is policy-first.
It should remain stable enough to drive implementation and cross-repo handoff work even before replay artifacts are refreshed.

Working rule:
1. live seam clauses belong in this document and the assurance map,
2. replay packs, run outputs, and handoff packets are evidence layers that support but do not replace the canonical seam wording,
3. stage-promotion or closure claims still require replay and cross-repo acknowledgment.

## 4. Architectural Position
FEC/F3E sits between:
1. OxFml parse/bind/semantic-plan/evaluation logic, and
2. OxCalc coordinator publication and scheduling logic.

It is the transactional evaluator publication seam.

FEC/F3E session state is evaluator-operational state.
It must not be confused with ownership of canonical formula syntax, bind artifacts, or higher workbook/document versions.
Those canonical artifacts remain externalized and versionable above the session boundary.

## 5. Session Identity and Fences
### 5.0 Prepare semantics
`prepare` is the pre-session validation and request-shaping step.

It exists to:
1. validate that the formula identity is known,
2. capture the current fence tuple before session open,
3. determine whether session open is admissible for the requested profile and capability scope,
4. reject early when the request cannot legally progress to evaluation.

### 5.1 Stable identity
Every session is anchored by stable evaluator identity:
1. `formula_stable_id`
2. `formula_token`
3. `snapshot_epoch`
4. `bind_hash`
5. `profile_version`

Human-readable labels are metadata only. They are not contractual identity.
Typed supporting identities such as name ids and spill-range ids remain part of seam evidence where the evaluator exposes them.

These fence members are intentionally a mixed category:
1. `formula_stable_id` is stable logical identity,
2. `formula_token`, `snapshot_epoch`, and `profile_version` are version/fence keys,
3. `bind_hash` is a bind-result fingerprint used operationally as a fence key.

### 5.2 Fence rules
`commit` must reject when any required fence no longer matches:
1. formula token mismatch,
2. snapshot epoch mismatch,
3. bind hash mismatch,
4. profile-version mismatch,
5. capability-view mismatch,
6. expired or aborted session.

Fence consequence rule:
1. incompatible or stale candidate work is rejected rather than partially published,
2. fence incompatibility must yield typed reject detail sufficient for deterministic replay,
3. no evaluator-side accepted state may be treated as published state until commit acceptance succeeds.

## 6. Capability Model
`capability_view` is the evaluator-side declaration and validation step for optional or profile-gated behaviors.

Rules:
1. capability decisions are session-bound,
2. capability denial must be machine-typed,
3. capability state must be revalidated at commit,
4. no hidden capability assumptions are allowed between execute and commit.

Host-query rule:
1. when function semantics depend on cell, workbook, or environment facts, the capability view must expose typed host-query capabilities rather than raw host objects or ad hoc callbacks.

## 7. Overlay Model
FEC/F3E mediates session-local overlay participation.

The baseline overlay families are:
1. calc-time dependency overlay,
2. spill overlay,
3. format dependency overlay.

Overlay rules:
1. overlays are derived state, never mutation of canonical structural truth,
2. overlay writes are session-local until commit,
3. overlay reuse requires exact fence match on epoch, token, bind hash, and profile version,
4. overlay eviction must be deterministic and epoch-safe.

## 8. Execute Semantics
`execute` runs one semantic plan against one fenced session context.

Execution may:
1. materialize prepared arguments for OxFunc,
2. discover dynamic references,
3. register spill and format overlay entries,
4. produce typed evaluator facts for later publication,
5. construct an `AcceptedCandidateResult` when evaluation succeeds under the active session fences,
6. terminate with a typed reject when the session cannot legally continue.

Execution must not:
1. publish partial global state,
2. mutate OxCalc-owned scheduler policy,
3. replace typed evaluator failures with opaque error strings.

## 9. Accepted Candidate Result Contract
Successful evaluation is not itself publication.

The accepted evaluator outcome is an `AcceptedCandidateResult`.
It is a non-published candidate payload presented for coordinator-controlled commit acceptance.

The canonical candidate-result shape is defined in:
1. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`

Rules:
1. accepted candidate results and committed publication are distinct layers,
2. accepted candidate results must carry enough structured content for one coherent atomic publication if the coordinator accepts them,
3. accepted candidate results must carry the compatibility basis needed for deterministic accept-versus-reject decisions,
4. accepted candidate results must surface or make derivable any runtime-discovered evaluator effects that materially affect coordinator correctness.

Coordinator-relevant runtime-derived effect families include at least:
1. dynamic-reference discoveries,
2. spill discoveries, conflicts, and typed spill events,
3. format-dependency discoveries,
4. capability-sensitive execution observations where they affect accept/reject or publication consequences,
5. execution-profile or execution-restriction facts where safe scheduling or publication interpretation depends on them.

## 10. Commit Bundle Contract
Accepted commits promote an `AcceptedCandidateResult` into one atomic published derived bundle.

Evaluator success does not itself imply publication.
Publication occurs only when the coordinator accepts the candidate result under compatible fences.

The baseline bundle shape is:
1. `value_delta`
2. `shape_delta`
3. `topology_delta`
4. optional `format_delta`
5. optional `display_delta`
6. trace fragment or trace correlation metadata

`topology_delta` carries evaluator facts and dependency evidence, not scheduler policy judgments.
Global recalc policy remains OxCalc-owned.

The current minimum field sets for these payloads are defined in:
1. `../OXFML_MINIMUM_SEAM_SCHEMAS.md`

## 11. Spill Event Contract
Spill semantics are explicit shape facts, not inferred side effects.

The baseline spill-event families are:
1. `SpillTakeover`
2. `SpillClearance`
3. `SpillBlocked`

Each spill event must carry enough typed context to let OxCalc derive invalidation behavior without guessing.

## 12. Reject Taxonomy
Rejects are typed, replay-stable, and non-publishing.

Baseline reject families include:
1. token and snapshot fence mismatch,
2. capability denial,
3. session expiry or session abort,
4. bind mismatch,
5. structural conflict,
6. dynamic-reference failure classes,
7. profile-version mismatch,
8. resource exhaustion,
9. internal invariant violation.

Reject consequence rule:
1. rejected work publishes no accepted state,
2. fence and capability incompatibilities must produce structured reject detail rather than ambiguous failure classes,
3. reject detail must be sufficient for deterministic replay and coordinator diagnostics,
4. the minimum typed reject-context schemas are defined in `../OXFML_MINIMUM_SEAM_SCHEMAS.md`.

## 13. Trace Contract
Tracing is part of the seam contract.

Each traced event must be:
1. versioned,
2. schema-validatable,
3. correlated to formula/session identity,
4. sufficient to replay session outcome classification.

Trace correlation must be sufficient to distinguish:
1. candidate-result construction,
2. commit acceptance versus commit rejection,
3. reject-with-no-publish outcomes,
4. surfaced runtime-derived effect reporting that influences coordinator correctness.

## 14. Formal Modeling Hooks
FEC/F3E must be specified so the seam can be checked as part of DNA Calc's near-formal core.

Lean-oriented seam artifacts should include:
1. typed session-state ADTs,
2. commit bundle shape,
3. reject taxonomy,
4. no-publish-on-reject rule,
5. fence-soundness rules for the sequential baseline.

TLA+-oriented seam artifacts should include:
1. session lifecycle and state transitions,
2. snapshot/token/capability fence behavior,
3. concurrent commit contention and retry behavior,
4. session expiry and abort cleanup,
5. publish authority and atomicity rules.

Replay artifacts remain the concrete witness layer tying these models back to executable behavior.

The canonical seam-level witness map is defined in:
1. `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`

## 15. Boundary Discipline
### 15.1 OxCalc boundary
FEC/F3E publishes evaluator facts and typed outcomes.
OxCalc consumes those facts and owns scheduler and global publication policy.

### 15.2 OxFunc boundary
OxFml prepares arguments and results for OxFunc while preserving reference- and provenance-sensitive distinctions.
OxFunc semantic rules do not erase OxCalc policy boundaries.

## 16. Stage Guidance
The baseline spec is written for Stage 1 sequential coordinator semantics first.

Stage 2 and beyond require additional closure for:
1. deterministic contention replay,
2. session-registry concurrency hardening,
3. epoch-pinning overlay GC safety,
4. parallel reduction determinism where applicable.

## 17. Related Documents
1. `README.md`
2. `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
3. `FEC_F3E_TESTING_AND_REPLAY.md`
4. `../OXFML_SYSTEM_DESIGN.md`
5. `../OXFML_FORMALIZATION_AND_VERIFICATION.md`
6. `../formula-language/OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
