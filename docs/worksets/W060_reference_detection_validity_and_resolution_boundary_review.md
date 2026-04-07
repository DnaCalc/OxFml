# W060: Reference Detection, Validity, And Resolution Boundary Review

## Purpose
Review and correct the cross-repo design and implementation split for reference detection, reference validity, workbook-grid extents, and evaluation-time reference resolution so workbook truth is supplied through explicit context/interfaces rather than inferred or hardcoded inside OxFml or OxFunc.

This workset exists because the current implementation has drifted away from the intended boundary in several places:
1. OxFunc currently hardcodes worksheet max rows/max cols for A1 parsing,
2. OxFml still locally decides some A1 validity questions for absent-cell handling,
3. OxFml still embeds a local workbook-like resolver/cache model for evaluation,
4. the current `ReferenceResolver` seam is too weak to express workbook-profile truth such as grid extents and in-bounds validity,
5. whole-row/whole-column value-only dereference still lacks an explicit host/model-backed path.

## Position and Dependencies
- **Depends on**: `W059`
- **Blocks**: `none`
- **Cross-repo**:
  - OxFunc matching review and implementation workset required
  - OxCalc matching host/context and coordinator-facing packet review required
  - handoff packets will be required once the OxFml-side review is translated into concrete seam changes

## Scope
### In scope
1. review the current OxFml/OxFunc/OxCalc split for:
   - reference parsing versus reference validity,
   - workbook-grid extents,
   - absent-cell versus invalid-reference classification,
   - reference-preserved versus value-required dereference lanes,
   - whole-row/whole-column and other bounded-grid reference families,
2. document the intended ownership split for:
   - reference grammar and normalized reference structure,
   - workbook-profile truth and grid extents,
   - host-backed reference validity,
   - single-reference dereference,
   - multi-reference or multi-area semantic combination,
3. define the minimum explicit runtime/FEX context needed for honest reference resolution,
4. identify and remove current drift where OxFml or OxFunc locally hardcodes workbook truth instead of consuming it from explicit context,
5. define the required OxCalc-facing host/context packet or provider surface for workbook reference truth,
6. produce bounded implementation slices in OxFml needed to consume the corrected seam,
7. file and track matching OxFunc and OxCalc handoff packets with exact requested changes and evidence.

### Out of scope
1. broad non-reference formula-language review outside the reference/extent/resolution topic,
2. UI/editor-only rendering concerns,
3. speculative support for workbook families whose reference model is not yet admitted by bounded spec.

## Deliverables
1. OxFml workset-owned design review packet recording:
   - current drift,
   - intended ownership,
   - required context/seam additions,
   - explicit non-goals.
2. Updated OxFml spec/doctrine text that states:
   - workbook-grid truth does not live in OxFunc constants,
   - OxFml does not locally invent reference-validity truth,
   - reference resolution requires explicit host/model context.
3. Exact outbound handoff packets to OxFunc and OxCalc for the matching implementation work.
4. OxFml implementation updates for any local drift that can be corrected once the seam/context is clarified.
5. Deterministic evidence for the exercised boundary cases, including at minimum:
   - in-bounds direct cell reference,
   - out-of-range direct cell reference,
   - whole-row/whole-column preserved-reference lanes,
   - whole-row/whole-column value-required lanes under the admitted context path,
   - invalid versus absent-cell distinction.

## Review Questions
1. Which repo owns workbook-grid extents and reference-validity truth?
2. What exact data belongs in FEX or adjacent runtime context so OxFml/OxFunc can evaluate references honestly?
3. What should the `ReferenceResolver` contract become, if anything, to carry:
   - grid limits,
   - in-bounds checks,
   - whole-row/whole-column dereference,
   - host-backed invalid-reference classification?
4. Which current OxFml local helpers are legitimate test/bootstrap scaffolds, and which are ownership drift?
5. Which current OxFunc A1 parsing helpers should remain lexical utilities, and which should stop embedding workbook-profile constants?
6. What exactly must OxCalc supply as host/coordinator context for this seam to be honest end to end?

## Candidate Correction Areas
1. OxFunc:
   - remove fixed `EXCEL_MAX_ROWS` / `EXCEL_MAX_COLS` as semantic truth carriers,
   - stop treating workbook-grid profile as library-local constant truth,
   - consume grid/extent truth from passed-in evaluation context or adjacent reference context.
2. OxFml:
   - stop using local A1-shape parsing as the final word on absent vs invalid references,
   - narrow local `LocalReferenceResolver` responsibilities to explicit test/bootstrap lanes only, or replace it with a clearer host/context abstraction,
   - stop treating local sparse cell maps as sufficient proxy for workbook reference truth where they are not.
3. OxCalc:
   - define the host/runtime packet or provider surface that supplies workbook-grid profile and reference validity/dereference truth,
   - make the host contract explicit for whole-row/whole-column and out-of-range reference handling.

## Gate Model
### Entry gate
- `W059` operator-boundary review and multi-area Style A follow-through landed
- current drift areas identified in OxFml and OxFunc code

### Exit gate
- OxFml design review packet written and linked from this workset
- matching OxFunc and OxCalc handoff packets filed
- OxFml spec updated to freeze the intended reference-validity/extents ownership split
- at least one bounded implementation slice landed in OxFml for the corrected seam or explicit context consumption path
- deterministic tests exist for the first exercised corrected boundary cases

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | no |
| 2 | Conformance matrix rows updated? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Cross-repo impact assessed and handoff filed if needed? | no |
| 5 | All required tests pass? | no |
| 6 | No known semantic gaps remain in declared scope? | no |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | no |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - write the OxFml review packet for current drift and intended seam
  - file matching OxFunc and OxCalc handoff packets
  - define the minimum explicit reference/workbook context contract
  - decide the first bounded implementation slice after review freeze
- claim_confidence: draft
