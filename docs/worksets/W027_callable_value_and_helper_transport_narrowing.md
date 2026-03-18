# W027: Callable-Value and Helper-Transport Narrowing

## Purpose
Narrow the callable-value and helper-result seam enough that OxFml and OxFunc can stop relying on replay-summary-only treatment for the current local helper floor while still keeping publication restrictions distinct from semantic admissibility.

## Position and Dependencies
- **Depends on**: `W026`
- **Blocks**: later richer higher-order semantics and any downstream callable-value integration planning
- **Cross-repo**: OxFunc remains the owner of callable semantic behavior beyond the OxFml helper/evaluator floor; OxFml remains the owner of helper syntax, lexical capture, invocation planning, and replay-preserved helper artifacts

## Scope
### In scope
1. Define the minimum callable-value carrier facts needed beyond the current replayable summary surface.
2. Narrow helper-result and prepared-result callable provenance enough for downstream OxFunc consumption.
3. Keep lexical capture, helper shadowing, and invocation planning explicit in the canonical artifact and replay surfaces.
4. Add deterministic replay and proving-host artifacts for the narrowed callable carrier.

### Out of scope
1. Full higher-order function breadth or currying completeness.
2. Broad UDF transport closure for callable values.
3. Coordinator-facing publication-policy changes unless a new callable publication consequence actually appears.

## Deliverables
1. A narrower callable-value carrier baseline shared between OxFml and OxFunc.
2. Updated helper-result and prepared-result provenance surfaces for callable values.
3. Deterministic replay/proving artifacts showing lexical-capture-sensitive callable transport beyond the current summary-only floor.

## Gate Model
### Entry gate
- `W026` has narrowed the surrounding library-context and availability surfaces enough that callable transport can be tightened honestly.

### Exit gate
- Callable-value minimum carrier facts are explicit in the canonical OxFml/OxFunc boundary.
- Helper-result callable provenance is narrower and better exercised than the current replay-summary-only floor.
- Deterministic replay artifacts prove lexical-capture-sensitive callable transport over at least one direct and one helper-bound invocation path.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | yes |
| 6 | No known semantic gaps remain in declared scope? | yes |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes: richer higher-order callable breadth, UDF transport closure, and any callable-publication coordinator consequences remain open outside this narrowing workset
- claim_confidence: high
