use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use oxfml_core::binding::{
    BinaryOp, BindContext, BindRequest, BoundExpr, NameKind, NormalizedReference, ReferenceExpr,
    UnaryOp, bind_formula, bind_formula_incremental,
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

            host_name_resolver: None,

            reference_bind_profile: None,
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

        host_name_resolver: None,

        reference_bind_profile: None,
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

        host_name_resolver: None,

        reference_bind_profile: None,
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

            host_name_resolver: None,

            reference_bind_profile: None,
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

        host_name_resolver: None,

        reference_bind_profile: None,
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

            host_name_resolver: None,

            reference_bind_profile: None,
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

        host_name_resolver: None,

        reference_bind_profile: None,
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

            host_name_resolver: None,

            reference_bind_profile: None,
        },
        Some(&bind.bound_formula),
    );
    assert!(!incremental_bind.reused_bound_formula);
}

#[test]
fn bind_inherits_explicit_sheet_qualifier_across_simple_a1_range() {
    let source = FormulaSourceRecord::new("fixture-sheet-qualified-range", 1, "=SUM(Alpha!A1:A2)");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.is_empty());
    assert_eq!(bind.bound_formula.normalized_references.len(), 1);
    match &bind.bound_formula.normalized_references[0] {
        NormalizedReference::Area(area) => {
            assert_eq!(area.sheet_id, "Alpha");
            assert_eq!(area.top_left.row, 1);
            assert_eq!(area.top_left.col, 1);
            assert_eq!(area.height, 2);
            assert_eq!(area.width, 1);
        }
        other => panic!("expected normalized area reference, got {other:?}"),
    }
}

#[test]
fn bind_preserves_array_literal_multiplied_by_unary_negative_literal_shape() {
    let source = FormulaSourceRecord::new("fixture-array-negation", 1, "={1,2,3;2,3,4}*-1");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    let BoundExpr::Binary { op, left, right } = &bind.bound_formula.root else {
        panic!("expected multiply root, got {:?}", bind.bound_formula.root);
    };
    assert_eq!(*op, BinaryOp::Multiply);
    assert!(matches!(left.as_ref(), BoundExpr::ArrayLiteral(_)));

    let BoundExpr::Unary {
        op: right_op,
        expr: unary_expr,
    } = right.as_ref()
    else {
        panic!("expected unary negative literal, got {:?}", right);
    };
    assert_eq!(*right_op, UnaryOp::Negate);
    assert_eq!(
        unary_expr.as_ref(),
        &BoundExpr::NumberLiteral("1".to_string())
    );
}

#[test]
fn bind_prefers_builtin_function_over_colliding_helper_local_in_call_position() {
    let source = FormulaSourceRecord::new(
        "fixture-builtin-collision-helper-call",
        1,
        "=LET(gcd,LAMBDA(self,a,b,IF(b=0,a,self(self,b,MOD(a,b)))),gcd(gcd,48,36))",
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.is_empty());
    let BoundExpr::FunctionCall {
        function_name,
        args,
        ..
    } = &bind.bound_formula.root
    else {
        panic!("expected LET call root, got {:?}", bind.bound_formula.root);
    };
    assert_eq!(function_name, "LET");

    let BoundExpr::FunctionCall {
        function_name,
        args: invocation_args,
        ..
    } = &args[2]
    else {
        panic!("expected builtin function call body, got {:?}", args[2]);
    };
    assert_eq!(function_name, "GCD");
    let BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(name))) =
        &invocation_args[0]
    else {
        panic!(
            "expected helper-local name as first arg, got {:?}",
            invocation_args[0]
        );
    };
    assert_eq!(name.kind, NameKind::HelperLocal);
    assert_eq!(name.name, "gcd");
}

#[test]
fn bind_surfaces_t_builtin_arity_authoring_reject_for_ftc_0444() {
    let source = FormulaSourceRecord::new(
        "fixture-builtin-collision-arity-helper-call",
        1,
        "=LET(THUNK,LAMBDA(x,LAMBDA(x)),t,THUNK(42),t())",
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .starts_with("built-in function call 'T' rejects 0 arguments at the authoring boundary")
    }));
    let BoundExpr::FunctionCall {
        function_name,
        args,
        ..
    } = &bind.bound_formula.root
    else {
        panic!("expected LET call root, got {:?}", bind.bound_formula.root);
    };
    assert_eq!(function_name, "LET");

    let BoundExpr::FunctionCall {
        function_name,
        args: invocation_args,
        ..
    } = &args[4]
    else {
        panic!("expected builtin function call body, got {:?}", args[4]);
    };
    assert_eq!(function_name, "T");
    assert!(invocation_args.is_empty());
}

#[test]
fn bind_surfaces_generic_builtin_arity_authoring_reject_for_plain_gcd_zero_arity() {
    let source = FormulaSourceRecord::new("fixture-builtin-arity-gcd-zero", 1, "=GCD()");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.starts_with(
            "built-in function call 'GCD' rejects 0 arguments at the authoring boundary",
        )
    }));
    let BoundExpr::FunctionCall {
        function_name,
        args,
        ..
    } = &bind.bound_formula.root
    else {
        panic!("expected GCD call root, got {:?}", bind.bound_formula.root);
    };
    assert_eq!(function_name, "GCD");
    assert!(args.is_empty());
}

#[test]
fn bind_accepts_error_literal_inside_array_call_for_ftc_0837() {
    let source = FormulaSourceRecord::new(
        "fixture-ftc-0837-error-literal-array",
        1,
        "=SUM(TOCOL({1,2,#N/A;4,5,6},1))",
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    assert!(parse.green_tree.diagnostics.is_empty());
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.is_empty());

    let BoundExpr::FunctionCall {
        function_name,
        args,
        ..
    } = &bind.bound_formula.root
    else {
        panic!("expected SUM call root, got {:?}", bind.bound_formula.root);
    };
    assert_eq!(function_name, "SUM");

    let BoundExpr::FunctionCall {
        function_name,
        args: tocol_args,
        ..
    } = &args[0]
    else {
        panic!("expected TOCOL inner call, got {:?}", args[0]);
    };
    assert_eq!(function_name, "TOCOL");

    let BoundExpr::ArrayLiteral(rows) = &tocol_args[0] else {
        panic!("expected TOCOL array argument, got {:?}", tocol_args[0]);
    };
    let BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(error))) = &rows[0][2]
    else {
        panic!("expected #N/A error literal cell, got {:?}", rows[0][2]);
    };
    assert_eq!(error.error_class, "#N/A");
    assert_eq!(error.source_text, "#N/A");
}

#[test]
fn bind_accepts_filterxml_embedded_quoted_xml_string_for_ftc_1041() {
    let source = FormulaSourceRecord::new(
        "fixture-ftc-1041-filterxml-quoted-xml",
        1,
        "=FILTERXML(\"<items><item id=\"\"1\"\">apple</item><item id=\"\"2\"\">banana</item></items>\",\"//item[@id=2]\")",
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    assert!(parse.green_tree.diagnostics.is_empty());
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.is_empty());

    let BoundExpr::FunctionCall {
        function_name,
        args,
        ..
    } = &bind.bound_formula.root
    else {
        panic!(
            "expected FILTERXML call root, got {:?}",
            bind.bound_formula.root
        );
    };
    assert_eq!(function_name, "FILTERXML");
    assert_eq!(args.len(), 2);
    assert_eq!(
        args[0],
        BoundExpr::StringLiteral(
            "\"<items><item id=\"\"1\"\">apple</item><item id=\"\"2\"\">banana</item></items>\""
                .to_string()
        )
    );
    assert_eq!(
        args[1],
        BoundExpr::StringLiteral("\"//item[@id=2]\"".to_string())
    );
}

#[test]
fn parse_rejects_unbalanced_deep_parenthesis_family_for_ftc_0916_and_ftc_0987() {
    let source = FormulaSourceRecord::new(
        "fixture-ftc-0916-ftc-0987-deep-parenthesis",
        1,
        "=((((((((((((((((1+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)",
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });

    assert_eq!(parse.green_tree.diagnostics.len(), 1);
    assert_eq!(
        parse.green_tree.diagnostics[0].message,
        "expected ')'".to_string()
    );

    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.is_empty());
    assert!(matches!(bind.bound_formula.root, BoundExpr::Binary { .. }));
}

#[test]
fn bind_prefers_helper_local_name_over_cell_like_reference_text() {
    let source = FormulaSourceRecord::new(
        "fixture-returned-lambda-helper-name",
        1,
        "=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),add5,adder(5),add5(10))",
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });

    assert!(bind.bound_formula.diagnostics.is_empty());
    let BoundExpr::FunctionCall {
        function_name,
        args,
        ..
    } = &bind.bound_formula.root
    else {
        panic!("expected LET call root, got {:?}", bind.bound_formula.root);
    };
    assert_eq!(function_name, "LET");

    let BoundExpr::Invocation { callee, .. } = &args[4] else {
        panic!("expected helper invocation body, got {:?}", args[4]);
    };
    let BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(name))) =
        callee.as_ref()
    else {
        panic!("expected helper-local name callee, got {:?}", callee);
    };
    assert_eq!(name.kind, NameKind::HelperLocal);
    assert_eq!(name.name, "add5");
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
        SyntaxKind::ArrayLiteralExpr => "ArrayLiteralExpr",
        SyntaxKind::IdentifierExpr => "IdentifierExpr",
        SyntaxKind::QuotedIdentifierExpr => "QuotedIdentifierExpr",
        SyntaxKind::QualifiedReferenceExpr => "QualifiedReferenceExpr",
        SyntaxKind::HostMemberReferenceExpr => "HostMemberReferenceExpr",
        SyntaxKind::HostReferenceCollectionExpr => "HostReferenceCollectionExpr",
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
        SyntaxKind::OmittedArgExpr => "OmittedArgExpr",
        SyntaxKind::MissingExpr => "MissingExpr",
    }
}

fn bound_expr_name(expr: &BoundExpr) -> &'static str {
    match expr {
        BoundExpr::NumberLiteral(_) => "NumberLiteral",
        BoundExpr::StringLiteral(_) => "StringLiteral",
        BoundExpr::LogicalLiteral(_) => "LogicalLiteral",
        BoundExpr::ArrayLiteral(_) => "ArrayLiteral",
        BoundExpr::OmittedArgument => "OmittedArgument",
        BoundExpr::HelperParameterName(_) => "HelperParameterName",
        BoundExpr::HelperOptionalParameterName(_) => "HelperOptionalParameterName",
        BoundExpr::Binary { .. } => "Binary",
        BoundExpr::Unary { .. } => "Unary",
        BoundExpr::FunctionCall { .. } => "FunctionCall",
        BoundExpr::Invocation { .. } => "Invocation",
        BoundExpr::Reference(_) => "Reference",
        BoundExpr::HostReference(_) => "HostReference",
        BoundExpr::HostStructuralSelector(_) => "HostStructuralSelector",
        BoundExpr::HostReferenceCollection(_) => "HostReferenceCollection",
        BoundExpr::ImplicitIntersection(_) => "ImplicitIntersection",
    }
}
