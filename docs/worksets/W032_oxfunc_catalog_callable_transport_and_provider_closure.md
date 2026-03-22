# W032: OxFunc Catalog, Callable Transport, and Provider Closure

## Purpose
Narrow the remaining OxFml/OxFunc seam around broader catalog breadth, callable-value transport, and stage-aware availability/provider outcomes so the next semantic/runtime wave stops relying on provisional carriers and note-ledger alignment alone.

## Position and Dependencies
- **Depends on**: `W026`, `W027`, `W029`
- **Blocks**: `W034`, `W035`
- **Cross-repo**: OxFml remains authoritative for grammar, bind, semantic-plan, and evaluator meaning; OxFunc remains authoritative for catalog truth, callable semantic value behavior, and function/operator semantic traits

## Scope
### In scope
1. Narrow the minimum external library-context snapshot field set needed for semantic planning, replay correlation, and runtime/provider distinctions.
2. Define the preferred normative runtime library-context interface so OxFml does not depend on build-time catalog-file ingestion for implementation use.
3. Broaden OxFunc catalog coverage for the currently exercised semantic lanes where OxFml still uses narrow local metadata.
4. Narrow the minimum callable-value carrier and helper-result transport facts beyond the current summary-plus-detail baseline.
5. Tighten the split between:
   - early formula rejection,
   - semantic-plan unsupported/gated states,
   - runtime capability denial,
   - post-dispatch or provider-failure outcomes.
6. Add deterministic replay and proving artifacts for the narrowed catalog/callable/provider families.

### Out of scope
1. Full user-defined function product surface.
2. Final downstream publication policy for callable values in every host mode.
3. OxCalc-owned coordinator scheduling or retry policy.
4. Broad formula-language review work better handled by `W031`.

## Deliverables
1. A narrower canonical OxFml/OxFunc boundary for library-context snapshot, callable carrier, and provider-stage outcomes.
2. A preferred runtime library-context interface model that supports immutable snapshots plus runtime registration/removal.
3. Broader exercised semantic-plan and prepared-result evidence for those lanes.
4. Updated outbound observation notes for OxFunc if the narrowed baseline materially changes the shared reading.
5. A focused prep artifact for pinning down the `LET` / `LAMBDA` callable seam once the general callable floor is narrow enough.
6. A concrete OxFml-side ask for a pinned OxFunc catalog snapshot export or stable pointer suitable for semantic-plan and test consumption.
7. A bounded integration-round posture that uses concrete exports and concrete seam mismatches rather than indefinite note-only narrowing.

## Gate Model
### Entry gate
- `W026` has established stage-aware availability summaries and snapshot refs.
- `W027` has established a stronger local callable-value baseline.
- `W029` has established async-coupled external-provider runtime facts worth connecting back to the shared OxFunc taxonomy.

### Exit gate
- The smallest honest shared library-context snapshot is narrower than today.
- The preferred runtime library-context interface is explicit enough that OxFml can consume built-ins and later runtime registrations without build-time catalog-file dependence.
- The callable-value carrier is narrower than the current provisional summary-only transport posture.
- The availability/provider taxonomy is exercised across semantic-plan and runtime paths without collapsing distinct failure classes.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - broader OxFunc catalog coverage for currently exercised semantic lanes is still partial, even though first-pass `W044` snapshot consumption now covers selected ordinary, seam-heavy, and higher-order helper rows
  - the preferred runtime library-context interface direction is now explicit and OxFunc has now confirmed the runtime snapshot/provider model as the normative implementation direction, but OxFml still needs a real consumer/modeling pass rather than only a prose agreement
  - the split between accepted-unresolved, semantic-plan gated, runtime capability denied, and post-dispatch provider-unavailable outcomes now has dedicated deterministic evidence and a first checked Lean artifact, but the full edit-rejection policy boundary and final seam lock are still not final
  - the callable-value carrier now covers helper-local, helper-bound, and adopted defined-name callable lanes, and OxFml now executes `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` through a local typed callable-invoker bridge, but the carrier is still provisional rather than final
  - the `LET` / `LAMBDA` seam pin-down agenda is now explicit, and typed invocation over opaque callable identity is now locally exercised; the remaining open question is the final carrier vs provenance field split rather than whether the invocation boundary itself is viable
  - first-pass local consumption of the downstream `W044` snapshot export now exists for selected seam-heavy, ordinary, and higher-order helper rows, but broader replacement of narrow local catalog assumptions is still incomplete
  - higher-order helper rows now have first local end-to-end runtime evidence for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY`; `ISOMITTED` is no longer a first-freeze seam blocker, but broader callable-family closure still remains outside the exercised runtime floor
  - the next useful narrowing step is now to use the exercised typed-invocation floor plus broader `W044` field or interpretation mismatches to lock a smaller honest callable carrier while moving the successor-packet locks into `W041`, `W042`, and `W043`
  - the final OxFunc note in this round now treats the note exchange as converged; the next work is local execution of `W041`, `W042`, and `W043` rather than another clarification round
  - the typed context/query bundle, returned publication-aware value surface, and runtime provider-consumer model for the already-covered application scope are now explicit next-lock lanes, but they are not yet canonically frozen on the OxFml side
  - broader UDF/product transport and final callable publication policy remain outside this workset scope
- claim_confidence: moderate
