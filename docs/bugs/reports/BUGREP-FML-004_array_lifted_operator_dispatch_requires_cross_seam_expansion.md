# BUGREP-FML-004: Array-Lifted Operator Dispatch Requires Cross-Seam Expansion

## Intake
- **Report id**: `BUGREP-FML-004`
- **Filed**: 2026-04-07
- **Status**: triaged
- **Reported against ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Reported against kind**: `commit`
- **Source channel**: `local_boundary_review`
- **Canonical bug id**: `BUG-FML-004`

## Observed Facts
During `W059`, scalar binary arithmetic was successfully moved onto OxFunc `FUNC.OP_*` dispatch, but array-lifted ordinary arithmetic rows failed when routed through the same surface.

Observed local failures before compatibility fallback:

| formula | observed failure |
|---|---|
| `={1,2,3;2,3,4}*-1` | `OxFunc surface evaluation failed for OP_MULTIPLY: Value` |
| `={1,2;3,4}+{10,20;30,40}` | `OxFunc surface evaluation failed for OP_ADD: Value` |
| `={1,2;6,8}/{1,0;3,2}` | `OxFunc surface evaluation failed for OP_DIVIDE: Value` |

## Notes
- This is not evidence that ordinary operator semantic ownership belongs back in OxFml.
- It is evidence that the current prepared-call seam is still too narrow for array-lifted ordinary operator rows.
- `W059` currently uses a clearly marked temporary compatibility fallback for array-involved binary arithmetic while this stream remains open.

## Evidence
1. `crates/oxfml_core/src/eval/mod.rs`
2. `../OxFunc/crates/oxfunc_core/src/functions/binary_numeric.rs`
3. `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
4. `crates/oxfml_core/tests/evaluator_tests.rs`
5. `cargo test -p oxfml_core`
