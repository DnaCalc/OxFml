# OxFml Delta, Effect, Trace, and Reject Taxonomies

## 1. Purpose
This document defines the current canonical taxonomy layer for:
1. seam delta families,
2. evaluator-fact families,
3. reject-context families,
4. trace-event families.

The artifact-shapes document defines which containers exist.
This document defines the expected semantic contents of the most important families inside those containers.

This document should be read together with:
1. `OXFML_CANONICAL_ARTIFACT_SHAPES.md`
2. `OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
3. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`
4. `fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`

## 2. Working Rule
The seam must not use broad buckets like "other topology info" or "generic reject detail" as a substitute for typed taxonomies.

Until implementation begins:
1. taxonomy families should be explicit,
2. exact field names may remain open,
3. new families should be added only when replay or coordinator correctness requires them.

## 3. Delta Family Taxonomy
### 3.1 `value_delta`
`value_delta` carries worksheet-visible value consequences.

The current minimum families are:
1. scalar value replacement
2. error value replacement
3. array payload publication where the value consequence is visible at the formula locus
4. blankness transition where worksheet-visible cell value semantics change

`value_delta` must not be used for:
1. dependency-only changes,
2. scheduler policy,
3. purely display-only formatting effects.

### 3.2 `shape_delta`
`shape_delta` carries occupancy and shape consequences.

The current minimum families are:
1. spill extent establishment
2. spill extent shrink/expand
3. spill occupancy clearance
4. blocked-shape state at intended spill targets
5. array-shape publication changes that affect visible occupied region

### 3.3 `topology_delta`
`topology_delta` carries coordinator-consumable evaluator facts and dependency consequences.

The current minimum families are:
1. dynamic dependency additions
2. dynamic dependency removals
3. dependency classification changes
4. runtime-discovered reference target facts
5. invalidation-relevant spill facts
6. format-dependency facts that affect future invalidation behavior
7. capability-sensitive execution facts when they alter coordinator interpretation

`topology_delta` must not contain:
1. global scheduling decisions,
2. fairness policy,
3. coordinator-local publication heuristics.

### 3.4 `format_delta`
`format_delta` carries semantic formatting consequences that must cross the seam.

The current minimum families are:
1. format recommendation or adjustment linked to formula semantics
2. semantic format changes required for stable downstream evaluation meaning

Open boundary:
1. the exact split between `format_delta` and prepared-result `format_hint` remains profile-sensitive.

### 3.5 `display_delta`
`display_delta` is optional and only for publication-surface consequences that are explicit seam obligations.

Current rule:
1. if a display-facing consequence is purely renderer/UI-local, it does not belong here,
2. if a display-facing consequence is required for evaluator/publication semantics, it may appear here.

## 4. Evaluator-Fact Taxonomy
Evaluator facts are pre-publication execution observations that may feed `topology_delta`, typed event sets, or trace payloads.

### 4.1 Dynamic Reference Facts
Current minimum families:
1. discovered target region identity
2. discovered target shape/classification
3. failure-to-resolve dynamic target
4. change in discovered target compared with previously known shape

### 4.2 Spill Facts
Current minimum families:
1. intended spill anchor and intended extent
2. spill blocked reason and blocking loci
3. spill takeover confirmation
4. spill clearance confirmation
5. spill reconfiguration under changed result shape

### 4.3 Format Dependency Facts
Current minimum families:
1. formula depends on semantic format state
2. formula depends on locale/date-system-sensitive rendering/parsing context
3. format-dependency token/classification needed for later invalidation

### 4.4 Capability-Sensitive Execution Facts
Current minimum families:
1. feature/capability path exercised
2. capability-denied path classification
3. fallback path chosen due to capability profile

### 4.5 Fact Relationship Rule
Evaluator facts:
1. may remain local if they have no coordinator or replay consequence,
2. must be surfaced or made derivable when they affect accept/reject/publication correctness,
3. must remain typed rather than collapsed into free-form diagnostics.

## 5. Spill Event Taxonomy
The canonical spill-event families remain:
1. `SpillTakeover`
2. `SpillClearance`
3. `SpillBlocked`

Required typed context for every spill event should include at least:
1. anchor identity
2. intended extent or affected extent
3. blocking or cleared loci where applicable
4. correlation to candidate result / commit attempt

## 6. Reject Taxonomy Refinement
The seam already defines top-level reject families.
This section defines the current minimum typed-context families inside `RejectRecord`.

### 6.1 Fence Mismatch Context
Minimum context:
1. mismatched fence member kind
2. expected versus observed values where capturable
3. stale-versus-incompatible classification

### 6.2 Capability Denial Context
Minimum context:
1. denied capability or profile gate
2. phase where denial occurred
3. whether fallback existed

### 6.3 Session Expiry / Abort Context
Minimum context:
1. expiry versus explicit abort
2. affected session identity
3. whether evaluation had already produced a candidate result

### 6.4 Bind Mismatch Context
Minimum context:
1. relevant bind artifact identity/fingerprint
2. mismatch classification
3. whether the mismatch was discovered before or during commit

### 6.5 Structural Conflict Context
Minimum context:
1. conflicting structural locus or shape
2. conflict kind
3. whether the conflict is recoverable by retry

### 6.6 Dynamic-Reference Failure Context
Minimum context:
1. failing dynamic-reference family
2. resolution failure class
3. any partial target identity that was available

### 6.7 Resource / Invariant Context
Minimum context:
1. resource exhaustion versus invariant violation classification
2. replay-safe machine detail
3. optional implementation-only debug detail kept out of canonical minimums

## 7. Trace Event Taxonomy
Trace events must be sufficient to replay and diagnose the seam lifecycle without confusing candidate construction with publication.

### 7.1 Lifecycle Events
Current minimum families:
1. `PrepareStarted`
2. `PrepareRejected`
3. `SessionOpened`
4. `CapabilityViewResolved`
5. `ExecuteStarted`
6. `ExecuteCompleted`

### 7.2 Candidate / Commit Events
Current minimum families:
1. `AcceptedCandidateResultBuilt`
2. `CommitStarted`
3. `CommitAccepted`
4. `CommitRejected`

### 7.3 Reject Events
Current minimum families:
1. `RejectIssued`
2. `FenceMismatchRejected`
3. `CapabilityDeniedRejected`
4. `SessionExpiredRejected`

This does not mean each family must be a separate top-level enum variant in every implementation.
It means replay must be able to distinguish them.

### 7.4 Effect and Overlay Events
Current minimum families:
1. `DynamicReferenceDiscovered`
2. `SpillEventObserved`
3. `FormatDependencyObserved`
4. `OverlayRegistered`
5. `OverlayEvicted`

### 7.5 Correlation Rule
Every trace event should be able to correlate to some combination of:
1. formula identity
2. session identity
3. candidate-result identity or fingerprint
4. commit attempt identity
5. reject identity or fingerprint

## 8. Candidate-vs-Published Consequence Rule
When a candidate result exists and commit later rejects it:
1. trace and reject artifacts must show that a candidate existed,
2. no published bundle is emitted,
3. the failure reason must remain machine-typed and replayable.

## 9. Testing and Replay Implications
The minimum replay families implied by this taxonomy are:
1. candidate-result built then commit accepted
2. candidate-result built then commit rejected on fence mismatch
3. execution rejected before candidate-result construction
4. spill blocked / cleared / reconfigured with surfaced effects
5. format dependency discovery affecting later invalidation behavior

## 10. Replay Appliance Mapping
Foundation replay registries provide normalized mismatch, predicate, and severity families.
OxFml local taxonomy remains authoritative for source meaning and source family membership.

### 10.1 Predicate-Family Mapping
The current additive predicate mapping is:
1. typed reject-family preservation
   - normalized predicate: `pred.reject.family_present`
   - local source authority: `RejectRecord.reject_code` plus typed reject-context family
2. no-publish reason preservation
   - normalized predicate: `pred.publication.not_published_reason`
   - local source authority: candidate-versus-commit boundary plus typed reject context
3. mismatch-presence preservation over effect or topology surfaces
   - normalized predicate: `pred.diff.mismatch_present`
   - local source authority: typed delta, fact, and trace families in this document
4. invariant or failure preservation where resource or machine rules matter
   - normalized predicate: `pred.invariant.failed`
   - local source authority: `ResourceInvariantContext`

### 10.2 Mismatch-Family Mapping
The current additive mismatch mapping is:
1. candidate or commit value/shape/view consequence mismatch
   - normalized mismatch: `mm.result.state` and/or `mm.view.value`
2. typed reject-family or reject-context mismatch
   - normalized mismatch: `mm.reject.kind`
3. lifecycle or effect trace mismatch
   - normalized mismatch: `mm.trace.event`
4. missing required replay sidecar payload where semantic surface is otherwise preserved
   - normalized mismatch: `mm.sidecar.payload`
5. missing required capability, lifecycle, or registry binding used for replay governance
   - normalized mismatch: `mm.evidence.binding`

Mapping rule:
1. normalized mismatch ids classify cross-lane comparison surface,
2. local taxonomy ids still determine what the underlying OxFml event, reject, or effect means.

### 10.3 Severity Alignment
The current additive severity alignment is:
1. `sev.semantic`
   - value, shape, topology, reject, candidate, commit, and required effect mismatches
2. `sev.coverage`
   - missing lifecycle, registry, or required evidence bindings that affect assurance or promotion claims
3. `sev.instrumentation`
   - missing optional enrichments, sidecars, or opaque-preserved payloads that do not alter replay truth
4. `sev.informational`
   - optional explanation-text or advisory projection differences only

### 10.4 Authority Rule
Normalized replay mappings:
1. are additive and versioned,
2. may be used for shared diff and explain tooling,
3. may not silently replace OxFml taxonomy ids or typed family meanings.

## 11. Open Decisions
The following remain open:
1. exact nested payload structure inside each delta family
2. whether some evaluator facts appear only in `topology_delta` versus separate fact sets
3. exact trace event naming and granularity
4. whether some reject-context families should be split further for Stage 2 concurrency work

## 12. Working Rule
Until implementation begins:
1. use these taxonomies as the semantic baseline,
2. keep publication-facing consequences typed,
3. defer only where exercised evidence is genuinely still missing.
