# BUG-FML-010: Scientific Numeric Boundary `1E+308` Semantics Are Unpinned

## Summary
- **Bug id**: `BUG-FML-010`
- **Opened**: 2026-04-08
- **Status**: triaged
- **Owner workset**: `W061`

## Source Refs
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reproduced on ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`

## Ownership And Root Cause
- **Ownership class**: unknown
- **Root cause class**: vague_spec
- **Root cause summary**: the current corpus row mixes several unpinned entry and evaluation lanes around the `1E+308` boundary. We have not yet pinned which behavior belongs to:
  - worksheet formula entry,
  - worksheet calculation overflow,
  - `.xlsx` load,
  - COM formula/value assignment,
  - OxXlPlay capture,
  - OxFml parser/eval.

## Reproduction
1. Compare these observations:
   - interactive Excel appears to reject `1E+308`,
   - `9.99999999999999E+307` is accepted and may display as `1E+308`,
   - OxFml currently parses/evaluates `=1E+308*2` and returns `inf`,
   - OxXlPlay reportedly captures `0` for the same corpus lane.
2. Observe that the case does not yet have one trusted worksheet-semantic expectation.

## Spec Relationship
- **Spec references**:
  1. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
  2. `../Foundation/ARCHITECTURE_AND_REQUIREMENTS.md`
- **Spec state at intake**: vague
- **Notes**: this is currently a boundary-definition and lane-pin problem, not just an implementation mismatch.

## Investigation Log
1. 2026-04-08: corpus row `FTC-0050` surfaced as `=1E+308*2`.
2. 2026-04-08: local review identified conflicting observations across OxFml, OxXlPlay, and interactive Excel entry.
3. 2026-04-08: the topic was split out from the main corpus batch so it can be investigated without over-claiming OxFml fault.

## Fix Plan
1. Pin the exact lane behaviors separately:
   - interactive worksheet entry
   - `.xlsx` open/load
   - COM `Formula`
   - COM `Value` / `Value2`
   - worksheet overflow from valid entered literals
2. Reclassify the current corpus row as:
   - valid and pinned, or
   - invalid/ambiguous and replaced.
3. Only after that, decide whether OxFml needs:
   - numeric-domain clamping,
   - `#NUM!` overflow projection,
   - parser/admission narrowing for workbook-facing lanes.

## Similar-Risk Scan
### Adjacent families to check
1. largest accepted entered scientific literals near `9.99999999999999E+307`
2. overflow-from-valid-input rows such as multiplying a valid boundary literal
3. negative and subnormal scientific boundary cases
4. capture-lane handling in OxXlPlay and downstream verification

### Check method
1. pin the behaviors lane by lane instead of assuming one worksheet rule covers every entry path
2. split corpus rows into admitted-literal and overflow-from-valid-input families

### Results
1. no final result yet
2. current ownership remains intentionally unassigned

## Linked Reports
1. `BUGREP-FML-007`
2. `BUGREP-FML-005`

## Evidence
1. `FTC-0050`
2. local OxFml evaluation observation for `=1E+308*2`
3. interactive Excel entry observation recorded during investigation

## Closure Checklist
- [ ] fix landed
- [ ] validation passed
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix updated if required
- [ ] handoff filed if required
- [x] linked reports updated
