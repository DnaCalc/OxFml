# W077: Strict Excel Grid BindProfile And R1C1 Identity

## Purpose

Process `HANDOFF-DNATREECALC-001` into an OxFml-owned formula-language plan for the `strict-excel-grid` profile: typed `BindProfile`, symbolic relative references, A1 `$` fidelity, grid bounds, caller-independent bind identity, compiled-plan caching, and translation/rebind APIs.

## Depends on

- W037 R1C1 formula-channel floor.
- W074 name/call and host-context identity lessons.
- W075 compiled-plan optimization floor.
- OxCalc W061 grid model and reference-machine planning.

## Scope

1. Define the public `BindProfile` shape and default-preserving migration rule.
2. Define `GridBounds` and the out-of-bounds to `#REF!` contract.
3. Define symbolic reference ADTs for relative R1C1 and caller-relative A1 references.
4. Remove caller anchor from bind identity only when the profile proves caller-independent symbolic refs are active.
5. Define compiled-plan cache keys and per-cell caller-coordinate instantiation.
6. Define translation/rebind APIs for fill, paste, region stamping, and insert/delete shifting.
7. Fix non-default host-reference syntax reuse penalties before `strict-excel-grid` makes non-default syntax common.

## Non-goals

- OxCalc grid storage or dependency graph implementation.
- OxFunc function semantics.
- Spill placement/blockage arbitration; OxFml only preserves syntax, capability facts, and pass-through shape facts.
- Any default-behavior drift for existing TreeCalc or ordinary worksheet channels.

## Closure condition

W077 closes only when default-profile behavior is regression-proven unchanged, `strict-excel-grid` bind fixtures prove caller-independent identity and `$` fidelity, grid-bound translations produce deterministic `#REF!`, plan-cache identity is explicit, and OxCalc has acknowledged the public packet shape for W061.

## Initial lanes

1. Spec update and type-shape freeze.
2. Default behavior guardrails and fixture inventory.
3. Symbolic R1C1 and caller-relative A1 bind records.
4. Bind fingerprint/cache-key migration.
5. Translation/rebind API design and focused fixtures.
6. OxCalc acknowledgement packet.