# BUGREP-FML-007: Scientific Numeric Boundary `1E+308` Lane Is Unpinned

## Intake
- **Report id**: `BUGREP-FML-007`
- **Filed**: 2026-04-08
- **Source channel**: local investigation
- **Reporter/source**: `Foundation corpus / FTC-0050` follow-on review
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reported against kind**: commit
- **Canonical bug id**: `BUG-FML-010`
- **Status**: triaged

## Observed Symptom
The `FTC-0050` row `=1E+308*2` is not a cleanly pinned worksheet-boundary case.
Current observations disagree across lanes:
1. OxFml returns `inf`,
2. OxXlPlay reportedly captures `0`,
3. interactive Excel entry appears to reject `1E+308`,
4. `9.99999999999999E+307` is accepted and displayed as `1E+308`.

## Reproduction
1. Evaluate `=1E+308*2` through current OxFml head.
2. Compare with the current downstream capture lane and interactive Excel entry.
3. Observe that the worksheet/formula/file/COM behaviors are not yet pinned as one agreed semantic case.

## Initial Ownership Read
- **Initial classification**: unknown
- **Reason**: this may involve a mix of workbook-lane admission, worksheet numeric-domain semantics, corpus-case validity, and downstream capture behavior rather than a single confirmed OxFml-local defect.

## Links
1. `docs/bugs/streams/BUG-FML-010_scientific_numeric_boundary_1e308_semantics_are_unpinned.md`
2. `docs/bugs/reports/BUGREP-FML-005_foundation_corpus_verification_batch_ftc_0001_0100.md`
3. `docs/worksets/W061_foundation_corpus_verification_intake_round_001.md`

## Triage Notes
1. This topic needs the lane split pinned explicitly:
   - interactive worksheet entry
   - `.xlsx` open/load
   - COM `Formula`
   - COM `Value` / `Value2`
   - OxFml parser/eval
2. The likely clean corpus outcome is to replace `FTC-0050` with a boundary-valid entered-literal case plus a separate overflow-from-valid-input case.
