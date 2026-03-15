# W006: Formatting Semantics and Host-Query Follow-Through

## Purpose
Close the remaining evaluator-facing semantic lanes that cut across OxFml formatting semantics, host-query behavior, and publication consequences.

## Position and Dependencies
- **Depends on**: `W003`, `W004`
- **Blocks**: `W008` and later formatting-semantic implementation and downstream `TEXT` / `CELL` / `INFO` proving work
- **Cross-repo**: OxFunc boundary alignment required; OxCalc seam review may be used on an ad hoc basis if new publication-visible consequences are promoted

## Scope
### In scope
1. Tighten semantic formatting behavior that crosses the evaluator seam.
2. Tighten host-query effects that can influence prepared results, publication hints, or format dependencies.
3. Narrow the semantic-vs-display boundary for evaluator-owned formatting consequences.
4. Plan the first proving scenarios for `TEXT`, `VALUE`, `NOW`, `TODAY`, `CELL`, and `INFO`-adjacent lanes.

### Out of scope
1. Pure renderer/UI display behavior.
2. Full formatting mini-language completion.
3. OxCalc scheduler policy.

## Deliverables
1. A narrowed formatting-semantic seam baseline.
2. A clearer rule set for host-query and formatting interactions with prepared results and publication.
3. Updated fixture/pack planning for semantic formatting and host-query scenarios.

## Gate Model
### Entry gate
- OxFunc boundary and host-query capability baseline are explicit enough to refine.
- FEC/F3E schema and replay planning exists for publication-facing consequences.

### Exit gate
- Semantic formatting versus display-only boundaries are explicit enough for implementation-start.
- Host-query and formatting consequence rules are explicit enough for proving-scenario authoring.
- Any new coordinator-visible consequences are handed off if mature enough.

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
- open_lanes: broader formatting mini-language coverage remains outside this workset; current publication hints and format dependency facts are still local baseline behavior rather than pack-grade validated corpus; and no external Excel empirical run set has been authored from this repo yet
- claim_confidence: validated
