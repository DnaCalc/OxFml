//! W062 R3.8 — binder consults `ReferenceSyntaxCapabilities` as routing gates.
//!
//! These tests prove the D2 §3 contract: the shared grammar always parses every
//! reference shape, but the *binder* consults the active profile's
//! `syntax_capabilities()` before routing a parsed shape to a `bind_*` method.
//! A capability flag that is `false` suppresses that shape's routing and sends
//! it to the profile-symbolic typed rejection path instead of silently binding.
//!
//! Default-preservation is proven by the untouched existing suite (the trait
//! default `syntax_capabilities()` admits every shape); here we drive a fake
//! profile with individual flags flipped OFF and assert the shape is no longer
//! routed, then flip the flag ON and assert it binds exactly as before.

use oxfml_core::source::FormulaChannelKind;
use oxfml_core::syntax::token::TextSpan;
use oxfml_core::{
    BindContext, BindRequest, BoundExpr, FormulaSourceRecord, NormalizedReference, ProfilePayload,
    ProfileReferenceRecord, ProfileVersion, ReferenceAtomBindRequest, ReferenceAtomBindResult,
    ReferenceBindProfile, ReferenceExpr, ReferenceNameBindRequest, ReferenceNormalFormKey,
    ReferencePolicy, ReferenceSelectorBindRequest, ReferenceSelectorSyntax, ReferenceSourceInfo,
    ReferenceStructuredBindRequest, ReferenceSyntaxCapabilities, ReferenceValidity, bind_formula,
};
use oxfml_core::{ParseRequest, parse_formula};
use oxfml_core::{project_red_view, source::FormulaToken};

/// A symbolic profile whose syntax capabilities are configurable per test. It
/// binds `A1`-style address atoms, `Table[Column]` structured references, and
/// `Base.@CHILDREN` / `Base.[member]` host selectors — one bind method per
/// gated shape — so a single flag flip isolates the routing under test.
struct GatedProfile {
    capabilities: ReferenceSyntaxCapabilities,
}

impl GatedProfile {
    fn new(capabilities: ReferenceSyntaxCapabilities) -> Self {
        Self { capabilities }
    }

    fn record(
        &self,
        payload_kind: &str,
        source_text: &str,
        normal_form_key: &str,
        source_channel: FormulaChannelKind,
        source_span: TextSpan,
    ) -> ProfileReferenceRecord {
        ProfileReferenceRecord {
            profile_id: self.profile_id().to_string(),
            profile_version: ProfileVersion::v1(),
            source_info: ReferenceSourceInfo {
                source_channel,
                source_span,
                source_text: source_text.to_string(),
                parsed_qualifier: None,
                address_fidelity: Some(source_text.to_string()),
            },
            profile_payload: ProfilePayload::textual(payload_kind, source_text),
            normal_form_key: ReferenceNormalFormKey(normal_form_key.to_string()),
            render_hint: Some(source_text.to_string()),
            validity: ReferenceValidity::ValidNow,
        }
    }
}

impl ReferenceBindProfile for GatedProfile {
    fn profile_id(&self) -> &str {
        "gated.profile.v1"
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn syntax_capabilities(&self) -> ReferenceSyntaxCapabilities {
        self.capabilities
    }

    fn selector_syntax(&self) -> Vec<ReferenceSelectorSyntax> {
        vec![ReferenceSelectorSyntax::collection("CHILDREN", "children")]
    }

    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        if !looks_like_cell_ref(&request.source_text) {
            return ReferenceAtomBindResult::LegacyCompatibility;
        }
        ReferenceAtomBindResult::Bound(self.record(
            "atom",
            &request.source_text,
            &format!("cell:{}", request.source_text.to_ascii_uppercase()),
            request.source_channel,
            request.source_span,
        ))
    }

    fn bind_name(&self, request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult {
        if request.source_text.eq_ignore_ascii_case("Base") {
            return ReferenceAtomBindResult::Bound(self.record(
                "name",
                &request.source_text,
                "name:BASE",
                request.source_channel,
                request.source_span,
            ));
        }
        ReferenceAtomBindResult::Unsupported
    }

    fn bind_structured_reference(
        &self,
        request: &ReferenceStructuredBindRequest,
    ) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::Bound(self.record(
            "structured",
            &request.source_text,
            &format!("structured:{}", request.source_text),
            request.source_channel,
            request.source_span,
        ))
    }

    fn bind_selector(&self, request: &ReferenceSelectorBindRequest) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::Bound(self.record(
            "selector",
            &request.source_text,
            &format!("selector:{}", request.selector_family),
            request.source_channel,
            request.source_span,
        ))
    }
}

fn looks_like_cell_ref(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    let mut saw_letter = false;
    while matches!(chars.peek(), Some(c) if c.is_ascii_alphabetic()) {
        saw_letter = true;
        chars.next();
    }
    let mut saw_digit = false;
    while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
        saw_digit = true;
        chars.next();
    }
    saw_letter && saw_digit && chars.next().is_none()
}

fn bind(
    formula: &str,
    channel: FormulaChannelKind,
    profile: &dyn ReferenceBindProfile,
) -> oxfml_core::BoundFormula {
    let source = FormulaSourceRecord::new("gating-test", 1, formula.to_string())
        .with_formula_channel_kind(channel);
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let request = BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            formula_token: FormulaToken("gating-token".to_string()),
            ..BindContext::default()
        },
        reference_bind_profile: Some(profile),
    };
    bind_formula(request).bound_formula
}

fn profile_record(bound: &BoundExpr) -> Option<&ProfileReferenceRecord> {
    match bound {
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record))) => {
            Some(record)
        }
        _ => None,
    }
}

fn is_ref_error(bound: &BoundExpr) -> bool {
    matches!(
        bound,
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(_)))
    )
}

// ---- structured_references (acceptance criterion) ---------------------------

#[test]
fn structured_reference_flag_on_routes_to_bind_structured_reference() {
    let profile = GatedProfile::new(ReferenceSyntaxCapabilities::all());
    let bound = bind("=Table1[Amount]", FormulaChannelKind::WorksheetA1, &profile);
    let record = profile_record(&bound.root).expect("structured reference should bind");
    assert_eq!(record.profile_payload.payload_kind, "structured");
    assert!(bound.diagnostics.is_empty(), "{:?}", bound.diagnostics);
}

#[test]
fn structured_reference_flag_off_receives_no_structured_routing() {
    let capabilities = ReferenceSyntaxCapabilities {
        structured_references: false,
        ..ReferenceSyntaxCapabilities::all()
    };
    let profile = GatedProfile::new(capabilities);
    let bound = bind("=Table1[Amount]", FormulaChannelKind::WorksheetA1, &profile);
    // With no structured routing, a symbolic profile lands on the typed
    // unresolved-identifier error path instead of a structured record.
    assert!(
        profile_record(&bound.root).is_none(),
        "structured routing must be suppressed when the flag is off",
    );
    assert!(is_ref_error(&bound.root));
    assert!(!bound.diagnostics.is_empty());
}

// ---- spill_references -------------------------------------------------------

#[test]
fn spill_flag_on_binds_spill_reference() {
    let profile = GatedProfile::new(ReferenceSyntaxCapabilities::all());
    let bound = bind("=A1#", FormulaChannelKind::WorksheetA1, &profile);
    assert!(
        matches!(
            &bound.root,
            BoundExpr::Reference(ReferenceExpr::Spill { .. })
        ),
        "expected spill node, got {:?}",
        bound.root
    );
    assert!(bound.diagnostics.is_empty(), "{:?}", bound.diagnostics);
}

#[test]
fn spill_flag_off_suppresses_spill_routing_with_typed_diagnostic() {
    let capabilities = ReferenceSyntaxCapabilities {
        spill_references: false,
        ..ReferenceSyntaxCapabilities::all()
    };
    let profile = GatedProfile::new(capabilities);
    let bound = bind("=A1#", FormulaChannelKind::WorksheetA1, &profile);
    assert!(
        !matches!(
            &bound.root,
            BoundExpr::Reference(ReferenceExpr::Spill { .. })
        ),
        "spill routing must be suppressed when the flag is off",
    );
    assert!(
        bound
            .diagnostics
            .iter()
            .any(|d| d.message.contains("spill reference")),
        "expected a spill capability diagnostic, got {:?}",
        bound.diagnostics
    );
}

// ---- host_references --------------------------------------------------------

#[test]
fn host_reference_flag_on_routes_selector() {
    let profile = GatedProfile::new(ReferenceSyntaxCapabilities::all());
    let bound = bind(
        "=SUM(Base.@CHILDREN)",
        FormulaChannelKind::WorksheetA1,
        &profile,
    );
    let BoundExpr::FunctionCall { args, .. } = &bound.root else {
        panic!("expected SUM call, got {:?}", bound.root);
    };
    let record = profile_record(&args[0]).expect("selector should bind");
    assert_eq!(record.profile_payload.payload_kind, "selector");
}

#[test]
fn host_reference_flag_off_receives_no_selector_routing() {
    let capabilities = ReferenceSyntaxCapabilities {
        host_references: false,
        ..ReferenceSyntaxCapabilities::all()
    };
    let profile = GatedProfile::new(capabilities);
    let bound = bind(
        "=SUM(Base.@CHILDREN)",
        FormulaChannelKind::WorksheetA1,
        &profile,
    );
    let BoundExpr::FunctionCall { args, .. } = &bound.root else {
        panic!("expected SUM call, got {:?}", bound.root);
    };
    assert!(
        profile_record(&args[0]).is_none(),
        "selector routing must be suppressed when host_references is off",
    );
    assert!(is_ref_error(&args[0]));
}

// ---- r1c1_references / a1_references (channel gate) -------------------------

#[test]
fn r1c1_flag_on_binds_atom_on_r1c1_channel() {
    let profile = GatedProfile::new(ReferenceSyntaxCapabilities::all());
    let bound = bind("=RC1", FormulaChannelKind::WorksheetR1C1, &profile);
    let record = profile_record(&bound.root).expect("R1C1 atom should bind when admitted");
    assert_eq!(record.profile_payload.payload_kind, "atom");
}

#[test]
fn r1c1_flag_off_rejects_r1c1_channel_bind() {
    let capabilities = ReferenceSyntaxCapabilities {
        r1c1_references: false,
        ..ReferenceSyntaxCapabilities::all()
    };
    let profile = GatedProfile::new(capabilities);
    let bound = bind("=RC1", FormulaChannelKind::WorksheetR1C1, &profile);
    assert!(
        profile_record(&bound.root).is_none(),
        "R1C1 atom must not bind when r1c1_references is off",
    );
    assert!(is_ref_error(&bound.root));
    assert!(
        bound
            .diagnostics
            .iter()
            .any(|d| d.message.contains("R1C1")),
        "expected an R1C1 capability diagnostic, got {:?}",
        bound.diagnostics
    );
}

#[test]
fn r1c1_flag_off_does_not_affect_a1_channel_atoms() {
    // r1c1 off must not gate A1-channel atoms — the gate is channel-scoped.
    let capabilities = ReferenceSyntaxCapabilities {
        r1c1_references: false,
        ..ReferenceSyntaxCapabilities::all()
    };
    let profile = GatedProfile::new(capabilities);
    let bound = bind("=A1", FormulaChannelKind::WorksheetA1, &profile);
    let record = profile_record(&bound.root).expect("A1 atom still binds");
    assert_eq!(record.profile_payload.payload_kind, "atom");
}

#[test]
fn a1_flag_off_rejects_a1_channel_bind() {
    let capabilities = ReferenceSyntaxCapabilities {
        a1_references: false,
        ..ReferenceSyntaxCapabilities::all()
    };
    let profile = GatedProfile::new(capabilities);
    let bound = bind("=A1", FormulaChannelKind::WorksheetA1, &profile);
    assert!(
        profile_record(&bound.root).is_none(),
        "A1 atom must not bind when a1_references is off",
    );
    assert!(is_ref_error(&bound.root));
    assert!(
        bound.diagnostics.iter().any(|d| d.message.contains("A1")),
        "expected an A1 capability diagnostic, got {:?}",
        bound.diagnostics
    );
}

// ---- default preservation ---------------------------------------------------

#[test]
fn all_capabilities_is_the_trait_default() {
    // The trait default admits every shape; a profile that does not override
    // syntax_capabilities() must equal all(). This is the default-preserving
    // guardrail: the existing suite runs entirely on this default.
    struct DefaultProfile;
    impl ReferenceBindProfile for DefaultProfile {
        fn profile_id(&self) -> &str {
            "default.profile"
        }
    }
    assert_eq!(
        DefaultProfile.syntax_capabilities(),
        ReferenceSyntaxCapabilities::all()
    );
}
