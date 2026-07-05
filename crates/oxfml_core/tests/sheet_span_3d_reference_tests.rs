//! W078 — 3D sheet-range reference grammar (`Sheet1:Sheet3!A1`).
//!
//! These tests prove the W078 contract:
//!  - the bounded `Colon Identifier Bang` lookahead produces a distinct
//!    `SheetSpan3DReferenceExpr` node for `Sheet1:Sheet3!A1` (and leaves the
//!    trailing `:B2` of `Sheet1:Sheet3!A1:B2` to the range loop);
//!  - the lookahead is guarded by the mandatory trailing `!`, so ordinary
//!    ranges (`A1:A3`), name unions (`name:name`), bare sheet ranges
//!    (`Sheet1:Sheet3`), and single-sheet qualified ranges (`Sheet1!A1:B2`)
//!    are byte-for-byte unchanged;
//!  - the binder consults the profile's `sheet_span_3d_references` capability
//!    as a routing gate (W062 D2 §3): flag ON binds a distinct
//!    `NormalizedReference::SheetSpan3D`; flag OFF emits a typed `#REF!`
//!    capability rejection, never a silently-wrong range;
//!  - the 3D span stays distinct from a same-sheet multi-area and from a range;
//!  - the authored source round-trips through the green tree unchanged.

use oxfml_core::binding::{
    BindContext, BindRequest, BoundExpr, NormalizedReference, ReferenceExpr, bind_formula,
};
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaChannelKind, FormulaSourceRecord, FormulaToken};
use oxfml_core::syntax::green::{GreenChild, GreenNode, GreenTreeRoot, SyntaxKind};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::syntax::token::TokenKind;
use oxfml_core::{
    ProfilePayload, ProfileReferenceRecord, ProfileVersion, ReferenceAtomBindRequest,
    ReferenceAtomBindResult, ReferenceBindProfile, ReferenceNameBindRequest,
    ReferenceNormalFormKey, ReferencePolicy, ReferenceSourceInfo, ReferenceSyntaxCapabilities,
    ReferenceValidity,
};

// ---- grammar helpers --------------------------------------------------------

fn parse(formula: &str) -> GreenTreeRoot {
    let source = FormulaSourceRecord::new("w078-test", 1, formula.to_string());
    parse_formula(ParseRequest { source }).green_tree
}

/// The single top-level expression node under `FormulaRoot`.
fn root_expr(tree: &GreenTreeRoot) -> &GreenNode {
    tree.root
        .children
        .iter()
        .find_map(|child| match child {
            GreenChild::Node(node) => Some(node.as_ref()),
            GreenChild::Token(_) => None,
        })
        .expect("formula root should carry an expression node")
}

/// Reconstruct the authored source from the green tree's full-fidelity tokens.
fn round_trip_source(tree: &GreenTreeRoot) -> String {
    tree.full_fidelity_tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .map(|t| t.text.as_str())
        .collect()
}

fn find_kind(node: &GreenNode, kind: SyntaxKind) -> bool {
    if node.kind == kind {
        return true;
    }
    node.children.iter().any(|child| match child {
        GreenChild::Node(n) => find_kind(n, kind),
        GreenChild::Token(_) => false,
    })
}

// ---- grammar: 3D text parses to the span expr -------------------------------

#[test]
fn sheet_span_3d_parses_to_span_expr_not_range() {
    let tree = parse("=Sheet1:Sheet3!A1");
    let expr = root_expr(&tree);
    assert_eq!(
        expr.kind,
        SyntaxKind::SheetSpan3DReferenceExpr,
        "Sheet1:Sheet3!A1 must parse as a 3D span, not a range; got {:?}",
        expr.kind
    );
    // Children order: start_sheet, colon, end_sheet, bang, target.
    let tokens: Vec<_> = expr
        .children
        .iter()
        .filter_map(|c| match c {
            GreenChild::Token(t) => Some((t.kind, t.text.clone())),
            GreenChild::Node(_) => None,
        })
        .collect();
    assert_eq!(tokens[0], (TokenKind::Identifier, "Sheet1".to_string()));
    assert_eq!(tokens[1].0, TokenKind::Colon);
    assert_eq!(tokens[2], (TokenKind::Identifier, "Sheet3".to_string()));
    assert_eq!(tokens[3].0, TokenKind::Bang);
    assert_eq!(tree.diagnostics.len(), 0, "{:?}", tree.diagnostics);
}

#[test]
fn sheet_span_3d_with_area_target_leaves_trailing_range_to_loop() {
    // `Sheet1:Sheet3!A1:B2` → RangeExpr(SheetSpan3D(Sheet1,Sheet3,A1), B2):
    // the span production claims the FIRST `:`; the range loop claims the
    // second, exactly paralleling the qualified-range `Sheet1!A1:B2` shape.
    let tree = parse("=Sheet1:Sheet3!A1:B2");
    let expr = root_expr(&tree);
    assert_eq!(expr.kind, SyntaxKind::RangeExpr);
    let left = expr
        .children
        .iter()
        .find_map(|c| match c {
            GreenChild::Node(n) => Some(n.as_ref()),
            GreenChild::Token(_) => None,
        })
        .unwrap();
    assert_eq!(left.kind, SyntaxKind::SheetSpan3DReferenceExpr);
    assert_eq!(tree.diagnostics.len(), 0, "{:?}", tree.diagnostics);
}

// ---- grammar: ordinary shapes are untouched (ambiguity guard) ---------------

#[test]
fn plain_range_a1_a3_is_untouched() {
    let tree = parse("=A1:A3");
    assert_eq!(root_expr(&tree).kind, SyntaxKind::RangeExpr);
    assert!(
        !find_kind(root_expr(&tree), SyntaxKind::SheetSpan3DReferenceExpr),
        "A1:A3 must not produce a 3D span node",
    );
}

#[test]
fn name_colon_name_is_untouched() {
    let tree = parse("=name:name");
    assert_eq!(root_expr(&tree).kind, SyntaxKind::RangeExpr);
    assert!(!find_kind(
        root_expr(&tree),
        SyntaxKind::SheetSpan3DReferenceExpr
    ));
}

#[test]
fn bare_sheet_range_without_bang_stays_a_range() {
    // The trailing `!` is mandatory: `Sheet1:Sheet3` (no bang) is an ordinary
    // range, never a 3D span.
    let tree = parse("=Sheet1:Sheet3");
    assert_eq!(root_expr(&tree).kind, SyntaxKind::RangeExpr);
    assert!(!find_kind(
        root_expr(&tree),
        SyntaxKind::SheetSpan3DReferenceExpr
    ));
}

#[test]
fn single_sheet_qualified_range_is_untouched() {
    // `Sheet1!A1:B2` → RangeExpr(QualifiedReferenceExpr(Sheet1,A1), B2), the
    // pre-W078 shape — no 3D span node anywhere.
    let tree = parse("=Sheet1!A1:B2");
    let expr = root_expr(&tree);
    assert_eq!(expr.kind, SyntaxKind::RangeExpr);
    assert!(find_kind(expr, SyntaxKind::QualifiedReferenceExpr));
    assert!(
        !find_kind(expr, SyntaxKind::SheetSpan3DReferenceExpr),
        "single-sheet qualified range must not become a 3D span",
    );
}

#[test]
fn quoted_start_sheet_forms_a_span() {
    // A quoted *start* sheet is safe (it is the already-bumped leading
    // identifier, disambiguated by the trailing `!`): `'My Sheet':Sheet3!A1`
    // parses as a 3D span.
    let tree = parse("='My Sheet':Sheet3!A1");
    assert_eq!(
        root_expr(&tree).kind,
        SyntaxKind::SheetSpan3DReferenceExpr,
        "'My Sheet':Sheet3!A1 must parse as a 3D span",
    );
    assert_eq!(tree.diagnostics.len(), 0, "{:?}", tree.diagnostics);
}

#[test]
fn quoted_end_sheet_is_a_typed_limitation_not_a_span() {
    // The end sheet is restricted to a plain Identifier so the tested
    // whole-column qualified-range form `'Sheet'!A:'Sheet'!C` is not mis-split.
    // The cost is that a quoted *end* sheet is NOT recognized as a span; it
    // fails safe to the pre-W078 range shape with no diagnostic (a recorded
    // limitation, see the W078 workset doc). This test pins that behavior so a
    // future broadening must consciously update it.
    let tree = parse("=Sheet1:'Q2 Data'!A1");
    assert_ne!(
        root_expr(&tree).kind,
        SyntaxKind::SheetSpan3DReferenceExpr,
        "quoted-end form is a typed limitation, not a span",
    );
    // And the whole-column qualified range it protects still parses as a range.
    let range = parse("='Annual Data'!A:'Annual Data'!C");
    assert_eq!(root_expr(&range).kind, SyntaxKind::RangeExpr);
    assert!(!find_kind(
        root_expr(&range),
        SyntaxKind::SheetSpan3DReferenceExpr
    ));
}

// ---- render round-trip ------------------------------------------------------

#[test]
fn authored_3d_source_round_trips_through_green_tree() {
    for formula in ["=Sheet1:Sheet3!A1", "=Sheet1:Sheet3!A1:B2"] {
        let tree = parse(formula);
        assert_eq!(
            round_trip_source(&tree),
            formula,
            "3D span source must round-trip byte-for-byte",
        );
    }
}

// ---- bind gating: profile plumbing ------------------------------------------

/// A minimal symbolic profile whose `sheet_span_3d_references` capability is
/// configurable per test. It binds nothing itself for the span path (the 3D
/// bind is grammar-driven and gated only by the capability flag); it exists to
/// carry the flag and to land other identifiers on the typed rejection path.
struct SpanProfile {
    capabilities: ReferenceSyntaxCapabilities,
}

impl ReferenceBindProfile for SpanProfile {
    fn profile_id(&self) -> &str {
        "span.profile.v1"
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn syntax_capabilities(&self) -> ReferenceSyntaxCapabilities {
        self.capabilities
    }

    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        // Bind any single atom so `A1`/`B2` targets do not perturb diagnostics.
        ReferenceAtomBindResult::Bound(ProfileReferenceRecord {
            profile_id: self.profile_id().to_string(),
            profile_version: ProfileVersion::v1(),
            source_info: ReferenceSourceInfo {
                source_channel: request.source_channel,
                source_span: request.source_span,
                source_text: request.source_text.clone(),
                parsed_qualifier: None,
                address_fidelity: Some(request.source_text.clone()),
            },
            profile_payload: ProfilePayload::textual("atom", &request.source_text),
            normal_form_key: ReferenceNormalFormKey(format!(
                "cell:{}",
                request.source_text.to_ascii_uppercase()
            )),
            render_hint: Some(request.source_text.clone()),
            validity: ReferenceValidity::ValidNow,
        })
    }

    fn bind_name(&self, _request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::Unsupported
    }
}

fn bind(formula: &str, profile: &dyn ReferenceBindProfile) -> oxfml_core::BoundFormula {
    let source = FormulaSourceRecord::new("w078-bind", 1, formula.to_string())
        .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let request = BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            formula_token: FormulaToken("w078-token".to_string()),
            ..BindContext::default()
        },
        reference_bind_profile: Some(profile),
    };
    bind_formula(request).bound_formula
}

fn span_atom(bound: &BoundExpr) -> Option<&NormalizedReference> {
    match bound {
        BoundExpr::Reference(ReferenceExpr::Atom(atom)) => Some(atom),
        _ => None,
    }
}

// ---- bind gating: flag on / off ---------------------------------------------

#[test]
fn flag_on_binds_distinct_sheet_span_3d_reference() {
    let profile = SpanProfile {
        capabilities: ReferenceSyntaxCapabilities::all(), // sheet_span_3d on
    };
    let bound = bind("=Sheet1:Sheet3!A1", &profile);
    let atom = span_atom(&bound.root).expect("3D span should bind as an atom");
    let NormalizedReference::SheetSpan3D(span) = atom else {
        panic!("expected a distinct SheetSpan3D reference, got {atom:?}");
    };
    assert_eq!(span.start_sheet, "Sheet1");
    assert_eq!(span.end_sheet, "Sheet3");
    assert_eq!(span.target, "A1");
    // Distinct from a range and from any multi-area/name shape.
    assert!(!matches!(
        &bound.root,
        BoundExpr::Reference(ReferenceExpr::Range { .. })
    ));
    assert!(bound.diagnostics.is_empty(), "{:?}", bound.diagnostics);
}

#[test]
fn flag_on_span_normal_form_is_stable_and_distinct_from_range() {
    let profile = SpanProfile {
        capabilities: ReferenceSyntaxCapabilities::all(),
    };
    let a = bind("=Sheet1:Sheet3!A1", &profile);
    let b = bind("=Sheet1:Sheet3!A1", &profile);
    let key_a = span_atom(&a.root).unwrap().to_string();
    let key_b = span_atom(&b.root).unwrap().to_string();
    assert_eq!(key_a, key_b, "span normal form must be stable");
    assert!(
        key_a.starts_with("sheet-span-3d:"),
        "span normal form must be a distinct 3D key, got {key_a}",
    );
    // A same-sheet range over the same target must NOT bind as the 3D span:
    // it is a distinct `ReferenceExpr::Range`, never a `SheetSpan3D` atom.
    let range = bind("=A1:A3", &profile);
    assert!(
        matches!(
            &range.root,
            BoundExpr::Reference(ReferenceExpr::Range { .. })
        ),
        "A1:A3 must bind as a range, distinct from the 3D span; got {:?}",
        range.root
    );
}

#[test]
fn flag_off_rejects_span_with_typed_ref_error() {
    let profile = SpanProfile {
        capabilities: ReferenceSyntaxCapabilities {
            sheet_span_3d_references: false,
            ..ReferenceSyntaxCapabilities::all()
        },
    };
    let bound = bind("=Sheet1:Sheet3!A1", &profile);
    // Typed rejection: a #REF! error atom, never a silently-wrong range or a
    // bound 3D span.
    match span_atom(&bound.root) {
        Some(NormalizedReference::Error(err)) => {
            assert_eq!(err.error_class, "#REF!");
        }
        other => panic!("expected a typed #REF! rejection, got {other:?}"),
    }
    assert!(
        bound
            .diagnostics
            .iter()
            .any(|d| d.message.contains("3D sheet-span")),
        "expected a 3D sheet-span capability diagnostic, got {:?}",
        bound.diagnostics
    );
}

#[test]
fn no_profile_default_admits_the_span() {
    // With no profile the binder admits every shape (`all()`), so the 3D span
    // binds rather than being rejected. This is the default-preserving stance.
    let source = FormulaSourceRecord::new("w078-noprofile", 1, "=Sheet1:Sheet3!A1".to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bound = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext::default(),
        reference_bind_profile: None,
    })
    .bound_formula;
    assert!(matches!(
        span_atom(&bound.root),
        Some(NormalizedReference::SheetSpan3D(_))
    ));
}
