use crate::source::FormulaChannelKind;
use crate::syntax::token::TextSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileVersion(pub String);

impl ProfileVersion {
    pub fn v1() -> Self {
        Self("v1".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceProfileFingerprint(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceNormalFormKey(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferencePolicy {
    FormulaOnly,
    LegacyCompatibility,
    ProfileSymbolic,
    HostExtended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceFingerprintPolicy {
    IncludeCallerAnchor,
    ExcludeCallerAnchorForTemplate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReferenceSyntaxCapabilities {
    pub a1_references: bool,
    pub r1c1_references: bool,
    pub host_references: bool,
    pub structured_references: bool,
    pub spill_references: bool,
}

impl ReferenceSyntaxCapabilities {
    pub const fn worksheet_legacy() -> Self {
        Self {
            a1_references: true,
            r1c1_references: true,
            host_references: false,
            structured_references: true,
            spill_references: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindProfile {
    pub profile_id: String,
    pub profile_version: ProfileVersion,
    pub reference_policy: ReferencePolicy,
    pub fingerprint_policy: ReferenceFingerprintPolicy,
    pub syntax_capabilities: ReferenceSyntaxCapabilities,
}

impl BindProfile {
    pub fn legacy_compatibility() -> Self {
        Self {
            profile_id: "formula-legacy".to_string(),
            profile_version: ProfileVersion::v1(),
            reference_policy: ReferencePolicy::LegacyCompatibility,
            fingerprint_policy: ReferenceFingerprintPolicy::IncludeCallerAnchor,
            syntax_capabilities: ReferenceSyntaxCapabilities::worksheet_legacy(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReferenceOperatorCapabilities {
    pub range: bool,
    pub union: bool,
    pub intersection: bool,
    pub spill: bool,
}

impl ReferenceOperatorCapabilities {
    pub const fn worksheet_legacy() -> Self {
        Self {
            range: true,
            union: true,
            intersection: true,
            spill: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceValidity {
    ValidNow,
    ValidAfterInstantiation,
    InvalidStatic,
    InvalidForCurrentPlacement,
    DynamicOrHostSensitive,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceSourceInfo {
    pub source_channel: FormulaChannelKind,
    pub source_span: TextSpan,
    pub source_text: String,
    pub parsed_qualifier: Option<String>,
    pub address_fidelity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfilePayload {
    pub payload_kind: String,
    pub encoding: String,
    pub data: String,
}

impl ProfilePayload {
    pub fn textual(payload_kind: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            payload_kind: payload_kind.into(),
            encoding: "text".to_string(),
            data: data.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileReferenceRecord {
    pub profile_id: String,
    pub profile_version: ProfileVersion,
    pub source_info: ReferenceSourceInfo,
    pub profile_payload: ProfilePayload,
    pub normal_form_key: ReferenceNormalFormKey,
    pub render_hint: Option<String>,
    pub validity: ReferenceValidity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceProfileFingerprintContext {
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
    pub structure_context_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceAtomBindRequest {
    pub source_channel: FormulaChannelKind,
    pub source_span: TextSpan,
    pub source_text: String,
    pub parsed_qualifier: Option<String>,
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceSelectorKind {
    Collection,
    StructuralSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceSelectorSyntax {
    pub token_text: String,
    pub selector_family: String,
    pub selector_kind: ReferenceSelectorKind,
}

impl ReferenceSelectorSyntax {
    pub fn collection(token_text: impl Into<String>, selector_family: impl Into<String>) -> Self {
        Self {
            token_text: token_text.into(),
            selector_family: selector_family.into(),
            selector_kind: ReferenceSelectorKind::Collection,
        }
    }

    pub fn structural_selector(
        token_text: impl Into<String>,
        selector_family: impl Into<String>,
    ) -> Self {
        Self {
            token_text: token_text.into(),
            selector_family: selector_family.into(),
            selector_kind: ReferenceSelectorKind::StructuralSelector,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceNameBindRequest {
    pub source_channel: FormulaChannelKind,
    pub source_span: TextSpan,
    pub source_text: String,
    pub parsed_qualifier: Option<String>,
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
}

impl ReferenceNameBindRequest {
    pub fn from_atom(request: &ReferenceAtomBindRequest) -> Self {
        Self {
            source_channel: request.source_channel,
            source_span: request.source_span,
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
            workbook_id: request.workbook_id.clone(),
            sheet_id: request.sheet_id.clone(),
            caller_row: request.caller_row,
            caller_col: request.caller_col,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSelectorBindRequest {
    pub source_channel: FormulaChannelKind,
    pub source_span: TextSpan,
    pub source_text: String,
    pub selector_token_text: String,
    pub selector_family: String,
    pub selector_kind: ReferenceSelectorKind,
    pub base: Option<ProfileReferenceRecord>,
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceStructuredBindRequest {
    pub source_channel: FormulaChannelKind,
    pub source_span: TextSpan,
    pub source_text: String,
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceAtomBindResult {
    Bound(ProfileReferenceRecord),
    LegacyCompatibility,
    Rejected {
        validity: ReferenceValidity,
        message: String,
    },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceRangeEndpointBindRequest {
    pub source_span: TextSpan,
    pub source_text: String,
    pub target_text: String,
    pub parsed_qualifier: Option<String>,
    pub sheet_id: String,
    pub external_target_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceRangeBindRequest {
    pub source_channel: FormulaChannelKind,
    pub source_span: TextSpan,
    pub source_text: String,
    pub left: ReferenceRangeEndpointBindRequest,
    pub right: ReferenceRangeEndpointBindRequest,
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceRangeBindResult {
    Bound(ProfileReferenceRecord),
    LegacyCompatibility,
    Rejected {
        validity: ReferenceValidity,
        message: String,
    },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceDependencyEnvelope {
    None,
    Static {
        profile_id: String,
        dependency_key: String,
    },
    Dynamic {
        profile_id: String,
        request_key: String,
    },
    HostSensitive {
        profile_id: String,
        reason: String,
    },
    Unsupported {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceTransformKind {
    StructuralEdit,
    RenderModeChange,
    HostSpecific(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceTransformRequest {
    pub reference: ProfileReferenceRecord,
    pub transform_kind: ReferenceTransformKind,
    pub payload: Option<ProfilePayload>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceTransformOutcome {
    Unchanged,
    Shifted,
    Expanded,
    Shrunk,
    Split,
    PartiallyInvalid,
    FullyInvalid,
    DynamicOrHostSensitive,
    Unsupported,
    GeometryCoupledOpaqueConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceTransformResult {
    pub outcome: ReferenceTransformOutcome,
    pub reference: Option<ProfileReferenceRecord>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceRenderRequest {
    pub reference: ProfileReferenceRecord,
    pub target_channel: FormulaChannelKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceRenderResult {
    pub rendered_text: Option<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceEditorContext {
    pub source_channel: FormulaChannelKind,
    pub formula_text: String,
    pub cursor_offset: usize,
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
    pub formula_token: String,
    pub structure_context_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCompletionRequest {
    pub editor_context: ReferenceEditorContext,
    pub replacement_span: TextSpan,
    pub prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCompletionProposal {
    pub proposal_id: String,
    pub display_text: String,
    pub insert_text: String,
    pub replacement_span: Option<TextSpan>,
    pub documentation_ref: Option<String>,
    pub profile_payload: Option<ProfilePayload>,
    pub requires_revalidation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCompletionResult {
    pub proposals: Vec<ReferenceCompletionProposal>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceInstantiationRequest {
    pub bound_reference: ProfileReferenceRecord,
    pub runtime_host_formula_context: RuntimeHostFormulaContext,
    pub purpose: ReferenceInstantiationPurpose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostFormulaContext {
    pub profile_id: String,
    pub context_payload: Option<ProfilePayload>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceInstantiationPurpose {
    StaticDependencyExtraction,
    RuntimeEvaluation,
    Rendering,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstantiatedReference {
    ReferenceLike {
        profile_id: String,
        identity_key: String,
    },
    StaticDependencyEnvelope(ReferenceDependencyEnvelope),
    DynamicDependencyRequest(ReferenceDependencyEnvelope),
    RefError,
    Unsupported {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaSourceIdentity {
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaTemplateIdentity {
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedFormulaIdentity {
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDependencyIdentity {
    pub key: String,
}

pub trait ReferenceBindProfile {
    fn profile_id(&self) -> &str;

    fn profile_version(&self) -> ProfileVersion {
        ProfileVersion::v1()
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::LegacyCompatibility
    }

    fn fingerprint_policy(&self) -> ReferenceFingerprintPolicy {
        ReferenceFingerprintPolicy::IncludeCallerAnchor
    }

    fn fingerprint(
        &self,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceProfileFingerprint {
        ReferenceProfileFingerprint(format!(
            "{}:{}",
            self.profile_id(),
            self.profile_version().0
        ))
    }

    fn operator_capabilities(&self) -> ReferenceOperatorCapabilities {
        ReferenceOperatorCapabilities::worksheet_legacy()
    }

    fn bind_atom(&self, _request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::LegacyCompatibility
    }

    fn bind_name(&self, _request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::LegacyCompatibility
    }

    fn selector_syntax(&self) -> Vec<ReferenceSelectorSyntax> {
        Vec::new()
    }

    fn bind_selector(&self, _request: &ReferenceSelectorBindRequest) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::LegacyCompatibility
    }

    fn bind_structured_reference(
        &self,
        _request: &ReferenceStructuredBindRequest,
    ) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::LegacyCompatibility
    }

    fn bind_range(&self, _request: &ReferenceRangeBindRequest) -> ReferenceRangeBindResult {
        ReferenceRangeBindResult::LegacyCompatibility
    }

    fn normal_form_key(
        &self,
        reference: &ProfileReferenceRecord,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceNormalFormKey {
        reference.normal_form_key.clone()
    }

    fn dependency_hints(
        &self,
        _reference: &ProfileReferenceRecord,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceDependencyEnvelope {
        ReferenceDependencyEnvelope::None
    }

    fn instantiate_reference(
        &self,
        request: &ReferenceInstantiationRequest,
    ) -> InstantiatedReference {
        if request.runtime_host_formula_context.profile_id != self.profile_id() {
            return InstantiatedReference::Unsupported {
                reason: format!(
                    "runtime host formula context profile '{}' does not match reference profile '{}'",
                    request.runtime_host_formula_context.profile_id,
                    self.profile_id()
                ),
            };
        }

        match request.bound_reference.validity {
            ReferenceValidity::ValidNow | ReferenceValidity::ValidAfterInstantiation => {
                match request.purpose {
                    ReferenceInstantiationPurpose::StaticDependencyExtraction => {
                        InstantiatedReference::StaticDependencyEnvelope(
                            ReferenceDependencyEnvelope::Static {
                                profile_id: self.profile_id().to_string(),
                                dependency_key: request.bound_reference.normal_form_key.0.clone(),
                            },
                        )
                    }
                    ReferenceInstantiationPurpose::RuntimeEvaluation
                    | ReferenceInstantiationPurpose::Rendering => {
                        InstantiatedReference::ReferenceLike {
                            profile_id: self.profile_id().to_string(),
                            identity_key: request.bound_reference.normal_form_key.0.clone(),
                        }
                    }
                }
            }
            ReferenceValidity::DynamicOrHostSensitive => {
                InstantiatedReference::DynamicDependencyRequest(
                    ReferenceDependencyEnvelope::Dynamic {
                        profile_id: self.profile_id().to_string(),
                        request_key: request.bound_reference.normal_form_key.0.clone(),
                    },
                )
            }
            ReferenceValidity::InvalidStatic | ReferenceValidity::InvalidForCurrentPlacement => {
                InstantiatedReference::RefError
            }
            ReferenceValidity::Unsupported => InstantiatedReference::Unsupported {
                reason: format!(
                    "reference '{}' is unsupported by profile '{}'",
                    request.bound_reference.normal_form_key.0,
                    self.profile_id()
                ),
            },
        }
    }

    fn transform_reference(&self, request: &ReferenceTransformRequest) -> ReferenceTransformResult {
        ReferenceTransformResult {
            outcome: ReferenceTransformOutcome::Unsupported,
            reference: Some(request.reference.clone()),
            diagnostics: vec!["reference transform is not supported by this profile".to_string()],
        }
    }

    fn render_reference(&self, request: &ReferenceRenderRequest) -> ReferenceRenderResult {
        ReferenceRenderResult {
            rendered_text: request.reference.render_hint.clone(),
            diagnostics: Vec::new(),
        }
    }

    fn reference_completion_proposals(
        &self,
        _request: &ReferenceCompletionRequest,
    ) -> ReferenceCompletionResult {
        ReferenceCompletionResult {
            proposals: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}
