use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use oxfml_core::binding::{BindContext, BindRequest, BoundExpr, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::green::{GreenChild, SyntaxKind};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};

#[derive(Debug, Deserialize)]
struct ParseBindFixture {
    case_id: String,
    formula: String,
    expected_root_kind: String,
    expected_parse_diagnostics: usize,
    expected_bound_root_kind: String,
    expected_reference_count: usize,
    expected_bind_diagnostics: usize,
}

#[test]
fn parse_and_bind_fixtures_match_initial_w002_slice() {
    let fixtures = load_fixtures();
    for (index, fixture) in fixtures.iter().enumerate() {
        let source =
            FormulaSourceRecord::new(format!("fixture-{index}"), 1, fixture.formula.clone());
        let parse = parse_formula(ParseRequest {
            source: source.clone(),
        });

        let root_kind = parse
            .green_tree
            .root
            .children
            .iter()
            .find_map(|child| match child {
                GreenChild::Node(node) => Some(node.kind),
                GreenChild::Token(_) => None,
            })
            .unwrap();

        assert_eq!(
            syntax_kind_name(root_kind),
            fixture.expected_root_kind,
            "root syntax mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            parse.green_tree.diagnostics.len(),
            fixture.expected_parse_diagnostics,
            "parse diagnostics mismatch for {}",
            fixture.case_id
        );

        let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
        assert_eq!(red.root().span.start, 0, "red root span should start at 0");

        let bind = bind_formula(BindRequest {
            source: source.clone(),
            green_tree: parse.green_tree,
            red_projection: red,
            context: BindContext {
                structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
                formula_token: source.formula_token(),
                ..BindContext::default()
            },
        });

        assert_eq!(
            bound_expr_name(&bind.bound_formula.root),
            fixture.expected_bound_root_kind,
            "bound root mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            bind.bound_formula.normalized_references.len(),
            fixture.expected_reference_count,
            "reference count mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            bind.bound_formula.diagnostics.len(),
            fixture.expected_bind_diagnostics,
            "bind diagnostics mismatch for {}",
            fixture.case_id
        );
    }
}

#[test]
fn unresolved_identifier_becomes_typed_bind_diagnostic() {
    let source = FormulaSourceRecord::new("fixture-unresolved", 1, "=UnknownName");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext::default(),
    });

    assert_eq!(bind.bound_formula.unresolved_references.len(), 1);
    assert_eq!(bind.bound_formula.diagnostics.len(), 1);
}

fn load_fixtures() -> Vec<ParseBindFixture> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("parse_bind_cases.json");
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn syntax_kind_name(kind: SyntaxKind) -> &'static str {
    match kind {
        SyntaxKind::FormulaRoot => "FormulaRoot",
        SyntaxKind::NumberLiteralExpr => "NumberLiteralExpr",
        SyntaxKind::StringLiteralExpr => "StringLiteralExpr",
        SyntaxKind::IdentifierExpr => "IdentifierExpr",
        SyntaxKind::CallExpr => "CallExpr",
        SyntaxKind::InvokeExpr => "InvokeExpr",
        SyntaxKind::ArgumentList => "ArgumentList",
        SyntaxKind::BinaryExpr => "BinaryExpr",
        SyntaxKind::PrefixExpr => "PrefixExpr",
        SyntaxKind::PostfixExpr => "PostfixExpr",
        SyntaxKind::GroupingExpr => "GroupingExpr",
        SyntaxKind::RangeExpr => "RangeExpr",
        SyntaxKind::IntersectionExpr => "IntersectionExpr",
        SyntaxKind::UnionExpr => "UnionExpr",
        SyntaxKind::MissingExpr => "MissingExpr",
    }
}

fn bound_expr_name(expr: &BoundExpr) -> &'static str {
    match expr {
        BoundExpr::NumberLiteral(_) => "NumberLiteral",
        BoundExpr::StringLiteral(_) => "StringLiteral",
        BoundExpr::HelperParameterName(_) => "HelperParameterName",
        BoundExpr::Binary { .. } => "Binary",
        BoundExpr::FunctionCall { .. } => "FunctionCall",
        BoundExpr::Invocation { .. } => "Invocation",
        BoundExpr::Reference(_) => "Reference",
        BoundExpr::ImplicitIntersection(_) => "ImplicitIntersection",
    }
}
