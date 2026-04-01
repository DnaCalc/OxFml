pub mod binding;
pub mod carrier;
pub mod consumer;
pub mod eval;
mod host;
pub mod interface;
mod language_service;
mod oxfunc_adapter;
pub mod red;
pub mod scheduler;
pub mod seam;
pub mod semantics;
mod session;
pub mod source;
pub mod syntax;

#[doc(hidden)]
pub mod substrate {
    pub mod host {
        pub use crate::host::*;
    }

    pub mod session {
        pub use crate::session::*;
    }
}

#[doc(hidden)]
pub mod test_support {
    pub mod oxfunc_adapter {
        pub use crate::oxfunc_adapter::*;
    }
}

pub use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalCallRequest, RegisteredExternalDescriptor,
    RegisteredExternalOriginKind, RegisteredExternalProvider, RegisteredExternalProviderError,
    RegisteredExternalTarget, RegisteredProcedureSpec,
};

pub use binding::{
    BindContext, BindDiagnostic, BindRequest, BindResult, BoundExpr, BoundFormula,
    IncrementalBindResult, NormalizedReference, ReferenceExpr, bind_formula,
    bind_formula_incremental,
};
pub use carrier::{
    CarrierRestrictionCode, CarrierValidationDisposition, ConditionalFormattingCarrierSpec,
    DataValidationCarrierSpec, FormulaCarrierValidation, validate_conditional_formatting_formula,
    validate_data_validation_formula,
};
pub use eval::{
    CallableCaptureMode, CallableDefinedNameBinding, CallableInvocationModel, CallableOriginKind,
    CallableValueCarrier, CallableValueProfile, DefinedNameBinding, EvaluationBackend,
    EvaluationContext, EvaluationError, EvaluationOutput, EvaluationTrace, PreparedArgument,
    PreparedBlanknessClass, PreparedCall, PreparedEvaluationMode, PreparedResult,
    PreparedResultClass, PreparedSourceClass, PreparedStructureClass, evaluate_formula,
};
pub use interface::{
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
    DynamicReferenceFailureContext, Extent, FenceMismatchContext, FenceSnapshot, FormatDelta,
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
    IncrementalParseResult, ParseRequest, ParseResult, parse_formula, parse_formula_incremental,
};
