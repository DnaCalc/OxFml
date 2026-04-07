# HANDOFF_OXFUNC_001 — Operator Value Surface And Array-Lift Expansion

## Header
1. handoff id: `HANDOFF-OXFUNC-001`
2. date: `2026-04-07`
3. from repo: `OxFml`
4. to repo: `OxFunc`
5. related workset or feature id: `W059`

## Purpose
Record a formal OxFml-to-OxFunc bug and seam-expansion request for ordinary operator execution.

Historical note:
1. the array-lifted binary-operator concern recorded below has since been rechecked in the local workspace,
2. current local validation now shows admitted array-lifted ordinary arithmetic rows travel through the OxFunc-backed path,
3. this handoff therefore remains a historical seam note unless OxFunc reports a contradictory downstream read.

The immediate trigger was the `^` bug revisit, but the real scope is broader:
1. scalar ordinary binary arithmetic can now be routed through OxFunc `FUNC.OP_*` rows from OxFml,
2. array-involved ordinary binary arithmetic cannot yet travel through the same prepared-call/value seam without falling back locally,
3. the wider operator family still needs a clean shared execution plan rather than one-off local patches.

## Current OxFml Read
Current OxFml read of the operator boundary is:
1. OxFml should own lexical grammar, parse structure, precedence, associativity, and bind-time operator/reference shaping.
2. OxFunc should own ordinary operator semantic truth.
3. The recent local `^` symptom made it obvious that OxFml had drifted into local ordinary-operator execution.
4. The scalar arithmetic slice can now be corrected locally by dispatching:
   - `BinaryOp::Add -> FUNC.OP_ADD`
   - `BinaryOp::Subtract -> FUNC.OP_SUBTRACT`
   - `BinaryOp::Multiply -> FUNC.OP_MULTIPLY`
   - `BinaryOp::Divide -> FUNC.OP_DIVIDE`
   - `BinaryOp::Power -> FUNC.OP_POWER`
5. That scalar dispatch is now locally validated in OxFml.
6. But array-involved binary arithmetic still fails through the current OxFml→OxFunc prepared-call/value surface unless OxFml keeps a temporary compatibility fallback.

## Concrete Failure Observed
When OxFml routed array-involved binary arithmetic directly through the current OxFunc operator surface, retained evaluator rows failed with worksheet `Value` outcomes such as:
1. `={1,2,3;2,3,4}*-1` -> `OxFunc surface evaluation failed for OP_MULTIPLY: Value`
2. `={1,2;3,4}+{10,20;30,40}` -> `OxFunc surface evaluation failed for OP_ADD: Value`
3. `={1,2;6,8}/{1,0;3,2}` -> `OxFunc surface evaluation failed for OP_DIVIDE: Value`

Current OxFml evidence points to the present OxFunc prepared-call/value surface as the limiting step, not to the ownership direction being wrong.

Relevant OxFunc evidence already visible from OxFml:
1. `../OxFunc/crates/oxfunc_core/src/functions/binary_numeric.rs`
2. `../OxFunc/crates/oxfunc_core/src/functions/operator_arithmetic_family.rs`
3. `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

Key local read from `surface_dispatch.rs`:
1. the current prepared call-arg model does not yet carry full array payloads,
2. the current binary numeric surface therefore behaves as scalar-only for this lane.

## Wider Scope OxFml Intends To Deal With
This handoff is not only about the first binary-array symptom.
OxFml wants to deal with the broader ordinary-operator family coherently.

### A. Ordinary operator rows OxFunc already owns
OxFml can already see OxFunc operator rows for:
1. arithmetic:
   - `FUNC.OP_ADD`
   - `FUNC.OP_SUBTRACT`
   - `FUNC.OP_MULTIPLY`
   - `FUNC.OP_DIVIDE`
   - `FUNC.OP_POWER`
   - `FUNC.OP_NEGATE`
   - `FUNC.OP_UNARY_PLUS`
   - `FUNC.OP_PERCENT`
2. concat and comparisons:
   - `FUNC.OP_CONCAT`
   - `FUNC.OP_EQUAL`
   - `FUNC.OP_NOT_EQUAL`
   - `FUNC.OP_LESS_THAN`
   - `FUNC.OP_LESS_EQUAL`
   - `FUNC.OP_GREATER_THAN`
   - `FUNC.OP_GREATER_EQUAL`
3. reference operators:
   - `FUNC.OP_RANGE_REF`
   - `FUNC.OP_INTERSECTION_REF`
   - `FUNC.OP_UNION_REF`
   - `FUNC.OP_SPILL_REF`
   - trim-ref rows

### B. Ordinary operator rows OxFml does not yet fully express
Current OxFml status:
1. present in token/parser/binder:
   - binary arithmetic `+ - * / ^`
   - reference operators `: , <space>`
   - `@`
   - `#`
2. still missing in OxFml token/parser/binder:
   - postfix `%`
   - concatenation `&`
   - comparisons `< <= > >= = <>` as ordinary comparison operators
3. still partial in OxFml execution boundary:
   - scalar binary arithmetic now dispatches to OxFunc
   - array-involved binary arithmetic still uses a temporary OxFml fallback
   - union/intersection references still do not execute through the admitted lane in OxFml
   - unary arithmetic still needs explicit boundary review

## Requested OxFunc Intake
OxFml’s request to OxFunc is:
1. treat the current scalar-only prepared-call/value limitation for ordinary operators as a real bug / seam gap, not as acceptable final behavior,
2. scope the fix broadly enough to support array-lifted ordinary operator execution rather than only the first `OP_ADD`/`OP_MULTIPLY` symptom,
3. clarify the intended owner path for:
   - scalar ordinary operator execution,
   - array-lifted ordinary operator execution,
   - worksheet-error carriage from operator surfaces,
4. identify the minimum OxFunc-side changes needed so OxFml can remove the temporary array-compatibility fallback for binary arithmetic,
5. keep the wider operator family in view so the seam does not get patched one operator at a time.

## Requested Broad Fix Shape
OxFml’s preferred broad direction is:
1. the OxFml→OxFunc prepared-call/value surface should carry enough value structure for ordinary operator rows to handle:
   - scalar/scalar
   - scalar/array
   - array/scalar
   - same-shape array/array
2. worksheet error outcomes from ordinary operators should remain typed worksheet values, not fatal transport failures,
3. the same widened value surface should be usable by:
   - binary arithmetic rows
   - unary arithmetic rows
   - postfix percent
   - concat / comparison rows where admitted
4. once that surface exists, OxFml can narrow further and stop keeping local semantic compatibility code for those operator families.

## Current OxFml Temporary State
OxFml has taken only a partial local corrective step:
1. scalar binary arithmetic now dispatches into OxFunc rows,
2. array-involved binary arithmetic currently falls back to a clearly marked temporary OxFml compatibility path,
3. this is recorded locally as:
   - `BUG-FML-003`
   - `BUG-FML-004`
   - `W059`
4. OxFml does not treat that temporary fallback as the final seam.

## Evidence
1. `crates/oxfml_core/src/eval/mod.rs`
2. `crates/oxfml_core/tests/evaluator_tests.rs`
3. `crates/oxfml_core/tests/fixtures/callable_transport_cases.json`
4. `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
5. `docs/bugs/streams/BUG-FML-003_ordinary_operator_semantics_should_dispatch_to_oxfunc.md`
6. `docs/bugs/streams/BUG-FML-004_array_lifted_operator_dispatch_needs_seam_expansion.md`
7. `docs/worksets/W059_operator_semantic_dispatch_boundary_correction.md`
8. `cargo test -p oxfml_core`
