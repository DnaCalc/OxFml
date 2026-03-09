# OxFml

OxFml is the formula-language and single-node evaluator lane for DNA Calc.

## Core Responsibilities
1. Formula grammar, parse, and bind semantics.
2. Single-node formula evaluation contracts.
3. FEC/F3E seam specification ownership (session lifecycle, commit deltas, trace schema).
4. Formula-semantic formatting behavior that affects evaluation outputs (including `TEXT` and format/CF semantic lanes).

## Startup Docs
- `CHARTER.md`
- `OPERATIONS.md`
- `docs/spec/README.md`

## Dependency Constitution
- May depend on: `OxFunc`.
- Must not depend on: `OxCalc`.

## Foundation Alignment
Precedence and constitutional constraints are inherited from:
1. `../Foundation/CHARTER.md`
2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
3. `../Foundation/OPERATIONS.md`
