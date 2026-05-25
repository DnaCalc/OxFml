*Posted by Codex agent on behalf of @govert*

# HO-FN-016 Function Call Target And Index Dispatch Ack

Status: `acknowledged`
Direction: OxFml -> OxFunc
Responds to: `../OxFunc/docs/handoffs/HO-FN-016_function_call_target_and_index_dispatch.md`
Source workset: `OxFunc/W096`
OxFml owner workset: `W075`
Acknowledged date: 2026-05-07

## Acknowledgement

OxFml acknowledges the HO-FN-016 ownership split:

1. OxFunc owns function/operator semantics, catalog-index dispatch, resolved function-call target handles, scratch buffers, and function metadata.
2. OxFml owns formula structure, binding, lexical scope, LET/LAMBDA frames, reference binding, child evaluation order, trace policy, and compiled plan consumption.
3. `FunctionCallTarget`, `FunctionExecutionContextBundle`, and `FunctionCallScratch` are the right seam for generic compiled formula evaluation without moving function-specific semantics into OxFml.
4. `FunctionCallTarget` metadata is the preferred source for future purity, volatility, host-interaction, callable-argument, and hoistability gates.

## Initial OxFml Response

The longer optimizer response is opened as `W075`.

Current OxFml work has a first consumption floor:

1. compiled evaluation plans retain resolved `FunctionCallTarget` handles for ordinary calls, operators, special operator lanes, reference operators, implicit intersection, and built-in callable slots;
2. evaluator FEC provider setup flows through `FunctionExecutionContextBundle`;
3. built-in callable `invoke_many` uses reusable `FunctionCallScratch` for repeated function-target invocation;
4. value-only versus prepared-call tracing is an explicit OxFml-owned `EvaluationTraceMode`, defaulting to value-only;
5. no OxFml `INDEX`, arithmetic, `HSTACK`, or helper-function semantic implementation is accepted as the optimizer direction.

## Remaining W075 Lanes

1. lexical slot-frame execution for lambda and LET locals,
2. generic compiled expression-node lowering beyond the current resolved function-call target floor,
3. reusable trace templates for prepared-call trace mode,
4. metadata-driven hoisting under explicit execution-context policy,
5. deterministic performance fixtures and before/after evidence,
6. narrow OxFunc metadata follow-ups if W075 discovers gaps.

## Status Axes

- execution_state: acknowledged
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W075 optimizer execution,
  - slot-frame evidence,
  - trace-template evidence,
  - metadata-driven hoisting evidence,
  - deterministic performance evidence.
