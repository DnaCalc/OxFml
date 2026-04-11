# W061: Foundation Corpus Verification Intake Round 001

## Purpose
Process the first `FTC-0001` through `FTC-0100` verification batch reported through DNA OneCalc, classify each divergence against current OxFml head, fix the OxFml-owned slice, and file exact sibling handoffs for remaining non-OxFml-owned semantic gaps.

## Position and Dependencies
- **Depends on**: `W058`, `W059`
- **Blocks**: untracked corpus-regression drift and stale attribution of already-fixed operator families
- **Cross-repo**:
  - `OxFunc` for the broader floating numeric comparison family split confirmed in `HO-FN-008`
  - `DnaOneCalc` only for the non-OxFml display-summary attribution note

## Scope
### In scope
1. Record the corpus batch against the exact current OxFml ref.
2. Separate stale-on-head reports from still-live current-head issues.
3. Fix the OxFml-owned scientific numeric literal parse gap.
4. Add focused regression coverage for:
   - current-head operator support that the corpus still reported as missing,
   - current-head negative fractional power `#NUM!` behavior,
   - scientific numeric literal support.
5. File exact OxFunc handoff text for the remaining live semantic cases and correct any stale intake reads once the reply arrives.

### Out of scope
1. Fixing OxFunc-owned semantic behavior directly from this repo.
2. Fixing downstream display-summary formatting inside `DnaOneCalc`.
3. Broader formula-corpus expansion beyond the first `FTC-0001` through `FTC-0100` batch.

## Deliverables
1. A recorded corpus intake report and updated bug registers.
2. One new OxFml bug stream for scientific numeric literals.
3. One outbound OxFunc handoff covering the remaining live semantic gaps, plus local correction of any reply-driven intake mistakes.
4. OxFml code/tests proving scientific numeric literals are now admitted on current head.
5. Honest current-head notes for stale duplicate reports and non-OxFml-owned attribution.

## Gate Model
### Entry gate
- Current-head review of the reported formulas completed.
- Ownership split identified.

### Exit gate
- Corpus intake is recorded at the current exact ref.
- OxFml-owned scientific literal support is exercised in deterministic tests.
- Stale duplicate reports are linked honestly rather than left as open active bugs.
- Remaining live semantic gaps are handed off to OxFunc with exact formulas and observed behavior, and any corrected-intake rows are updated honestly after reply.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | no |
| 2 | Conformance matrix rows updated? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | yes |
| 6 | No known semantic gaps remain in declared scope? | no |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | no |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - record the exact landed OxFunc ref for `HO-FN-008` once available instead of relying on sibling working-tree validation
  - decide whether scientific exponent-literal admission needs explicit formula-language matrix promotion
  - update any broader worklist/spec pointers if this corpus lane is promoted beyond the current intake/fix slice
- claim_confidence: provisional
