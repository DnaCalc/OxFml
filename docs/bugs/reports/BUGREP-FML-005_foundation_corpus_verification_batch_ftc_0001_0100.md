# BUGREP-FML-005: Foundation Corpus Verification Batch FTC-0001 Through FTC-0100

## Intake
- **Report id**: `BUGREP-FML-005`
- **Filed**: 2026-04-08
- **Source channel**: downstream handoff
- **Reporter/source**: `DNA OneCalc verify-formula` corpus run
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reported against kind**: commit
- **Canonical bug id**: `unassigned`
- **Status**: triaged

## Observed Symptom
The first `FTC-0001` through `FTC-0100` corpus batch reported multiple OxFml-attributed divergences, but current-head triage shows the batch mixes three different kinds of findings: stale duplicate operator reports already fixed on current head, one real current-head OxFml lexer gap for scientific numeric literals, and several non-OxFml-owned or downstream-attribution items.

## Reproduction
1. Run the Foundation corpus formulas through `dnaonecalc-host verify-formula` against current OxFml head.
2. Compare OxFml-evaluated output with Excel.
3. Observed batch included stale operator parse failures, one scientific numeric literal parse gap, OxFunc-owned semantic mismatches, and one downstream display-summary mismatch attribution.

## Initial Ownership Read
- **Initial classification**: unknown
- **Reason**: the incoming batch was mixed and required current-head triage rather than direct acceptance as one OxFml bug.

## Links
1. `docs/bugs/streams/BUG-FML-005_scientific_numeric_literals_are_not_lexed.md`
2. `docs/bugs/streams/BUG-FML-006_if_empty_text_condition_diverges_from_excel.md`
3. `docs/bugs/streams/BUG-FML-007_floating_comparison_equality_diverges_from_excel.md`
4. `docs/bugs/streams/BUG-FML-008_multi_character_comparison_tokens_advance_incorrectly.md`
5. `docs/bugs/streams/BUG-FML-003_ordinary_operator_semantics_should_dispatch_to_oxfunc.md`
6. `docs/handoffs/HANDOFF-OXFUNC-003_CORPUS_IF_EMPTY_TEXT_AND_FLOAT_COMPARE.md`

## Triage Notes
1. Single-character comparison and non-literal `&` reports were stale on the reported ref and remain covered under `BUG-FML-003`.
2. Multi-character comparison operators (`<>`, `<=`, `>=`) were still live on the reported ref because of a lexer cursor bug; they are now tracked under `BUG-FML-008`.
3. Negative fractional exponent `=(-1)^0.5` is stale on current head; local regression now proves `#NUM!`.
4. Scientific numeric literals remained a current-head OxFml lexer gap and are tracked under `BUG-FML-005`.
5. `=IF("",1,2)` was a corrected intake. `HO-FN-008` shows current Excel replay yields `#VALUE!`, and the live failure reduced to the local OxFml worksheet-error projection bug tracked under `BUG-FML-009`.
6. `=0.1+0.2=0.3` is part of a broader OxFunc semantic lane, not just ordinary compare operators. `HO-FN-008` widens that lane to operators, criteria/database, and `SWITCH`, while `MATCH`, `XMATCH`, and `DELTA` remain exact.
7. The `0.30000000000000004` display-summary complaint is not currently treated as an OxFml bug; the live DNA OneCalc bridge formats numbers locally with `number.to_string()`.
8. `FTC-0050 =1E+308*2` has been split into `BUG-FML-010` because the lane itself is not yet pinned across interactive Excel entry, `.xlsx` load, COM assignment, downstream capture, and OxFml eval.
