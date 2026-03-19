# W031: MS-OE376 Formula and Formatting Rule Review

## Purpose
Review the remaining Excel formula-language and formula-adjacent formatting-rule surfaces documented in the Microsoft Open Specifications (`MS-OE376`) and convert the relevant parts into explicit OxFml planning, spec deltas, and future implementation slices without copying Microsoft prose mechanically.

## Position and Dependencies
- **Depends on**: `W019`, `W026`, `W030`
- **Blocks**: later broader formula-language closure, structured-reference realization, and any stronger conditional-formatting or external-name formula implementation planning
- **Cross-repo**: OxFml remains the owner of formula-language, evaluator, and formula-semantic formatting meaning; OxCalc and OxFunc consume the resulting clarified seams rather than owning them

## Scope
### In scope
1. Review the relevant `MS-OE376` formula and formatting-rule sections and classify them as:
   - already canonical in OxFml,
   - partially represented,
   - still missing or under-specified.
2. Cover at minimum the following families:
   - structure references,
   - conditional formatting formulas,
   - name formulas,
   - external name formulas,
   - R1C1 formulas,
   - data validation formulas,
   - related formula-significant formatting or table-rule surfaces where they materially affect OxFml semantics.
3. Identify which parts belong to:
   - grammar and parsing,
   - bind/reference normalization,
   - evaluator/runtime behavior,
   - formula-semantic formatting,
   - proving-host and replay evidence,
   - later host-policy or coordinator-facing consequences.
4. Produce a gap map and a sequenced follow-on backlog rather than trying to absorb every surface into one implementation packet.

### Out of scope
1. Full implementation of all reviewed grammar families.
2. Blind promotion of `MS-OE376` wording into OxFml canonical text.
3. UI-only formatting behavior with no formula-semantic consequence.
4. Cross-repo handoff unless the review exposes a genuinely coordinator-facing seam change.

## Deliverables
1. An OxFml-owned review note or spec update set mapping the relevant `MS-OE376` sections into OxFml semantic ownership.
2. An explicit classification of what is already covered, partially covered, or missing in current OxFml docs and local evidence.
3. A sequenced follow-on backlog for implementation-facing work such as structured references, external-name handling, R1C1 formulas, and conditional-formatting-language realization.
4. Updated planning/index docs so these `MS-OE376` review outcomes are tracked rather than left implicit.

## Source Baseline
The minimum review set for this workset should include:
1. `MS-OE376` Structure References  
   `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/bcd72180-31a3-423b-8f83-d224b2286da3`
2. `MS-OE376` Conditional Formatting Formulas  
   `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/bfd22bea-d7b6-49cb-94cb-feb4d58a65ea`
3. `MS-OE376` Name Formulas  
   `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/36f59210-dd88-44c8-a24c-95b33d7742d5`
4. `MS-OE376` External Name Formulas  
   `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/75328f70-50a7-43af-a4da-3abade67f5f9`
5. `MS-OE376` R1C1 formulas  
   `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/9ccaf3c0-9941-460f-a4b4-a8e8bce7cf9f`
6. `MS-OE376` Data Validation Formulas  
   `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/02389aab-a27f-47fc-b58e-6bb431ef9c37`
7. `MS-OE376` formulas umbrella section where needed for context  
   `https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/0ea8ed49-6460-4385-a811-b8e8e70763c2`

## Gate Model
### Entry gate
- `W019` has established the current formula-language closure floor.
- `W026` has narrowed the library-context and availability taxonomy enough that rule-review outcomes can be assigned cleanly to grammar, bind, semantic-planning, or runtime surfaces.
- `W030` has narrowed the semantic-format versus display boundary enough that formula-adjacent formatting rule review does not collapse into general UI behavior.

### Exit gate
- The relevant `MS-OE376` rule families are explicitly classified against current OxFml coverage.
- Missing or partial areas are assigned to concrete follow-on OxFml planning lanes rather than left as generic future work.
- The review outcome makes clear which areas are parser/binder work, which are evaluator/runtime work, and which are formula-semantic formatting or host-policy work.

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
