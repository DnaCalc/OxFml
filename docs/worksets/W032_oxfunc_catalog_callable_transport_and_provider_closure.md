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
2. Broaden OxFunc catalog coverage for the currently exercised semantic lanes where OxFml still uses narrow local metadata.
3. Narrow the minimum callable-value carrier and helper-result transport facts beyond the current summary-plus-detail baseline.
4. Tighten the split between:
   - early formula rejection,
   - semantic-plan unsupported/gated states,
   - runtime capability denial,
   - post-dispatch or provider-failure outcomes.
5. Add deterministic replay and proving artifacts for the narrowed catalog/callable/provider families.

### Out of scope
1. Full user-defined function product surface.
2. Final downstream publication policy for callable values in every host mode.
3. OxCalc-owned coordinator scheduling or retry policy.
4. Broad formula-language review work better handled by `W031`.

## Deliverables
1. A narrower canonical OxFml/OxFunc boundary for library-context snapshot, callable carrier, and provider-stage outcomes.
2. Broader exercised semantic-plan and prepared-result evidence for those lanes.
3. Updated outbound observation notes for OxFunc if the narrowed baseline materially changes the shared reading.
4. A focused prep artifact for pinning down the `LET` / `LAMBDA` callable seam once the general callable floor is narrow enough.

## Gate Model
### Entry gate
- `W026` has established stage-aware availability summaries and snapshot refs.
- `W027` has established a stronger local callable-value baseline.
- `W029` has established async-coupled external-provider runtime facts worth connecting back to the shared OxFunc taxonomy.

### Exit gate
- The smallest honest shared library-context snapshot is narrower than today.
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
  - broader OxFunc catalog coverage for currently exercised semantic lanes is still partial
  - the split between early formula rejection, semantic-plan unsupported states, runtime capability denial, and provider-failure outcomes is narrower locally but not yet final
  - the callable-value carrier now covers helper-local and adopted defined-name callable lanes, but it is still provisional rather than final
  - the `LET` / `LAMBDA` seam pin-down agenda is now explicit, but the final carrier vs provenance vs invocation split is still open
  - broader UDF/product transport and final callable publication policy remain outside this workset scope
- claim_confidence: moderate
