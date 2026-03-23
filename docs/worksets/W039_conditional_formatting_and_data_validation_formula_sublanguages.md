# W039: Conditional-Formatting and Data-Validation Formula Sublanguages

## Purpose
Define the OxFml treatment of conditional-formatting and data-validation formulas as restricted, host-sensitive formula-bearing lanes rather than assuming they are identical to worksheet formulas or leaving them undocumented.

## Position and Dependencies
- **Depends on**: `W031`, `W030`, `W034`
- **Blocks**: later stronger format/host-policy integration and empirical-pack promotion for non-cell formula carriers
- **Cross-repo**: OxFml owns formula sublanguage semantics, admission, and seam-significant effects; host-policy consumers consume the resulting clarified boundaries

## Scope
### In scope
1. Define canonical OxFml treatment of conditional-formatting formulas as a restricted worksheet-like sublanguage.
2. Define canonical OxFml treatment of data-validation formulas as a host-sensitive formula-bearing lane.
3. Model formula-host surfaces that are semantically relevant in those lanes, including rule target ranges and threshold/value carriers where needed.
4. Classify which restrictions belong to parse, bind, evaluation context, or seam-significant formatting/runtime effects, without collapsing CF and DV into one identical restriction set.
5. Add deterministic replay/proving artifacts for the first exercised CF and DV formula families.

### Out of scope
1. Full conditional-format rendering policy.
2. UI/editor behavior for validation prompts.
3. Broad workbook-style management unrelated to formula semantics.
4. Generic style/render work with no formula-host consequence.

## Deliverables
1. Canonical OxFml sublanguage rules for CF and DV formulas.
2. A formula-host model for semantically relevant CF/DV rule fields such as target ranges and threshold/value carriers.
3. Narrower formula-semantic versus display/UI boundary for those lanes.
4. Deterministic replay/proving artifacts for the exercised local floor.

## Gate Model
### Entry gate
- `W031` has classified CF and DV formulas as partial/missing sublanguage lanes.
- `W030` has narrowed the semantic-format versus display boundary enough to separate formula-significant rules from UI-only behavior.
- `W034` has widened runtime/coordinator consequence boundaries enough to classify seam-significant outcomes where they exist.

### Exit gate
- CF and DV formulas are explicitly treated as distinct sublanguage/carrier lanes in OxFml docs.
- CF and DV host fields with formula-semantic meaning are modeled explicitly rather than treated as generic formatting metadata.
- The current local floor includes deterministic evidence for at least one nontrivial CF lane and one nontrivial DV lane.
- Remaining CF/DV semantic gaps are explicit rather than implicit.

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
