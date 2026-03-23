# W047: First Host Readiness Execution Batch

## Purpose
Run one bounded execution batch that closes the immediate first-host readiness gaps identified in the current planning pass:
1. finish `W037` R1C1 channel support,
2. finish `W039` CF/DV sublanguage support,
3. freeze the exact first-host FEC packet and replay-capture path through `W046`,
4. keep host-managed name/external-name ownership explicit instead of silently expanding OxFml scope.

This workset exists to keep the next implementation push small, coherent, and implementation-facing.

## Position and Dependencies
- **Depends on**: `W041`, `W042`, `W043`, `W045`
- **Blocks**: first direct single-cell host implementation and first replay-aware host harness packet
- **Cross-repo**: OxFml owns the execution batch and resulting canonical docs; OxCalc consumes the host/FEC reading already converged at note level; OxFunc remains authoritative for function semantics and runtime library-context truth

## Scope
### In scope
1. Execute `W037` to closure for the first honest R1C1 floor.
2. Execute `W039` to closure for the first honest CF/DV formula sublanguage floor.
3. Execute `W046` to freeze the exact first-host implementation and replay-capture packet.
4. Tighten the FEC-facing host requirements so unsupported/unavailable provider behavior is explicit for the currently admitted host slice:
   - `INFO`
   - `CELL`
   - `RTD`
5. Keep `W038` explicitly narrowed to host-managed name/external-name boundary work rather than pulling that ownership into this batch.

### Out of scope
1. Full Excel language or built-in-function closure.
2. Pack-grade replay promotion.
3. Full workbook graph hosting or distributed coordinator policy.
4. Broader name/external-name workbook management beyond the host boundary statement.

## Deliverables
1. `W037` closed with explicit R1C1 channel docs and deterministic evidence.
2. `W039` closed with explicit CF/DV sublanguage docs and deterministic evidence.
3. `W046` advanced enough to freeze the exact first-host implementation packet for the current local floor.
4. Explicit host behavior for unsupported/unavailable `INFO`, `CELL`, and `RTD`.
5. Explicit replay-capture projection guidance from `HostRecalcOutput` into the replay appliance for the first host packet.

## Gate Model
### Entry gate
- `W045` has already converged the host/runtime draft for first-slice implementation planning.
- The remaining immediate first-host blockers are now small enough to batch deliberately.

### Exit gate
- `W037` and `W039` are no longer open immediate language blockers for the first host packet.
- The first-host implementation packet is explicit enough for implementation use on the current local floor.
- Unsupported/unavailable provider behavior is explicit for the admitted `INFO` / `CELL` / `RTD` slice.
- Replay-capture from the first host packet is explicit enough for implementation use.

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
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated
