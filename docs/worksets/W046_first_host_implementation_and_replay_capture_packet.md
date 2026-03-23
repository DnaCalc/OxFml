# W046: First Host Implementation and Replay Capture Packet

## Purpose
Specify the exact first-host implementation packet for running OxFml and OxFunc as a single-cell Excel formula evaluation tool, including host lifecycle, unavailable-provider behavior, and replay-appliance capture from the host-facing output packet.

## Position and Dependencies
- **Depends on**: `W037`, `W039`, `W041`, `W042`, `W043`, `W045`
- **Blocks**: first direct host implementation, replay-aware host harness work
- **Cross-repo**: OxFml owns the canonical direct-host packet and replay projection rules; OxCalc consumes the integrated-host reading; OxFunc continues to own function semantics and runtime library-context truth

## Scope
### In scope
1. Freeze the first direct-host implementation workflow over the currently covered local floor.
2. Specify exact host behavior when `INFO`, `CELL`, or `RTD` are unsupported, unavailable, or denied.
3. Specify the exact host-facing packet a first single-cell implementation should consume.
4. Specify how a host records replay-facing execution traces and bundle projections from `HostRecalcOutput`.
5. List exactly what remains outside the first host implementation packet.

### Out of scope
1. Full workbook graph hosting.
2. Pack-grade replay promotion.
3. Full Excel formula-language or built-in-function closure.
4. Final cross-process ABI or UI packaging.

## Deliverables
1. A canonically documented first-host implementation packet.
2. Explicit unavailable-provider behavior for the currently admitted host-query/provider slice.
3. Explicit replay-capture projection rules from the host-facing output packet.
4. An explicit residual list for unsupported language, provider, and publication families.

## Gate Model
### Entry gate
- `W037` and `W039` have removed the biggest remaining “not ordinary worksheet formula” gaps from the immediate first-host scope.
- `W041`, `W042`, `W043`, and `W045` have made the host/query, return-surface, runtime-provider, and host-contract packet explicit enough to freeze a first host recipe honestly.

### Exit gate
- The first direct-host implementation packet is explicit enough for implementation use.
- Unsupported/unavailable behavior for `INFO`, `CELL`, and `RTD` is explicit.
- Replay capture from the host-facing packet is explicit.
- Remaining out-of-scope language and provider gaps are explicitly listed.

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
