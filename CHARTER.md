# CHARTER.md — OxFml Charter

## 1. Mission
OxFml defines and validates the formula-language and evaluator seam for DNA Calc.

It is the permanent specification owner for the FEC/F3E contract and provides stable evaluator-side interfaces consumed by OxCalc coordinators.

## 2. Precedence
When guidance conflicts, precedence is:
1. `../Foundation/CHARTER.md`
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
3. `../Foundation/OPERATIONS.md`
4. this `CHARTER.md`
5. this repo `OPERATIONS.md`

## 3. Scope
In scope:
1. Formula grammar, parse, bind, and normalized reference representation.
2. Evaluator execution semantics for single-node calculation sessions.
3. FEC/F3E protocol ownership for evaluator-side clauses.
4. Commit output contract for `value_delta`, `shape_delta`, `topology_delta`, and optional display/format deltas.
5. Trace schema and reject-detail taxonomy for deterministic replay.
6. Formula-semantic formatting behavior crossing the seam.

Out of scope:
1. Multi-node scheduling policy and global recalc policy ownership (OxCalc).
2. Function kernel semantics (OxFunc).
3. UI/rendering-only display behavior.

## 4. FEC/F3E Ownership Rule
1. OxFml is the canonical owner of the shared FEC/F3E protocol specification files.
2. OxCalc co-defines coordinator-facing clauses through explicit handoff packets.
3. Foundation keeps a read-only mirror for cross-program conformance governance.

## 5. Clean-room Rule
Allowed sources:
1. public specifications and documentation,
2. published research,
3. reproducible black-box observations.

Disallowed sources:
1. proprietary or restricted sources,
2. reverse engineering of internals,
3. decompilation/disassembly of Excel internals.

## 6. Definition of Done (Lane)
A spec/policy change is done only when:
1. seam contract text is updated,
2. conformance matrix rows are updated,
3. replay/trace impact is documented,
4. cross-repo handoff impact is recorded when coordinator-facing clauses change.
