use std::cell::RefCell;

use oxfml_core::TypedContextQueryBundle;
use oxfml_core::consumer::runtime::{
    RuntimeAuthoredInputResult, RuntimeEnvironment, RuntimeFormulaRequest,
};
use oxfml_core::{
    BindContext, BindRequest, BoundExpr, BoundFormula, CompileSemanticPlanRequest,
    FormulaSourceRecord, InstantiatedReference, NormalizedReference, PlacedFormulaIdentity,
    ProfilePayload, ProfileReferenceRecord, ProfileVersion, ReferenceAtomBindRequest,
    ReferenceAtomBindResult, ReferenceBindProfile, ReferenceDependencyEnvelope, ReferenceExpr,
    ReferenceFingerprintPolicy, ReferenceInstantiationPurpose, ReferenceInstantiationRequest,
    ReferenceNormalFormKey, ReferencePolicy, ReferenceProfileFingerprint,
    ReferenceProfileFingerprintContext, ReferenceRangeBindRequest, ReferenceRangeBindResult,
    ReferenceSourceInfo, ReferenceTransformKind, ReferenceTransformOutcome,
    ReferenceTransformRequest, ReferenceTransformResult, ReferenceValidity,
    RuntimeHostFormulaContext, StructureContextVersion, bind_formula, bind_formula_incremental,
    compile_semantic_plan,
};
use oxfml_core::{EvaluationContext, evaluate_formula};
use oxfml_core::{ParseRequest, parse_formula};
use oxfml_core::{project_red_view, source::FormulaToken};
use oxfunc_core::resolver::{
    ReferenceComposeOperation, ReferenceComposeRequest, ReferenceDereferenceRequest,
    ReferenceResolutionError, ReferenceSystemError, ReferenceSystemProvider,
};
use oxfunc_core::value::{
    CalcValue, ReferenceIdentity, ReferenceKind, ReferenceLike, ReferenceSystemId,
};

struct FakeSymbolicProfile;

impl ReferenceBindProfile for FakeSymbolicProfile {
    fn profile_id(&self) -> &str {
        "fake.symbolic.v1"
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn fingerprint_policy(&self) -> ReferenceFingerprintPolicy {
        ReferenceFingerprintPolicy::ExcludeCallerAnchorForTemplate
    }

    fn fingerprint(
        &self,
        context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceProfileFingerprint {
        ReferenceProfileFingerprint(format!(
            "{}:{}:{}:{}",
            self.profile_id(),
            self.profile_version().0,
            context.workbook_id,
            context.sheet_id
        ))
    }

    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        if !looks_like_fake_cell_ref(&request.source_text) {
            return ReferenceAtomBindResult::LegacyCompatibility;
        }

        let normal_text = request.source_text.to_ascii_uppercase();
        ReferenceAtomBindResult::Bound(ProfileReferenceRecord {
            profile_id: self.profile_id().to_string(),
            profile_version: ProfileVersion::v1(),
            source_info: ReferenceSourceInfo {
                source_channel: request.source_channel,
                source_span: request.source_span,
                source_text: request.source_text.clone(),
                parsed_qualifier: request.parsed_qualifier.clone(),
                address_fidelity: Some(request.source_text.clone()),
            },
            profile_payload: ProfilePayload::textual(
                "fake-cell-ref",
                format!("sheet={};source={}", request.sheet_id, request.source_text),
            ),
            normal_form_key: ReferenceNormalFormKey(format!("fake-cell:{normal_text}")),
            render_hint: Some(request.source_text.clone()),
            validity: ReferenceValidity::ValidAfterInstantiation,
        })
    }

    fn bind_range(&self, request: &ReferenceRangeBindRequest) -> ReferenceRangeBindResult {
        if !request.left.target_text.eq_ignore_ascii_case("alpha")
            || !request.right.target_text.eq_ignore_ascii_case("beta")
        {
            return ReferenceRangeBindResult::LegacyCompatibility;
        }

        ReferenceRangeBindResult::Bound(ProfileReferenceRecord {
            profile_id: self.profile_id().to_string(),
            profile_version: ProfileVersion::v1(),
            source_info: ReferenceSourceInfo {
                source_channel: request.source_channel,
                source_span: request.source_span,
                source_text: request.source_text.clone(),
                parsed_qualifier: None,
                address_fidelity: Some(request.source_text.clone()),
            },
            profile_payload: ProfilePayload::textual(
                "fake-range-ref",
                format!(
                    "left={};right={};sheet={}",
                    request.left.target_text, request.right.target_text, request.sheet_id
                ),
            ),
            normal_form_key: ReferenceNormalFormKey(format!(
                "fake-range:{}:{}",
                request.left.target_text.to_ascii_uppercase(),
                request.right.target_text.to_ascii_uppercase()
            )),
            render_hint: Some(request.source_text.clone()),
            validity: ReferenceValidity::ValidAfterInstantiation,
        })
    }

    /// Exercise the transform seam at the OxFml trait level. The outcome is driven by the
    /// transform payload's `data` so a test can pin `Shifted` vs `PartiallyInvalid` vs
    /// `FullyInvalid` deterministically. A structural edit that keeps the reference valid
    /// shifts it (and rewrites the normal-form key); an edit that clips part of the target
    /// is partially invalid; an edit that deletes the whole target is fully invalid.
    fn transform_reference(&self, request: &ReferenceTransformRequest) -> ReferenceTransformResult {
        let directive = request
            .payload
            .as_ref()
            .map(|payload| payload.data.as_str())
            .unwrap_or("");

        match (&request.transform_kind, directive) {
            (ReferenceTransformKind::StructuralEdit, "shift") => {
                let mut shifted = request.reference.clone();
                shifted.normal_form_key =
                    ReferenceNormalFormKey(format!("{}+shift", shifted.normal_form_key.0));
                ReferenceTransformResult {
                    outcome: ReferenceTransformOutcome::Shifted,
                    reference: Some(shifted),
                    diagnostics: Vec::new(),
                }
            }
            (ReferenceTransformKind::StructuralEdit, "clip") => {
                let mut clipped = request.reference.clone();
                clipped.validity = ReferenceValidity::InvalidForCurrentPlacement;
                ReferenceTransformResult {
                    outcome: ReferenceTransformOutcome::PartiallyInvalid,
                    reference: Some(clipped),
                    diagnostics: vec!["partial target overlap after structural edit".to_string()],
                }
            }
            (ReferenceTransformKind::StructuralEdit, "delete") => ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::FullyInvalid,
                reference: None,
                diagnostics: vec!["target fully deleted by structural edit".to_string()],
            },
            _ => ReferenceTransformResult {
                outcome: ReferenceTransformOutcome::Unchanged,
                reference: Some(request.reference.clone()),
                diagnostics: Vec::new(),
            },
        }
    }
}

#[test]
fn fake_profile_binds_atom_to_profile_symbolic_reference() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-atom", "=A1+1", 8, 4, Some(&profile));

    assert_eq!(bound.normalized_references.len(), 1);
    match &bound.normalized_references[0] {
        NormalizedReference::ProfileSymbolic(record) => {
            assert_eq!(record.profile_id, "fake.symbolic.v1");
            assert_eq!(record.normal_form_key.0, "fake-cell:A1");
            assert_eq!(record.source_info.address_fidelity.as_deref(), Some("A1"));
        }
        other => panic!("expected profile symbolic reference, got {other:?}"),
    }
}

#[test]
fn symbolic_profile_keeps_range_as_expression_composition() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-range", "=A1:B2", 8, 4, Some(&profile));

    assert_eq!(bound.normalized_references.len(), 2);
    match &bound.root {
        BoundExpr::Reference(ReferenceExpr::Range { start, end }) => {
            assert_profile_symbolic_atom(start, "fake-cell:A1");
            assert_profile_symbolic_atom(end, "fake-cell:B2");
        }
        other => panic!("expected profile symbolic range expression, got {other:?}"),
    }
}

#[test]
fn symbolic_profile_can_bind_range_fragments_without_oxfml_grammar() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for(
        "profile-range-fragment",
        "=alpha:beta",
        8,
        4,
        Some(&profile),
    );

    assert_profile_symbolic_normalized_reference(&bound, "fake-range:ALPHA:BETA");
    match &bound.root {
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record))) => {
            assert_eq!(record.source_info.source_text, "alpha:beta");
            assert_eq!(
                record.source_info.address_fidelity.as_deref(),
                Some("alpha:beta")
            );
        }
        other => panic!("expected profile-bound range atom, got {other:?}"),
    }
}

#[test]
fn symbolic_profile_does_not_capture_external_workbook_references() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for(
        "profile-external",
        "=[Book.xlsx]Sheet2!A1",
        8,
        4,
        Some(&profile),
    );

    assert_eq!(bound.normalized_references.len(), 1);
    match &bound.normalized_references[0] {
        NormalizedReference::External(external) => {
            assert_eq!(external.target_summary, "[Book.xlsx]Sheet2!A1");
        }
        other => panic!("expected external reference to bypass profile, got {other:?}"),
    }
}

#[test]
fn symbolic_profile_template_identity_is_caller_independent() {
    let profile = FakeSymbolicProfile;
    let first = bind_formula_for("profile-shared-a", "=A1+1", 2, 2, Some(&profile));
    let second = bind_formula_for("profile-shared-b", "=A1+1", 19, 7, Some(&profile));

    assert_eq!(
        first.formula_template_identity,
        second.formula_template_identity
    );
    assert_ne!(
        first.placed_formula_identity,
        second.placed_formula_identity
    );
    assert_ne!(
        first.formula_source_identity,
        second.formula_source_identity
    );
}

#[test]
fn default_binding_keeps_caller_anchor_in_bind_fingerprint() {
    let first = bind_formula_for("legacy-a", "=A1+1", 2, 2, None);
    let second = bind_formula_for("legacy-a", "=A1+1", 19, 7, None);

    assert_ne!(
        first.bind_context_fingerprint,
        second.bind_context_fingerprint
    );
    assert_ne!(
        first.placed_formula_identity,
        second.placed_formula_identity
    );
}

#[test]
fn incremental_reuse_refreshes_placed_identity_for_new_anchor() {
    let profile = FakeSymbolicProfile;
    let source = FormulaSourceRecord::new("profile-incremental", 1, "=A1+1".to_string());
    let first = bind_formula_request(source.clone(), 2, 2, Some(&profile), None);
    let previous_placed_identity = first.bound_formula.placed_formula_identity.clone();

    let second = bind_formula_request(source, 19, 7, Some(&profile), Some(&first.bound_formula));

    assert!(second.reused_bound_formula);
    assert_eq!(
        first.bound_formula.formula_template_identity,
        second.bound_formula.formula_template_identity
    );
    assert_ne!(
        previous_placed_identity,
        second.bound_formula.placed_formula_identity
    );
    assert_ne!(
        PlacedFormulaIdentity { key: String::new() },
        second.bound_formula.placed_formula_identity
    );
}

#[test]
fn profile_symbolic_reference_evaluation_routes_through_reference_provider() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-provider", "=A1", 2, 2, Some(&profile));
    let semantic_plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bound.clone(),
        oxfunc_catalog_identity: "oxfunc:test".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot: None,
    })
    .semantic_plan;
    let provider = RecordingProvider::default();
    let mut context = EvaluationContext::new(&bound, &semantic_plan);
    context.reference_system_provider = Some(&provider);

    let output = evaluate_formula(context).expect("profile reference evaluation should run");

    assert_eq!(output.oxfunc_value, CalcValue::number(42.0));
    let seen = provider.seen.borrow();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].system,
        ReferenceSystemId("fake.symbolic.v1".to_string())
    );
    match &seen[0].identity {
        ReferenceIdentity::Opaque(handle) => {
            assert_eq!(
                String::from_utf8(handle.id.bytes.clone()).expect("normal key bytes"),
                "fake-cell:A1"
            );
        }
        other => panic!("expected opaque profile reference identity, got {other:?}"),
    }
}

#[test]
fn runtime_environment_supplies_reference_profile_to_authored_bind() {
    let profile = FakeSymbolicProfile;
    let runtime = RuntimeEnvironment::new().with_reference_bind_profile(&profile);

    let result = runtime.interpret_authored_input(FormulaSourceRecord::new(
        "runtime-profile-authored",
        1,
        "=A1".to_string(),
    ));

    let RuntimeAuthoredInputResult::Formula(bound) = result else {
        panic!("expected authored formula bind result, got {result:?}");
    };
    assert_profile_symbolic_normalized_reference(&bound, "fake-cell:A1");
}

#[test]
fn runtime_environment_supplies_reference_profile_through_execution() {
    let profile = FakeSymbolicProfile;
    let provider = RecordingProvider::default();
    let runtime = RuntimeEnvironment::new().with_reference_bind_profile(&profile);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime-profile-exec", 1, "=A1".to_string()),
        TypedContextQueryBundle::default().with_reference_system_provider(Some(&provider)),
    );

    let result = runtime
        .execute(request)
        .expect("runtime profile execution should succeed");

    assert_eq!(result.published_worksheet_value, CalcValue::number(42.0));
    let seen = provider.seen.borrow();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].system,
        ReferenceSystemId("fake.symbolic.v1".to_string())
    );
}

#[test]
fn runtime_environment_forwards_profile_range_composition_to_reference_provider() {
    let profile = FakeSymbolicProfile;
    let provider = ComposingProvider::default();
    let runtime = RuntimeEnvironment::new().with_reference_bind_profile(&profile);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new(
            "runtime-profile-range-compose",
            1,
            "=SUM(A1:B1)".to_string(),
        ),
        TypedContextQueryBundle::default().with_reference_system_provider(Some(&provider)),
    );

    let result = runtime
        .execute(request)
        .expect("runtime profile range composition should execute");

    assert_eq!(result.published_worksheet_value, CalcValue::number(5.0));
    let composed = provider.composed.borrow();
    assert!(!composed.is_empty());
    for request in composed.iter() {
        assert_eq!(request.operation, ReferenceComposeOperation::Range);
        assert_eq!(
            request.lhs.system,
            ReferenceSystemId("fake.symbolic.v1".to_string())
        );
        assert_eq!(
            request.rhs.system,
            ReferenceSystemId("fake.symbolic.v1".to_string())
        );
    }
    assert!(
        provider
            .dereferenced
            .borrow()
            .iter()
            .any(|reference| reference.target() == "fake-range:A1:B1")
    );
}

#[test]
fn default_profile_instantiation_returns_reference_like_and_static_dependency() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-instantiation", "=A1", 2, 2, Some(&profile));
    let record = profile_symbolic_record(&bound).clone();
    let host_context = RuntimeHostFormulaContext {
        profile_id: profile.profile_id().to_string(),
        context_payload: None,
    };

    let runtime = profile.instantiate_reference(&ReferenceInstantiationRequest {
        bound_reference: record.clone(),
        runtime_host_formula_context: host_context.clone(),
        purpose: ReferenceInstantiationPurpose::RuntimeEvaluation,
    });
    assert_eq!(
        runtime,
        InstantiatedReference::ReferenceLike {
            profile_id: "fake.symbolic.v1".to_string(),
            identity_key: "fake-cell:A1".to_string(),
        }
    );

    let dependency = profile.instantiate_reference(&ReferenceInstantiationRequest {
        bound_reference: record,
        runtime_host_formula_context: host_context,
        purpose: ReferenceInstantiationPurpose::StaticDependencyExtraction,
    });
    assert_eq!(
        dependency,
        InstantiatedReference::StaticDependencyEnvelope(ReferenceDependencyEnvelope::Static {
            profile_id: "fake.symbolic.v1".to_string(),
            dependency_key: "fake-cell:A1".to_string(),
        })
    );
}

#[test]
fn default_profile_instantiation_classifies_dynamic_invalid_and_mismatched_context() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-instantiation-classes", "=A1", 2, 2, Some(&profile));
    let mut record = profile_symbolic_record(&bound).clone();
    let host_context = RuntimeHostFormulaContext {
        profile_id: profile.profile_id().to_string(),
        context_payload: None,
    };

    record.validity = ReferenceValidity::DynamicOrHostSensitive;
    let dynamic = profile.instantiate_reference(&ReferenceInstantiationRequest {
        bound_reference: record.clone(),
        runtime_host_formula_context: host_context.clone(),
        purpose: ReferenceInstantiationPurpose::RuntimeEvaluation,
    });
    assert_eq!(
        dynamic,
        InstantiatedReference::DynamicDependencyRequest(ReferenceDependencyEnvelope::Dynamic {
            profile_id: "fake.symbolic.v1".to_string(),
            request_key: "fake-cell:A1".to_string(),
        })
    );

    record.validity = ReferenceValidity::InvalidForCurrentPlacement;
    let invalid = profile.instantiate_reference(&ReferenceInstantiationRequest {
        bound_reference: record.clone(),
        runtime_host_formula_context: host_context,
        purpose: ReferenceInstantiationPurpose::RuntimeEvaluation,
    });
    assert_eq!(invalid, InstantiatedReference::RefError);

    record.validity = ReferenceValidity::ValidAfterInstantiation;
    let mismatched = profile.instantiate_reference(&ReferenceInstantiationRequest {
        bound_reference: record,
        runtime_host_formula_context: RuntimeHostFormulaContext {
            profile_id: "other.profile.v1".to_string(),
            context_payload: None,
        },
        purpose: ReferenceInstantiationPurpose::RuntimeEvaluation,
    });
    assert!(matches!(
        mismatched,
        InstantiatedReference::Unsupported { .. }
    ));
}

#[test]
fn profile_transform_reference_shifts_valid_structural_edit() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-transform-shift", "=A1", 2, 2, Some(&profile));
    let record = profile_symbolic_record(&bound).clone();

    let result = profile.transform_reference(&ReferenceTransformRequest {
        reference: record,
        transform_kind: ReferenceTransformKind::StructuralEdit,
        payload: Some(ProfilePayload::textual("edit", "shift")),
    });

    assert_eq!(result.outcome, ReferenceTransformOutcome::Shifted);
    let shifted = result
        .reference
        .expect("a shifted transform must yield a rewritten reference");
    assert_eq!(shifted.normal_form_key.0, "fake-cell:A1+shift");
    assert!(result.diagnostics.is_empty());
}

#[test]
fn profile_transform_reference_reports_partially_invalid_clip() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-transform-clip", "=A1", 2, 2, Some(&profile));
    let record = profile_symbolic_record(&bound).clone();

    let result = profile.transform_reference(&ReferenceTransformRequest {
        reference: record,
        transform_kind: ReferenceTransformKind::StructuralEdit,
        payload: Some(ProfilePayload::textual("edit", "clip")),
    });

    assert_eq!(result.outcome, ReferenceTransformOutcome::PartiallyInvalid);
    let clipped = result
        .reference
        .expect("a partially-invalid transform still yields the surviving reference");
    assert_eq!(
        clipped.validity,
        ReferenceValidity::InvalidForCurrentPlacement
    );
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn profile_transform_reference_reports_fully_invalid_delete() {
    let profile = FakeSymbolicProfile;
    let bound = bind_formula_for("profile-transform-delete", "=A1", 2, 2, Some(&profile));
    let record = profile_symbolic_record(&bound).clone();

    let result = profile.transform_reference(&ReferenceTransformRequest {
        reference: record,
        transform_kind: ReferenceTransformKind::StructuralEdit,
        payload: Some(ProfilePayload::textual("edit", "delete")),
    });

    assert_eq!(result.outcome, ReferenceTransformOutcome::FullyInvalid);
    assert!(
        result.reference.is_none(),
        "a fully-invalid transform drops the reference"
    );
    assert!(!result.diagnostics.is_empty());
}

/// Regression for HANDOFF-DNATREECALC-001 item 8: a non-default host-reference syntax
/// profile must NOT pay a plan-reuse penalty across placements. Plan reuse in
/// `bind_formula_incremental` keys off `(formula_stable_id, green_tree_key,
/// bind_context_fingerprint)`. Under the default `IncludeCallerAnchor` policy the
/// fingerprint folds in the caller anchor, so the same formula at a different cell
/// re-binds from scratch (the penalty). Under a profile whose `fingerprint_policy` is
/// `ExcludeCallerAnchorForTemplate`, the caller anchor is dropped from the fingerprint,
/// so a placement at a new anchor REUSES the bound artifact (and its downstream compiled
/// plan) while refreshing only the placed identity.
#[test]
fn non_default_syntax_profile_reuses_plan_across_placements() {
    let profile = FakeSymbolicProfile;
    let source = FormulaSourceRecord::new("profile-reuse-penalty", 1, "=A1+1".to_string());

    let first = bind_formula_request(source.clone(), 2, 2, Some(&profile), None);
    let previous_placed = first.bound_formula.placed_formula_identity.clone();

    // Same formula, different caller cell, under the non-default syntax profile.
    let second = bind_formula_request(source, 40, 11, Some(&profile), Some(&first.bound_formula));

    // No penalty: the bound artifact (and thus the compiled plan keyed off its template
    // identity) is reused rather than re-bound.
    assert!(
        second.reused_bound_formula,
        "non-default syntax profile must reuse the bound artifact across placements"
    );
    assert_eq!(
        first.bound_formula.formula_template_identity,
        second.bound_formula.formula_template_identity,
        "template identity (the plan cache key input) must be shared across placements"
    );
    assert_eq!(
        first.bound_formula.bind_context_fingerprint, second.bound_formula.bind_context_fingerprint,
        "caller anchor must be excluded from the bind fingerprint under the profile"
    );
    // Placement still refreshes so per-cell identity stays distinct.
    assert_ne!(
        previous_placed, second.bound_formula.placed_formula_identity,
        "placed identity must still refresh per caller cell"
    );
}

/// Companion to the regression above: prove the penalty is real under the DEFAULT path,
/// so the profile's reuse is a genuine improvement and not a vacuous assertion.
#[test]
fn default_syntax_pays_plan_reuse_penalty_across_placements() {
    let source = FormulaSourceRecord::new("default-reuse-penalty", 1, "=A1+1".to_string());

    let first = bind_formula_request(source.clone(), 2, 2, None, None);
    let second = bind_formula_request(source, 40, 11, None, Some(&first.bound_formula));

    assert!(
        !second.reused_bound_formula,
        "default path must NOT reuse across placements (caller anchor is in the fingerprint)"
    );
    assert_ne!(
        first.bound_formula.bind_context_fingerprint,
        second.bound_formula.bind_context_fingerprint
    );
}

fn bind_formula_for(
    stable_id: &str,
    formula: &str,
    caller_row: u32,
    caller_col: u32,
    profile: Option<&dyn ReferenceBindProfile>,
) -> BoundFormula {
    let source = FormulaSourceRecord::new(stable_id, 1, formula.to_string());
    bind_formula_request(source, caller_row, caller_col, profile, None).bound_formula
}

fn bind_formula_request(
    source: FormulaSourceRecord,
    caller_row: u32,
    caller_col: u32,
    profile: Option<&dyn ReferenceBindProfile>,
    previous: Option<&BoundFormula>,
) -> oxfml_core::IncrementalBindResult {
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let request = BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            caller_row,
            caller_col,
            formula_token: FormulaToken("shared-template-token".to_string()),
            structure_context_version: StructureContextVersion(
                "profile-test-struct-v1".to_string(),
            ),
            ..BindContext::default()
        },
        reference_bind_profile: profile,
    };

    match previous {
        Some(previous) => bind_formula_incremental(request, Some(previous)),
        None => {
            let bind = bind_formula(request);
            oxfml_core::IncrementalBindResult {
                bound_formula: bind.bound_formula,
                reused_bound_formula: false,
            }
        }
    }
}

#[derive(Default)]
struct RecordingProvider {
    seen: RefCell<Vec<ReferenceLike>>,
}

impl ReferenceSystemProvider for RecordingProvider {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        self.seen.borrow_mut().push(request.reference.clone());
        Ok(CalcValue::number(42.0))
    }
}

#[derive(Default)]
struct ComposingProvider {
    composed: RefCell<Vec<ReferenceComposeRequest>>,
    dereferenced: RefCell<Vec<ReferenceLike>>,
}

impl ReferenceSystemProvider for ComposingProvider {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        self.dereferenced
            .borrow_mut()
            .push(request.reference.clone());
        if request.reference.target() == "fake-range:A1:B1" {
            return Ok(CalcValue::number(5.0));
        }
        Err(ReferenceResolutionError::UnresolvedReference {
            target: request.reference.target().to_string(),
        })
    }

    fn compose_references(
        &self,
        request: &ReferenceComposeRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        self.composed.borrow_mut().push(request.clone());
        if request.operation == ReferenceComposeOperation::Range {
            return Ok(ReferenceLike::new(ReferenceKind::Area, "fake-range:A1:B1"));
        }
        Err(ReferenceSystemError::ProviderFailure {
            detail: "fake_profile_unexpected_compose".to_string(),
        })
    }
}

fn looks_like_fake_cell_ref(text: &str) -> bool {
    let split = text
        .find(|ch: char| ch.is_ascii_digit())
        .unwrap_or(text.len());
    split > 0
        && split < text.len()
        && text[..split].chars().all(|ch| ch.is_ascii_alphabetic())
        && text[split..].chars().all(|ch| ch.is_ascii_digit())
}

fn assert_profile_symbolic_normalized_reference(bound: &BoundFormula, expected_key: &str) {
    assert_eq!(bound.normalized_references.len(), 1);
    assert_eq!(
        profile_symbolic_record(bound).normal_form_key.0,
        expected_key
    );
}

fn profile_symbolic_record(bound: &BoundFormula) -> &ProfileReferenceRecord {
    match &bound.normalized_references[0] {
        NormalizedReference::ProfileSymbolic(record) => record,
        other => panic!("expected profile symbolic reference, got {other:?}"),
    }
}

fn assert_profile_symbolic_atom(reference: &ReferenceExpr, expected_key: &str) {
    match reference {
        ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record)) => {
            assert_eq!(record.normal_form_key.0, expected_key);
        }
        other => panic!("expected profile symbolic atom, got {other:?}"),
    }
}
