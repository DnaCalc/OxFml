# HANDOFF-OXFUNC-003: Corpus IF Empty-Text Condition And Floating Comparison Follow-On

## Purpose
Record the remaining live semantic divergences from the first Foundation corpus verification batch after current-head OxFml triage.

## Core Message
The incoming corpus batch initially attributed several issues to OxFml, but current-head triage reduces the remaining live semantic divergences to OxFunc-owned behavior in two places:
1. `FUNC.IF` condition coercion for empty text
2. ordinary numeric comparison equality semantics for floating results

OxFml has already confirmed that the earlier comparison-operator parse failures and non-literal `&` failures are stale on current head, and `=(-1)^0.5` already produces `#NUM!` locally. The remaining current-head divergences are semantic truth lanes in OxFunc.

## Exact Cases
1. `=IF("",1,2)`
   - Excel observed result: `2`
   - current OxFunc read: `if_fn.rs` condition coercion falls through generic numeric coercion for text values
2. `=0.1+0.2=0.3`
   - Excel observed result: `TRUE`
   - current OxFunc read: `operator_compare_concat_family.rs` uses exact `partial_cmp` equality for numbers

## Current Evidence
1. `../OxFunc/crates/oxfunc_core/src/functions/if_fn.rs`
2. `../OxFunc/crates/oxfunc_core/src/functions/operator_compare_concat_family.rs`
3. `docs/bugs/streams/BUG-FML-006_if_empty_text_condition_diverges_from_excel.md`
4. `docs/bugs/streams/BUG-FML-007_floating_comparison_equality_diverges_from_excel.md`

## Requested OxFunc Follow-On
1. Review and fix `IF` condition coercion so empty text matches Excel’s observed false branch behavior.
2. Review and fix ordinary floating comparison equality semantics for `=` so the admitted Excel-observed tolerance behavior is reproduced.
3. Scan adjacent families:
   - `<>` and ordered comparisons after any equality-tolerance change
   - other boolish condition-coercion helpers
   - criteria-family numeric equality helpers if they intentionally share comparison semantics

## OxFml Boundary Statement
OxFml will not patch these locally. They are semantic truth lanes that belong in OxFunc.

## Expected Reply
1. ownership confirmation,
2. exact bug/workset ids in OxFunc,
3. whether criteria-family comparisons will be treated as part of the same semantic family,
4. any required OxFml follow-on after OxFunc lands the changes.
