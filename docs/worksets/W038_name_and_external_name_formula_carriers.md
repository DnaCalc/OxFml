# W038: Name and External-Name Formula Carriers

## Purpose
Make defined-name formulas and external-name formulas first-class OxFml carrier lanes, with explicit grammar/bind/runtime consequences, instead of leaving them implied inside generic scoped-name or external-reference handling.

## Position and Dependencies
- **Depends on**: `W031`, `W032`, `W036`
- **Blocks**: later stronger external-host and formula-carrier policy work
- **Cross-repo**: OxFml owns formula-carrier meaning, bind/runtime consequences, and replay artifacts; OxFunc contributes catalog/provider truth where needed; OxCalc consumes resulting seam-significant effects

## Scope
### In scope
1. Define canonical OxFml treatment of name formulas and external-name formulas as formula-bearing carriers.
2. Narrow grammar and bind treatment for those carriers.
3. Tighten stage-aware runtime/provider behavior for external-name outcomes.
4. Add deterministic replay/proving artifacts for the exercised local carrier baseline.

### Out of scope
1. Full workbook-management semantics for every name-like object.
2. Broad distributed coordinator policy.
3. R1C1 and CF/DV sublanguage work.

## Deliverables
1. A canonical OxFml carrier model for name formulas and external-name formulas.
2. A narrower split between name-scope, external-provider, and error/prohibition outcomes.
3. Deterministic replay evidence for the first exercised carrier families.

## Gate Model
### Entry gate
- `W031` has classified name and external-name formulas as partial/missing carrier lanes.
- `W032` has narrowed library-context and provider taxonomy enough for honest carrier modeling.
- `W036` has widened table/reference semantics enough that name-carrier work does not collapse into generic reference handling.

### Exit gate
- Name formulas and external-name formulas are canonically modeled as formula-bearing carriers.
- External-name provider/failure behavior is narrower than the current generic external-reference posture.
- Remaining carrier gaps are explicitly listed.

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
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes: broader workbook/object-management semantics and wider provider-host policy remain outside this workset scope
- claim_confidence: draft
