# W076: Formula Drill Trace Model

## Purpose
Process DNA OneCalc's formula drill-down handoff into an OxFml-owned
expression-trace lane.

DNA OneCalc currently has a useful audit harness, but the surface it can render
from OxFml is too close to the raw prepared-call trace. The requested target is
an OxFml-owned drill-ready tree that explains how a formula evaluated without
forcing the host to reconstruct formula semantics.

Inbound handoff:
1. `../DnaOneCalc/docs/HANDOFF_OXFML_FORMULA_DRILL_TRACE_MODEL.md`

## Position and Dependencies
- **Depends on**: `W054`, `W068`, `W075`
- **Blocks**: DNA OneCalc replacement of the current prepared-call formula
  drill projection with a semantic tree projection.
- **Cross-repo**: DNA OneCalc owns UI mapping and host-facing verdict policy;
  OxFml owns formula structure, evaluator trace projection, source-span
  correlation, LET/LAMBDA binding projection, lazy branch disposition, and
  runtime facade publication; OxFunc owns function/operator semantics and
  function metadata used for argument labels.

Reviewed inbound observations:
1. OxFunc notes reviewed: no conflicting request; argument names/roles should
   continue to come from OxFunc registry/call metadata where available.
2. OxCalc notes reviewed: no coordinator-facing FEC/F3E change is required for
   this first formula-drill lane; the artifact is consumer-runtime facing.
3. DnaOneCalc handoff processed: the request is accepted as an OxFml-owned
   trace-projection gap, not a host UI reconstruction task.

## Scope

### In Scope
1. Define the public `FormulaDrillTrace` V1 contract.
2. Preserve expression tree structure separately from evaluation order.
3. Define source-span, stable node id, diagnostic-link, and causal-error
   fields.
4. Define branch disposition for lazy/choice forms.
5. Define LET/LAMBDA binding projection requirements.
6. Define argument-name and argument-role source rules.
7. Define typed array/rich-value preview rules.
8. Emit the first runtime projection through `RuntimeFormulaResult` or an
   equivalent stable projection service.
9. Add deterministic evidence for the minimal DNA OneCalc corpus.

### Out of Scope
1. Moving function/operator semantics out of OxFunc.
2. DNA OneCalc UI implementation.
3. Pack-grade replay promotion.
4. Full Excel formula-language closure.
5. Reopening the frozen `OxFml_V1` consumer facade beyond this additive result
   artifact.

## Bead Set

### B076-01: FormulaDrillTrace V1 Public Contract

- **Tracker**: `fml-ldv.1`
- **Status**: in_progress
- **Owner**: OxFml
- **Effect**: define the additive public trace contract in the consumer facade,
  public API sketch, and DNA OneCalc downstream consumer contract.
- **Evidence target**: spec text names the artifact shape, node shape, state
  vocabulary, branch disposition, LET/LAMBDA flow, diagnostic links, error
  causality, typed previews, and minimum corpus.
- **Current note**: this is `spec_drafted` until runtime emission and tests
  exist; it must not be reported as implementation evidence.

### B076-02: Runtime Projection And Corpus Evidence

- **Tracker**: `fml-ldv.2`
- **Status**: planned
- **Owner**: OxFml
- **Effect**: emit the first `FormulaDrillTrace` through the runtime facade or
  approved projection service and add deterministic corpus evidence.
- **Evidence target**: focused W076 tests plus `cargo test -p oxfml_core`
  before any gate/closure claim.

## Minimal Acceptance Corpus
1. `=SUM(1,2,3)`
2. `=SUM(IF(TRUE,2,3),4)`
3. `=IF(FALSE,SUM(1,2),SUM(3,4))`
4. `=LET(x,1,y,2,SUM(x,y))`
5. `=1/0`
6. `=SEQUENCE(2,2)`
7. `=SUM(`

## Gate Model

### Entry Gate
- DnaOneCalc handoff reviewed.
- OxFunc and OxCalc inbound observation ledgers reviewed.
- Parent and child beads exist in `.beads`.

### Mid-Gate A: Contract
- Consumer facade contract names `FormulaDrillTrace` V1.
- Public API sketch names the intended code-facing projection shape.
- DNA OneCalc downstream contract says the host must consume the OxFml trace
  rather than reconstructing semantics locally.

### Mid-Gate B: Runtime Projection
- Runtime facade or approved projection service exposes the artifact.
- Focused corpus tests assert structure, source spans, branch disposition,
  LET binding flow, error causality, diagnostics, and typed array preview.

### Exit Gate
- Contract text and implementation agree.
- Focused W076 evidence passes.
- `cargo test -p oxfml_core` passes.
- DNA OneCalc follow-up status or handoff note is prepared, with downstream
  uptake explicitly left open until acknowledged.
- Pre-Closure Verification Checklist and Completion Claim Self-Audit pass.

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
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - runtime projection,
  - focused corpus evidence,
  - full validation,
  - DNA OneCalc uptake acknowledgement
- claim_confidence: draft
