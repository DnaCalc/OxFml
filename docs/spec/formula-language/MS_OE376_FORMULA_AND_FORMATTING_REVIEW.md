# MS-OE376 Formula And Formatting Review

## 1. Purpose
This document records the OxFml-owned review outcome for the remaining formula and formula-adjacent rule families surfaced by `MS-OE376`.

It does not mirror Microsoft prose mechanically.
It translates the reviewed source material into:
1. OxFml ownership boundaries,
2. current-coverage classification,
3. evidence posture,
4. concrete follow-on work packets.

## 2. Authority And Reading Rule
1. `MS-OE376` is treated here as an upstream source of Excel behavior and carrier-shape signals.
2. OxFml canonical docs remain authoritative for OxFml meaning, artifact boundaries, reject semantics, and replay consequences.
3. If `MS-OE376` families imply carrier or host distinctions that OxFml does not yet model, this document records them as missing or partial rather than silently importing them into live semantics.

## 3. Reviewed Source Baseline
This pass was driven from Foundation-owned reference processing rooted at:
1. `../Foundation/reference/runs/20260318-ms-oe376-full-detail-pass-01/outputs`
2. `../Foundation/reference/downloads/learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/db9b9b72-b10b-4e7e-844c-09f88c972219.md`
3. `../Foundation/reference/runs/20260305-ms-formatting-formal-pass-01/outputs/FORMATTING_FORMAL_FINDINGS.md`

High-signal source anchors used in this review:
1. structured references:
   - `SPEC-discovered-ms-oe376-88e93023-48236`
   - `CONF-discovered-ms-oe376-220816-823374c7-1423`
2. conditional-formatting formula restrictions:
   - `CONF-discovered-ms-oe376-220816-823374c7-1427`
   - `CONF-discovered-ms-oe376-220816-823374c7-1428`
   - `CONF-discovered-ms-oe376-220816-823374c7-1429`
   - `CONF-discovered-ms-oe376-220816-823374c7-1430`
3. data-validation formula restrictions:
   - `CONF-discovered-ms-oe376-220816-823374c7-1431`
4. defined-name uniqueness and carrier presence:
   - `CONF-discovered-ms-oe376-220816-823374c7-0362`
   - `CONF-discovered-ms-oe376-220816-823374c7-0363`
   - `SPEC-discovered-ms-oe376-88e93023-48424`
5. external name formulas:
   - `SPEC-discovered-ms-oe376-88e93023-48443`
   - `SPEC-discovered-ms-oe376-88e93023-48448`
   - `SPEC-discovered-ms-oe376-88e93023-48451`
6. R1C1 formulas:
   - `CONF-discovered-ms-oe376-220816-823374c7-1434`
   - `SPEC-discovered-ms-oe376-88e93023-48474`
   - `SPEC-discovered-ms-oe376-88e93023-48487`
7. historical DV/name carrier presence:
   - `SPEC-discovered-ms-oe376-171212-fc69605e-23574`
   - `SPEC-discovered-ms-oe376-88e93023-51882`

## 4. Family Classification

| family | source signal | current OxFml coverage | current local evidence posture | review classification | follow-on owner |
|---|---|---|---|---|---|
| Structured references | `MS-OE376` treats intra-table references as explicit formula syntax and carrier surface, including omitted-table-name forms and current-row-sensitive semantics. | `FML-R-009` exists in [EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md](./EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md); normalized-reference docs already mention row-context-sensitive structured refs in [OXFML_NORMALIZED_REFERENCE_ADTS.md](./OXFML_NORMALIZED_REFERENCE_ADTS.md). | local parser acceptance and some structured-reference evidence exist, but qualifier breadth, omitted-table-name binding, binder shapes, and table-context runtime meaning are still narrow. | `partial` | `W036` |
| Conditional-formatting formulas | `MS-OE376` constrains CF formulas by forbidding array constants, structured references, union/intersection, and 3-D references, while also exposing formula-bearing host fields such as `cfRule/formula` and threshold lanes such as `cfvo@val`. | formatting semantics acknowledge conditional-format overlays and observability boundaries in [EXCEL_FORMATTING_HIERARCHY_AND_VISIBILITY_MODEL.md](../formatting/EXCEL_FORMATTING_HIERARCHY_AND_VISIBILITY_MODEL.md). | there is source-backed constraint evidence, but no OxFml-local canonical CF formula sublanguage contract, host-profile model, or dedicated replay family for CF formula admission/execution. | `partial_to_missing` | `W039` |
| Name formulas | `MS-OE376` treats defined names as formula-bearing carriers with workbook/sheet scoping rules. | scoped name-resolution rule exists in `FML-R-008`; proving-host docs already allow defined-name bindings. | name collision and scope evidence exists, but non-cell formula-bearing carrier semantics are not first-class in canonical OxFml docs. | `partial` | `W038` |
| External name formulas | `MS-OE376` exposes external-name expression families and error-bearing external-name outcomes, with a narrower same-external-book restriction than generic external references. | OxFml has external references, availability taxonomy, and provider/runtime lanes, but not an explicit external-name formula carrier model. | some external-reference parsing and provider/runtime evidence exists, but external-name grammar, same-book restriction, bind shape, and carrier semantics are still under-specified. | `missing_to_partial` | `W038` |
| R1C1 formulas | `MS-OE376` treats R1C1 as a distinct formula language channel and requires R1C1-style references in that carrier. | no canonical OxFml doc currently models R1C1 as a first-class formula channel; current parser/binder baseline is A1-first. | no dedicated local deterministic replay family exists for R1C1 parse/bind/eval. | `missing` | `W037` |
| Data-validation formulas | `MS-OE376` exposes `dataValidation/formula1` and `formula2` as separate formula-bearing lanes, with restriction pressure similar to CF formulas but not obviously identical. | no dedicated OxFml-local DV formula sublanguage contract exists today. | no dedicated local replay family exists for DV formula admission, host binding, or evaluation context. | `missing` | `W039` |
| Formula-significant table or formatting surfaces | `MS-OE376` plus current formatting findings indicate formula-adjacent table and conditional-format rules that matter semantically even if they are not general UI behavior. | OxFml already distinguishes semantic format vs display and table/CF overlays in [EXCEL_FORMATTING_HIERARCHY_AND_VISIBILITY_MODEL.md](../formatting/EXCEL_FORMATTING_HIERARCHY_AND_VISIBILITY_MODEL.md). | local evidence covers only seam-significant format/display subsets, not the full formula-bearing carrier consequences of tables, CF, or DV. | `partial` | `W036` and `W039` |

## 5. Cross-Cutting Ownership Outcome

### 5.1 Grammar And Parsing
The review implies four distinct parser lanes, not one generic backlog:
1. structured-reference grammar and qualifier breadth,
2. R1C1 formula and reference grammar,
3. name and external-name formula-bearing carriers,
4. restricted sublanguages for conditional-formatting and data-validation formulas.

### 5.2 Bind And Reference Normalization
The reviewed material sharpens four bind lanes:
1. table-context-sensitive structured-reference resolution,
2. R1C1 relative/absolute translation under caller position,
3. explicit defined-name and external-name carrier identity,
4. typed refusal or deferment for formula carriers not yet executable in a given host/runtime profile.

Critical bind consequence:
1. omitted structured-reference table names cannot be resolved from syntax alone and require enclosing-table context,
2. structured-reference table identifiers must remain distinct from user-defined names,
3. external-name formulas need explicit external-book identity rather than generic external-reference flattening.

### 5.3 Evaluator And Runtime
The review does not authorize silent reuse of worksheet-formula semantics everywhere.
OxFml must keep open the possibility that:
1. conditional-formatting formulas are a restricted worksheet-like sublanguage,
2. data-validation formulas are a distinct host/evaluation lane,
3. external-name formulas depend on runtime/provider outcomes that differ from plain worksheet references,
4. name formulas are formula-bearing carriers whose storage and update semantics are not identical to grid-cell formulas.

Critical runtime consequence:
1. R1C1 is not just a render/view mode in this source set; it is a separate formula-entry lane whose admissibility depends on R1C1 references.

### 5.4 Semantic Formatting And Host Policy
Two consequences are important:
1. formula-significant table and conditional-formatting surfaces belong in OxFml semantics only where they alter formula admission, bind meaning, evaluator context, or seam-significant effects,
2. UI-only formatting remains out of scope for OxFml even when `MS-OE376` documents it near formula language.
3. rule-host surfaces such as `sqref`, rule type, operator, time-period, threshold fields, and priority are part of formula-host semantics for CF/DV lanes rather than generic styling noise.

## 6. Explicit Non-Authorizations
This review does not authorize:
1. blind promotion of `MS-OE376` wording into OxFml canonical rule text,
2. treating structured references, R1C1, CF formulas, and DV formulas as one interchangeable parser feature,
3. assuming name formulas and grid-cell formulas share identical host, replay, or publication semantics,
4. collapsing external-name/provider failure into a generic `#NAME?` lane without stage-aware typing,
5. treating UI formatting rules as OxFml formula semantics unless formula admission, bind, evaluation, or seam-significant effects depend on them.

## 7. High-Risk Distinctions From The Review
1. structured references are context-sensitive and are not universally admissible across all formula-bearing carriers,
2. `#This Row` is a true row-context lane, not mere surface sugar,
3. conditional-formatting and data-validation formulas should be treated as restricted sublanguages until local evidence proves wider reuse safely,
4. external-name formulas are narrower than generic external references,
5. R1C1 should be modeled as a formula channel, not only as a presentation mode.
6. conditional-formatting and data-validation restrictions are similar but not safely identical; current source support does not justify silently collapsing them into one ban list,
7. `:` remains admissible pressure for CF/DV even where union and intersection are restricted,
8. whitespace preservation on formula-bearing CF/DV fields is part of carrier fidelity rather than purely cosmetic serialization.

## 8. Follow-On Workset Shaping
This review yields the following concrete follow-on backlog:
1. `W036` structured references and table formula semantics realization,
2. `W037` R1C1 formula channels and reference translation,
3. `W038` name and external-name formula carriers,
4. `W039` conditional-formatting and data-validation formula sublanguages.

Working sequence:
1. `W036` and `W037` can proceed after the current formula-language baseline without waiting for all runtime/distributed work.
2. `W038` should follow the current OxFml/OxFunc catalog/provider narrowing so external-name carrier semantics do not race ahead of library-context truth.
3. `W039` should build on the semantic-format/display boundary already narrowed in `W030` and the future runtime consequence work from `W034` where CF/DV outcomes become seam-significant.

## 9. W031 Outcome
Current `W031` result:
1. the relevant `MS-OE376` families are now classified against current OxFml coverage,
2. the review has been converted into explicit OxFml-owned backlog shape,
3. several reviewed families remain intentionally unpromoted into live rule text because OxFml does not yet have the local replay/evaluator floor to claim stronger semantics honestly.

That means this review is useful now, but it is not itself a claim that the reviewed families are locally realized.
