pub mod binding;
pub mod carrier;
pub mod consumer;
pub mod eval;
pub mod format;
mod host;
pub mod interface;
mod language_service;
mod oxfunc_adapter;
pub mod publication;
pub mod red;
pub mod scheduler;
pub mod seam;
pub mod semantics;
mod session;
pub mod source;
pub mod syntax;

#[doc(hidden)]
pub mod test_support {
    pub mod host {
        pub use crate::host::*;
    }

    pub mod session {
        pub use crate::session::*;
    }

    pub mod oxfunc_adapter {
        pub use crate::oxfunc_adapter::*;
    }

    pub mod random {
        use oxfunc_core::functions::rand_fn::RandomProvider;

        pub struct FixedRandomProvider {
            pub value: f64,
        }

        impl RandomProvider for FixedRandomProvider {
            fn random_unit(&self) -> f64 {
                self.value
            }
        }

        pub static FIXED_RANDOM_PROVIDER_025: FixedRandomProvider =
            FixedRandomProvider { value: 0.25 };
        pub static FIXED_RANDOM_PROVIDER_05: FixedRandomProvider =
            FixedRandomProvider { value: 0.5 };
    }
}

pub use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalCallRequest, RegisteredExternalDescriptor,
    RegisteredExternalOriginKind, RegisteredExternalProvider, RegisteredExternalProviderError,
    RegisteredExternalTarget, RegisteredProcedureSpec,
};

pub use binding::{
    BindContext, BindDiagnostic, BindProfile, BindRequest, BindResult, BoundExpr, BoundFormula,
    BoundHostReferenceCollection, BoundHostStructuralSelector, FormulaSourceIdentity,
    FormulaTemplateIdentity, HostNameBindRecord, HostReferenceCollectionResolveRequest,
    HostReferenceCollectionResolveResult, HostStructuralSelectorResolveRequest,
    HostStructuralSelectorResolveResult, IncrementalBindResult, InstantiatedReference,
    NormalizedReference, PlacedFormulaIdentity, ProfilePayload, ProfileReferenceRecord,
    ProfileVersion, ReferenceAtomBindRequest, ReferenceAtomBindResult, ReferenceBindProfile,
    ReferenceCompletionProposal, ReferenceCompletionRequest, ReferenceCompletionResult,
    ReferenceDependencyEnvelope, ReferenceEditorContext, ReferenceExpr, ReferenceFingerprintPolicy,
    ReferenceInstantiationPurpose, ReferenceInstantiationRequest, ReferenceNormalFormKey,
    ReferenceOperatorCapabilities, ReferencePolicy, ReferenceProfileFingerprint,
    ReferenceProfileFingerprintContext, ReferenceRangeBindRequest, ReferenceRangeBindResult,
    ReferenceRangeEndpointBindRequest, ReferenceRenderRequest, ReferenceRenderResult,
    ReferenceSourceInfo, ReferenceSyntaxCapabilities, ReferenceTransformKind,
    ReferenceTransformOutcome, ReferenceTransformRequest, ReferenceTransformResult,
    ReferenceValidity, RuntimeDependencyIdentity, RuntimeHostFormulaContext,
    StructuredReferenceBindDiagnosticLink, StructuredReferenceBindRecord,
    StructuredReferenceSelectedRegion, StructuredReferenceSourceTokenKind, StructuredSectionKind,
    bind_formula, bind_formula_incremental,
};
pub use carrier::{
    CarrierRestrictionCode, CarrierValidationDisposition, ConditionalFormattingCarrierSpec,
    DataValidationCarrierSpec, FormulaCarrierValidation, validate_conditional_formatting_formula,
    validate_data_validation_formula,
};
pub use eval::{
    CallableCaptureMode, CallableCapturedRef, CallableDefinedNameBinding, CallableInvocationModel,
    CallableOriginKind, CallableValueCarrier, CallableValueProfile, DefinedNameBinding,
    EvaluationBackend, EvaluationContext, EvaluationError, EvaluationOutput, EvaluationTrace,
    EvaluationTraceMode, PortableCallableValue, PreparedArgument, PreparedBlanknessClass,
    PreparedCall, PreparedEvaluationMode, PreparedResult, PreparedResultClass, PreparedSourceClass,
    PreparedStructureClass, evaluate_formula,
};
pub use interface::{
    HostFunctionInvocation, HostFunctionProvider, HostFunctionProviderError,
    HostProviderOutcomeKind, HostProviderOutcomeSurface, InMemoryLibraryContextProvider,
    LibraryContextFieldClass, LibraryContextProvider, LibraryContextSnapshotRef,
    PinnedLibraryContextView, RegisteredExternalCatalogController,
    RegisteredExternalCatalogMutationRequest, RegisteredExternalCatalogMutationResult,
    RegisteredExternalHostRegistrationRequest, RegisteredExternalRegistrationChannel,
    RegisteredExternalUnregisterRequest, ReturnedValueSurface, ReturnedValueSurfaceKind,
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
    TypedContextQueryBundle, TypedContextQueryBundleSpec, TypedContextQueryFamily,
    classify_library_context_field,
};
pub use publication::{
    ArrayCellFormat, ArrayCellFormatGrid, AverageRuleOptions, CfIcon, ColorScaleRuleOptions,
    ColorScaleRuleStop, ConditionalFormattingRank, ConditionalFormattingThreshold,
    ConditionalFormattingTypedRule, DataBarDirection, DataBarFill, DataBarRuleOptions,
    IconSetRuleOptions, LocaleFormatContextSurface, RankRuleOptions,
    VerificationConditionalFormattingRule, VerificationPublicationContext,
    VerificationPublicationSurface, build_verification_publication_surface,
};
pub use red::{
    IncrementalRedProjectionResult, RedNode, RedProjection, project_red_view,
    project_red_view_incremental,
};
pub use scheduler::{
    ExecutionContract, ExecutionRestriction, ReplaySensitivityClass, SchedulerLaneClass,
    build_execution_contract,
};
pub use seam::{
    AcceptDecision, AcceptedCandidateResult, CapabilityDenialContext, CapabilityEffectFact,
    CommitBundle, CommitRequest, DependencyConsequenceFact, DisplayDelta, DynamicReferenceFact,
    DynamicReferenceFailureContext, ExecutionOutcomeKind, ExecutionOutcomeStage,
    ExecutionOutcomeSurface, Extent, FenceMismatchContext, FenceSnapshot, FormatDelta,
    FormatDependencyFact, Locus, RejectCode, RejectContext, RejectRecord, ResourceInvariantContext,
    SessionTerminationContext, ShapeDelta, ShapeOutcomeClass, SpillEvent, SpillEventKind,
    SpillFact, StructuralConflictContext, TopologyDelta, TraceEvent, TraceEventKind, TracePayload,
    ValueDelta, ValuePayload, WorksheetValueClass, commit_candidate,
};
pub use semantics::{
    CompileSemanticPlanRequest, CompileSemanticPlanResult, EvaluationRequirement,
    ExecutionProfileSummary, FormulaDeterminismClass, FormulaThreadSafetyClass,
    FormulaVolatilityClass, FunctionAvailabilitySummary, FunctionPlanBinding,
    HelperEnvironmentProfile, LibraryAvailabilityState, LibraryContextSnapshot,
    LibraryContextSnapshotEntry, RegistrationSourceKind, SemanticDiagnostic, SemanticPlan,
    compile_semantic_plan,
};
pub use source::{
    FormulaChannelKind, FormulaSourceRecord, FormulaStableId, FormulaTextVersion, FormulaToken,
    StructureContextVersion,
};
pub use syntax::green::{GreenTreeRoot, SyntaxKind};
pub use syntax::parser::{
    HostReferenceCollectionSyntax, HostReferenceStructuralSelectorSyntax,
    HostReferenceSyntaxProfile, IncrementalParseResult, ParseRequest, ParseResult, parse_formula,
    parse_formula_incremental, parse_formula_incremental_with_host_reference_syntax,
    parse_formula_with_host_reference_syntax,
};
