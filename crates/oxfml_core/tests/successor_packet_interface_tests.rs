use std::collections::BTreeMap;

use oxfml_core::binding::NameKind;
use oxfml_core::eval::EvaluationContext;
use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::interface::{
    HostProviderOutcomeKind, InMemoryLibraryContextProvider, LibraryContextProvider,
    LibraryContextSnapshotRef, PinnedLibraryContextView, ReturnedValueSurface,
    ReturnedValueSurfaceKind, TypedContextQueryBundle, TypedContextQueryFamily,
    classify_library_context_field,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult};
use oxfunc_core::host_info::{
    CellInfoQuery, HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, InfoQuery,
    ResolvedWebImage,
};
use oxfunc_core::value::{
    EvalValue, ExcelText, ExtendedValue, NumberFormatHint, PresentationHint, ReferenceLike,
    RichObjectValue, RichValue, RichValueData, RichValueType,
};

mod common;

#[test]
fn typed_context_query_bundle_freeze_candidate_stays_capability_scoped() {
    let locale = oxfml_en_us_locale_context();
    let bundle = TypedContextQueryBundle::new(
        Some(&MockHostInfoProvider),
        Some(&MockRtdProvider),
        Some(&locale),
        Some(46000.0),
        Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
    );

    let spec = bundle.freeze_candidate_spec();

    assert_eq!(
        spec.families,
        vec![
            TypedContextQueryFamily::ReferenceResolver,
            TypedContextQueryFamily::CellInfo,
            TypedContextQueryFamily::Info,
            TypedContextQueryFamily::Image,
            TypedContextQueryFamily::FormulaText,
            TypedContextQueryFamily::SheetIndex,
            TypedContextQueryFamily::SheetCount,
            TypedContextQueryFamily::AggregateReferenceContext,
            TypedContextQueryFamily::WidthConversionMode,
            TypedContextQueryFamily::Translate,
            TypedContextQueryFamily::Rtd,
            TypedContextQueryFamily::NowSerial,
            TypedContextQueryFamily::RandomProvider,
            TypedContextQueryFamily::LocaleFormatContext,
        ]
    );
}

#[test]
fn evaluation_context_round_trips_typed_context_query_bundle() {
    let compiled = common::compile_formula(
        "formula:test",
        "=SUM(1,2)",
        BTreeMap::<String, NameKind>::new(),
        "struct:v1",
        "oxfunc:test",
    );
    let locale = oxfml_en_us_locale_context();
    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    let bundle = TypedContextQueryBundle::new(
        Some(&MockHostInfoProvider),
        Some(&MockRtdProvider),
        Some(&locale),
        Some(45000.0),
        Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_05),
    );

    context.apply_typed_context_query_bundle(bundle);
    let round_tripped = context.typed_context_query_bundle();

    assert_eq!(
        round_tripped.freeze_candidate_spec(),
        bundle.freeze_candidate_spec()
    );
}

#[test]
fn returned_value_surface_keeps_three_way_split() {
    let ordinary =
        ReturnedValueSurface::from_extended_value(&ExtendedValue::Core(EvalValue::Number(42.0)));
    assert_eq!(ordinary.kind, ReturnedValueSurfaceKind::OrdinaryValue);
    assert_eq!(ordinary.payload_summary, "Number");
    assert_eq!(ordinary.rich_value_type_name, None);

    let with_presentation =
        ReturnedValueSurface::from_extended_value(&ExtendedValue::ValueWithPresentation {
            value: EvalValue::Number(42.0),
            hint: PresentationHint::number_format(NumberFormatHint::Currency),
        });
    assert_eq!(
        with_presentation.kind,
        ReturnedValueSurfaceKind::ValueWithPresentation
    );
    assert!(with_presentation.presentation_hint.is_some());
    assert_eq!(with_presentation.rich_value_type_name, None);

    let provider_outcome =
        ReturnedValueSurface::from_rtd_provider_result(&RtdProviderResult::CapabilityDenied);
    assert_eq!(
        provider_outcome.kind,
        ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        provider_outcome
            .host_provider_outcome
            .expect("typed outcome")
            .outcome_kind,
        HostProviderOutcomeKind::CapabilityDenied
    );
}

#[test]
fn returned_value_surface_preserves_image_rich_value_class() {
    let rich_value = ReturnedValueSurface::from_extended_value(&ExtendedValue::RichValue(
        Box::new(RichValue::Object(RichObjectValue {
            value_type: RichValueType {
                type_name: "_webimage".to_string(),
                required_keys: vec!["WebImageIdentifier".to_string()],
                key_flags: vec![],
            },
            fallback: RichValueData::Text(ExcelText::from_interop_assignment("Sphere")),
            kvps: vec![],
        })),
    ));
    assert_eq!(rich_value.kind, ReturnedValueSurfaceKind::RichValue);
    assert_eq!(rich_value.payload_summary, "RichValue(_webimage)");
    assert_eq!(
        rich_value.rich_value_type_name.as_deref(),
        Some("_webimage")
    );
    assert_eq!(rich_value.presentation_hint, None);
    assert_eq!(rich_value.host_provider_outcome, None);
}

#[test]
fn runtime_library_context_provider_pins_and_looks_up_snapshots() {
    let snapshot = test_snapshot();
    let snapshot_ref = LibraryContextSnapshotRef::from(&snapshot);
    let provider = InMemoryLibraryContextProvider::new(snapshot.clone());

    assert_eq!(provider.current_snapshot(), snapshot);
    let entry = provider
        .lookup_surface(&snapshot_ref, "SUM")
        .expect("lookup by surface name");
    assert_eq!(entry.surface_stable_id.as_deref(), Some("FUNC.SUM"));
    assert_eq!(
        provider.snapshot_by_identity(&snapshot_ref),
        Some(test_snapshot())
    );
}

#[test]
fn pinned_library_context_view_prefers_pinned_snapshot_ref_over_provider_current_snapshot() {
    let pinned_snapshot = test_snapshot();
    let pinned_snapshot_ref = LibraryContextSnapshotRef::from(&pinned_snapshot);
    let mut current_snapshot = test_snapshot();
    current_snapshot.snapshot_version = "v2".to_string();
    current_snapshot.entries.push(LibraryContextSnapshotEntry {
        surface_name: "TAKE".to_string(),
        canonical_id: Some("FUNC.TAKE".to_string()),
        surface_stable_id: Some("surface:take".to_string()),
        name_resolution_table_ref: None,
        semantic_trait_profile_ref: None,
        gating_profile_ref: None,
        metadata_status: Some("runtime_snapshot".to_string()),
        special_interface_kind: None,
        admission_interface_kind: None,
        preparation_owner: None,
        runtime_boundary_kind: None,
        interface_contract_ref: Some("contract:take".to_string()),
        registration_source_kind: RegistrationSourceKind::BuiltIn,
        parse_bind_state: LibraryAvailabilityState::CatalogKnown,
        semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
        runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
        post_dispatch_state: None,
    });
    let current_snapshot_ref = LibraryContextSnapshotRef::from(&current_snapshot);
    let provider = InMemoryLibraryContextProvider::with_snapshots(
        current_snapshot_ref,
        vec![pinned_snapshot.clone(), current_snapshot],
    );

    let view = PinnedLibraryContextView::new(Some(&provider), Some(&pinned_snapshot_ref), None);
    let resolved = view
        .resolve_snapshot()
        .expect("pinned snapshot should resolve");

    assert_eq!(view.effective_snapshot_ref(), Some(pinned_snapshot_ref));
    assert_eq!(resolved, pinned_snapshot);
    assert!(
        !resolved
            .entries
            .iter()
            .any(|entry| entry.surface_name.eq_ignore_ascii_case("TAKE"))
    );
}

#[test]
fn single_formula_host_can_consume_runtime_library_context_provider() {
    let provider = InMemoryLibraryContextProvider::new(test_snapshot());
    let locale = oxfml_en_us_locale_context();
    let mut host = SingleFormulaHost::new("formula:sum", "=SUM(1,2)");

    let output = host
        .recalc_with_rtd_provider(
            Some(&MockHostInfoProvider),
            Some(&MockRtdProvider),
            Some(&locale),
            Some(&provider),
        )
        .expect("host recalc");

    assert_eq!(
        output.semantic_plan.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::new("snapshot:host", "v1"))
    );
}

#[test]
fn library_context_field_classification_preserves_runtime_vs_export_split() {
    assert_eq!(
        classify_library_context_field("surface_stable_id"),
        Some(oxfml_core::LibraryContextFieldClass::RuntimeSemantic)
    );
    assert_eq!(
        classify_library_context_field("xlcall_builtin_symbol"),
        Some(oxfml_core::LibraryContextFieldClass::CompatibilityMetadata)
    );
    assert_eq!(classify_library_context_field("arity_shape_note"), None);
}

#[test]
fn grouped_query_family_runs_surface_bundle_specs_and_return_packets() {
    let locale = oxfml_en_us_locale_context();
    let cases = vec![
        (
            "host-info-info",
            "=INFO(\"system\")",
            Some(&MockHostInfoProvider as &dyn HostInfoProvider),
            None,
            vec![TypedContextQueryFamily::Info],
            vec![TypedContextQueryFamily::Rtd],
            ReturnedValueSurfaceKind::OrdinaryValue,
            "Text",
        ),
        (
            "host-info-info-unsupported",
            "=INFO(\"directory\")",
            Some(&MockHostInfoProvider as &dyn HostInfoProvider),
            None,
            vec![TypedContextQueryFamily::Info],
            vec![TypedContextQueryFamily::Rtd],
            ReturnedValueSurfaceKind::TypedHostProviderOutcome,
            "UnsupportedQuery",
        ),
        (
            "host-info-cell",
            "=CELL(\"filename\",A1)",
            Some(&MockHostInfoProvider as &dyn HostInfoProvider),
            None,
            vec![TypedContextQueryFamily::CellInfo],
            vec![TypedContextQueryFamily::Rtd],
            ReturnedValueSurfaceKind::OrdinaryValue,
            "Text",
        ),
        (
            "host-info-cell-provider-failure",
            "=CELL(\"filename\",A1)",
            Some(&FailingHostInfoProvider as &dyn HostInfoProvider),
            None,
            vec![TypedContextQueryFamily::CellInfo],
            vec![TypedContextQueryFamily::Rtd],
            ReturnedValueSurfaceKind::TypedHostProviderOutcome,
            "ProviderFailure",
        ),
        (
            "rtd-runtime-value",
            "=RTD(\"prog\",\"server\",\"topic\")",
            None,
            Some(&MockRtdProvider as &dyn RtdProvider),
            vec![TypedContextQueryFamily::Rtd],
            vec![
                TypedContextQueryFamily::Info,
                TypedContextQueryFamily::CellInfo,
            ],
            ReturnedValueSurfaceKind::TypedHostProviderOutcome,
            "Number",
        ),
        (
            "rtd-runtime-capability-denied",
            "=RTD(\"prog\",\"server\",\"topic\")",
            None,
            Some(&CapabilityDeniedRtdProvider as &dyn RtdProvider),
            vec![TypedContextQueryFamily::Rtd],
            vec![
                TypedContextQueryFamily::Info,
                TypedContextQueryFamily::CellInfo,
            ],
            ReturnedValueSurfaceKind::TypedHostProviderOutcome,
            "CapabilityDenied",
        ),
    ];

    for (
        scenario_id,
        formula,
        host_info,
        rtd_provider,
        required_families,
        absent_families,
        expected_kind,
        expected_payload_summary,
    ) in cases
    {
        let mut host = SingleFormulaHost::new(format!("formula:{scenario_id}"), formula);
        let output = host
            .recalc_with_rtd_provider(host_info, rtd_provider, Some(&locale), None)
            .expect("host recalc");

        for family in &required_families {
            assert!(
                output.typed_query_bundle_spec.families.contains(family),
                "missing required family {:?} for {scenario_id}",
                family
            );
        }
        for family in &absent_families {
            assert!(
                !output.typed_query_bundle_spec.families.contains(family),
                "unexpected family {:?} for {scenario_id}",
                family
            );
        }

        assert_eq!(
            output.returned_value_surface,
            output.evaluation.returned_value_surface
        );
        assert_eq!(
            output.returned_value_surface,
            output.candidate_result.returned_value_surface
        );
        assert_eq!(output.returned_value_surface.kind, expected_kind);
        assert_eq!(
            output.returned_value_surface.payload_summary,
            expected_payload_summary
        );

        match &output.commit_decision {
            oxfml_core::AcceptDecision::Accepted(bundle) => {
                assert_eq!(bundle.returned_value_surface, output.returned_value_surface);
            }
            oxfml_core::AcceptDecision::Rejected(_) => {
                panic!("expected accepted commit bundle for {scenario_id}")
            }
        }
    }
}

struct MockHostInfoProvider;

impl HostInfoProvider for MockHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        match query {
            CellInfoQuery::Filename => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "[Book1]Sheet1".encode_utf16().collect(),
            ))),
            _ => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
        }
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        match query {
            InfoQuery::System => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "pcdos".encode_utf16().collect(),
            ))),
            _ => Err(HostInfoError::UnsupportedInfoQuery(query)),
        }
    }

    fn query_image(&self, _request: &ImageRequest) -> Result<ImageProviderResult, HostInfoError> {
        Ok(ImageProviderResult::Image(ResolvedWebImage {
            web_image_identifier: "img-1".to_string(),
            published_fallback: ExcelText::from_interop_assignment("-2146826273"),
        }))
    }
}

struct FailingHostInfoProvider;

impl HostInfoProvider for FailingHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        match query {
            CellInfoQuery::Filename => Err(HostInfoError::ProviderFailure {
                detail: "host offline".to_string(),
            }),
            _ => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
        }
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedInfoQuery(query))
    }
}

struct MockRtdProvider;

impl RtdProvider for MockRtdProvider {
    fn resolve_rtd(
        &self,
        _request: &oxfunc_core::functions::rtd_fn::RtdRequest,
    ) -> RtdProviderResult {
        RtdProviderResult::Value(EvalValue::Number(7.0))
    }
}

struct CapabilityDeniedRtdProvider;

impl RtdProvider for CapabilityDeniedRtdProvider {
    fn resolve_rtd(
        &self,
        _request: &oxfunc_core::functions::rtd_fn::RtdRequest,
    ) -> RtdProviderResult {
        RtdProviderResult::CapabilityDenied
    }
}

fn test_snapshot() -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "snapshot:host".to_string(),
        snapshot_version: "v1".to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: "SUM".to_string(),
            canonical_id: Some("FUNC.SUM".to_string()),
            surface_stable_id: Some("FUNC.SUM".to_string()),
            name_resolution_table_ref: Some("table:default".to_string()),
            semantic_trait_profile_ref: Some("traits:sum".to_string()),
            gating_profile_ref: Some("gating:open".to_string()),
            metadata_status: Some("curated".to_string()),
            special_interface_kind: None,
            admission_interface_kind: Some("value_call".to_string()),
            preparation_owner: Some("oxfunc".to_string()),
            runtime_boundary_kind: Some("surface_dispatch".to_string()),
            interface_contract_ref: Some("oxfunc.surface.sum.v1".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
        }],
    }
}
