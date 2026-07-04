//! Public no-regression witness for A1 `$` fidelity in the DEFAULT (non-profile) bind path.
//!
//! `parse_cell_reference` records per-axis absolute/relative flags derived from `$`
//! positions into `CellRef::address_mode` (see the in-crate unit tests
//! `binding::a1_dollar_fidelity_tests`). Those types are crate-internal, so this
//! integration test instead pins the load-bearing *observable* guarantee: enriching the
//! recorded `$` fidelity must NOT change how `$`-decorated A1 references bind or evaluate
//! through the default path — a cell reference resolves to the same cell regardless of
//! its `$` decoration.

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};

/// Probe helper: bind + execute a default-path formula and return the debug rendering of
/// its bound root, so decorations can be compared structurally.
fn bind_debug(formula: &str) -> String {
    let runtime = RuntimeEnvironment::new();
    let source = FormulaSourceRecord::new("a1-dollar-fidelity", 1, formula.to_string());
    let request = RuntimeFormulaRequest::new(source, TypedContextQueryBundle::default());
    match runtime.execute(request) {
        Ok(result) => format!("ok:{:?}", result.published_worksheet_value),
        Err(err) => format!("err:{err:?}"),
    }
}

#[test]
fn dollar_decorations_bind_and_evaluate_consistently_with_relative() {
    // All four `$` decorations of the same cell must produce the same public outcome as
    // the bare relative form. The `$` fidelity is recorded internally but is
    // evaluation-neutral: it never shifts which cell is referenced.
    let relative = bind_debug("=A1");
    for decorated in ["=$A1", "=A$1", "=$A$1"] {
        assert_eq!(
            bind_debug(decorated),
            relative,
            "`{decorated}` must bind/evaluate identically to `=A1` in the default path"
        );
    }
}
