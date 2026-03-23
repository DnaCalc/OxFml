# OxFml R1C1 Formula Channel

## Purpose
This document defines the first honest OxFml-local `R1C1` formula-channel floor.

It exists so `R1C1` is treated as a real formula-entry channel with caller-anchor-sensitive
translation, not only as an A1 presentation concern.

## Current Local Floor
For the current local floor, OxFml supports `WorksheetR1C1` formulas that use:
1. absolute `R1C1` cell references such as `R2C3`,
2. relative `R[dr]C[dc]` cell references translated against the caller anchor,
3. mixed absolute/relative `R1C[2]` or `R[-1]C3` cell references,
4. same-sheet or qualified area ranges built from those cell references.

Current translation rule:
1. row-relative and column-relative parts resolve against the current caller row/column,
2. absolute parts remain absolute,
3. translated meaning is preserved in the existing normalized reference forms,
4. caller-anchor use is explicit on the normalized reference.

## Canonical Artifact Treatment
The formula source must identify the channel explicitly as `WorksheetR1C1`.

The parser remains token-level agnostic about A1 versus `R1C1` for the first local floor.
The channel-sensitive translation happens during bind.

For the current local floor:
1. `R1C1` references normalize into the same `CellRef` / `AreaRef` forms used elsewhere,
2. `address_mode` records absolute versus relative origin,
3. `caller_anchor_used` records whether translation depended on the caller anchor.

## Explicit Residuals
The following remain outside the current local floor and must not be overclaimed:
1. dedicated `R1C1` whole-row and whole-column carrier parity,
2. broader `R1C1`-specific grammar families beyond the current cell and area slice,
3. non-worksheet carriers that independently use `R1C1`,
4. broader relative-reference seam freezing with OxCalc beyond the current caller-anchor facts.

## Current Deterministic Evidence
The current local evidence lives in:
1. `crates/oxfml_core/tests/w047_host_readiness_tests.rs`
2. `crates/oxfml_core/src/source.rs`
3. `crates/oxfml_core/src/binding/mod.rs`
4. `crates/oxfml_core/src/syntax/lexer.rs`
