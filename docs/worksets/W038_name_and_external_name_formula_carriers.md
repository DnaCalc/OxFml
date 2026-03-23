# W038: Name and External-Name Host Resolution Boundary

## Purpose
Clarify the OxFml boundary for host-managed name and external-name formulas, so OxFml does not over-own workbook carrier management while still specifying the exact bind, resolution, runtime, and FEC consequences it must preserve.

## Position and Dependencies
- **Depends on**: `W031`, `W032`, `W045`
- **Blocks**: later stronger host-managed name/external-name integration work
- **Cross-repo**: host owns name and external-name carrier management; OxFml owns bind/runtime/FEC consequences once those carriers are presented for evaluation; OxFunc contributes catalog/provider truth where needed; OxCalc consumes resulting seam-significant effects

## Scope
### In scope
1. Define the exact OxFml boundary for host-managed name and external-name formulas.
2. Specify what a host must present to OxFml when evaluation of a name-managed formula is requested.
3. Narrow bind, unresolved, and runtime/provider behavior for external-name outcomes once presented to OxFml.
4. Add deterministic replay/proving artifacts for the exercised local boundary and FEC consequences.

### Out of scope
1. Full workbook-management semantics for every name-like object.
2. Broad distributed coordinator policy.
3. R1C1 and CF/DV sublanguage work.

## Deliverables
1. A canonical OxFml boundary statement for host-managed name and external-name formulas.
2. A narrower split between host-owned carrier management, OxFml bind/runtime consequences, and external-provider/error outcomes.
3. Deterministic replay evidence for the first exercised boundary families.

## Gate Model
### Entry gate
- `W031` has classified name and external-name formulas as partial/missing lanes.
- `W032` has narrowed library-context and provider taxonomy enough for honest boundary modeling.
- `W045` has made the first direct-host versus integrated-host split explicit.

### Exit gate
- The host-managed carrier boundary is explicit.
- External-name provider/failure behavior is narrower than the current generic external-reference posture.
- Remaining boundary gaps are explicitly listed.

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
  - semantic-plan classification now distinguishes mixed/deferred name handling and adopted defined-name callable origin, but the host-managed presentation contract for name/external-name evaluation is still not specified exactly
  - external-name formulas now have a first checked Lean boundary artifact for same-external-book restriction and provider-stage runtime typing, but the exact host-presented FEC contract is still missing
  - broader workbook/object-management semantics and wider provider-host policy remain outside this workset scope
- claim_confidence: draft
