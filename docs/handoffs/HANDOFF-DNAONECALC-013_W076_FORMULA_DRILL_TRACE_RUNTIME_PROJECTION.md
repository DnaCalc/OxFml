*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-013: W076 Formula Drill Trace Runtime Projection

## Status
- direction: outbound
- source_workset: W076
- target_workset: DnaOneCalc-TBD
- filed_date: 2026-05-16
- acknowledgement_status: pending

## Summary
OxFml now exposes a first formula drill-down projection for DnaOneCalc intake.
The host should consume the OxFml-owned `FormulaDrillTrace` artifact instead of
reconstructing tree structure, branch choice, argument labels, or error
causality from raw `prepared_calls`.

## Public Runtime Surface
1. `RuntimeFormulaResult.formula_drill_trace`
   - Present on successful runtime formula execution.
   - Carries `FormulaDrillTrace` with source text, deterministic node ids,
     parent/child tree structure, separate `evaluation_order`, diagnostics,
     final value, and projection-loss fields.
2. `RuntimeEnvironment::formula_drill_trace_for_source(...)`
   - Projection-only path for diagnostic-producing source such as incomplete
     formulas that do not produce a `RuntimeFormulaResult`.

## Exercised Formula Families
The local W076 evidence covers:
1. `=SUM(1,2,3)` named SUM arguments and final value.
2. `=SUM(IF(TRUE,2,3),4)` nested IF under SUM argument 1, skipped false branch,
   and post-order evaluation sequence.
3. `=IF(FALSE,SUM(1,2),SUM(3,4))` skipped true branch and evaluated false SUM.
4. `=LET(x,1,y,2,SUM(x,y))` LET binding nodes, body node, nested SUM, and
   visible resolved values for `x` and `y` through the SUM argument nodes.
5. `=1/0` divide operator node, operand nodes, final `#DIV/0!`, and causal
   error link to the divide node.
6. `=SEQUENCE(2,2)` typed array shape and preview.
7. `=SUM(` partial call plus source-linked diagnostic placeholder.
8. `=SUM(SUM(1,2),SUM(3,4))` post-order prepared-call correlation for
   same-named nested/sibling calls.

## Evidence
- `crates/oxfml_core/tests/w076_formula_drill_trace_tests.rs`
- `cargo test -p oxfml_core --test w076_formula_drill_trace_tests -- --nocapture`
  passes with 8 tests.
- `cargo test -p oxfml_core` passes.

## DnaOneCalc Intake Notes
1. Render `FormulaDrillTrace.nodes` as the structural source of truth.
2. Use `evaluation_order` only as a timeline/debug ordering, not as the tree.
3. Prefer `argument_name`, `argument_role`, and `argument_name_source` over
   host-local argument-label inference.
4. Use `branch_disposition` and `evaluation_state` to display skipped branches;
   do not infer laziness from missing prepared-call rows.
5. Use `source_span` and `diagnostics[*].node_id` for editor focus.
6. Use `FormulaDrillValuePreview` for arrays/rich values; do not parse display
   strings to recover array shape.

## Open Lane
DNA OneCalc uptake and acknowledgement remain open. This handoff does not close
the downstream UI mapping or host-verdict lane.
