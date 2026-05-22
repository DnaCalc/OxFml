use std::collections::{BTreeMap, BTreeSet};

use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalDescriptor, RegisteredExternalProvider,
    RegisteredExternalProviderError,
};
use oxfunc_core::functions::rand_fn::RandomProvider;
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult};
use oxfunc_core::host_info::{HostInfoError, HostInfoProvider};
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::registry::builtin_registry;
use oxfunc_core::value::{EvalValue, ExtendedValue, PresentationHint, WorksheetErrorCode};

use crate::semantics::{LibraryContextSnapshot, LibraryContextSnapshotEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypedContextQueryFamily {
    ReferenceResolver,
    CellInfo,
    Info,
    Image,
    FormulaText,
    SheetIndex,
    SheetCount,
    AggregateReferenceContext,
    WidthConversionMode,
    Translate,
    Rtd,
    RegisteredExternal,
    HostFunction,
    NowSerial,
    RandomProvider,
    LocaleFormatContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedContextQueryBundleSpec {
    pub families: Vec<TypedContextQueryFamily>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableRef {
    pub table_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TableRegionKind {
    Headers,
    Data,
    Totals,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableCallerRegion {
    pub table_id: String,
    pub region_kind: TableRegionKind,
    pub data_row_offset: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableColumnDescriptor {
    pub column_id: String,
    pub column_name: String,
    pub ordinal: u32,
    pub column_range_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableDescriptor {
    pub table_id: String,
    pub table_name: String,
    pub workbook_scope_ref: String,
    pub sheet_scope_ref: String,
    pub table_range_ref: String,
    pub row_membership_identity: Option<String>,
    pub row_order_identity: Option<String>,
    pub header_region_ref: Option<String>,
    pub totals_region_ref: Option<String>,
    pub header_row_present: bool,
    pub totals_row_present: bool,
    pub columns: Vec<TableColumnDescriptor>,
}

#[derive(Clone, Copy)]
pub struct TypedContextQueryBundle<'a> {
    pub host_info: Option<&'a dyn HostInfoProvider>,
    pub rtd_provider: Option<&'a dyn RtdProvider>,
    pub registered_external_provider: Option<&'a dyn RegisteredExternalProvider>,
    pub host_function_provider: Option<&'a dyn HostFunctionProvider>,
    pub locale_ctx: Option<&'a LocaleFormatContext<'a>>,
    pub now_serial: Option<f64>,
    pub random_provider: Option<&'a dyn RandomProvider>,
}

impl std::fmt::Debug for TypedContextQueryBundle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedContextQueryBundle")
            .field("host_info_enabled", &self.host_info.is_some())
            .field("rtd_provider_enabled", &self.rtd_provider.is_some())
            .field(
                "registered_external_provider_enabled",
                &self.registered_external_provider.is_some(),
            )
            .field(
                "host_function_provider_enabled",
                &self.host_function_provider.is_some(),
            )
            .field("locale_ctx_enabled", &self.locale_ctx.is_some())
            .field("now_serial_enabled", &self.now_serial.is_some())
            .field("random_provider_enabled", &self.random_provider.is_some())
            .finish()
    }
}

impl<'a> Default for TypedContextQueryBundle<'a> {
    fn default() -> Self {
        Self {
            host_info: None,
            rtd_provider: None,
            registered_external_provider: None,
            host_function_provider: None,
            locale_ctx: None,
            now_serial: None,
            random_provider: None,
        }
    }
}

impl<'a> TypedContextQueryBundle<'a> {
    pub fn new(
        host_info: Option<&'a dyn HostInfoProvider>,
        rtd_provider: Option<&'a dyn RtdProvider>,
        locale_ctx: Option<&'a LocaleFormatContext<'a>>,
        now_serial: Option<f64>,
        random_provider: Option<&'a dyn RandomProvider>,
    ) -> Self {
        Self {
            host_info,
            rtd_provider,
            registered_external_provider: None,
            host_function_provider: None,
            locale_ctx,
            now_serial,
            random_provider,
        }
    }

    pub fn with_registered_external_provider(
        mut self,
        registered_external_provider: Option<&'a dyn RegisteredExternalProvider>,
    ) -> Self {
        self.registered_external_provider = registered_external_provider;
        self
    }

    pub fn with_host_function_provider(
        mut self,
        host_function_provider: Option<&'a dyn HostFunctionProvider>,
    ) -> Self {
        self.host_function_provider = host_function_provider;
        self
    }

    pub fn freeze_candidate_spec(&self) -> TypedContextQueryBundleSpec {
        let mut families = BTreeSet::from([TypedContextQueryFamily::ReferenceResolver]);
        if self.host_info.is_some() {
            families.extend([
                TypedContextQueryFamily::CellInfo,
                TypedContextQueryFamily::Info,
                TypedContextQueryFamily::Image,
                TypedContextQueryFamily::FormulaText,
                TypedContextQueryFamily::SheetIndex,
                TypedContextQueryFamily::SheetCount,
                TypedContextQueryFamily::AggregateReferenceContext,
                TypedContextQueryFamily::WidthConversionMode,
                TypedContextQueryFamily::Translate,
            ]);
        }
        if self.rtd_provider.is_some() {
            families.insert(TypedContextQueryFamily::Rtd);
        }
        if self.registered_external_provider.is_some() {
            families.insert(TypedContextQueryFamily::RegisteredExternal);
        }
        if self.host_function_provider.is_some() {
            families.insert(TypedContextQueryFamily::HostFunction);
        }
        if self.now_serial.is_some() {
            families.insert(TypedContextQueryFamily::NowSerial);
        }
        if self.random_provider.is_some() {
            families.insert(TypedContextQueryFamily::RandomProvider);
        }
        if self.locale_ctx.is_some() {
            families.insert(TypedContextQueryFamily::LocaleFormatContext);
        }

        TypedContextQueryBundleSpec {
            families: families.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostFunctionInvocation {
    pub function_name: String,
    pub args: Vec<EvalValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostFunctionProviderError {
    pub message: String,
}

impl HostFunctionProviderError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub trait HostFunctionProvider {
    fn invoke_host_function(
        &self,
        invocation: &HostFunctionInvocation,
    ) -> Result<EvalValue, HostFunctionProviderError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegisteredExternalRegistrationChannel {
    WorksheetRegisterId,
    HostApiRegistration,
    VbaProjectShimRegistration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredExternalHostRegistrationRequest {
    pub registration_channel: RegisteredExternalRegistrationChannel,
    pub register_id_request: RegisterIdRequest,
    pub stable_registration_id_hint: Option<String>,
    pub display_name_hint: Option<String>,
    pub help_text_hint: Option<String>,
    pub source_project_ref: Option<String>,
    pub source_module_ref: Option<String>,
    pub source_procedure_ref: Option<String>,
    pub host_execution_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredExternalUnregisterRequest {
    pub registration_channel: RegisteredExternalRegistrationChannel,
    pub stable_registration_id: String,
    pub host_execution_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisteredExternalCatalogMutationRequest {
    Register(RegisteredExternalHostRegistrationRequest),
    Unregister(RegisteredExternalUnregisterRequest),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisteredExternalCatalogMutationResult {
    RegisterApplied {
        descriptor: RegisteredExternalDescriptor,
        host_execution_profile: Option<String>,
    },
    UnregisterApplied {
        stable_registration_id: String,
        host_execution_profile: Option<String>,
    },
}

pub trait RegisteredExternalCatalogController {
    fn apply_mutation(
        &self,
        request: &RegisteredExternalCatalogMutationRequest,
    ) -> Result<RegisteredExternalCatalogMutationResult, RegisteredExternalProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibraryContextSnapshotRef {
    pub snapshot_id: String,
    pub snapshot_version: String,
}

impl LibraryContextSnapshotRef {
    pub fn new(snapshot_id: impl Into<String>, snapshot_version: impl Into<String>) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            snapshot_version: snapshot_version.into(),
        }
    }

    pub fn from_compound_ref(value: &str) -> Option<Self> {
        let (snapshot_id, snapshot_version) = value.split_once('@')?;
        if snapshot_id.is_empty() || snapshot_version.is_empty() {
            return None;
        }
        Some(Self::new(snapshot_id, snapshot_version))
    }

    pub fn compound_ref(&self) -> String {
        format!("{}@{}", self.snapshot_id, self.snapshot_version)
    }
}

impl From<&LibraryContextSnapshot> for LibraryContextSnapshotRef {
    fn from(value: &LibraryContextSnapshot) -> Self {
        Self {
            snapshot_id: value.snapshot_id.clone(),
            snapshot_version: value.snapshot_version.clone(),
        }
    }
}

pub trait LibraryContextProvider {
    fn current_snapshot(&self) -> LibraryContextSnapshot;

    fn current_snapshot_ref(&self) -> LibraryContextSnapshotRef {
        LibraryContextSnapshotRef::from(&self.current_snapshot())
    }

    fn snapshot_by_identity(
        &self,
        snapshot_ref: &LibraryContextSnapshotRef,
    ) -> Option<LibraryContextSnapshot>;

    fn lookup_surface(
        &self,
        snapshot_ref: &LibraryContextSnapshotRef,
        surface_key: &str,
    ) -> Option<LibraryContextSnapshotEntry> {
        self.snapshot_by_identity(snapshot_ref)?
            .entries
            .into_iter()
            .find(|entry| snapshot_entry_matches_surface_key(entry, surface_key))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryLibraryContextProvider {
    current_snapshot_ref: LibraryContextSnapshotRef,
    snapshots: BTreeMap<LibraryContextSnapshotRef, LibraryContextSnapshot>,
}

impl InMemoryLibraryContextProvider {
    pub fn new(current_snapshot: LibraryContextSnapshot) -> Self {
        let current_snapshot_ref = LibraryContextSnapshotRef::from(&current_snapshot);
        let mut snapshots = BTreeMap::new();
        snapshots.insert(current_snapshot_ref.clone(), current_snapshot);
        Self {
            current_snapshot_ref,
            snapshots,
        }
    }

    pub fn with_snapshots(
        current_snapshot_ref: LibraryContextSnapshotRef,
        snapshots: Vec<LibraryContextSnapshot>,
    ) -> Self {
        let snapshots = snapshots
            .into_iter()
            .map(|snapshot| (LibraryContextSnapshotRef::from(&snapshot), snapshot))
            .collect();
        Self {
            current_snapshot_ref,
            snapshots,
        }
    }
}

impl LibraryContextProvider for InMemoryLibraryContextProvider {
    fn current_snapshot(&self) -> LibraryContextSnapshot {
        self.snapshots
            .get(&self.current_snapshot_ref)
            .expect("current snapshot ref should resolve")
            .clone()
    }

    fn current_snapshot_ref(&self) -> LibraryContextSnapshotRef {
        self.current_snapshot_ref.clone()
    }

    fn snapshot_by_identity(
        &self,
        snapshot_ref: &LibraryContextSnapshotRef,
    ) -> Option<LibraryContextSnapshot> {
        self.snapshots.get(snapshot_ref).cloned()
    }
}

#[derive(Clone, Copy)]
pub struct PinnedLibraryContextView<'a> {
    provider: Option<&'a dyn LibraryContextProvider>,
    snapshot_ref: Option<&'a LibraryContextSnapshotRef>,
    snapshot: Option<&'a LibraryContextSnapshot>,
}

impl<'a> std::fmt::Debug for PinnedLibraryContextView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PinnedLibraryContextView")
            .field("provider_enabled", &self.provider.is_some())
            .field("snapshot_ref", &self.snapshot_ref)
            .field("inline_snapshot_enabled", &self.snapshot.is_some())
            .finish()
    }
}

impl<'a> PinnedLibraryContextView<'a> {
    pub fn new(
        provider: Option<&'a dyn LibraryContextProvider>,
        snapshot_ref: Option<&'a LibraryContextSnapshotRef>,
        snapshot: Option<&'a LibraryContextSnapshot>,
    ) -> Self {
        Self {
            provider,
            snapshot_ref,
            snapshot,
        }
    }

    pub fn resolve_snapshot(&self) -> Option<LibraryContextSnapshot> {
        match (self.provider, self.snapshot_ref, self.snapshot) {
            (Some(provider), Some(snapshot_ref), _) => provider.snapshot_by_identity(snapshot_ref),
            (_, _, Some(snapshot)) => Some(snapshot.clone()),
            (Some(provider), None, None) => Some(provider.current_snapshot()),
            (None, Some(_), None) => None,
            (None, None, None) => None,
        }
    }

    pub fn effective_snapshot_ref(&self) -> Option<LibraryContextSnapshotRef> {
        match (self.provider, self.snapshot_ref, self.snapshot) {
            (Some(_), Some(snapshot_ref), _) => Some(snapshot_ref.clone()),
            (_, _, Some(snapshot)) => Some(LibraryContextSnapshotRef::from(snapshot)),
            (Some(provider), None, None) => Some(provider.current_snapshot_ref()),
            (None, Some(_), None) => None,
            (None, None, None) => None,
        }
    }

    pub fn provider(&self) -> Option<&'a dyn LibraryContextProvider> {
        self.provider
    }

    pub fn snapshot_ref(&self) -> Option<&'a LibraryContextSnapshotRef> {
        self.snapshot_ref
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryContextFieldClass {
    RuntimeSemantic,
    CompatibilityMetadata,
    ExportDescription,
}

pub fn classify_library_context_field(field_name: &str) -> Option<LibraryContextFieldClass> {
    match field_name {
        "snapshot_id"
        | "snapshot_generation"
        | "snapshot_version"
        | "source_commit_short"
        | "source_commit_full"
        | "source_tree_state"
        | "surface_stable_id"
        | "entry_kind"
        | "registration_source_kind"
        | "canonical_surface_name"
        | "name_resolution_table_ref"
        | "semantic_trait_profile_ref"
        | "gating_profile_ref"
        | "metadata_status"
        | "special_interface_kind"
        | "admission_interface_kind"
        | "preparation_owner"
        | "runtime_boundary_kind"
        | "interface_contract_ref" => Some(LibraryContextFieldClass::RuntimeSemantic),
        "xlcall_builtin_symbol" | "xlcall_builtin_code" | "xlfn_name" | "_xlfn_name" => {
            Some(LibraryContextFieldClass::CompatibilityMetadata)
        }
        "arg_preparation_profile" | "explanatory_note" => {
            Some(LibraryContextFieldClass::ExportDescription)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnedValueSurfaceKind {
    OrdinaryValue,
    RichValue,
    ValueWithPresentation,
    TypedHostProviderOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostProviderOutcomeKind {
    Value,
    UnsupportedQuery,
    CapabilityDenied,
    NoValueYet,
    ConnectionFailed,
    ProviderError,
    ProviderFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostProviderOutcomeSurface {
    pub outcome_kind: HostProviderOutcomeKind,
    pub worksheet_error: Option<WorksheetErrorCode>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnedValueSurface {
    pub kind: ReturnedValueSurfaceKind,
    pub payload_summary: String,
    pub rich_value_type_name: Option<String>,
    pub producer_capability_set_keys: Vec<String>,
    pub exercised_capability_keys: Vec<String>,
    pub presentation_hint: Option<PresentationHint>,
    pub host_provider_outcome: Option<HostProviderOutcomeSurface>,
}

impl ReturnedValueSurface {
    pub fn from_extended_value(value: &ExtendedValue) -> Self {
        match value {
            ExtendedValue::Core(core) => Self {
                kind: ReturnedValueSurfaceKind::OrdinaryValue,
                payload_summary: eval_value_summary(core),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: None,
            },
            ExtendedValue::RichValue(rich) => Self {
                kind: ReturnedValueSurfaceKind::RichValue,
                payload_summary: format!("RichValue({})", rich.value_type.type_name),
                rich_value_type_name: Some(rich.value_type.type_name.clone()),
                producer_capability_set_keys: rich_value_producer_capability_set_keys(
                    &rich.value_type.type_name,
                ),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: None,
            },
            ExtendedValue::ValueWithPresentation { value, hint } => Self {
                kind: ReturnedValueSurfaceKind::ValueWithPresentation,
                payload_summary: eval_value_summary(value),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: Some(*hint),
                host_provider_outcome: None,
            },
            ExtendedValue::ErrorWithMetadata { code, .. } => Self {
                kind: ReturnedValueSurfaceKind::OrdinaryValue,
                payload_summary: format!("Error({code:?})"),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: None,
            },
        }
    }

    pub fn from_extended_value_with_capability_keys(
        value: &ExtendedValue,
        producer_capability_set_keys: Vec<String>,
        exercised_capability_keys: Vec<String>,
    ) -> Self {
        let mut surface = Self::from_extended_value(value);
        surface.producer_capability_set_keys = producer_capability_set_keys;
        surface.exercised_capability_keys = exercised_capability_keys;
        surface
    }

    pub fn from_host_info_error(error: &HostInfoError) -> Self {
        let (outcome_kind, detail) = match error {
            HostInfoError::UnsupportedCellInfoQuery(_)
            | HostInfoError::UnsupportedInfoQuery(_)
            | HostInfoError::UnsupportedFormulaTextQuery
            | HostInfoError::UnsupportedSheetIndexQuery
            | HostInfoError::UnsupportedSheetCountQuery
            | HostInfoError::UnsupportedAggregateReferenceContextQuery
            | HostInfoError::UnsupportedWidthConversionProfileQuery(_)
            | HostInfoError::UnsupportedImageQuery
            | HostInfoError::UnsupportedTranslateQuery => {
                (HostProviderOutcomeKind::UnsupportedQuery, None)
            }
            HostInfoError::ProviderFailure { detail } => (
                HostProviderOutcomeKind::ProviderFailure,
                Some(detail.clone()),
            ),
        };

        Self {
            kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
            payload_summary: format!("{outcome_kind:?}"),
            rich_value_type_name: None,
            producer_capability_set_keys: Vec::new(),
            exercised_capability_keys: Vec::new(),
            presentation_hint: None,
            host_provider_outcome: Some(HostProviderOutcomeSurface {
                outcome_kind,
                worksheet_error: None,
                detail,
            }),
        }
    }

    pub fn from_rtd_provider_result(result: &RtdProviderResult) -> Self {
        match result {
            RtdProviderResult::Value(value) => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: eval_value_summary(value),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::Value,
                    worksheet_error: None,
                    detail: None,
                }),
            },
            RtdProviderResult::NoValueYet => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: "NoValueYet".to_string(),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::NoValueYet,
                    worksheet_error: Some(WorksheetErrorCode::NA),
                    detail: None,
                }),
            },
            RtdProviderResult::CapabilityDenied => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: "CapabilityDenied".to_string(),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::CapabilityDenied,
                    worksheet_error: Some(WorksheetErrorCode::Blocked),
                    detail: None,
                }),
            },
            RtdProviderResult::ConnectionFailed => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: "ConnectionFailed".to_string(),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::ConnectionFailed,
                    worksheet_error: Some(WorksheetErrorCode::Connect),
                    detail: None,
                }),
            },
            RtdProviderResult::ProviderError(code) => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: format!("ProviderError({code:?})"),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::ProviderError,
                    worksheet_error: Some(*code),
                    detail: None,
                }),
            },
        }
    }

    pub fn from_rtd_eval_value(value: &EvalValue) -> Self {
        match value {
            EvalValue::Error(WorksheetErrorCode::NA) => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: "NoValueYet".to_string(),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::NoValueYet,
                    worksheet_error: Some(WorksheetErrorCode::NA),
                    detail: None,
                }),
            },
            EvalValue::Error(WorksheetErrorCode::Blocked) => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: "CapabilityDenied".to_string(),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::CapabilityDenied,
                    worksheet_error: Some(WorksheetErrorCode::Blocked),
                    detail: None,
                }),
            },
            EvalValue::Error(WorksheetErrorCode::Connect) => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: "ConnectionFailed".to_string(),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::ConnectionFailed,
                    worksheet_error: Some(WorksheetErrorCode::Connect),
                    detail: None,
                }),
            },
            EvalValue::Error(code) => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: format!("ProviderError({code:?})"),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::ProviderError,
                    worksheet_error: Some(*code),
                    detail: None,
                }),
            },
            _ => Self {
                kind: ReturnedValueSurfaceKind::TypedHostProviderOutcome,
                payload_summary: eval_value_summary(value),
                rich_value_type_name: None,
                producer_capability_set_keys: Vec::new(),
                exercised_capability_keys: Vec::new(),
                presentation_hint: None,
                host_provider_outcome: Some(HostProviderOutcomeSurface {
                    outcome_kind: HostProviderOutcomeKind::Value,
                    worksheet_error: None,
                    detail: None,
                }),
            },
        }
    }
}

fn rich_value_producer_capability_set_keys(type_name: &str) -> Vec<String> {
    match type_name {
        "_webimage" => builtin_registry()
            .lookup_by_surface_name("IMAGE")
            .map(|entry| entry.meta.producer_capability_set_keys.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn snapshot_entry_matches_surface_key(
    entry: &LibraryContextSnapshotEntry,
    surface_key: &str,
) -> bool {
    entry.surface_name == surface_key
        || entry
            .canonical_id
            .as_ref()
            .is_some_and(|canonical_id| canonical_id == surface_key)
        || entry
            .surface_stable_id
            .as_ref()
            .is_some_and(|stable_id| stable_id == surface_key)
}

fn eval_value_summary(value: &EvalValue) -> String {
    match value {
        EvalValue::Number(_) => "Number".to_string(),
        EvalValue::Text(_) => "Text".to_string(),
        EvalValue::Logical(_) => "Logical".to_string(),
        EvalValue::Error(code) => format!("Error({code:?})"),
        EvalValue::Array(array) => {
            let shape = array.shape();
            format!("Array({}x{})", shape.rows, shape.cols)
        }
        EvalValue::Reference(reference) => format!("Reference({})", reference.target),
        EvalValue::Lambda(lambda) => format!("Lambda({})", lambda.callable_token),
    }
}
