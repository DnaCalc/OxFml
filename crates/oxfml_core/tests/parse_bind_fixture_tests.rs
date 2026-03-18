use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use oxfml_core::binding::{
    BindContext, BindRequest, BoundExpr, bind_formula, bind_formula_incremental,
};
use oxfml_core::red::{project_red_view, project_red_view_incremental};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::green::{GreenChild, SyntaxKind};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula, parse_formula_incremental};

#[derive(Debug, Deserialize)]
struct ParseBindFixture {
    case_id: String,
    formula: String,
    expected_root_kind: String,
    expected_parse_diagnostics: usize,
    expected_bound_root_kind: String,
    expected_reference_count: usize,
    expected_bind_diagnostics: usize,
    expected_normalized_references: Option<Vec<String>>,
    expected_unresolved_reasons: Option<Vec<String>>,
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
        if let Some(expected_references) = &fixture.expected_normalized_references {
            let normalized = bind
                .bound_formula
                .normalized_references
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            assert_eq!(
                normalized, *expected_references,
                "normalized references mismatch for {}",
                fixture.case_id
            );
        }
        if let Some(expected_reasons) = &fixture.expected_unresolved_reasons {
            let reasons = bind
                .bound_formula
                .unresolved_references
                .iter()
                .map(|record| record.reason.clone())
                .collect::<Vec<_>>();
            assert_eq!(
                reasons, *expected_reasons,
                "unresolved reasons mismatch for {}",
                fixture.case_id
            );
        }
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

#[test]
fn incremental_parse_red_and_bind_reuse_same_immutable_artifacts() {
    let source =
        FormulaSourceRecord::new("fixture-incremental", 1, "='Annual Data'!A:'Annual Data'!C");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree.clone(),
        red_projection: red.clone(),
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },
    });

    let incremental_parse = parse_formula_incremental(
        ParseRequest {
            source: source.clone(),
        },
        Some(&parse.green_tree),
    );
    assert!(incremental_parse.reused_green_tree);

    let incremental_red = project_red_view_incremental(
        source.formula_stable_id.clone(),
        &incremental_parse.green_tree,
        Some(&red),
    );
    assert!(incremental_red.reused_red_projection);

    let incremental_bind = bind_formula_incremental(
        BindRequest {
            source: source.clone(),
            green_tree: incremental_parse.green_tree,
            red_projection: incremental_red.red_projection,
            context: BindContext {
                structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
                formula_token: source.formula_token(),
                ..BindContext::default()
            },
        },
        Some(&bind.bound_formula),
    );
    assert!(incremental_bind.reused_bound_formula);
}

#[test]
fn incremental_reference_breadth_reuse_keeps_whole_row_bindings() {
    let source = FormulaSourceRecord::new("fixture-incremental-whole-row", 1, "=1:3");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree.clone(),
        red_projection: red.clone(),
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },
    });

    let incremental_parse = parse_formula_incremental(
        ParseRequest {
            source: source.clone(),
        },
        Some(&parse.green_tree),
    );
    assert!(incremental_parse.reused_green_tree);

    let incremental_red = project_red_view_incremental(
        source.formula_stable_id.clone(),
        &incremental_parse.green_tree,
        Some(&red),
    );
    assert!(incremental_red.reused_red_projection);

    let incremental_bind = bind_formula_incremental(
        BindRequest {
            source: source.clone(),
            green_tree: incremental_parse.green_tree,
            red_projection: incremental_red.red_projection,
            context: BindContext {
                structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
                formula_token: source.formula_token(),
                ..BindContext::default()
            },
        },
        Some(&bind.bound_formula),
    );
    assert!(incremental_bind.reused_bound_formula);
}

#[test]
fn incremental_bind_invalidates_when_bind_context_changes() {
    let source = FormulaSourceRecord::new("fixture-incremental-context", 1, "=InputValue");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree.clone(),
        red_projection: red.clone(),
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },
    });

    let mut names = std::collections::BTreeMap::new();
    names.insert(
        "InputValue".to_string(),
        oxfml_core::binding::NameKind::ValueLike,
    );
    let incremental_bind = bind_formula_incremental(
        BindRequest {
            source: source.clone(),
            green_tree: parse.green_tree,
            red_projection: red,
            context: BindContext {
                structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
                formula_token: source.formula_token(),
                names,
                ..BindContext::default()
            },
        },
        Some(&bind.bound_formula),
    );
    assert!(!incremental_bind.reused_bound_formula);
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
        SyntaxKind::QuotedIdentifierExpr => "QuotedIdentifierExpr",
        SyntaxKind::QualifiedReferenceExpr => "QualifiedReferenceExpr",
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
