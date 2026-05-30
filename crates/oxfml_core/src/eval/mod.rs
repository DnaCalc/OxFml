use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use oxfunc_core::function::ArgPreparationProfile;
use oxfunc_core::function_call::{
    FunctionCallScratch, FunctionCallTarget, FunctionExecutionContextBundle,
};
use oxfunc_core::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalCallRequest, RegisteredExternalProvider,
    parse_call_request, parse_register_id_request,
};
use oxfunc_core::functions::callable_helpers::{
    CallableInvocationBatch, CallableInvocationError, CallableInvoker,
};
use oxfunc_core::functions::cell::{CellEvalError, eval_cell_surface};
use oxfunc_core::functions::if_fn::{eval_if_surface, map_if_error_to_ws};
use oxfunc_core::functions::iferror::{eval_iferror_surface, map_iferror_error_to_ws};
use oxfunc_core::functions::image_fn::eval_image_surface_extended_with_capabilities;
use oxfunc_core::functions::info_fn::{InfoEvalError, eval_info_surface};
use oxfunc_core::functions::rand_fn::RandomProvider;
use oxfunc_core::functions::rtd_fn::RtdProvider;
use oxfunc_core::functions::surface_dispatch::{
    FUNC_ID_CALL, FUNC_ID_CELL, FUNC_ID_HSTACK, FUNC_ID_HYPERLINK, FUNC_ID_IMAGE, FUNC_ID_INDEX,
    FUNC_ID_INFO, FUNC_ID_NOW, FUNC_ID_OP_ADD, FUNC_ID_OP_CONCAT, FUNC_ID_OP_DIVIDE,
    FUNC_ID_OP_EQUAL, FUNC_ID_OP_GREATER_EQUAL, FUNC_ID_OP_GREATER_THAN,
    FUNC_ID_OP_IMPLICIT_INTERSECTION, FUNC_ID_OP_INTERSECTION_REF, FUNC_ID_OP_LESS_EQUAL,
    FUNC_ID_OP_LESS_THAN, FUNC_ID_OP_MULTIPLY, FUNC_ID_OP_NEGATE, FUNC_ID_OP_NOT_EQUAL,
    FUNC_ID_OP_PERCENT, FUNC_ID_OP_POWER, FUNC_ID_OP_RANGE_REF, FUNC_ID_OP_SPILL_REF,
    FUNC_ID_OP_SUBTRACT, FUNC_ID_OP_UNARY_PLUS, FUNC_ID_OP_UNION_REF, FUNC_ID_REGISTER_ID,
    FUNC_ID_RTD, FUNC_ID_TAKE, FUNC_ID_TODAY, FUNC_ID_XLOOKUP, eval_surface_extended_call,
};
use oxfunc_core::host_info::HostInfoProvider;
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::resolver::resolve_eval_value as resolve_oxfunc_eval_value;
use oxfunc_core::resolver::{
    CallerContext as OxFuncCallerContext, RefResolutionError, ReferenceResolver,
    ReferenceTextResolver, ResolvedReferenceValues, ResolverCapabilities,
};
use oxfunc_core::value::{
    ArrayCellValue, CallArgValue, CallableArityShape as OxCallableArityShape,
    CallableCaptureMode as OxCallableCaptureMode, CallableOriginKind as OxCallableOriginKind,
    EvalArray, EvalValue, ExcelText, ExtendedValue, LambdaValue as OxLambdaValue, ReferenceKind,
    ReferenceLike, WorksheetErrorCode,
};
use stacker::maybe_grow;

use crate::binding::{
    AreaRef, BinaryOp, BoundExpr, BoundFormula, CellRef, ErrorRef, NameKind, NameRef,
    NormalizedReference, ReferenceExpr, StructuredResolvedRef, StructuredSectionKind,
};
use crate::interface::{
    HostFunctionInvocation, HostFunctionProvider, ReturnedValueSurface, TypedContextQueryBundle,
};
use crate::semantics::{LibraryAvailabilityState, SemanticPlan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedStructureClass {
    DirectScalar,
    ArrayLike,
    ReferenceVisible,
    Omitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedSourceClass {
    Literal,
    HelperParameter,
    FunctionCall,
    CellReference,
    AreaReference,
    WholeRowReference,
    WholeColumnReference,
    NameReference,
    ExternalReference,
    SpillReference,
    ImplicitIntersection,
    BinaryExpression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedEvaluationMode {
    EagerValue,
    ReferencePreserved,
    CallerContextScalarized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedBlanknessClass {
    NonBlank,
    Omitted,
    EmptyCell,
    EmptyText,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedArgument {
    pub ordinal: usize,
    pub structure_class: PreparedStructureClass,
    pub source_class: PreparedSourceClass,
    pub evaluation_mode: PreparedEvaluationMode,
    pub blankness_class: PreparedBlanknessClass,
    pub caller_context_sensitive: bool,
    pub reference_target: Option<String>,
    pub opaque_reason: Option<String>,
    /// Value this argument resolved to before being passed to the
    /// function. `Some` for `EagerValue` / `CallerContextScalarized`
    /// args where the function actually receives a value; `None` for
    /// `ReferencePreserved` args, omitted/missing args, helper-parameter
    /// name slots, and lambda-body expression slots where the function
    /// receives the raw expression / reference rather than a resolved
    /// value.
    pub resolved_value: Option<EvalValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedCall {
    pub function_name: String,
    pub function_id: &'static str,
    pub arg_preparation_profile: ArgPreparationProfile,
    pub prepared_arguments: Vec<PreparedArgument>,
    pub register_id_request: Option<RegisterIdRequest>,
    pub registered_external_call_request: Option<RegisteredExternalCallRequest>,
    pub locale_profile_id: Option<String>,
    pub date_system: Option<String>,
    pub host_query_enabled: bool,
    /// Value this call evaluated to. `Some` when the call ran to
    /// completion (including completing with an error code); `None`
    /// when the call short-circuited before its trace entry was
    /// finalised (very rare — only LAMBDA carrier construction, which
    /// returns a registry handle rather than a computed value).
    pub returned_value: Option<EvalValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedResultClass {
    Scalar,
    Array,
    Reference,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedResult {
    pub result_class: PreparedResultClass,
    pub structure_class: PreparedStructureClass,
    pub payload_summary: String,
    pub blankness_class: PreparedBlanknessClass,
    pub reference_target: Option<String>,
    pub callable_carrier: Option<CallableValueCarrier>,
    pub callable_profile: Option<String>,
    pub callable_profile_detail: Option<CallableValueProfile>,
    pub deferred_reason: Option<String>,
    pub format_hint: Option<String>,
    pub publication_hint: Option<String>,
    pub capability_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableOriginKind {
    HelperLambda,
    DefinedNameCallable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableInvocationModel {
    TypedInvocationOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableCaptureMode {
    NoCapture,
    LexicalCapture,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableValueCarrier {
    pub origin_kind: CallableOriginKind,
    pub invocation_model: CallableInvocationModel,
    pub capture_mode: CallableCaptureMode,
    pub arity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableValueProfile {
    pub arity: usize,
    pub required_arity: usize,
    pub parameter_names: Vec<String>,
    pub optional_parameter_names: Vec<String>,
    pub capture_names: Vec<String>,
    pub body_kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallableDefinedNameBinding {
    pub summary: String,
    pub carrier: CallableValueCarrier,
    pub profile: CallableValueProfile,
    pub params: Vec<String>,
    pub optional_parameter_names: Vec<String>,
    pub body: BoundExpr,
    pub closure: BTreeMap<String, DefinedNameBinding>,
}

/// A single captured-reference dependency fact surfaced alongside a portable
/// callable result. The host uses these to build invalidation edges so that a
/// caller re-evaluates when a captured reference's value changes, and to
/// re-supply each captured value into the consuming scope before invoking the
/// callable (the body resolves captures live, not from a baked snapshot — see
/// `PortableCallableValue`). OxFml owns the free-vs-bound analysis that produces
/// these facts; the host never inspects the callable body.
#[derive(Debug, Clone, PartialEq)]
pub struct CallableCapturedRef {
    /// The free name as written in the callable body (e.g. a defined-name). For a
    /// free name that the binder lowered to a `#NAME?` error reference because it
    /// is not yet defined in the producing scope, this preserves the original
    /// name text so the host can track it as a pending dependency.
    pub name: String,
    /// A stable identity string for the captured reference, suitable as a
    /// dependency-edge key. For a defined-name (resolved or pending) this is
    /// `name:<Name>`; for other reference kinds (cell, area, ...) it mirrors
    /// `NormalizedReference`'s `Display` form.
    pub identity: String,
    /// The resolved binding for this captured name in the *defining* scope, when
    /// available. This is informational (a dependency snapshot the host may use
    /// to seed re-supply); it is NOT baked into the callable's closure. `None`
    /// means the name is free but currently unresolved in the supplied
    /// defined-name scope (e.g. a not-yet-defined name, or a non-defined-name
    /// reference such as a cell) — still a dependency the host should track so a
    /// later definition triggers re-evaluation.
    pub binding: Option<DefinedNameBinding>,
}

/// A portable callable value surfaced when a formula's top-level result is a
/// callable. The host can store `binding` opaquely and hand it back later (see
/// `set_defined_name_callable`); `captured_refs` carry the dependency facts the
/// host needs to wire invalidation edges.
///
/// Capture model (oracle-faithful): captured top-level defined names are resolved
/// at INVOCATION time, not at definition time, mirroring Excel's resolution of
/// workbook-level defined names. The producer therefore does NOT snapshot captured
/// values into `binding.closure`; it surfaces each captured-ref identity in
/// `captured_refs`. The host re-supplies the current value of each captured ref
/// into the consuming scope (alongside the re-supplied callable), where the body
/// resolves it live. This keeps the `captured_refs` invalidation edges meaningful:
/// changing a captured name re-evaluates the consumer against the new value rather
/// than against a stale baked snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct PortableCallableValue {
    pub binding: CallableDefinedNameBinding,
    pub captured_refs: Vec<CallableCapturedRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationTrace {
    pub prepared_calls: Vec<PreparedCall>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EvaluationTraceMode {
    #[default]
    ValueOnly,
    PreparedCalls,
}

const SPECIAL_LET_FUNCTION_ID: &str = "SPECIAL.LET";
const SPECIAL_LAMBDA_FUNCTION_ID: &str = "SPECIAL.LAMBDA";
const SPECIAL_LEGACY_SINGLE_FUNCTION_ID: &str = "SPECIAL.LEGACY_SINGLE";
const SPECIAL_EXTERNAL_REFERENCE_DEFERRED_FUNCTION_ID: &str = "SPECIAL.EXTERNAL_REFERENCE_DEFERRED";
const HELPER_LAMBDA_INVOCATION_CONTRACT_REF: &str = "oxfml.helper_lambda.invoke.v1";
const BUILTIN_CALLABLE_INVOCATION_CONTRACT_REF: &str = "oxfml.builtin_callable.invoke.v1";
const LOCAL_CALLABLE_RECURSION_BUDGET_UNITS: usize = 16_383;
const LOCAL_CALLABLE_RECURSION_BASE_COST_UNITS: usize = 3;
const LOCAL_CALLABLE_RECURSION_LAMBDA_ARG_COST_UNITS: usize = 1;
const LOCAL_CALLABLE_STACK_RED_ZONE_BYTES: usize = 2 * 1024 * 1024;
const LOCAL_CALLABLE_STACK_GROW_BYTES: usize = 128 * 1024 * 1024;
const LOCAL_CALLABLE_STACK_REPROBE_INTERVAL: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationBackend {
    LocalBootstrap,
    OxFuncBacked,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledFormulaPlan {
    root: CompiledExpr,
    helper_slot_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum CompiledExpr {
    NumberLiteral {
        source: String,
        value: Option<f64>,
    },
    StringLiteral(ExcelText),
    LogicalLiteral(bool),
    PrecomputedValue {
        value: EvalValue,
        source: Box<CompiledExpr>,
    },
    ArrayLiteral(Vec<Vec<CompiledExpr>>),
    OmittedArgument,
    HelperParameterName {
        name: String,
        slot: Option<usize>,
    },
    HelperOptionalParameterName {
        name: String,
        slot: Option<usize>,
    },
    Binary {
        op: BinaryOp,
        call_target: FunctionCallTarget,
        left: Box<CompiledExpr>,
        right: Box<CompiledExpr>,
    },
    Unary {
        op: crate::binding::UnaryOp,
        call_target: FunctionCallTarget,
        expr: Box<CompiledExpr>,
    },
    FunctionCall {
        function_name: String,
        call_target: CompiledFunctionCallTarget,
        args: Vec<CompiledExpr>,
    },
    ResolvedFunctionCall {
        function_name: String,
        call_target: CompiledFunctionCallTarget,
        args: Vec<CompiledExpr>,
    },
    Let {
        args: Vec<CompiledExpr>,
        slot_only: bool,
    },
    LambdaLiteral {
        args: Vec<CompiledExpr>,
    },
    If {
        args: Vec<CompiledExpr>,
    },
    IfError {
        args: Vec<CompiledExpr>,
    },
    BuiltinCallable(CompiledBuiltinCallable),
    Invocation {
        callee: Box<CompiledExpr>,
        args: Vec<CompiledExpr>,
    },
    Reference(CompiledReferenceExpr),
    ImplicitIntersection {
        call_target: FunctionCallTarget,
        expr: Box<CompiledExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum CompiledReferenceExpr {
    Atom(NormalizedReference),
    HelperLocalSlot {
        name: NameRef,
        slot: usize,
    },
    Spill {
        call_target: FunctionCallTarget,
        anchor: Box<CompiledReferenceExpr>,
    },
    Range {
        call_target: FunctionCallTarget,
        start: Box<CompiledReferenceExpr>,
        end: Box<CompiledReferenceExpr>,
    },
    Union {
        call_target: FunctionCallTarget,
        left: Box<CompiledReferenceExpr>,
        right: Box<CompiledReferenceExpr>,
    },
    Intersection {
        call_target: FunctionCallTarget,
        left: Box<CompiledReferenceExpr>,
        right: Box<CompiledReferenceExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledFunctionCallTarget {
    special_form: CompiledFunctionSpecialForm,
    function_call_target: Option<FunctionCallTarget>,
    special_operator_call_target: Option<FunctionCallTarget>,
    callable_argument_ordinals: Vec<usize>,
}

impl CompiledFunctionCallTarget {
    fn from_surface_name(surface_name: &str, argc: usize) -> Self {
        let special_form = CompiledFunctionSpecialForm::from_surface_name(surface_name);
        let function_call_target = FunctionCallTarget::from_surface_name(surface_name).ok();
        let callable_argument_ordinals = function_call_target
            .as_ref()
            .map(|call_target| call_target.callable_argument_ordinals_for_arity(argc))
            .unwrap_or_default();
        Self {
            special_form,
            function_call_target,
            special_operator_call_target: match special_form {
                CompiledFunctionSpecialForm::LegacySingle => Some(
                    function_call_target_from_function_id(FUNC_ID_OP_IMPLICIT_INTERSECTION),
                ),
                _ => None,
            },
            callable_argument_ordinals,
        }
    }

    fn function_id(&self) -> Option<&'static str> {
        self.function_call_target
            .as_ref()
            .map(FunctionCallTarget::function_id)
    }

    fn has_special_form(&self, special_form: CompiledFunctionSpecialForm) -> bool {
        self.special_form == special_form
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompiledFunctionSpecialForm {
    None,
    Let,
    Lambda,
    If,
    IfError,
    LegacySingle,
}

impl CompiledFunctionSpecialForm {
    fn from_surface_name(surface_name: &str) -> Self {
        match surface_name {
            "LET" => Self::Let,
            "LAMBDA" => Self::Lambda,
            "IF" => Self::If,
            "IFERROR" => Self::IfError,
            "_XLFN.SINGLE" | "SINGLE" => Self::LegacySingle,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledBuiltinCallable {
    call_target: FunctionCallTarget,
}

impl CompiledBuiltinCallable {
    fn from_surface_name(surface_name: &str) -> Option<Self> {
        Some(Self {
            call_target: FunctionCallTarget::from_surface_name(surface_name).ok()?,
        })
    }
}

fn function_call_target_from_function_id(function_id: &'static str) -> FunctionCallTarget {
    FunctionCallTarget::from_function_id(function_id)
        .expect("OxFunc built-in function id must resolve to a function-call target")
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationOutput {
    pub result: PreparedResult,
    pub oxfunc_value: EvalValue,
    pub returned_value_surface: ReturnedValueSurface,
    /// Set when the formula's top-level result is a callable. Carries the
    /// portable callable payload plus captured-ref dependency facts so a host can
    /// store the callable and re-supply it later. `None` for ordinary results.
    pub portable_callable: Option<PortableCallableValue>,
    pub trace: EvaluationTrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefinedNameBinding {
    Value(EvalValue),
    Reference(ReferenceLike),
    Callable(CallableDefinedNameBinding),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparseReferenceValuesBinding {
    pub reference: ReferenceLike,
    pub values: ResolvedReferenceValues,
}

#[derive(Debug, Clone, PartialEq)]
enum HelperBinding {
    Arg(CallArgValue),
    EmptyHstackCarrier(CallArgValue),
    Lambda {
        params: Rc<[LambdaParam]>,
        body: Rc<CompiledExpr>,
        closure: HelperBindingFrame,
    },
}

fn helper_name_key(name: &str) -> String {
    name.to_ascii_uppercase()
}

#[derive(Debug, Default, PartialEq)]
struct HelperBindingFrame {
    layers: Vec<Rc<BTreeMap<String, HelperBindingEntry>>>,
    slots: Rc<Vec<RefCell<Option<HelperBinding>>>>,
}

#[derive(Debug, Clone, PartialEq)]
struct HelperBindingEntry {
    display_name: String,
    slot: Option<usize>,
    binding: HelperBinding,
}

impl HelperBindingFrame {
    fn child(&self) -> Self {
        let mut layers = self.layers.clone();
        layers.push(Rc::new(BTreeMap::new()));
        Self {
            layers,
            slots: self.slots.clone(),
        }
    }

    fn with_slot_count(slot_count: usize) -> Self {
        Self {
            layers: Vec::new(),
            slots: empty_slot_cells(slot_count),
        }
    }

    fn with_min_slot_count(mut self, min_slot_count: usize) -> Self {
        if self.slots.len() >= min_slot_count {
            return self;
        }
        let mut expanded_slots = self
            .slots
            .iter()
            .map(|cell| RefCell::new(cell.borrow().clone()))
            .collect::<Vec<_>>();
        expanded_slots.extend((expanded_slots.len()..min_slot_count).map(|_| RefCell::new(None)));
        self.slots = Rc::new(expanded_slots);
        self
    }

    fn contains(&self, name: &str) -> bool {
        let key = helper_name_key(name);
        self.layers
            .iter()
            .rev()
            .any(|layer| layer.contains_key(&key))
    }

    fn get(&self, name: &str) -> Option<&HelperBinding> {
        let key = helper_name_key(name);
        self.layers
            .iter()
            .rev()
            .find_map(|layer| layer.get(&key).map(|entry| &entry.binding))
    }

    fn get_slot_clone(&self, slot: usize) -> Option<HelperBinding> {
        self.slots.get(slot).and_then(|cell| cell.borrow().clone())
    }

    fn get_mut_key(&mut self, key: &str) -> Option<&mut HelperBinding> {
        let layer_index = self
            .layers
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, layer)| layer.contains_key(key).then_some(index))?;
        Rc::make_mut(&mut self.layers[layer_index])
            .get_mut(key)
            .map(|entry| &mut entry.binding)
    }

    fn insert(&mut self, name: String, binding: HelperBinding) {
        self.insert_key(helper_name_key(&name), name, None, binding);
    }

    fn insert_key(
        &mut self,
        key: String,
        display_name: String,
        slot: Option<usize>,
        binding: HelperBinding,
    ) {
        if self.layers.is_empty() {
            self.layers.push(Rc::new(BTreeMap::new()));
        }
        if let Some(slot) = slot {
            self.set_slot(slot, binding.clone());
        }
        Rc::make_mut(
            self.layers
                .last_mut()
                .expect("helper frame has a top layer"),
        )
        .insert(
            key,
            HelperBindingEntry {
                display_name,
                slot,
                binding,
            },
        );
    }

    fn set_key_binding(&mut self, key: &str, slot: Option<usize>, binding: HelperBinding) {
        if let Some(slot) = slot {
            self.set_slot(slot, binding.clone());
        }
        if let Some(existing) = self.get_mut_key(key) {
            *existing = binding;
        } else {
            debug_assert!(
                false,
                "attempted to update helper binding key {key} before priming it"
            );
        }
    }

    fn set_slot(&self, slot: usize, binding: HelperBinding) {
        if let Some(cell) = self.slots.get(slot) {
            *cell.borrow_mut() = Some(binding);
        } else {
            debug_assert!(
                false,
                "attempted to update helper slot {slot} outside the compiled slot frame"
            );
        }
    }

    fn is_empty(&self) -> bool {
        self.layers.iter().all(|layer| layer.is_empty())
    }

    fn keys(&self) -> impl Iterator<Item = &String> {
        self.layers
            .iter()
            .flat_map(|layer| layer.values().map(|entry| &entry.display_name))
    }

    fn entries(&self) -> impl Iterator<Item = &HelperBindingEntry> {
        self.layers.iter().flat_map(|layer| layer.values())
    }
}

impl Clone for HelperBindingFrame {
    fn clone(&self) -> Self {
        Self {
            layers: self.layers.clone(),
            slots: clone_slot_cells(&self.slots),
        }
    }
}

fn empty_slot_cells(slot_count: usize) -> Rc<Vec<RefCell<Option<HelperBinding>>>> {
    Rc::new((0..slot_count).map(|_| RefCell::new(None)).collect())
}

fn clone_slot_cells(
    slots: &Rc<Vec<RefCell<Option<HelperBinding>>>>,
) -> Rc<Vec<RefCell<Option<HelperBinding>>>> {
    Rc::new(
        slots
            .iter()
            .map(|cell| RefCell::new(cell.borrow().clone()))
            .collect(),
    )
}

impl FromIterator<(String, HelperBinding)> for HelperBindingFrame {
    fn from_iter<T: IntoIterator<Item = (String, HelperBinding)>>(iter: T) -> Self {
        let mut frame = Self::default();
        for (name, binding) in iter {
            frame.insert(name, binding);
        }
        frame
    }
}

fn helper_binding_contains(helper_bindings: &HelperBindingFrame, name: &str) -> bool {
    helper_bindings.contains(name)
}

fn helper_binding_get<'a>(
    helper_bindings: &'a HelperBindingFrame,
    name: &str,
) -> Option<&'a HelperBinding> {
    helper_bindings.get(name)
}

fn insert_helper_slot_binding(
    helper_bindings: &mut HelperBindingFrame,
    name: String,
    slot: Option<usize>,
    binding: HelperBinding,
) {
    helper_bindings.insert_key(helper_name_key(&name), name, slot, binding);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LambdaParam {
    name: String,
    optional: bool,
    slot: Option<usize>,
}

fn lambda_param_from_expr(expr: &CompiledExpr) -> Option<LambdaParam> {
    match expr {
        CompiledExpr::HelperParameterName { name, slot } => Some(LambdaParam {
            name: name.clone(),
            optional: false,
            slot: *slot,
        }),
        CompiledExpr::HelperOptionalParameterName { name, slot } => Some(LambdaParam {
            name: name.clone(),
            optional: true,
            slot: *slot,
        }),
        _ => None,
    }
}

fn lambda_params_from_exprs(args: &[CompiledExpr]) -> Option<Rc<[LambdaParam]>> {
    args.iter()
        .map(lambda_param_from_expr)
        .collect::<Option<Vec<_>>>()
        .map(|params| Rc::from(params.into_boxed_slice()))
}

fn lambda_slot_count(params: &[LambdaParam], body: &CompiledExpr) -> usize {
    params
        .iter()
        .filter_map(|param| param.slot)
        .chain(compiled_expr_max_helper_slot(body))
        .max()
        .map_or(0, |slot| slot + 1)
}

fn compiled_expr_max_helper_slot(expr: &CompiledExpr) -> Option<usize> {
    match expr {
        CompiledExpr::HelperParameterName { slot, .. }
        | CompiledExpr::HelperOptionalParameterName { slot, .. } => *slot,
        CompiledExpr::Binary { left, right, .. } => compiled_expr_max_helper_slot(left)
            .into_iter()
            .chain(compiled_expr_max_helper_slot(right))
            .max(),
        CompiledExpr::Unary { expr, .. }
        | CompiledExpr::ImplicitIntersection { expr, .. }
        | CompiledExpr::PrecomputedValue { source: expr, .. } => {
            compiled_expr_max_helper_slot(expr)
        }
        CompiledExpr::ArrayLiteral(rows) => rows
            .iter()
            .flatten()
            .filter_map(compiled_expr_max_helper_slot)
            .max(),
        CompiledExpr::FunctionCall { args, .. }
        | CompiledExpr::ResolvedFunctionCall { args, .. }
        | CompiledExpr::Let { args, .. }
        | CompiledExpr::LambdaLiteral { args }
        | CompiledExpr::If { args }
        | CompiledExpr::IfError { args } => {
            args.iter().filter_map(compiled_expr_max_helper_slot).max()
        }
        CompiledExpr::Invocation { callee, args } => compiled_expr_max_helper_slot(callee)
            .into_iter()
            .chain(args.iter().filter_map(compiled_expr_max_helper_slot))
            .max(),
        CompiledExpr::Reference(reference) => compiled_reference_max_helper_slot(reference),
        CompiledExpr::NumberLiteral { .. }
        | CompiledExpr::StringLiteral(_)
        | CompiledExpr::LogicalLiteral(_)
        | CompiledExpr::OmittedArgument
        | CompiledExpr::BuiltinCallable(_) => None,
    }
}

fn compiled_reference_max_helper_slot(reference: &CompiledReferenceExpr) -> Option<usize> {
    match reference {
        CompiledReferenceExpr::HelperLocalSlot { slot, .. } => Some(*slot),
        CompiledReferenceExpr::Spill { anchor, .. } => compiled_reference_max_helper_slot(anchor),
        CompiledReferenceExpr::Range { start, end, .. }
        | CompiledReferenceExpr::Union {
            left: start,
            right: end,
            ..
        }
        | CompiledReferenceExpr::Intersection {
            left: start,
            right: end,
            ..
        } => compiled_reference_max_helper_slot(start)
            .into_iter()
            .chain(compiled_reference_max_helper_slot(end))
            .max(),
        CompiledReferenceExpr::Atom(_) => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LambdaBinding {
    origin_kind: CallableOriginKind,
    params: Rc<[LambdaParam]>,
    body: Rc<CompiledExpr>,
    closure: HelperBindingFrame,
}

#[derive(Debug, Clone, PartialEq)]
struct RegisteredCallableBinding {
    value: OxLambdaValue,
    lambda: LambdaBinding,
}

#[derive(Debug)]
struct CallableRecursionState {
    current_cost_units: usize,
    max_cost_units: usize,
}

struct CallableRecursionGuard<'a> {
    state: &'a RefCell<CallableRecursionState>,
    cost_units: usize,
}

impl Drop for CallableRecursionGuard<'_> {
    fn drop(&mut self) {
        let mut state = self.state.borrow_mut();
        state.current_cost_units = state.current_cost_units.saturating_sub(self.cost_units);
    }
}

struct CallableStackGuard<'a> {
    depth: &'a Cell<usize>,
}

impl Drop for CallableStackGuard<'_> {
    fn drop(&mut self) {
        self.depth.set(self.depth.get().saturating_sub(1));
    }
}

#[derive(Debug)]
struct EvaluationFrameState {
    function_call_scratch: RefCell<Vec<FunctionCallScratch>>,
    callable_recursion_state: RefCell<CallableRecursionState>,
    callable_stack_guard_depth: Cell<usize>,
}

impl Default for EvaluationFrameState {
    fn default() -> Self {
        Self {
            function_call_scratch: RefCell::new(Vec::new()),
            callable_recursion_state: RefCell::new(CallableRecursionState {
                current_cost_units: 0,
                max_cost_units: LOCAL_CALLABLE_RECURSION_BUDGET_UNITS,
            }),
            callable_stack_guard_depth: Cell::new(0),
        }
    }
}

struct EvaluationFrame {
    trace: EvaluationTrace,
    callable_registry: RefCell<CallableRegistry>,
    root_helper_bindings: HelperBindingFrame,
    state: Rc<EvaluationFrameState>,
}

impl EvaluationFrame {
    fn new(_plan: &CompiledFormulaPlan) -> Self {
        Self {
            trace: EvaluationTrace {
                prepared_calls: Vec::new(),
            },
            callable_registry: RefCell::new(CallableRegistry::default()),
            root_helper_bindings: HelperBindingFrame::with_slot_count(_plan.helper_slot_count),
            state: Rc::new(EvaluationFrameState::default()),
        }
    }

    fn into_trace(self) -> EvaluationTrace {
        self.trace
    }
}

struct PooledFunctionCallScratch<'a> {
    pool: &'a RefCell<Vec<FunctionCallScratch>>,
    scratch: FunctionCallScratch,
}

impl Deref for PooledFunctionCallScratch<'_> {
    type Target = FunctionCallScratch;

    fn deref(&self) -> &Self::Target {
        &self.scratch
    }
}

impl DerefMut for PooledFunctionCallScratch<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scratch
    }
}

impl Drop for PooledFunctionCallScratch<'_> {
    fn drop(&mut self) {
        let mut reusable = std::mem::take(&mut self.scratch);
        reusable.clear();
        self.pool.borrow_mut().push(reusable);
    }
}

#[derive(Debug, Default)]
struct CallableRegistry {
    next_id: usize,
    bindings: BTreeMap<String, RegisteredCallableBinding>,
    builtin_call_targets: BTreeMap<String, FunctionCallTarget>,
    /// Compiled lambda bodies keyed by the body's structural identity, so a body
    /// of a given shape compiles once per frame instead of once per invocation.
    /// The key is the body's structural `Debug` form (collision-safe via `String`
    /// equality); bodies are compiled in a fresh zero-based scope, so the cached
    /// `CompiledExpr` is binding-independent (helper slots are filled per call).
    /// This is also the dedup/sharing substrate across distinct callables that
    /// happen to share a body shape.
    compiled_body_cache: BTreeMap<String, Rc<CompiledExpr>>,
    /// O(1) fast path over `compiled_body_cache`: maps a frame-stable body address
    /// to its compiled form so a defined-name callable invoked repeatedly (the hot
    /// `B(x)` loop case) resolves its body without recomputing the structural key.
    /// Safe because host-supplied bindings live in the frame's `defined_names` for
    /// the whole evaluation and this registry is frame-local, so addresses are
    /// stable and never reused within a frame. The address is only ever compared,
    /// never dereferenced; a miss simply falls back to the structural cache.
    compiled_body_by_addr: BTreeMap<usize, Rc<CompiledExpr>>,
}

impl CallableRegistry {
    fn register(&mut self, lambda: LambdaBinding) -> OxLambdaValue {
        self.next_id += 1;
        let token = callable_token(self.next_id, &lambda_value_summary_from_binding(&lambda));
        let oxfunc_value = OxLambdaValue::new(
            token.clone(),
            oxfunc_origin_kind_from_local(lambda.origin_kind),
            OxCallableArityShape::range(lambda_required_arity(&lambda.params), lambda.params.len()),
            if lambda.closure.is_empty() {
                OxCallableCaptureMode::NoCapture
            } else {
                OxCallableCaptureMode::LexicalCapture
            },
            HELPER_LAMBDA_INVOCATION_CONTRACT_REF,
        );
        self.bindings.insert(
            token,
            RegisteredCallableBinding {
                value: oxfunc_value.clone(),
                lambda,
            },
        );
        oxfunc_value
    }

    fn get(&self, token: &str) -> Option<&RegisteredCallableBinding> {
        self.bindings.get(token)
    }

    fn value(&self, token: &str) -> Option<OxLambdaValue> {
        self.bindings
            .get(token)
            .map(|binding| binding.value.clone())
    }

    fn register_builtin_call_target(&mut self, call_target: FunctionCallTarget) -> OxLambdaValue {
        let token = format!("oxfml.builtin::{}", call_target.function_id());
        self.builtin_call_targets
            .entry(token.clone())
            .or_insert_with(|| call_target.clone());
        let meta = call_target.function_meta();
        OxLambdaValue::new(
            token,
            OxCallableOriginKind::BuiltInCallable,
            OxCallableArityShape::range(meta.arity.min, meta.arity.max),
            OxCallableCaptureMode::NoCapture,
            BUILTIN_CALLABLE_INVOCATION_CONTRACT_REF,
        )
    }

    fn builtin_call_target(&self, token: &str) -> Option<FunctionCallTarget> {
        self.builtin_call_targets.get(token).cloned()
    }

    /// Get-or-compile a lambda body keyed by its structural identity. This is the
    /// per-frame compiled-body cache that keeps a defined-name callable invoked
    /// repeatedly (e.g. `B(x)` inside `MAP(arr, LAMBDA(x, B(x)))`) from
    /// recompiling its body on every call. Keying on structural identity (not the
    /// callable name) also dedups distinct callables that share a body shape, and
    /// is the substrate for later cross-formula sharing.
    fn get_or_compile_body(&mut self, body: &BoundExpr) -> Rc<CompiledExpr> {
        let addr = body as *const BoundExpr as usize;
        if let Some(cached) = self.compiled_body_by_addr.get(&addr) {
            return cached.clone();
        }
        let key = format!("{body:?}");
        let compiled = if let Some(cached) = self.compiled_body_cache.get(&key) {
            cached.clone()
        } else {
            let mut scope = CompileHelperScope::default();
            let compiled = Rc::new(compile_expr_for_evaluation(body, &mut scope));
            self.compiled_body_cache.insert(key, compiled.clone());
            compiled
        };
        self.compiled_body_by_addr.insert(addr, compiled.clone());
        compiled
    }
}

fn compile_formula_for_evaluation(bound_formula: &BoundFormula) -> CompiledFormulaPlan {
    let mut scope = CompileHelperScope::default();
    let root =
        precompute_context_free_expr(compile_expr_for_evaluation(&bound_formula.root, &mut scope));
    CompiledFormulaPlan {
        root,
        helper_slot_count: scope.next_slot,
    }
}

#[derive(Debug, Clone, Default)]
struct CompileHelperScope {
    helper_slots: BTreeMap<String, usize>,
    next_slot: usize,
}

impl CompileHelperScope {
    fn child(&self) -> Self {
        self.clone()
    }

    fn define_helper(&mut self, name: &str) -> usize {
        let slot = self.next_slot;
        self.next_slot += 1;
        self.helper_slots.insert(helper_name_key(name), slot);
        slot
    }

    fn helper_slot(&self, name: &str) -> Option<usize> {
        self.helper_slots.get(&helper_name_key(name)).copied()
    }
}

fn compile_expr_for_evaluation(expr: &BoundExpr, scope: &mut CompileHelperScope) -> CompiledExpr {
    match expr {
        BoundExpr::NumberLiteral(text) => CompiledExpr::NumberLiteral {
            source: text.clone(),
            value: parse_excel_numeric_literal(text).ok(),
        },
        BoundExpr::StringLiteral(text) => CompiledExpr::StringLiteral(
            ExcelText::from_utf16_code_units(decode_string_literal(text).encode_utf16().collect()),
        ),
        BoundExpr::LogicalLiteral(value) => CompiledExpr::LogicalLiteral(*value),
        BoundExpr::ArrayLiteral(rows) => CompiledExpr::ArrayLiteral(
            rows.iter()
                .map(|row| {
                    row.iter()
                        .map(|expr| compile_expr_for_evaluation(expr, scope))
                        .collect()
                })
                .collect(),
        ),
        BoundExpr::OmittedArgument => CompiledExpr::OmittedArgument,
        BoundExpr::HelperParameterName(name) => CompiledExpr::HelperParameterName {
            name: name.clone(),
            slot: scope.helper_slot(name),
        },
        BoundExpr::HelperOptionalParameterName(name) => CompiledExpr::HelperOptionalParameterName {
            name: name.clone(),
            slot: scope.helper_slot(name),
        },
        BoundExpr::Binary { op, left, right } => {
            let (_, function_id) = binary_operator_identity(*op);
            CompiledExpr::Binary {
                op: *op,
                call_target: function_call_target_from_function_id(function_id),
                left: Box::new(compile_expr_for_evaluation(left, scope)),
                right: Box::new(compile_expr_for_evaluation(right, scope)),
            }
        }
        BoundExpr::Unary { op, expr } => {
            let (_, function_id) = unary_operator_identity(*op);
            CompiledExpr::Unary {
                op: *op,
                call_target: function_call_target_from_function_id(function_id),
                expr: Box::new(compile_expr_for_evaluation(expr, scope)),
            }
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } => {
            let call_target =
                CompiledFunctionCallTarget::from_surface_name(function_name, args.len());
            if call_target.has_special_form(CompiledFunctionSpecialForm::Let) {
                return compile_let_call_for_evaluation(args, scope);
            }
            if call_target.has_special_form(CompiledFunctionSpecialForm::Lambda) {
                return compile_lambda_call_for_evaluation(args, scope);
            }
            if call_target.has_special_form(CompiledFunctionSpecialForm::If) {
                return CompiledExpr::If {
                    args: args
                        .iter()
                        .map(|arg| compile_expr_for_evaluation(arg, scope))
                        .collect(),
                };
            }
            if call_target.has_special_form(CompiledFunctionSpecialForm::IfError) {
                return CompiledExpr::IfError {
                    args: args
                        .iter()
                        .map(|arg| compile_expr_for_evaluation(arg, scope))
                        .collect(),
                };
            }
            let args = args
                .iter()
                .enumerate()
                .map(|(ordinal, arg)| {
                    if call_target.callable_argument_ordinals.contains(&ordinal) {
                        compile_callable_slot_expr_for_evaluation(arg, scope)
                    } else {
                        compile_expr_for_evaluation(arg, scope)
                    }
                })
                .collect();
            if call_target.has_special_form(CompiledFunctionSpecialForm::None)
                && call_target.function_call_target.is_some()
            {
                return CompiledExpr::ResolvedFunctionCall {
                    function_name: function_name.clone(),
                    call_target,
                    args,
                };
            }
            CompiledExpr::FunctionCall {
                function_name: function_name.clone(),
                call_target,
                args,
            }
        }
        BoundExpr::Invocation { callee, args } => CompiledExpr::Invocation {
            callee: Box::new(compile_expr_for_evaluation(callee, scope)),
            args: args
                .iter()
                .map(|arg| compile_expr_for_evaluation(arg, scope))
                .collect(),
        },
        BoundExpr::Reference(reference) => {
            CompiledExpr::Reference(compile_reference_for_evaluation(reference, scope))
        }
        BoundExpr::ImplicitIntersection(inner) => CompiledExpr::ImplicitIntersection {
            call_target: function_call_target_from_function_id(FUNC_ID_OP_IMPLICIT_INTERSECTION),
            expr: Box::new(compile_expr_for_evaluation(inner, scope)),
        },
    }
}

fn compile_let_call_for_evaluation(
    args: &[BoundExpr],
    scope: &mut CompileHelperScope,
) -> CompiledExpr {
    let mut let_scope = scope.child();
    let last_index = args.len().saturating_sub(1);
    let mut compiled_args = Vec::with_capacity(args.len());
    let mut index = 0usize;
    while index < last_index {
        let value_expr = compile_expr_for_evaluation(&args[index + 1], &mut let_scope);
        let binding_expr = match &args[index] {
            BoundExpr::HelperParameterName(name) => {
                let slot = let_scope.define_helper(name);
                CompiledExpr::HelperParameterName {
                    name: name.clone(),
                    slot: Some(slot),
                }
            }
            arg => compile_expr_for_evaluation(arg, &mut let_scope),
        };
        compiled_args.push(binding_expr);
        compiled_args.push(value_expr);
        index += 2;
    }
    if last_index < args.len() {
        compiled_args.push(compile_expr_for_evaluation(
            &args[last_index],
            &mut let_scope,
        ));
    }
    scope.next_slot = scope.next_slot.max(let_scope.next_slot);
    let slot_only = compiled_let_args_are_slot_only(&compiled_args);
    CompiledExpr::Let {
        args: compiled_args,
        slot_only,
    }
}

fn compiled_let_args_are_slot_only(args: &[CompiledExpr]) -> bool {
    if args.len() < 3 {
        return false;
    }
    let last_index = args.len() - 1;
    let mut index = 0usize;
    while index < last_index {
        if !matches!(
            args.get(index),
            Some(CompiledExpr::HelperParameterName { slot: Some(_), .. })
        ) {
            return false;
        }
        index += 2;
    }
    !args.iter().any(compiled_expr_contains_lambda_literal)
}

fn compiled_expr_contains_lambda_literal(expr: &CompiledExpr) -> bool {
    match expr {
        CompiledExpr::LambdaLiteral { .. } => true,
        CompiledExpr::PrecomputedValue { source, .. } => {
            compiled_expr_contains_lambda_literal(source)
        }
        CompiledExpr::ArrayLiteral(rows) => rows
            .iter()
            .flatten()
            .any(compiled_expr_contains_lambda_literal),
        CompiledExpr::Binary { left, right, .. } => {
            compiled_expr_contains_lambda_literal(left)
                || compiled_expr_contains_lambda_literal(right)
        }
        CompiledExpr::Unary { expr, .. } | CompiledExpr::ImplicitIntersection { expr, .. } => {
            compiled_expr_contains_lambda_literal(expr)
        }
        CompiledExpr::FunctionCall { args, .. }
        | CompiledExpr::ResolvedFunctionCall { args, .. }
        | CompiledExpr::Let { args, .. }
        | CompiledExpr::If { args }
        | CompiledExpr::IfError { args } => args.iter().any(compiled_expr_contains_lambda_literal),
        CompiledExpr::Invocation { callee, args } => {
            compiled_expr_contains_lambda_literal(callee)
                || args.iter().any(compiled_expr_contains_lambda_literal)
        }
        _ => false,
    }
}

fn compile_lambda_call_for_evaluation(
    args: &[BoundExpr],
    scope: &mut CompileHelperScope,
) -> CompiledExpr {
    let body_index = args.len().saturating_sub(1);
    let mut lambda_scope = scope.child();
    let mut compiled_args = Vec::with_capacity(args.len());
    for arg in &args[..body_index] {
        match arg {
            BoundExpr::HelperParameterName(name) => {
                let slot = lambda_scope.define_helper(name);
                compiled_args.push(CompiledExpr::HelperParameterName {
                    name: name.clone(),
                    slot: Some(slot),
                });
            }
            BoundExpr::HelperOptionalParameterName(name) => {
                let slot = lambda_scope.define_helper(name);
                compiled_args.push(CompiledExpr::HelperOptionalParameterName {
                    name: name.clone(),
                    slot: Some(slot),
                });
            }
            _ => compiled_args.push(compile_expr_for_evaluation(arg, &mut lambda_scope)),
        }
    }
    if body_index < args.len() {
        compiled_args.push(compile_expr_for_evaluation(
            &args[body_index],
            &mut lambda_scope,
        ));
    }
    scope.next_slot = scope.next_slot.max(lambda_scope.next_slot);
    CompiledExpr::LambdaLiteral {
        args: compiled_args,
    }
}

fn compile_reference_for_evaluation(
    reference: &ReferenceExpr,
    scope: &mut CompileHelperScope,
) -> CompiledReferenceExpr {
    match reference {
        ReferenceExpr::Atom(NormalizedReference::Name(name))
            if matches!(name.kind, NameKind::HelperLocal) =>
        {
            if let Some(slot) = scope.helper_slot(&name.name) {
                CompiledReferenceExpr::HelperLocalSlot {
                    name: name.clone(),
                    slot,
                }
            } else {
                CompiledReferenceExpr::Atom(NormalizedReference::Name(name.clone()))
            }
        }
        ReferenceExpr::Atom(atom) => CompiledReferenceExpr::Atom(atom.clone()),
        ReferenceExpr::Spill { anchor } => CompiledReferenceExpr::Spill {
            call_target: function_call_target_from_function_id(FUNC_ID_OP_SPILL_REF),
            anchor: Box::new(compile_reference_for_evaluation(anchor, scope)),
        },
        ReferenceExpr::Range { start, end } => CompiledReferenceExpr::Range {
            call_target: function_call_target_from_function_id(FUNC_ID_OP_RANGE_REF),
            start: Box::new(compile_reference_for_evaluation(start, scope)),
            end: Box::new(compile_reference_for_evaluation(end, scope)),
        },
        ReferenceExpr::Union { left, right } => CompiledReferenceExpr::Union {
            call_target: function_call_target_from_function_id(FUNC_ID_OP_UNION_REF),
            left: Box::new(compile_reference_for_evaluation(left, scope)),
            right: Box::new(compile_reference_for_evaluation(right, scope)),
        },
        ReferenceExpr::Intersection { left, right } => CompiledReferenceExpr::Intersection {
            call_target: function_call_target_from_function_id(FUNC_ID_OP_INTERSECTION_REF),
            left: Box::new(compile_reference_for_evaluation(left, scope)),
            right: Box::new(compile_reference_for_evaluation(right, scope)),
        },
    }
}

fn precompute_context_free_expr(expr: CompiledExpr) -> CompiledExpr {
    let expr = match expr {
        CompiledExpr::ArrayLiteral(rows) => CompiledExpr::ArrayLiteral(
            rows.into_iter()
                .map(|row| row.into_iter().map(precompute_context_free_expr).collect())
                .collect(),
        ),
        CompiledExpr::Binary {
            op,
            call_target,
            left,
            right,
        } => CompiledExpr::Binary {
            op,
            call_target,
            left: Box::new(precompute_context_free_expr(*left)),
            right: Box::new(precompute_context_free_expr(*right)),
        },
        CompiledExpr::Unary {
            op,
            call_target,
            expr,
        } => CompiledExpr::Unary {
            op,
            call_target,
            expr: Box::new(precompute_context_free_expr(*expr)),
        },
        CompiledExpr::FunctionCall {
            function_name,
            call_target,
            args,
        } => CompiledExpr::FunctionCall {
            function_name,
            call_target,
            args: args.into_iter().map(precompute_context_free_expr).collect(),
        },
        CompiledExpr::ResolvedFunctionCall {
            function_name,
            call_target,
            args,
        } => CompiledExpr::ResolvedFunctionCall {
            function_name,
            call_target,
            args: args.into_iter().map(precompute_context_free_expr).collect(),
        },
        CompiledExpr::Let { args, slot_only } => CompiledExpr::Let {
            args: args.into_iter().map(precompute_context_free_expr).collect(),
            slot_only,
        },
        CompiledExpr::If { args } => CompiledExpr::If {
            args: args.into_iter().map(precompute_context_free_expr).collect(),
        },
        CompiledExpr::IfError { args } => CompiledExpr::IfError {
            args: args.into_iter().map(precompute_context_free_expr).collect(),
        },
        CompiledExpr::Invocation { callee, args } => CompiledExpr::Invocation {
            callee: Box::new(precompute_context_free_expr(*callee)),
            args: args.into_iter().map(precompute_context_free_expr).collect(),
        },
        CompiledExpr::ImplicitIntersection { call_target, expr } => {
            CompiledExpr::ImplicitIntersection {
                call_target,
                expr: Box::new(precompute_context_free_expr(*expr)),
            }
        }
        CompiledExpr::LambdaLiteral { args } => CompiledExpr::LambdaLiteral {
            args: args.into_iter().map(precompute_context_free_expr).collect(),
        },
        other => other,
    };

    if expr_can_be_precomputed(&expr) {
        if let Some(value) = context_free_eval_value_for_expr(&expr) {
            return CompiledExpr::PrecomputedValue {
                value,
                source: Box::new(expr),
            };
        }
    }
    expr
}

fn expr_can_be_precomputed(expr: &CompiledExpr) -> bool {
    match expr {
        CompiledExpr::Binary {
            call_target,
            left,
            right,
            ..
        } => {
            call_target.is_context_free_pure()
                && expr_is_context_free_value(left)
                && expr_is_context_free_value(right)
        }
        CompiledExpr::Unary {
            call_target, expr, ..
        } => call_target.is_context_free_pure() && expr_is_context_free_value(expr),
        CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } => {
            call_target
                .function_call_target
                .as_ref()
                .is_some_and(FunctionCallTarget::is_context_free_pure)
                && args.iter().all(expr_is_context_free_value)
        }
        _ => false,
    }
}

fn expr_is_context_free_value(expr: &CompiledExpr) -> bool {
    match expr {
        CompiledExpr::NumberLiteral { value, .. } => value.is_some(),
        CompiledExpr::StringLiteral(_) | CompiledExpr::LogicalLiteral(_) => true,
        CompiledExpr::PrecomputedValue { .. } => true,
        _ => expr_can_be_precomputed(expr),
    }
}

fn context_free_eval_value_for_expr(expr: &CompiledExpr) -> Option<EvalValue> {
    match expr {
        CompiledExpr::NumberLiteral { value, .. } => value.map(EvalValue::Number),
        CompiledExpr::StringLiteral(text) => Some(EvalValue::Text(text.clone())),
        CompiledExpr::LogicalLiteral(value) => Some(EvalValue::Logical(*value)),
        CompiledExpr::PrecomputedValue { value, .. } => Some(value.clone()),
        CompiledExpr::Binary {
            call_target,
            left,
            right,
            ..
        } if call_target.is_context_free_pure() => {
            let args = [
                context_free_call_arg_for_expr(left)?,
                context_free_call_arg_for_expr(right)?,
            ];
            Some(invoke_context_free_function_call(call_target, &args))
        }
        CompiledExpr::Unary {
            call_target, expr, ..
        } if call_target.is_context_free_pure() => {
            let args = [context_free_call_arg_for_expr(expr)?];
            Some(invoke_context_free_function_call(call_target, &args))
        }
        CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } => {
            let function_call_target = call_target.function_call_target.as_ref()?;
            if !function_call_target.is_context_free_pure() {
                return None;
            }
            let mut call_args = args
                .iter()
                .map(context_free_call_arg_for_expr)
                .collect::<Option<Vec<_>>>()?;
            let trailing_omitted_count = args
                .iter()
                .rev()
                .take_while(|arg| matches!(arg, CompiledExpr::OmittedArgument))
                .count();
            for _ in 0..trailing_omitted_count {
                if !matches!(call_args.last(), Some(CallArgValue::MissingArg)) {
                    break;
                }
                call_args.pop();
            }
            if function_call_target.function_meta().function_id == FUNC_ID_HSTACK
                && args.iter().zip(call_args.iter()).any(|(arg, call_arg)| {
                    hstack_arg_should_collapse(arg, call_arg, &HelperBindingFrame::default())
                })
            {
                return Some(EvalValue::Error(WorksheetErrorCode::Calc));
            }
            Some(invoke_context_free_function_call(
                function_call_target,
                &call_args,
            ))
        }
        _ => None,
    }
}

fn context_free_call_arg_for_expr(expr: &CompiledExpr) -> Option<CallArgValue> {
    match expr {
        CompiledExpr::OmittedArgument => Some(CallArgValue::MissingArg),
        _ => context_free_eval_value_for_expr(expr).map(CallArgValue::Eval),
    }
}

fn invoke_context_free_function_call(
    call_target: &FunctionCallTarget,
    args: &[CallArgValue],
) -> EvalValue {
    let cell_values = BTreeMap::new();
    let defined_names = BTreeMap::new();
    let sparse_reference_values = BTreeMap::new();
    let callable_registry = RefCell::new(CallableRegistry::default());
    let resolver = LocalReferenceResolver {
        cell_values: &cell_values,
        defined_names: &defined_names,
        sparse_reference_values: &sparse_reference_values,
        caller_row: 1,
        caller_col: 1,
        callable_registry: &callable_registry,
    };
    let mut fec = FunctionExecutionContextBundle::new(&resolver);
    match call_target.invoke(args, &mut fec) {
        Ok(value) => value,
        Err(code) => EvalValue::Error(code),
    }
}

fn compile_callable_slot_expr_for_evaluation(
    expr: &BoundExpr,
    scope: &mut CompileHelperScope,
) -> CompiledExpr {
    if let Some(callable) = compile_builtin_callable_for_slot(expr) {
        return CompiledExpr::BuiltinCallable(callable);
    }

    compile_expr_for_evaluation(expr, scope)
}

fn compile_builtin_callable_for_slot(expr: &BoundExpr) -> Option<CompiledBuiltinCallable> {
    match expr {
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(error))) => {
            CompiledBuiltinCallable::from_surface_name(&error.source_text)
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if args.is_empty() => CompiledBuiltinCallable::from_surface_name(function_name),
        _ => None,
    }
}

pub struct EvaluationContext<'a> {
    pub bind_formula: &'a BoundFormula,
    pub plan: &'a SemanticPlan,
    compiled_plan: CompiledFormulaPlan,
    pub backend: EvaluationBackend,
    pub caller_row: usize,
    pub caller_col: usize,
    pub cell_values: BTreeMap<String, EvalValue>,
    pub defined_names: BTreeMap<String, DefinedNameBinding>,
    pub sparse_reference_values: BTreeMap<String, SparseReferenceValuesBinding>,
    pub locale_ctx: Option<&'a LocaleFormatContext<'a>>,
    pub host_info: Option<&'a dyn HostInfoProvider>,
    pub rtd_provider: Option<&'a dyn RtdProvider>,
    pub registered_external_provider: Option<&'a dyn RegisteredExternalProvider>,
    pub host_function_provider: Option<&'a dyn HostFunctionProvider>,
    pub now_serial: Option<f64>,
    pub random_provider: Option<&'a dyn RandomProvider>,
    pub reference_text_resolver: Option<&'a dyn ReferenceTextResolver>,
    pub trace_mode: EvaluationTraceMode,
    frame_state: Rc<EvaluationFrameState>,
}

impl<'a> EvaluationContext<'a> {
    pub fn new(bind_formula: &'a BoundFormula, plan: &'a SemanticPlan) -> Self {
        Self {
            bind_formula,
            plan,
            compiled_plan: compile_formula_for_evaluation(bind_formula),
            backend: EvaluationBackend::OxFuncBacked,
            caller_row: 1,
            caller_col: 1,
            cell_values: BTreeMap::new(),
            defined_names: BTreeMap::new(),
            sparse_reference_values: BTreeMap::new(),
            locale_ctx: None,
            host_info: None,
            rtd_provider: None,
            registered_external_provider: None,
            host_function_provider: None,
            now_serial: None,
            random_provider: None,
            reference_text_resolver: None,
            trace_mode: EvaluationTraceMode::default(),
            frame_state: Rc::new(EvaluationFrameState::default()),
        }
    }

    pub fn typed_context_query_bundle(&self) -> TypedContextQueryBundle<'a> {
        TypedContextQueryBundle::new(
            self.host_info,
            self.rtd_provider,
            self.locale_ctx,
            self.now_serial,
            self.random_provider,
        )
        .with_registered_external_provider(self.registered_external_provider)
        .with_host_function_provider(self.host_function_provider)
        .with_reference_text_resolver(self.reference_text_resolver)
    }

    pub fn apply_typed_context_query_bundle(&mut self, bundle: TypedContextQueryBundle<'a>) {
        self.host_info = bundle.host_info;
        self.rtd_provider = bundle.rtd_provider;
        self.registered_external_provider = bundle.registered_external_provider;
        self.host_function_provider = bundle.host_function_provider;
        self.locale_ctx = bundle.locale_ctx;
        self.now_serial = bundle.now_serial;
        self.random_provider = bundle.random_provider;
        self.reference_text_resolver = bundle.reference_text_resolver;
    }

    fn has_sparse_reference_values(&self, reference: &ReferenceLike) -> bool {
        self.sparse_reference_values
            .get(&reference.target)
            .is_some_and(|binding| binding.reference == *reference)
    }

    pub fn with_trace_mode(mut self, trace_mode: EvaluationTraceMode) -> Self {
        self.trace_mode = trace_mode;
        self
    }

    pub fn set_trace_mode(&mut self, trace_mode: EvaluationTraceMode) {
        self.trace_mode = trace_mode;
    }

    fn with_frame_state(mut self, frame_state: Rc<EvaluationFrameState>) -> Self {
        self.frame_state = frame_state;
        self
    }

    fn records_prepared_calls(&self) -> bool {
        matches!(self.trace_mode, EvaluationTraceMode::PreparedCalls)
    }

    fn enter_callable_stack_guard(&self) -> CallableStackGuard<'_> {
        self.frame_state
            .callable_stack_guard_depth
            .set(self.frame_state.callable_stack_guard_depth.get() + 1);
        CallableStackGuard {
            depth: &self.frame_state.callable_stack_guard_depth,
        }
    }

    fn take_function_call_scratch(&self, capacity: usize) -> PooledFunctionCallScratch<'_> {
        let mut scratch = self
            .frame_state
            .function_call_scratch
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| FunctionCallScratch::with_capacity(capacity));
        scratch.clear();
        let current_capacity = scratch.capacity();
        if current_capacity < capacity {
            scratch
                .call_args_mut()
                .reserve(capacity.saturating_sub(current_capacity));
        }
        PooledFunctionCallScratch {
            pool: &self.frame_state.function_call_scratch,
            scratch,
        }
    }
}

fn with_callable_stack_guard<T>(context: &EvaluationContext<'_>, f: impl FnOnce() -> T) -> T {
    let depth = context.frame_state.callable_stack_guard_depth.get();
    // Avoid stacker probes on every helper-loop iteration, but keep periodic
    // checks for genuinely deep recursive lambda chains.
    if depth > 0 && depth % LOCAL_CALLABLE_STACK_REPROBE_INTERVAL != 0 {
        let _guard = context.enter_callable_stack_guard();
        return f();
    }

    maybe_grow(
        LOCAL_CALLABLE_STACK_RED_ZONE_BYTES,
        LOCAL_CALLABLE_STACK_GROW_BYTES,
        || {
            let _guard = context.enter_callable_stack_guard();
            f()
        },
    )
}

pub fn evaluate_formula(
    context: EvaluationContext<'_>,
) -> Result<EvaluationOutput, EvaluationError> {
    let mut frame = EvaluationFrame::new(&context.compiled_plan);
    let context = context.with_frame_state(frame.state.clone());
    let mut resolver = LocalReferenceResolver {
        cell_values: &context.cell_values,
        defined_names: &context.defined_names,
        sparse_reference_values: &context.sparse_reference_values,
        caller_row: context.caller_row,
        caller_col: context.caller_col,
        callable_registry: &frame.callable_registry,
    };

    let value = evaluate_root_expr_value(
        &context.compiled_plan.root,
        &context,
        &mut resolver,
        &frame.root_helper_bindings,
        &frame.callable_registry,
        &mut frame.trace,
    )?;
    let value = dereference_final_output_value(value, &mut resolver)?;
    let output_value = sanitize_final_output_value(value, &frame.callable_registry);
    let portable_callable = portable_callable_for_top_level_result(
        &output_value,
        context.bind_formula,
        &context.defined_names,
    );

    Ok(EvaluationOutput {
        result: prepared_result_from_eval_value(&output_value, context.plan),
        returned_value_surface: returned_value_surface_for_output(
            &context.compiled_plan.root,
            &output_value,
            &context,
        ),
        oxfunc_value: output_value,
        portable_callable,
        trace: frame.into_trace(),
    })
}

/// Build a portable callable payload when the formula's top-level result is a
/// callable (`EvalValue::Lambda`) and the bound root is a `LAMBDA(...)` literal.
///
/// Coverage note (fml-ds0.20 narrowed AC, follow-up filed as fml-ds0.20.1): this
/// path intentionally covers only a bound root that is a bare `LAMBDA(...)`
/// literal. A callable produced via `LET`/`IF`/currying (e.g.
/// `=LET(f, LAMBDA(x,x+1), f)`) does NOT yield a portable payload here, because
/// its portable `body: BoundExpr` is not statically derivable from the bound
/// root without fabrication; those forms still publish `Error(Calc)`.
///
/// Two distinct concerns are kept separate so the surfaces never contradict each
/// other:
///   * Descriptive metadata (`carrier.capture_mode`, `profile.capture_names`,
///     `profile.body_kind`, `summary`) is derived from the runtime
///     `OxLambdaValue` — the evaluator's already-computed source of truth — via
///     the same helpers that feed `callable_profile_detail`/`callable_carrier`.
///     This guarantees the portable surface agrees with
///     `evaluation.result.callable_*` for the same value. In particular, a free
///     top-level defined-name (e.g. `Cap` in `=LAMBDA(x,MIN(x,Cap))`) is NOT a
///     lexical capture: the engine resolves it live at invocation time, so the
///     runtime reports `NoCapture` and the portable metadata follows suit.
///   * The re-supply payload (`params`, `optional_parameter_names`, `body`) is
///     read straight from the bound LAMBDA expression so the host can store and
///     hand the callable back. `closure` is left EMPTY for top-level
///     defined-name captures: baking a snapshot would shadow the consumer's live
///     value on re-supply and make the `captured_refs` invalidation edges
///     meaningless. The host re-supplies the current value of each captured ref
///     (tracked via `captured_refs`) into the consuming scope, where the body
///     resolves it live — matching Excel's invocation-time name resolution.
///
/// `captured_refs` always carries the captured-ref dependency identities (the
/// free names in the body that are not bound parameters) so the host can build
/// invalidation edges, including for names that are currently unresolved.
///
/// Returns `None` for non-callable results, or for callable results whose bound
/// root is not a directly portable LAMBDA literal (e.g. a curried or
/// LET-returned callable): we never fabricate a body we cannot honestly derive.
fn portable_callable_for_top_level_result(
    output_value: &EvalValue,
    bind_formula: &BoundFormula,
    defined_names: &BTreeMap<String, DefinedNameBinding>,
) -> Option<PortableCallableValue> {
    let EvalValue::Lambda(runtime_lambda) = output_value else {
        return None;
    };
    let (params, optional_params, body) = lambda_literal_params_and_body(&bind_formula.root)?;

    let bound_param_keys = params
        .iter()
        .chain(optional_params.iter())
        .map(|name| helper_name_key(name))
        .collect::<BTreeSet<_>>();
    let mut free_refs = Vec::new();
    collect_free_bound_references(body, &bound_param_keys, &mut free_refs);

    let captured_refs = free_refs
        .into_iter()
        .map(|reference| {
            let (name, identity) = captured_ref_name_and_identity(&reference);
            // The captured-ref *identity* is a dependency fact for the host; we do
            // NOT bake the captured value into `closure`. Top-level defined names
            // are re-resolved live at invocation time (see closure note below).
            let binding = match &reference {
                NormalizedReference::Name(name_ref) => defined_names.get(&name_ref.name).cloned(),
                _ => None,
            };
            CallableCapturedRef {
                name,
                identity,
                binding,
            }
        })
        .collect::<Vec<_>>();

    // Descriptive metadata mirrors the runtime lambda (the evaluator's source of
    // truth) so the portable surface never contradicts `evaluation.result`. The
    // runtime carrier/profile is the SAME data the runtime result publishes for an
    // `EvalValue::Lambda` (see PreparedResult for `EvalValue::Lambda`:
    // `callable_carrier`/`callable_profile_detail`). If either cannot be derived
    // we treat the result as not portable (return None) rather than fabricate a
    // fallback, so the portable surface can never silently diverge from
    // `evaluation.result.callable_*`.
    let carrier = callable_carrier_from_lambda_value(runtime_lambda)?;
    let profile = callable_profile_detail_from_lambda_value(runtime_lambda)?;
    let summary = lambda_summary(runtime_lambda).to_string();

    // The re-supply payload carries params/body verbatim from the bound AST. The
    // closure is intentionally empty: captured top-level defined names travel as
    // `captured_refs` identities and are re-resolved live by the consuming host,
    // not snapshotted here (which would shadow the consumer's live value).
    let closure = BTreeMap::new();

    let binding = CallableDefinedNameBinding {
        summary,
        carrier,
        profile,
        params,
        optional_parameter_names: optional_params,
        body: body.clone(),
        closure,
    };

    Some(PortableCallableValue {
        binding,
        captured_refs,
    })
}

/// Derive the host-facing `(name, identity)` for a captured free reference.
///
/// For a resolved defined-name we use the name verbatim and a `name:<Name>`
/// identity (matching `NormalizedReference`'s `Display`). For a free name that
/// the binder lowered to a `#NAME?` error reference (because it is not yet
/// defined in the producing scope), we recover the original name text from the
/// error's `source_text` so the host can still wire a pending invalidation edge
/// keyed on the name that will fire when the name later becomes defined. Other
/// reference kinds (cells, areas, ...) fall back to their `Display` identity.
fn captured_ref_name_and_identity(reference: &NormalizedReference) -> (String, String) {
    match reference {
        NormalizedReference::Name(name) => (name.name.clone(), format!("name:{}", name.name)),
        NormalizedReference::Error(error)
            if error.error_class == "#NAME?" && is_identifier_like(&error.source_text) =>
        {
            // An unresolved free name: surface the pending dependency by name so
            // the host can track it until the name becomes defined.
            (
                error.source_text.clone(),
                format!("name:{}", error.source_text),
            )
        }
        other => {
            let identity = other.to_string();
            (identity.clone(), identity)
        }
    }
}

/// Whether `text` is a plausible defined-name identifier (as opposed to a
/// synthetic error source-text like "range"/"union"/"intersection" or an empty
/// string). Used to distinguish an unresolved free *name* from other `#NAME?`
/// error references.
fn is_identifier_like(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.')
}

/// Extract `(required params, optional params, body)` from a bound `LAMBDA(...)`
/// literal expression, or `None` if the expression is not a LAMBDA literal.
fn lambda_literal_params_and_body(
    expr: &BoundExpr,
) -> Option<(Vec<String>, Vec<String>, &BoundExpr)> {
    let BoundExpr::FunctionCall {
        function_name,
        args,
    } = expr
    else {
        return None;
    };
    if !function_name.eq_ignore_ascii_case("LAMBDA") || args.is_empty() {
        return None;
    }
    let body_index = args.len() - 1;
    let mut params = Vec::new();
    let mut optional_params = Vec::new();
    for arg in &args[..body_index] {
        match arg {
            BoundExpr::HelperParameterName(name) => params.push(name.clone()),
            BoundExpr::HelperOptionalParameterName(name) => {
                params.push(name.clone());
                optional_params.push(name.clone());
            }
            // A non-parameter in the head means this is not a plain LAMBDA literal
            // we can portably describe; bail rather than guess.
            _ => return None,
        }
    }
    Some((params, optional_params, &args[body_index]))
}

/// Collect the free references in a bound expression that are not shadowed by the
/// supplied bound-parameter name keys. Mirrors the helper free-name analysis but
/// operates on the portable `BoundExpr` so the result is host-facing.
fn collect_free_bound_references(
    expr: &BoundExpr,
    bound_names: &BTreeSet<String>,
    out: &mut Vec<NormalizedReference>,
) {
    match expr {
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_) => {}
        BoundExpr::ArrayLiteral(rows) => {
            for row in rows {
                for cell in row {
                    collect_free_bound_references(cell, bound_names, out);
                }
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_free_bound_references(left, bound_names, out);
            collect_free_bound_references(right, bound_names, out);
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_free_bound_references(expr, bound_names, out);
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } => {
            if function_name.eq_ignore_ascii_case("LAMBDA") && !args.is_empty() {
                collect_free_bound_references_in_nested_lambda(args, bound_names, out);
                return;
            }
            if function_name.eq_ignore_ascii_case("LET") && args.len() >= 3 {
                collect_free_bound_references_in_let(args, bound_names, out);
                return;
            }
            for arg in args {
                collect_free_bound_references(arg, bound_names, out);
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_free_bound_references(callee, bound_names, out);
            for arg in args {
                collect_free_bound_references(arg, bound_names, out);
            }
        }
        BoundExpr::Reference(reference) => {
            collect_free_reference_atoms(reference, bound_names, out);
        }
    }
}

fn collect_free_bound_references_in_nested_lambda(
    args: &[BoundExpr],
    bound_names: &BTreeSet<String>,
    out: &mut Vec<NormalizedReference>,
) {
    let body_index = args.len() - 1;
    let mut nested_bound = bound_names.clone();
    for arg in &args[..body_index] {
        match arg {
            BoundExpr::HelperParameterName(name) | BoundExpr::HelperOptionalParameterName(name) => {
                nested_bound.insert(helper_name_key(name));
            }
            _ => {}
        }
    }
    collect_free_bound_references(&args[body_index], &nested_bound, out);
}

fn collect_free_bound_references_in_let(
    args: &[BoundExpr],
    bound_names: &BTreeSet<String>,
    out: &mut Vec<NormalizedReference>,
) {
    let last_index = args.len() - 1;
    let mut local_bound = bound_names.clone();
    let mut index = 0usize;
    while index + 1 < last_index {
        collect_free_bound_references(&args[index + 1], &local_bound, out);
        if let BoundExpr::HelperParameterName(name) = &args[index] {
            local_bound.insert(helper_name_key(name));
        }
        index += 2;
    }
    collect_free_bound_references(&args[last_index], &local_bound, out);
}

fn collect_free_reference_atoms(
    reference: &ReferenceExpr,
    bound_names: &BTreeSet<String>,
    out: &mut Vec<NormalizedReference>,
) {
    match reference {
        ReferenceExpr::Atom(atom) => {
            // A helper-local name shadowed by a bound parameter is not a free
            // captured reference; skip it.
            if let NormalizedReference::Name(name) = atom
                && matches!(name.kind, NameKind::HelperLocal)
                && bound_names.contains(&helper_name_key(&name.name))
            {
                return;
            }
            if !out.contains(atom) {
                out.push(atom.clone());
            }
        }
        ReferenceExpr::Spill { anchor } => {
            collect_free_reference_atoms(anchor, bound_names, out);
        }
        ReferenceExpr::Range { start, end }
        | ReferenceExpr::Union {
            left: start,
            right: end,
        }
        | ReferenceExpr::Intersection {
            left: start,
            right: end,
        } => {
            collect_free_reference_atoms(start, bound_names, out);
            collect_free_reference_atoms(end, bound_names, out);
        }
    }
}

fn returned_value_surface_for_output(
    root: &CompiledExpr,
    value: &EvalValue,
    context: &EvaluationContext<'_>,
) -> ReturnedValueSurface {
    if let Some(surface) = typed_surface_for_top_level_host_or_provider_call(root, context) {
        return surface;
    }

    if let Some(surface) = extended_surface_for_top_level_function_call(root, context) {
        return surface;
    }

    ReturnedValueSurface::from_extended_value(&ExtendedValue::Core(value.clone()))
}

fn dereference_final_output_value(
    value: EvalValue,
    resolver: &mut LocalReferenceResolver<'_>,
) -> Result<EvalValue, EvaluationError> {
    match value {
        EvalValue::Reference(reference) => {
            resolve_oxfunc_eval_value(resolver, &reference).map_err(map_resolution_error)
        }
        other => Ok(other),
    }
}

fn typed_surface_for_top_level_host_or_provider_call(
    root: &CompiledExpr,
    context: &EvaluationContext<'_>,
) -> Option<ReturnedValueSurface> {
    match root {
        CompiledExpr::FunctionCall {
            call_target, args, ..
        }
        | CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } if call_target.function_id() == Some(FUNC_ID_RTD) && context.rtd_provider.is_some() => {
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
                sparse_reference_values: &context.sparse_reference_values,
                caller_row: context.caller_row,
                caller_col: context.caller_col,
                callable_registry: &callable_registry,
            };
            match oxfunc_core::functions::rtd_fn::eval_rtd_surface(
                &call_args,
                &resolver,
                context.rtd_provider,
            ) {
                Ok(value) => Some(ReturnedValueSurface::from_rtd_eval_value(&value)),
                Err(_) => None,
            }
        }
        CompiledExpr::FunctionCall {
            call_target, args, ..
        }
        | CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } if call_target.function_id() == Some(FUNC_ID_INFO) && context.host_info.is_some() => {
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
                sparse_reference_values: &context.sparse_reference_values,
                caller_row: context.caller_row,
                caller_col: context.caller_col,
                callable_registry: &callable_registry,
            };
            match eval_info_surface(&call_args, &resolver, context.host_info) {
                Err(InfoEvalError::HostInfo(error)) => {
                    Some(ReturnedValueSurface::from_host_info_error(&error))
                }
                _ => None,
            }
        }
        CompiledExpr::FunctionCall {
            call_target, args, ..
        }
        | CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } if call_target.function_id() == Some(FUNC_ID_CELL) && context.host_info.is_some() => {
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
                sparse_reference_values: &context.sparse_reference_values,
                caller_row: context.caller_row,
                caller_col: context.caller_col,
                callable_registry: &callable_registry,
            };
            match eval_cell_surface(&call_args, &resolver, context.host_info) {
                Err(CellEvalError::HostInfo(error)) => {
                    Some(ReturnedValueSurface::from_host_info_error(&error))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn extended_surface_for_top_level_function_call(
    root: &CompiledExpr,
    context: &EvaluationContext<'_>,
) -> Option<ReturnedValueSurface> {
    match root {
        CompiledExpr::FunctionCall {
            call_target, args, ..
        }
        | CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } => {
            let function_id = call_target.function_id()?;
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
                sparse_reference_values: &context.sparse_reference_values,
                caller_row: context.caller_row,
                caller_col: context.caller_col,
                callable_registry: &callable_registry,
            };
            if function_id == FUNC_ID_IMAGE {
                let image_result = eval_image_surface_extended_with_capabilities(
                    &call_args,
                    &resolver,
                    context.host_info,
                )
                .ok()?;
                return Some(
                    ReturnedValueSurface::from_extended_value_with_capability_keys(
                        &image_result.value,
                        image_result.producer_capability_set_keys,
                        image_result.exercised_capability_keys,
                    ),
                );
            }
            if !matches!(function_id, FUNC_ID_HYPERLINK | FUNC_ID_NOW | FUNC_ID_TODAY) {
                return None;
            }
            let extended = eval_surface_extended_call(
                function_id,
                &call_args,
                &resolver,
                context.now_serial,
                context.random_provider,
                context.locale_ctx,
                context.host_info,
            )
            .ok()?;
            Some(ReturnedValueSurface::from_extended_value(&extended))
        }
        _ => None,
    }
}

fn build_top_level_call_args(
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    preserve_reference: bool,
) -> Result<Vec<CallArgValue>, EvaluationError> {
    let callable_registry = RefCell::new(CallableRegistry::default());
    let mut resolver = LocalReferenceResolver {
        cell_values: &context.cell_values,
        defined_names: &context.defined_names,
        sparse_reference_values: &context.sparse_reference_values,
        caller_row: context.caller_row,
        caller_col: context.caller_col,
        callable_registry: &callable_registry,
    };
    let helper_bindings = HelperBindingFrame::default();
    let mut trace = EvaluationTrace {
        prepared_calls: Vec::new(),
    };

    args.iter()
        .map(|arg| {
            evaluate_expr_as_call_arg(
                arg,
                context,
                &mut resolver,
                &helper_bindings,
                &callable_registry,
                preserve_reference,
                false,
                &mut trace,
            )
        })
        .collect()
}

fn evaluate_root_expr_value(
    expr: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if !context.bind_formula.root_expression_is_grouped {
        if let CompiledExpr::Binary {
            op,
            call_target,
            left,
            right,
        } = expr
        {
            if matches!(op, BinaryOp::Add | BinaryOp::Subtract) {
                let evaluation = evaluate_binary_operator_call_evaluation(
                    *op,
                    call_target,
                    left,
                    right,
                    context,
                    resolver,
                    helper_bindings,
                    callable_registry,
                    trace,
                )?;
                return Ok(publish_root_add_subtract_zero_reaching_result(
                    *op,
                    &evaluation.lhs,
                    &evaluation.rhs,
                    evaluation.value,
                ));
            }
        }
    }

    evaluate_expr_value(
        expr,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        trace,
    )
}

fn evaluate_expr_value(
    expr: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    match expr {
        CompiledExpr::NumberLiteral { source, value } => {
            value.map(EvalValue::Number).ok_or_else(|| EvaluationError {
                message: format!("failed to parse numeric literal {source}"),
            })
        }
        CompiledExpr::LogicalLiteral(value) => Ok(EvalValue::Logical(*value)),
        CompiledExpr::StringLiteral(text) => Ok(EvalValue::Text(text.clone())),
        CompiledExpr::PrecomputedValue { value, source } => {
            if context.records_prepared_calls() {
                evaluate_expr_value(
                    source,
                    context,
                    resolver,
                    helper_bindings,
                    callable_registry,
                    trace,
                )
            } else {
                Ok(value.clone())
            }
        }
        CompiledExpr::ArrayLiteral(rows) => evaluate_array_literal(
            rows,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::OmittedArgument => Ok(EvalValue::Error(WorksheetErrorCode::Value)),
        CompiledExpr::HelperParameterName { name, .. }
        | CompiledExpr::HelperOptionalParameterName { name, .. } => Err(EvaluationError {
            message: format!(
                "helper parameter {name} cannot be evaluated without helper-form environment support"
            ),
        }),
        CompiledExpr::Binary {
            op,
            call_target,
            left,
            right,
        } => evaluate_binary_operator_call(
            *op,
            call_target,
            left,
            right,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::Unary {
            op,
            call_target,
            expr,
        } => evaluate_unary_operator_call(
            *op,
            call_target,
            expr,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::FunctionCall {
            function_name,
            call_target,
            args,
        } => evaluate_function_call(
            function_name,
            call_target,
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::ResolvedFunctionCall {
            function_name,
            call_target,
            args,
        } => evaluate_ordinary_function_call(
            function_name,
            call_target,
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::Let { args, slot_only } => evaluate_let_call(
            args,
            *slot_only,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::LambdaLiteral { args } => {
            evaluate_lambda_call(args, helper_bindings, callable_registry, context, trace)
        }
        CompiledExpr::If { args } => evaluate_if_call(
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::IfError { args } => evaluate_iferror_call(
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::BuiltinCallable(callable) => Ok(EvalValue::Lambda(
            built_in_callable_lambda_from_call_target(&callable.call_target, callable_registry),
        )),
        CompiledExpr::Invocation { callee, args } => evaluate_invocation(
            callee,
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        CompiledExpr::Reference(reference) => {
            let arg = evaluate_reference_as_call_arg(
                reference,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                false,
                false,
                trace,
            )?;
            materialize_call_arg(arg, resolver)
        }
        CompiledExpr::ImplicitIntersection { call_target, expr } => {
            let arg = evaluate_expr_as_call_arg(
                expr,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                true,
                false,
                trace,
            )?;
            let args = [arg];
            Ok(evaluate_function_call_target_value(
                call_target,
                &args,
                context,
                resolver,
                callable_registry,
            ))
        }
    }
}

fn evaluate_function_call(
    function_name: &str,
    call_target: &CompiledFunctionCallTarget,
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    match call_target.special_form {
        CompiledFunctionSpecialForm::Let => {
            return evaluate_let_call(
                args,
                false,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            );
        }
        CompiledFunctionSpecialForm::Lambda => {
            return evaluate_lambda_call(args, helper_bindings, callable_registry, context, trace);
        }
        CompiledFunctionSpecialForm::If => {
            return evaluate_if_call(
                args,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            );
        }
        CompiledFunctionSpecialForm::IfError => {
            return evaluate_iferror_call(
                args,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            );
        }
        CompiledFunctionSpecialForm::LegacySingle => {
            let Some(special_operator_call_target) =
                call_target.special_operator_call_target.as_ref()
            else {
                return Ok(EvalValue::Error(WorksheetErrorCode::Name));
            };
            return evaluate_legacy_single_call(
                special_operator_call_target,
                args,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            );
        }
        CompiledFunctionSpecialForm::None => {}
    }

    evaluate_ordinary_function_call(
        function_name,
        call_target,
        args,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        trace,
    )
}

fn evaluate_ordinary_function_call(
    function_name: &str,
    call_target: &CompiledFunctionCallTarget,
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if runtime_capability_denied_for_function(context, function_name) {
        return Ok(EvalValue::Error(WorksheetErrorCode::Blocked));
    }

    let Some(function_call_target) = call_target.function_call_target.as_ref() else {
        if let Some(host_function_provider) = context.host_function_provider
            && context_allows_host_function_call(context, function_name)
        {
            return evaluate_host_function_call(
                function_name,
                args,
                host_function_provider,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            );
        }
        return Ok(EvalValue::Error(WorksheetErrorCode::Name));
    };
    let meta = function_call_target.function_meta();

    if context.backend == EvaluationBackend::LocalBootstrap {
        return Err(EvaluationError {
            message: format!(
                "local bootstrap backend does not support function calls: {function_name}"
            ),
        });
    }

    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        Vec::with_capacity(args.len())
    } else {
        Vec::new()
    };
    let mut scratch = context.take_function_call_scratch(args.len());
    for (ordinal, arg) in args.iter().enumerate() {
        let preserve_reference =
            meta.arg_preparation_profile == ArgPreparationProfile::RefsVisibleInAdapter;
        let callable_slot = call_target.callable_argument_ordinals.contains(&ordinal);
        let call_arg = evaluate_expr_as_call_arg(
            arg,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            callable_slot,
            trace,
        )?;
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(
                ordinal,
                arg,
                &call_arg,
                preserve_reference,
            ));
        }
        scratch.push_arg(call_arg);
    }

    let collapse_hstack_empty_carrier = meta.function_id == FUNC_ID_HSTACK
        && args
            .iter()
            .zip(scratch.call_args().iter())
            .any(|(arg, call_arg)| hstack_arg_should_collapse(arg, call_arg, helper_bindings));

    // Excel trailing omitted arguments behave like absent optional arguments at the
    // function-surface boundary. Preserve interior omissions as `MissingArg`, but do not
    // force trailing omitted placeholders into OxFunc's optional-argument lanes.
    let trailing_omitted_count = args
        .iter()
        .rev()
        .take_while(|arg| matches!(arg, CompiledExpr::OmittedArgument))
        .count();
    {
        let call_args = scratch.call_args_mut();
        for _ in 0..trailing_omitted_count {
            if !matches!(call_args.last(), Some(CallArgValue::MissingArg)) {
                break;
            }
            if records_prepared_calls {
                prepared_arguments.pop();
            }
            call_args.pop();
        }
    }

    let prepared_call_index = if records_prepared_calls {
        let register_id_request = if meta.function_id == FUNC_ID_REGISTER_ID {
            parse_register_id_request(scratch.call_args(), resolver).ok()
        } else {
            None
        };
        let registered_external_call_request = if meta.function_id == FUNC_ID_CALL {
            parse_call_request(scratch.call_args(), resolver).ok()
        } else {
            None
        };
        push_prepared_call_unchecked(
            trace,
            PreparedCall {
                function_name: function_name.to_string(),
                function_id: meta.function_id,
                arg_preparation_profile: meta.arg_preparation_profile,
                prepared_arguments,
                register_id_request,
                registered_external_call_request,
                locale_profile_id: context
                    .locale_ctx
                    .map(|ctx| format!("{:?}", ctx.profile.id)),
                date_system: context
                    .locale_ctx
                    .map(|ctx| format!("{:?}", ctx.date_system)),
                host_query_enabled: context.host_info.is_some(),
                returned_value: None,
            },
        )
    } else {
        None
    };

    let returned_value = if collapse_hstack_empty_carrier {
        EvalValue::Error(WorksheetErrorCode::Calc)
    } else {
        match evaluate_function_call_target_scratch(
            function_call_target,
            &scratch,
            context,
            resolver,
            callable_registry,
        ) {
            Ok(value) => {
                decode_callable_carrier_function_result(meta.function_id, value, callable_registry)
            }
            Err(_error)
                if allow_host_query_worksheet_error_fallback(
                    meta.function_id,
                    scratch.call_args(),
                    resolver,
                    context.host_info,
                ) =>
            {
                EvalValue::Error(WorksheetErrorCode::Value)
            }
            Err(code) => EvalValue::Error(code),
        }
    };

    record_prepared_call_returned_value(trace, prepared_call_index, &returned_value);
    Ok(returned_value)
}

fn runtime_capability_denied_for_function(
    context: &EvaluationContext<'_>,
    function_name: &str,
) -> bool {
    context.plan.availability_summaries.iter().any(|summary| {
        summary.surface_name.eq_ignore_ascii_case(function_name)
            && summary.runtime_capability_state
                == Some(LibraryAvailabilityState::HostProfileUnavailable)
    })
}

fn context_allows_host_function_call(context: &EvaluationContext<'_>, function_name: &str) -> bool {
    context.plan.availability_summaries.iter().any(|summary| {
        summary.surface_name.eq_ignore_ascii_case(function_name)
            && matches!(
                summary.runtime_boundary_kind.as_deref(),
                Some("host_callback" | "vba_host_callback")
            )
    })
}

fn evaluate_host_function_call(
    function_name: &str,
    args: &[CompiledExpr],
    host_function_provider: &dyn HostFunctionProvider,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if context.backend == EvaluationBackend::LocalBootstrap {
        return Err(EvaluationError {
            message: format!(
                "local bootstrap backend does not support host function calls: {function_name}"
            ),
        });
    }

    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        Vec::with_capacity(args.len())
    } else {
        Vec::new()
    };
    let mut invocation_args = Vec::with_capacity(args.len());
    for (ordinal, arg) in args.iter().enumerate() {
        let call_arg = evaluate_expr_as_call_arg(
            arg,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            false,
            false,
            trace,
        )?;
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(
                ordinal, arg, &call_arg, false,
            ));
        }
        let invocation_arg = match call_arg {
            CallArgValue::Eval(value) => value,
            CallArgValue::EmptyCell | CallArgValue::MissingArg | CallArgValue::Reference(_) => {
                EvalValue::Error(WorksheetErrorCode::Value)
            }
        };
        invocation_args.push(invocation_arg);
    }

    let prepared_call_index = if records_prepared_calls {
        push_prepared_call_unchecked(
            trace,
            PreparedCall {
                function_name: function_name.to_string(),
                function_id: "FUNC.HOST_CALLBACK",
                arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
                prepared_arguments,
                register_id_request: None,
                registered_external_call_request: None,
                locale_profile_id: context
                    .locale_ctx
                    .map(|ctx| format!("{:?}", ctx.profile.id)),
                date_system: context
                    .locale_ctx
                    .map(|ctx| format!("{:?}", ctx.date_system)),
                host_query_enabled: context.host_info.is_some(),
                returned_value: None,
            },
        )
    } else {
        None
    };

    let returned_value =
        match host_function_provider.invoke_host_function(&HostFunctionInvocation {
            function_name: function_name.to_string(),
            args: invocation_args,
        }) {
            Ok(value) => value,
            Err(_) => EvalValue::Error(WorksheetErrorCode::Value),
        };
    record_prepared_call_returned_value(trace, prepared_call_index, &returned_value);
    Ok(returned_value)
}

fn allow_host_query_worksheet_error_fallback(
    function_id: &str,
    call_args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
) -> bool {
    match function_id {
        FUNC_ID_INFO => matches!(
            eval_info_surface(call_args, resolver, host_info),
            Err(InfoEvalError::HostInfo(_))
        ),
        FUNC_ID_CELL => matches!(
            eval_cell_surface(call_args, resolver, host_info),
            Err(CellEvalError::HostInfo(_))
        ),
        _ => false,
    }
}

fn evaluate_function_call_target(
    call_target: &FunctionCallTarget,
    args: &[CallArgValue],
    context: &EvaluationContext<'_>,
    resolver: &LocalReferenceResolver<'_>,
    callable_registry: &RefCell<CallableRegistry>,
) -> Result<EvalValue, WorksheetErrorCode> {
    let callable_invoker = OxFmlCallableInvoker {
        context,
        callable_registry,
    };
    let mut fec = FunctionExecutionContextBundle::new(resolver);
    fec.now_serial = context.now_serial;
    fec.random_provider = context.random_provider;
    fec.locale_ctx = context.locale_ctx;
    fec.host_info = context.host_info;
    fec.callable_invoker = Some(&callable_invoker);
    fec.rtd_provider = context.rtd_provider;
    fec.registered_external_provider = context.registered_external_provider;
    fec.reference_text_resolver = context.reference_text_resolver;
    call_target.invoke(args, &mut fec)
}

fn evaluate_function_call_target_scratch(
    call_target: &FunctionCallTarget,
    scratch: &FunctionCallScratch,
    context: &EvaluationContext<'_>,
    resolver: &LocalReferenceResolver<'_>,
    callable_registry: &RefCell<CallableRegistry>,
) -> Result<EvalValue, WorksheetErrorCode> {
    let callable_invoker = OxFmlCallableInvoker {
        context,
        callable_registry,
    };
    let mut fec = FunctionExecutionContextBundle::new(resolver);
    fec.now_serial = context.now_serial;
    fec.random_provider = context.random_provider;
    fec.locale_ctx = context.locale_ctx;
    fec.host_info = context.host_info;
    fec.callable_invoker = Some(&callable_invoker);
    fec.rtd_provider = context.rtd_provider;
    fec.registered_external_provider = context.registered_external_provider;
    fec.reference_text_resolver = context.reference_text_resolver;
    call_target.invoke_scratch(scratch, &mut fec)
}

fn evaluate_function_call_target_value(
    call_target: &FunctionCallTarget,
    args: &[CallArgValue],
    context: &EvaluationContext<'_>,
    resolver: &LocalReferenceResolver<'_>,
    callable_registry: &RefCell<CallableRegistry>,
) -> EvalValue {
    match evaluate_function_call_target(call_target, args, context, resolver, callable_registry) {
        Ok(value) => value,
        Err(code) => EvalValue::Error(code),
    }
}

fn parse_excel_numeric_literal(text: &str) -> Result<f64, std::num::ParseFloatError> {
    let value = text.parse::<f64>()?;
    if numeric_literal_underflows_excel_admission_floor(text) {
        Ok(0.0)
    } else {
        Ok(value)
    }
}

fn numeric_literal_underflows_excel_admission_floor(raw: &str) -> bool {
    let mut text = raw.trim();
    if let Some(rest) = text.strip_prefix('+') {
        text = rest;
    } else if let Some(rest) = text.strip_prefix('-') {
        text = rest;
    }

    let (coefficient, exponent) = match text.find(|ch| ch == 'e' || ch == 'E') {
        Some(index) => {
            let exponent = match text[index + 1..].parse::<i32>() {
                Ok(exponent) => exponent,
                Err(_) => return false,
            };
            (&text[..index], exponent)
        }
        None => (text, 0),
    };

    let mut digits = Vec::new();
    let mut integer_digit_count = 0usize;
    let mut seen_decimal_point = false;
    for byte in coefficient.bytes() {
        match byte {
            b'0'..=b'9' => {
                if !seen_decimal_point {
                    integer_digit_count += 1;
                }
                digits.push(byte);
            }
            b'.' if !seen_decimal_point => seen_decimal_point = true,
            _ => return false,
        }
    }

    let Some(first_significant_digit) = digits.iter().position(|digit| *digit != b'0') else {
        return false;
    };

    let adjusted_exponent =
        exponent + integer_digit_count as i32 - first_significant_digit as i32 - 1;
    if adjusted_exponent < -308 {
        return true;
    }
    if adjusted_exponent > -308 {
        return false;
    }

    let significand = &digits[first_significant_digit..];
    let threshold = b"222507385850721";
    let width = significand.len().max(threshold.len());
    for index in 0..width {
        let actual = significand.get(index).copied().unwrap_or(b'0');
        let minimum = threshold.get(index).copied().unwrap_or(b'0');
        if actual != minimum {
            return actual < minimum;
        }
    }
    false
}

fn evaluate_expr_as_call_arg(
    expr: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
    callable_slot: bool,
    trace: &mut EvaluationTrace,
) -> Result<CallArgValue, EvaluationError> {
    if callable_slot {
        if let Some(callable_arg) =
            built_in_callable_arg_for_expr(expr, context, callable_registry)?
        {
            return Ok(callable_arg);
        }
    }

    match expr {
        CompiledExpr::OmittedArgument => Ok(CallArgValue::MissingArg),
        CompiledExpr::Reference(reference) => evaluate_reference_as_call_arg(
            reference,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            callable_slot,
            trace,
        ),
        CompiledExpr::ImplicitIntersection { call_target, expr } => {
            let arg = evaluate_expr_as_call_arg(
                expr,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                true,
                false,
                trace,
            )?;
            let args = [arg];
            Ok(CallArgValue::Eval(evaluate_function_call_target_value(
                call_target,
                &args,
                context,
                resolver,
                callable_registry,
            )))
        }
        _ => Ok(CallArgValue::Eval(evaluate_expr_value(
            expr,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        )?)),
    }
}

fn built_in_callable_arg_for_expr(
    expr: &CompiledExpr,
    _context: &EvaluationContext<'_>,
    callable_registry: &RefCell<CallableRegistry>,
) -> Result<Option<CallArgValue>, EvaluationError> {
    match expr {
        CompiledExpr::BuiltinCallable(callable) => {
            built_in_callable_arg_for_call_target(&callable.call_target, callable_registry)
                .map(Some)
        }
        CompiledExpr::FunctionCall {
            call_target, args, ..
        }
        | CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } if args.is_empty() => call_target
            .function_call_target
            .as_ref()
            .map(|call_target| {
                built_in_callable_arg_for_call_target(call_target, callable_registry)
            })
            .transpose(),
        _ => Ok(None),
    }
}

fn evaluate_reference_as_call_arg(
    reference: &CompiledReferenceExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
    callable_slot: bool,
    trace: &mut EvaluationTrace,
) -> Result<CallArgValue, EvaluationError> {
    match reference {
        CompiledReferenceExpr::Atom(NormalizedReference::Cell(cell)) => {
            call_arg_for_reference_like(
                reference_like_for_cell(cell),
                preserve_reference,
                context,
                resolver,
            )
        }
        CompiledReferenceExpr::Atom(NormalizedReference::Area(area)) => {
            call_arg_for_reference_like(
                reference_like_for_area(area),
                preserve_reference,
                context,
                resolver,
            )
        }
        CompiledReferenceExpr::Atom(NormalizedReference::WholeRow(rows)) => {
            call_arg_for_reference_like(
                ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: whole_row_target(rows),
                },
                preserve_reference,
                context,
                resolver,
            )
        }
        CompiledReferenceExpr::Atom(NormalizedReference::WholeColumn(columns)) => {
            call_arg_for_reference_like(
                ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: whole_column_target(columns),
                },
                preserve_reference,
                context,
                resolver,
            )
        }
        CompiledReferenceExpr::Atom(NormalizedReference::Name(name)) => call_arg_for_name(
            name,
            preserve_reference,
            callable_slot,
            context,
            resolver,
            helper_bindings,
            callable_registry,
        ),
        CompiledReferenceExpr::HelperLocalSlot { name, slot } => {
            if let Some(call_arg) = call_arg_for_helper_slot(
                helper_bindings,
                *slot,
                preserve_reference,
                resolver,
                callable_registry,
            ) {
                call_arg
            } else {
                call_arg_for_name(
                    name,
                    preserve_reference,
                    callable_slot,
                    context,
                    resolver,
                    helper_bindings,
                    callable_registry,
                )
            }
        }
        CompiledReferenceExpr::Atom(NormalizedReference::Structured(structured)) => {
            call_arg_for_reference_like(
                reference_like_for_structured(structured),
                preserve_reference,
                context,
                resolver,
            )
        }
        CompiledReferenceExpr::Atom(NormalizedReference::External(external)) => {
            let prepared_call_index = push_special_prepared_call(
                trace,
                "EXTERNAL_REFERENCE_DEFERRED",
                SPECIAL_EXTERNAL_REFERENCE_DEFERRED_FUNCTION_ID,
                ArgPreparationProfile::RefsVisibleInAdapter,
                if context.records_prepared_calls() {
                    vec![PreparedArgument {
                        ordinal: 0,
                        structure_class: PreparedStructureClass::ReferenceVisible,
                        source_class: PreparedSourceClass::ExternalReference,
                        evaluation_mode: PreparedEvaluationMode::ReferencePreserved,
                        blankness_class: PreparedBlanknessClass::NonBlank,
                        caller_context_sensitive: false,
                        reference_target: Some(external.target_summary.clone()),
                        opaque_reason: Some("external_reference_deferred".to_string()),
                        resolved_value: None,
                    }]
                } else {
                    Vec::new()
                },
                context,
            );
            let returned = EvalValue::Error(WorksheetErrorCode::Ref);
            record_prepared_call_returned_value(trace, prepared_call_index, &returned);
            Ok(CallArgValue::Eval(returned))
        }
        CompiledReferenceExpr::Atom(NormalizedReference::Error(error)) => Ok(CallArgValue::Eval(
            EvalValue::Error(error_code_for_error_ref(error)),
        )),
        CompiledReferenceExpr::Spill {
            call_target,
            anchor,
        } => evaluate_reference_operator_call(
            "OP_SPILL_REF",
            call_target,
            vec![anchor.as_ref()],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            trace,
        ),
        CompiledReferenceExpr::Range {
            call_target,
            start,
            end,
        } => evaluate_reference_operator_call(
            "OP_RANGE_REF",
            call_target,
            vec![start.as_ref(), end.as_ref()],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            trace,
        ),
        CompiledReferenceExpr::Union {
            call_target,
            left,
            right,
        } => evaluate_reference_operator_call(
            "OP_UNION_REF",
            call_target,
            vec![left.as_ref(), right.as_ref()],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            trace,
        ),
        CompiledReferenceExpr::Intersection {
            call_target,
            left,
            right,
        } => evaluate_reference_operator_call(
            "OP_INTERSECTION_REF",
            call_target,
            vec![left.as_ref(), right.as_ref()],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            trace,
        ),
    }
}

fn evaluate_reference_operator_call(
    function_name: &'static str,
    call_target: &FunctionCallTarget,
    operands: Vec<&CompiledReferenceExpr>,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
    trace: &mut EvaluationTrace,
) -> Result<CallArgValue, EvaluationError> {
    let function_id = call_target.function_id();
    let mut args = Vec::with_capacity(operands.len());
    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        Vec::with_capacity(operands.len())
    } else {
        Vec::new()
    };
    for (ordinal, operand) in operands.into_iter().enumerate() {
        let arg = evaluate_reference_as_call_arg(
            operand,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            true,
            false,
            trace,
        )?;
        if records_prepared_calls {
            let expr = CompiledExpr::Reference(operand.clone());
            prepared_arguments.push(prepared_argument_for_call_arg(ordinal, &expr, &arg, true));
        }
        args.push(arg);
    }

    let prepared_call_index = push_special_prepared_call(
        trace,
        function_name,
        function_id,
        ArgPreparationProfile::RefsVisibleInAdapter,
        prepared_arguments,
        context,
    );

    let result =
        evaluate_function_call_target(call_target, &args, context, resolver, callable_registry);
    let value = match result {
        Ok(value) => value,
        Err(code) => EvalValue::Error(code),
    };
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    call_arg_from_reference_operator_value(value, preserve_reference, resolver)
}

fn call_arg_from_reference_operator_value(
    value: EvalValue,
    preserve_reference: bool,
    resolver: &mut LocalReferenceResolver<'_>,
) -> Result<CallArgValue, EvaluationError> {
    match value {
        EvalValue::Reference(reference) if preserve_reference => {
            Ok(CallArgValue::Reference(reference))
        }
        EvalValue::Reference(reference) => resolve_oxfunc_eval_value(resolver, &reference)
            .map(call_arg_from_resolved_reference_value)
            .map_err(map_resolution_error),
        other => Ok(CallArgValue::Eval(other)),
    }
}

fn call_arg_for_reference_like(
    reference: ReferenceLike,
    preserve_reference: bool,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
) -> Result<CallArgValue, EvaluationError> {
    if preserve_reference || context.has_sparse_reference_values(&reference) {
        Ok(CallArgValue::Reference(reference))
    } else {
        resolve_oxfunc_eval_value(resolver, &reference)
            .map(call_arg_from_resolved_reference_value)
            .map_err(map_resolution_error)
    }
}

fn evaluate_array_literal(
    rows: &[Vec<CompiledExpr>],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    let height = rows.len();
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut array_rows = Vec::with_capacity(height);
    for row in rows {
        let mut array_row = Vec::with_capacity(width);
        for expr in row {
            let value = evaluate_expr_value(
                expr,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            )?;
            array_row.push(array_cell_value_from_eval_value(Some(value)));
        }
        while array_row.len() < width {
            array_row.push(ArrayCellValue::EmptyCell);
        }
        array_rows.push(array_row);
    }
    EvalArray::from_rows(array_rows)
        .map(EvalValue::Array)
        .ok_or_else(|| EvaluationError {
            message: "array literal produced an invalid rectangular shape".to_string(),
        })
}

fn evaluate_binary_operator_call(
    op: BinaryOp,
    call_target: &FunctionCallTarget,
    left: &CompiledExpr,
    right: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    evaluate_binary_operator_call_evaluation(
        op,
        call_target,
        left,
        right,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        trace,
    )
    .map(|evaluation| evaluation.value)
}

struct BinaryOperatorCallEvaluation {
    lhs: CallArgValue,
    rhs: CallArgValue,
    value: EvalValue,
}

fn publish_root_add_subtract_zero_reaching_result(
    op: BinaryOp,
    lhs: &CallArgValue,
    rhs: &CallArgValue,
    value: EvalValue,
) -> EvalValue {
    let EvalValue::Number(result) = value else {
        return value;
    };
    let (Some(lhs), Some(rhs)) = (scalar_number_call_arg(lhs), scalar_number_call_arg(rhs)) else {
        return EvalValue::Number(result);
    };
    if excel_root_add_subtract_reaches_zero(op, lhs, rhs, result) {
        EvalValue::Number(0.0)
    } else {
        EvalValue::Number(result)
    }
}

fn scalar_number_call_arg(arg: &CallArgValue) -> Option<f64> {
    match arg {
        CallArgValue::Eval(EvalValue::Number(number)) => Some(*number),
        _ => None,
    }
}

fn excel_root_add_subtract_reaches_zero(op: BinaryOp, lhs: f64, rhs: f64, result: f64) -> bool {
    if !lhs.is_finite()
        || !rhs.is_finite()
        || !result.is_finite()
        || lhs == 0.0
        || rhs == 0.0
        || result == 0.0
    {
        return false;
    }

    let is_cancellation = match op {
        BinaryOp::Add => lhs.is_sign_negative() != rhs.is_sign_negative(),
        BinaryOp::Subtract => lhs.is_sign_negative() == rhs.is_sign_negative(),
        _ => false,
    };
    if !is_cancellation {
        return false;
    }

    // Excel's documented "value reaches zero" compensation is shape-sensitive:
    // COM probes show it applies to the root add/sub publication, while the
    // same residual remains observable when consumed by an outer expression.
    result.abs() <= 5.0e-16 && result.abs() <= lhs.abs().max(rhs.abs()) * f64::EPSILON * 5.0
}

fn evaluate_binary_operator_call_evaluation(
    op: BinaryOp,
    call_target: &FunctionCallTarget,
    left: &CompiledExpr,
    right: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<BinaryOperatorCallEvaluation, EvaluationError> {
    let (function_name, function_id) = binary_operator_identity(op);
    let lhs = evaluate_expr_as_call_arg(
        left,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        false,
        false,
        trace,
    )?;
    let rhs = evaluate_expr_as_call_arg(
        right,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        false,
        false,
        trace,
    )?;

    let prepared_call_index = push_special_prepared_call(
        trace,
        function_name,
        function_id,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        if context.records_prepared_calls() {
            vec![
                prepared_argument_for_call_arg(0, left, &lhs, false),
                prepared_argument_for_call_arg(1, right, &rhs, false),
            ]
        } else {
            Vec::new()
        },
        context,
    );

    let args = [lhs.clone(), rhs.clone()];
    let result =
        evaluate_function_call_target(call_target, &args, context, resolver, callable_registry);
    let value = match result {
        Ok(value) => value,
        Err(code) => EvalValue::Error(code),
    };
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(BinaryOperatorCallEvaluation { lhs, rhs, value })
}

fn evaluate_unary_operator_call(
    op: crate::binding::UnaryOp,
    call_target: &FunctionCallTarget,
    expr: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    let (function_name, function_id) = unary_operator_identity(op);
    let arg = evaluate_expr_as_call_arg(
        expr,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        false,
        false,
        trace,
    )?;

    let prepared_call_index = push_special_prepared_call(
        trace,
        function_name,
        function_id,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        if context.records_prepared_calls() {
            vec![prepared_argument_for_call_arg(0, expr, &arg, false)]
        } else {
            Vec::new()
        },
        context,
    );

    let args = [arg];
    let result =
        evaluate_function_call_target(call_target, &args, context, resolver, callable_registry);
    let value = match result {
        Ok(value) => value,
        Err(code) => EvalValue::Error(code),
    };
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

fn binary_operator_identity(op: BinaryOp) -> (&'static str, &'static str) {
    match op {
        BinaryOp::Add => ("OP_ADD", FUNC_ID_OP_ADD),
        BinaryOp::Subtract => ("OP_SUBTRACT", FUNC_ID_OP_SUBTRACT),
        BinaryOp::Power => ("OP_POWER", FUNC_ID_OP_POWER),
        BinaryOp::Multiply => ("OP_MULTIPLY", FUNC_ID_OP_MULTIPLY),
        BinaryOp::Divide => ("OP_DIVIDE", FUNC_ID_OP_DIVIDE),
        BinaryOp::Concat => ("OP_CONCAT", FUNC_ID_OP_CONCAT),
        BinaryOp::Equal => ("OP_EQUAL", FUNC_ID_OP_EQUAL),
        BinaryOp::NotEqual => ("OP_NOT_EQUAL", FUNC_ID_OP_NOT_EQUAL),
        BinaryOp::LessThan => ("OP_LESS_THAN", FUNC_ID_OP_LESS_THAN),
        BinaryOp::LessEqual => ("OP_LESS_EQUAL", FUNC_ID_OP_LESS_EQUAL),
        BinaryOp::GreaterThan => ("OP_GREATER_THAN", FUNC_ID_OP_GREATER_THAN),
        BinaryOp::GreaterEqual => ("OP_GREATER_EQUAL", FUNC_ID_OP_GREATER_EQUAL),
    }
}

fn unary_operator_identity(op: crate::binding::UnaryOp) -> (&'static str, &'static str) {
    match op {
        crate::binding::UnaryOp::Plus => ("OP_UNARY_PLUS", FUNC_ID_OP_UNARY_PLUS),
        crate::binding::UnaryOp::Negate => ("OP_NEGATE", FUNC_ID_OP_NEGATE),
        crate::binding::UnaryOp::Percent => ("OP_PERCENT", FUNC_ID_OP_PERCENT),
    }
}

fn call_arg_for_name(
    name: &NameRef,
    preserve_reference: bool,
    callable_slot: bool,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
) -> Result<CallArgValue, EvaluationError> {
    if let Some(binding) = helper_binding_get(helper_bindings, &name.name) {
        return call_arg_for_helper_binding(
            binding,
            preserve_reference,
            resolver,
            callable_registry,
        );
    }

    let Some(binding) = context.defined_names.get(&name.name) else {
        if callable_slot {
            return Ok(CallArgValue::Eval(EvalValue::Error(
                WorksheetErrorCode::Name,
            )));
        }
        return Err(EvaluationError {
            message: format!("no binding available for defined name {}", name.name),
        });
    };

    match binding {
        DefinedNameBinding::Value(value) => Ok(CallArgValue::Eval(value.clone())),
        DefinedNameBinding::Reference(reference) => {
            if preserve_reference || context.has_sparse_reference_values(reference) {
                Ok(CallArgValue::Reference(reference.clone()))
            } else {
                resolver
                    .resolve_reference(reference)
                    .map(CallArgValue::Eval)
                    .map_err(map_resolution_error)
            }
        }
        DefinedNameBinding::Callable(binding) => {
            let lambda = lambda_binding_from_defined_name_binding(binding, callable_registry);
            Ok(CallArgValue::Eval(EvalValue::Lambda(
                callable_registry.borrow_mut().register(lambda),
            )))
        }
    }
}

fn call_arg_for_helper_slot(
    helper_bindings: &HelperBindingFrame,
    slot: usize,
    preserve_reference: bool,
    resolver: &mut LocalReferenceResolver<'_>,
    callable_registry: &RefCell<CallableRegistry>,
) -> Option<Result<CallArgValue, EvaluationError>> {
    let slot_cell = helper_bindings.slots.get(slot)?;
    let slot_ref = slot_cell.borrow();
    let binding = slot_ref.as_ref()?;
    Some(call_arg_for_helper_binding(
        binding,
        preserve_reference,
        resolver,
        callable_registry,
    ))
}

fn call_arg_for_helper_binding(
    binding: &HelperBinding,
    preserve_reference: bool,
    resolver: &mut LocalReferenceResolver<'_>,
    callable_registry: &RefCell<CallableRegistry>,
) -> Result<CallArgValue, EvaluationError> {
    match binding {
        HelperBinding::Arg(CallArgValue::Reference(reference))
        | HelperBinding::EmptyHstackCarrier(CallArgValue::Reference(reference)) => {
            if preserve_reference {
                Ok(CallArgValue::Reference(reference.clone()))
            } else {
                resolver
                    .resolve_reference(reference)
                    .map(call_arg_from_resolved_reference_value)
                    .map_err(map_resolution_error)
            }
        }
        HelperBinding::Arg(other) | HelperBinding::EmptyHstackCarrier(other) => Ok(other.clone()),
        HelperBinding::Lambda {
            params,
            body,
            closure,
        } => Ok(CallArgValue::Eval(EvalValue::Lambda(
            callable_registry.borrow_mut().register(LambdaBinding {
                origin_kind: CallableOriginKind::HelperLambda,
                params: params.clone(),
                body: body.clone(),
                closure: closure.clone(),
            }),
        ))),
    }
}

fn built_in_callable_arg_for_call_target(
    call_target: &FunctionCallTarget,
    callable_registry: &RefCell<CallableRegistry>,
) -> Result<CallArgValue, EvaluationError> {
    Ok(CallArgValue::Eval(EvalValue::Lambda(
        built_in_callable_lambda_from_call_target(call_target, callable_registry),
    )))
}

fn built_in_callable_lambda_from_call_target(
    call_target: &FunctionCallTarget,
    callable_registry: &RefCell<CallableRegistry>,
) -> OxLambdaValue {
    callable_registry
        .borrow_mut()
        .register_builtin_call_target(call_target.clone())
}

fn evaluate_let_call(
    args: &[CompiledExpr],
    slot_only: bool,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if args.len() < 2 {
        return Err(EvaluationError {
            message: "LET requires at least one binding pair and a final expression".to_string(),
        });
    }

    if slot_only {
        return evaluate_slot_only_let_call(
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        );
    }

    let mut local_bindings = helper_bindings.child();
    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        Vec::with_capacity(args.len())
    } else {
        Vec::new()
    };
    let last_index = args.len() - 1;
    let mut index = 0usize;
    while index < last_index {
        let CompiledExpr::HelperParameterName { name, slot } = &args[index] else {
            return Err(EvaluationError {
                message: "LET binding position did not contain a helper parameter".to_string(),
            });
        };
        if records_prepared_calls {
            prepared_arguments.push(PreparedArgument {
                ordinal: index,
                structure_class: PreparedStructureClass::DirectScalar,
                source_class: PreparedSourceClass::HelperParameter,
                evaluation_mode: PreparedEvaluationMode::EagerValue,
                blankness_class: PreparedBlanknessClass::NonBlank,
                caller_context_sensitive: false,
                reference_target: None,
                opaque_reason: None,
                resolved_value: None,
            });
        }
        if index + 1 >= args.len() {
            return Err(EvaluationError {
                message: format!("LET binding {name} is missing a value expression"),
            });
        }
        let binding_arg = evaluate_expr_as_call_arg(
            &args[index + 1],
            context,
            resolver,
            &local_bindings,
            callable_registry,
            true,
            false,
            trace,
        )?;
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(
                index + 1,
                &args[index + 1],
                &binding_arg,
                true,
            ));
        }
        let helper_binding = helper_binding_from_expr(
            &args[index + 1],
            binding_arg,
            &local_bindings,
            callable_registry,
        );
        insert_helper_slot_binding(&mut local_bindings, name.clone(), *slot, helper_binding);
        index += 2;
    }
    let body_arg = evaluate_expr_as_call_arg(
        &args[last_index],
        context,
        resolver,
        &local_bindings,
        callable_registry,
        false,
        false,
        trace,
    )?;
    if records_prepared_calls {
        prepared_arguments.push(prepared_argument_for_call_arg(
            last_index,
            &args[last_index],
            &body_arg,
            false,
        ));
    }
    let prepared_call_index = push_special_prepared_call(
        trace,
        "LET",
        SPECIAL_LET_FUNCTION_ID,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        prepared_arguments,
        context,
    );

    let value = materialize_call_arg(body_arg, resolver)?;
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

fn evaluate_slot_only_let_call(
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        Vec::with_capacity(args.len())
    } else {
        Vec::new()
    };
    let last_index = args.len() - 1;
    let mut index = 0usize;
    while index < last_index {
        let CompiledExpr::HelperParameterName {
            name,
            slot: Some(slot),
        } = &args[index]
        else {
            return Err(EvaluationError {
                message: "LET binding position did not contain a slotted helper parameter"
                    .to_string(),
            });
        };
        if records_prepared_calls {
            prepared_arguments.push(PreparedArgument {
                ordinal: index,
                structure_class: PreparedStructureClass::DirectScalar,
                source_class: PreparedSourceClass::HelperParameter,
                evaluation_mode: PreparedEvaluationMode::EagerValue,
                blankness_class: PreparedBlanknessClass::NonBlank,
                caller_context_sensitive: false,
                reference_target: None,
                opaque_reason: None,
                resolved_value: None,
            });
        }
        if index + 1 >= args.len() {
            return Err(EvaluationError {
                message: format!("LET binding {name} is missing a value expression"),
            });
        }
        let binding_arg = evaluate_expr_as_call_arg(
            &args[index + 1],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            true,
            false,
            trace,
        )?;
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(
                index + 1,
                &args[index + 1],
                &binding_arg,
                true,
            ));
        }
        let helper_binding = helper_binding_from_expr(
            &args[index + 1],
            binding_arg,
            helper_bindings,
            callable_registry,
        );
        helper_bindings.set_slot(*slot, helper_binding);
        index += 2;
    }
    let body_arg = evaluate_expr_as_call_arg(
        &args[last_index],
        context,
        resolver,
        helper_bindings,
        callable_registry,
        false,
        false,
        trace,
    )?;
    if records_prepared_calls {
        prepared_arguments.push(prepared_argument_for_call_arg(
            last_index,
            &args[last_index],
            &body_arg,
            false,
        ));
    }
    let prepared_call_index = push_special_prepared_call(
        trace,
        "LET",
        SPECIAL_LET_FUNCTION_ID,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        prepared_arguments,
        context,
    );

    let value = materialize_call_arg(body_arg, resolver)?;
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

fn coerce_excel_if_text_condition(
    condition: &CallArgValue,
    resolver: &mut LocalReferenceResolver<'_>,
) -> Result<Option<bool>, EvaluationError> {
    match condition {
        CallArgValue::Eval(EvalValue::Text(text)) => {
            let normalized = text.to_string_lossy().trim().to_ascii_uppercase();
            match normalized.as_str() {
                "TRUE" => Ok(Some(true)),
                "FALSE" => Ok(Some(false)),
                _ => Ok(None),
            }
        }
        CallArgValue::Reference(reference) => resolve_oxfunc_eval_value(resolver, reference)
            .map_err(map_resolution_error)
            .and_then(|value| coerce_excel_if_text_condition(&CallArgValue::Eval(value), resolver)),
        _ => Ok(None),
    }
}

fn evaluate_if_call(
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if !(2..=3).contains(&args.len()) {
        return Ok(EvalValue::Error(WorksheetErrorCode::Value));
    }

    let condition = evaluate_expr_as_call_arg(
        &args[0],
        context,
        resolver,
        helper_bindings,
        callable_registry,
        true,
        false,
        trace,
    )?;
    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        vec![prepared_argument_for_call_arg(
            0, &args[0], &condition, true,
        )]
    } else {
        Vec::new()
    };
    let normalized_condition =
        if let Some(value) = coerce_excel_if_text_condition(&condition, resolver)? {
            CallArgValue::Eval(EvalValue::Logical(value))
        } else {
            condition.clone()
        };

    if if_condition_resolves_to_array(&normalized_condition, resolver) {
        let true_arg = evaluate_expr_as_call_arg(
            &args[1],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            true,
            false,
            trace,
        )?;
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(1, &args[1], &true_arg, true));
        }
        let false_arg = if args.len() == 3 {
            let false_arg = evaluate_expr_as_call_arg(
                &args[2],
                context,
                resolver,
                helper_bindings,
                callable_registry,
                true,
                false,
                trace,
            )?;
            if records_prepared_calls {
                prepared_arguments.push(prepared_argument_for_call_arg(
                    2, &args[2], &false_arg, true,
                ));
            }
            false_arg
        } else {
            CallArgValue::MissingArg
        };

        let prepared_call_index = push_special_prepared_call(
            trace,
            "IF",
            "FUNC.IF",
            ArgPreparationProfile::RefsVisibleInAdapter,
            prepared_arguments,
            context,
        );
        let value = eval_if_surface(
            &[normalized_condition.clone(), true_arg, false_arg],
            resolver,
        )
        .unwrap_or_else(|error| EvalValue::Error(map_if_error_to_ws(&error)));
        record_prepared_call_returned_value(trace, prepared_call_index, &value);
        return Ok(value);
    }

    let condition_is_true = match eval_if_surface(
        &[
            normalized_condition.clone(),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(0.0)),
        ],
        resolver,
    ) {
        Ok(EvalValue::Number(n)) => n != 0.0,
        Ok(EvalValue::Logical(b)) => b,
        Ok(_) => false,
        Err(error) => {
            if records_prepared_calls {
                prepared_arguments.push(lazy_skipped_prepared_argument(
                    1,
                    &args[1],
                    "condition_invalid_lazy",
                ));
                if args.len() == 3 {
                    prepared_arguments.push(lazy_skipped_prepared_argument(
                        2,
                        &args[2],
                        "condition_invalid_lazy",
                    ));
                }
            }
            let prepared_call_index = push_special_prepared_call(
                trace,
                "IF",
                "FUNC.IF",
                ArgPreparationProfile::RefsVisibleInAdapter,
                prepared_arguments,
                context,
            );
            let value = EvalValue::Error(map_if_error_to_ws(&error));
            record_prepared_call_returned_value(trace, prepared_call_index, &value);
            return Ok(value);
        }
    };

    let mut call_args = vec![normalized_condition.clone()];
    if condition_is_true {
        let true_arg = evaluate_expr_as_call_arg(
            &args[1],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            true,
            false,
            trace,
        )?;
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(1, &args[1], &true_arg, true));
        }
        call_args.push(true_arg);
        if args.len() == 3 {
            call_args.push(CallArgValue::MissingArg);
            if records_prepared_calls {
                prepared_arguments.push(lazy_skipped_prepared_argument(
                    2,
                    &args[2],
                    "branch_not_evaluated_lazy",
                ));
            }
        }
    } else {
        call_args.push(CallArgValue::MissingArg);
        if records_prepared_calls {
            prepared_arguments.push(lazy_skipped_prepared_argument(
                1,
                &args[1],
                "branch_not_evaluated_lazy",
            ));
        }
        if args.len() == 3 {
            let false_arg = evaluate_expr_as_call_arg(
                &args[2],
                context,
                resolver,
                helper_bindings,
                callable_registry,
                true,
                false,
                trace,
            )?;
            if records_prepared_calls {
                prepared_arguments.push(prepared_argument_for_call_arg(
                    2, &args[2], &false_arg, true,
                ));
            }
            call_args.push(false_arg);
        }
    }

    let prepared_call_index = push_special_prepared_call(
        trace,
        "IF",
        "FUNC.IF",
        ArgPreparationProfile::RefsVisibleInAdapter,
        prepared_arguments,
        context,
    );
    let value = eval_if_surface(&call_args, resolver)
        .unwrap_or_else(|error| EvalValue::Error(map_if_error_to_ws(&error)));
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

fn if_condition_resolves_to_array(
    condition: &CallArgValue,
    resolver: &mut LocalReferenceResolver<'_>,
) -> bool {
    matches!(
        prepare_arg_values_only(condition, resolver),
        Ok(PreparedArgValue::Eval(EvalValue::Array(_)))
    )
}

fn evaluate_iferror_call(
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if args.len() != 2 {
        return Ok(EvalValue::Error(WorksheetErrorCode::Value));
    }

    let primary = evaluate_expr_as_call_arg(
        &args[0],
        context,
        resolver,
        helper_bindings,
        callable_registry,
        true,
        false,
        trace,
    )?;

    let primary_is_error = matches!(
        prepare_arg_values_only(&primary, resolver),
        Ok(PreparedArgValue::Eval(EvalValue::Error(_)))
    );

    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        vec![prepared_argument_for_call_arg(0, &args[0], &primary, true)]
    } else {
        Vec::new()
    };
    let mut call_args = vec![primary];
    if primary_is_error {
        let fallback = evaluate_expr_as_call_arg(
            &args[1],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            true,
            false,
            trace,
        )?;
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(1, &args[1], &fallback, true));
        }
        call_args.push(fallback);
    } else {
        if records_prepared_calls {
            prepared_arguments.push(lazy_skipped_prepared_argument(
                1,
                &args[1],
                "fallback_not_evaluated_lazy",
            ));
        }
        call_args.push(CallArgValue::MissingArg);
    }

    let prepared_call_index = push_special_prepared_call(
        trace,
        "IFERROR",
        "FUNC.IFERROR",
        ArgPreparationProfile::RefsVisibleInAdapter,
        prepared_arguments,
        context,
    );
    let value = eval_iferror_surface(&call_args, resolver)
        .unwrap_or_else(|error| EvalValue::Error(map_iferror_error_to_ws(&error)));
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

fn evaluate_lambda_call(
    args: &[CompiledExpr],
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    context: &EvaluationContext<'_>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if args.is_empty() {
        return Err(EvaluationError {
            message: "LAMBDA requires at least a body expression".to_string(),
        });
    }

    let body_index = args.len() - 1;
    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        Vec::with_capacity(args.len())
    } else {
        Vec::new()
    };
    let params = args[..body_index]
        .iter()
        .enumerate()
        .map(|(ordinal, arg)| {
            let param = lambda_param_from_expr(arg).ok_or_else(|| EvaluationError {
                message: "LAMBDA parameter did not bind as helper parameter".to_string(),
            })?;
            if records_prepared_calls {
                prepared_arguments.push(PreparedArgument {
                    ordinal,
                    structure_class: PreparedStructureClass::DirectScalar,
                    source_class: PreparedSourceClass::HelperParameter,
                    evaluation_mode: PreparedEvaluationMode::EagerValue,
                    blankness_class: PreparedBlanknessClass::NonBlank,
                    caller_context_sensitive: false,
                    reference_target: None,
                    opaque_reason: None,
                    resolved_value: None,
                });
            }
            Ok(param)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if records_prepared_calls {
        prepared_arguments.push(PreparedArgument {
            ordinal: body_index,
            structure_class: PreparedStructureClass::DirectScalar,
            source_class: prepared_source_class(&args[body_index]),
            evaluation_mode: PreparedEvaluationMode::EagerValue,
            blankness_class: PreparedBlanknessClass::NonBlank,
            caller_context_sensitive: false,
            reference_target: None,
            opaque_reason: None,
            resolved_value: None,
        });
    }
    let prepared_call_index = push_special_prepared_call(
        trace,
        "LAMBDA",
        SPECIAL_LAMBDA_FUNCTION_ID,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        prepared_arguments,
        context,
    );

    let parameter_names = lambda_param_names(&params);
    let capture_names = helper_capture_names(&args[body_index], &parameter_names, helper_bindings);
    let value = EvalValue::Lambda(callable_registry.borrow_mut().register(LambdaBinding {
        origin_kind: CallableOriginKind::HelperLambda,
        params: Rc::from(params.into_boxed_slice()),
        body: Rc::new(args[body_index].clone()),
        closure: helper_closure_from_names(helper_bindings, &capture_names),
    }));
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

fn evaluate_legacy_single_call(
    call_target: &FunctionCallTarget,
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    let Some(arg) = args.first() else {
        return Err(EvaluationError {
            message: "_xlfn.SINGLE requires one argument".to_string(),
        });
    };

    let prepared = evaluate_expr_as_call_arg(
        arg,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        true,
        false,
        trace,
    )?;
    let prepared_call_index = push_special_prepared_call(
        trace,
        "_XLFN.SINGLE",
        SPECIAL_LEGACY_SINGLE_FUNCTION_ID,
        ArgPreparationProfile::RefsVisibleInAdapter,
        if context.records_prepared_calls() {
            vec![prepared_argument_for_call_arg(0, arg, &prepared, true)]
        } else {
            Vec::new()
        },
        context,
    );
    let args = [prepared];
    let value = evaluate_function_call_target_value(
        call_target,
        &args,
        context,
        resolver,
        callable_registry,
    );
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

fn evaluate_invocation(
    callee: &CompiledExpr,
    args: &[CompiledExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if is_missing_helper_local_callee_name(callee, helper_bindings, &context.defined_names) {
        return Ok(EvalValue::Error(WorksheetErrorCode::Name));
    }
    let lambda = resolve_callable(
        callee,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        trace,
    )?;
    let required_arity = lambda_required_arity(&lambda.params);
    if args.len() > lambda.params.len() {
        return Ok(EvalValue::Error(WorksheetErrorCode::Value));
    }
    if args.len() < required_arity {
        return Err(EvaluationError {
            message: format!(
                "lambda invocation arity mismatch: expected {}..{}, got {}",
                required_arity,
                lambda.params.len(),
                args.len()
            ),
        });
    }
    let mut local_bindings = lambda.closure;
    let records_prepared_calls = context.records_prepared_calls();
    let mut prepared_arguments = if records_prepared_calls {
        Vec::with_capacity(args.len())
    } else {
        Vec::new()
    };
    let mut recursion_cost_units = LOCAL_CALLABLE_RECURSION_BASE_COST_UNITS;
    for (ordinal, (param, arg)) in lambda.params.iter().zip(args.iter()).enumerate() {
        let prepared = evaluate_expr_as_call_arg(
            arg,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            true,
            false,
            trace,
        )?;
        if matches!(prepared, CallArgValue::Eval(EvalValue::Lambda(_))) {
            recursion_cost_units += LOCAL_CALLABLE_RECURSION_LAMBDA_ARG_COST_UNITS;
        }
        if records_prepared_calls {
            prepared_arguments.push(prepared_argument_for_call_arg(
                ordinal, arg, &prepared, true,
            ));
        }
        insert_helper_slot_binding(
            &mut local_bindings,
            param.name.clone(),
            param.slot,
            HelperBinding::Arg(prepared),
        );
    }
    for param in lambda.params.iter().skip(args.len()) {
        insert_helper_slot_binding(
            &mut local_bindings,
            param.name.clone(),
            param.slot,
            HelperBinding::Arg(CallArgValue::MissingArg),
        );
    }
    let Some(_recursion_guard) = try_enter_callable_recursion(
        &context.frame_state.callable_recursion_state,
        recursion_cost_units,
    ) else {
        return Ok(EvalValue::Error(WorksheetErrorCode::Num));
    };
    let prepared_call_index = push_special_prepared_call(
        trace,
        "LAMBDA.INVOKE",
        "SPECIAL.LAMBDA_INVOKE",
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        prepared_arguments,
        context,
    );
    let value = with_callable_stack_guard(context, || {
        evaluate_expr_value(
            &lambda.body,
            context,
            resolver,
            &local_bindings,
            callable_registry,
            trace,
        )
    })?;
    record_prepared_call_returned_value(trace, prepared_call_index, &value);
    Ok(value)
}

/// The single entry point that resolves a callee expression to an invocable
/// `LambdaBinding`. All callable invocation funnels through here; it tries each
/// callable source in precedence order:
///   1. an immediate lambda literal or helper-local lambda (already compiled in
///      the formula plan / reused from the helper binding frame),
///   2. a host-supplied defined-name callable (its body compiled via the
///      structural compiled-body cache, closure rebuilt per call),
///   3. an evaluated expression that yields a lambda value (registry lookup).
fn resolve_callable(
    callee: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<LambdaBinding, EvaluationError> {
    if let Some(binding) = lambda_binding_for_callee(callee, helper_bindings, callable_registry) {
        return Ok(binding);
    }
    if let Some(binding) =
        lambda_binding_for_defined_name_callee(callee, &context.defined_names, callable_registry)
    {
        return Ok(binding);
    }
    lambda_binding_for_evaluated_callee(
        callee,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        trace,
    )
}

fn is_missing_helper_local_callee_name(
    callee: &CompiledExpr,
    helper_bindings: &HelperBindingFrame,
    defined_names: &BTreeMap<String, DefinedNameBinding>,
) -> bool {
    match callee {
        CompiledExpr::Reference(CompiledReferenceExpr::Atom(NormalizedReference::Name(name)))
            if matches!(name.kind, NameKind::HelperLocal) =>
        {
            !helper_binding_contains(helper_bindings, &name.name)
                && !defined_names.contains_key(&name.name)
        }
        _ => false,
    }
}

fn lambda_binding_for_evaluated_callee(
    callee: &CompiledExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<LambdaBinding, EvaluationError> {
    let value = evaluate_expr_value(
        callee,
        context,
        resolver,
        helper_bindings,
        callable_registry,
        trace,
    )?;
    let EvalValue::Lambda(lambda) = value else {
        return Err(EvaluationError {
            message: format!(
                "only immediate, helper-bound, defined-name, or lambda-valued callable invocation is supported; callee evaluated to {}",
                eval_value_kind(&value)
            ),
        });
    };
    callable_registry
        .borrow()
        .get(&lambda.callable_token)
        .map(|binding| binding.lambda.clone())
        .ok_or_else(|| EvaluationError {
            message: format!(
                "no callable binding available for lambda token {}",
                lambda.callable_token
            ),
        })
}

fn eval_value_kind(value: &EvalValue) -> &'static str {
    match value {
        EvalValue::Number(_) => "Number",
        EvalValue::Text(_) => "Text",
        EvalValue::Logical(_) => "Logical",
        EvalValue::Error(_) => "Error",
        EvalValue::Array(_) => "Array",
        EvalValue::Reference(_) => "Reference",
        EvalValue::Lambda(_) => "Lambda",
    }
}

fn try_enter_callable_recursion(
    state: &RefCell<CallableRecursionState>,
    cost_units: usize,
) -> Option<CallableRecursionGuard<'_>> {
    let mut state_ref = state.borrow_mut();
    if state_ref.current_cost_units.saturating_add(cost_units) > state_ref.max_cost_units {
        return None;
    }
    state_ref.current_cost_units += cost_units;
    drop(state_ref);
    Some(CallableRecursionGuard { state, cost_units })
}

fn helper_binding_from_expr(
    expr: &CompiledExpr,
    fallback: CallArgValue,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
) -> HelperBinding {
    match expr {
        CompiledExpr::LambdaLiteral { args } if !args.is_empty() => {
            let body_index = args.len() - 1;
            let params = lambda_params_from_exprs(&args[..body_index])
                .expect("evaluated LAMBDA literal parameters were validated earlier");
            let capture_names = helper_capture_names(
                &args[body_index],
                &lambda_param_names(&params),
                helper_bindings,
            );
            HelperBinding::Lambda {
                params,
                body: Rc::new(args[body_index].clone()),
                closure: helper_closure_from_names(helper_bindings, &capture_names),
            }
        }
        _ => {
            if let CallArgValue::Eval(EvalValue::Lambda(lambda)) = &fallback {
                if let Some(binding) = callable_registry.borrow().get(&lambda.callable_token) {
                    return HelperBinding::Lambda {
                        params: binding.lambda.params.clone(),
                        body: binding.lambda.body.clone(),
                        closure: binding.lambda.closure.clone(),
                    };
                }
            }
            if matches!(
                fallback,
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Calc))
            ) && expr_is_hstack_empty_carrier(expr, helper_bindings)
            {
                return HelperBinding::EmptyHstackCarrier(fallback);
            }
            HelperBinding::Arg(fallback)
        }
    }
}

fn expr_is_hstack_empty_carrier(expr: &CompiledExpr, helper_bindings: &HelperBindingFrame) -> bool {
    match expr {
        CompiledExpr::FunctionCall {
            call_target, args, ..
        }
        | CompiledExpr::ResolvedFunctionCall {
            call_target, args, ..
        } if call_target.function_id() == Some(FUNC_ID_TAKE) => {
            take_call_has_zero_column_extent(args)
        }
        CompiledExpr::FunctionCall {
            call_target, args, ..
        } if call_target.has_special_form(CompiledFunctionSpecialForm::If) => args
            .iter()
            .skip(1)
            .any(|arg| expr_is_hstack_empty_carrier(arg, helper_bindings)),
        CompiledExpr::If { args } => args
            .iter()
            .skip(1)
            .any(|arg| expr_is_hstack_empty_carrier(arg, helper_bindings)),
        CompiledExpr::Reference(CompiledReferenceExpr::Atom(NormalizedReference::Name(name)))
            if matches!(name.kind, crate::binding::NameKind::HelperLocal) =>
        {
            matches!(
                helper_binding_get(helper_bindings, &name.name),
                Some(HelperBinding::EmptyHstackCarrier(_))
            )
        }
        CompiledExpr::Reference(CompiledReferenceExpr::HelperLocalSlot { slot, .. }) => {
            matches!(
                helper_bindings.get_slot_clone(*slot),
                Some(HelperBinding::EmptyHstackCarrier(_))
            )
        }
        CompiledExpr::ImplicitIntersection { expr, .. } => {
            expr_is_hstack_empty_carrier(expr, helper_bindings)
        }
        CompiledExpr::PrecomputedValue { source, .. } => {
            expr_is_hstack_empty_carrier(source, helper_bindings)
        }
        _ => false,
    }
}

fn take_call_has_zero_column_extent(args: &[CompiledExpr]) -> bool {
    matches!(args.get(2), Some(expr) if expr_is_numeric_zero(expr))
}

fn expr_is_numeric_zero(expr: &CompiledExpr) -> bool {
    match expr {
        CompiledExpr::NumberLiteral { value, .. } => *value == Some(0.0),
        CompiledExpr::Unary {
            op: crate::binding::UnaryOp::Plus,
            expr,
            ..
        }
        | CompiledExpr::Unary {
            op: crate::binding::UnaryOp::Negate,
            expr,
            ..
        } => expr_is_numeric_zero(expr),
        CompiledExpr::PrecomputedValue { value, source } => {
            matches!(value, EvalValue::Number(number) if *number == 0.0)
                || expr_is_numeric_zero(source)
        }
        _ => false,
    }
}

fn hstack_arg_should_collapse(
    expr: &CompiledExpr,
    call_arg: &CallArgValue,
    helper_bindings: &HelperBindingFrame,
) -> bool {
    matches!(
        call_arg,
        CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Calc))
    ) && expr_is_hstack_empty_carrier(expr, helper_bindings)
}

fn lambda_binding_for_callee(
    callee: &CompiledExpr,
    helper_bindings: &HelperBindingFrame,
    callable_registry: &RefCell<CallableRegistry>,
) -> Option<LambdaBinding> {
    match callee {
        CompiledExpr::LambdaLiteral { args } if !args.is_empty() => {
            let body_index = args.len() - 1;
            let params = lambda_params_from_exprs(&args[..body_index])?;
            let capture_names = helper_capture_names(
                &args[body_index],
                &lambda_param_names(&params),
                helper_bindings,
            );
            Some(LambdaBinding {
                origin_kind: CallableOriginKind::HelperLambda,
                params,
                body: Rc::new(args[body_index].clone()),
                closure: helper_closure_from_names(helper_bindings, &capture_names),
            })
        }
        CompiledExpr::Reference(CompiledReferenceExpr::Atom(NormalizedReference::Name(name)))
            if matches!(name.kind, crate::binding::NameKind::HelperLocal) =>
        {
            match helper_binding_get(helper_bindings, &name.name) {
                Some(HelperBinding::Lambda {
                    params,
                    body,
                    closure,
                }) => Some(LambdaBinding {
                    origin_kind: CallableOriginKind::HelperLambda,
                    params: params.clone(),
                    body: body.clone(),
                    closure: closure.clone(),
                }),
                Some(HelperBinding::Arg(CallArgValue::Eval(EvalValue::Lambda(lambda)))) => {
                    callable_registry
                        .borrow()
                        .get(&lambda.callable_token)
                        .map(|binding| binding.lambda.clone())
                }
                _ => None,
            }
        }
        CompiledExpr::Reference(CompiledReferenceExpr::HelperLocalSlot { slot, .. }) => {
            match helper_bindings.get_slot_clone(*slot) {
                Some(HelperBinding::Lambda {
                    params,
                    body,
                    closure,
                }) => Some(LambdaBinding {
                    origin_kind: CallableOriginKind::HelperLambda,
                    params,
                    body,
                    closure,
                }),
                Some(HelperBinding::Arg(CallArgValue::Eval(EvalValue::Lambda(lambda)))) => {
                    callable_registry
                        .borrow()
                        .get(&lambda.callable_token)
                        .map(|binding| binding.lambda.clone())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn lambda_binding_for_defined_name_callee(
    callee: &CompiledExpr,
    defined_names: &BTreeMap<String, DefinedNameBinding>,
    callable_registry: &RefCell<CallableRegistry>,
) -> Option<LambdaBinding> {
    match callee {
        CompiledExpr::Reference(CompiledReferenceExpr::Atom(NormalizedReference::Name(name))) => {
            match defined_names.get(&name.name) {
                Some(DefinedNameBinding::Callable(binding)) => Some(
                    lambda_binding_from_defined_name_binding(binding, callable_registry),
                ),
                _ => None,
            }
        }
        _ => None,
    }
}

fn lambda_binding_from_defined_name_binding(
    binding: &CallableDefinedNameBinding,
    callable_registry: &RefCell<CallableRegistry>,
) -> LambdaBinding {
    // Compute the (cached) compiled body first so its registry borrow is released
    // before the closure loop below borrows the registry again (RefCell).
    let body = callable_registry
        .borrow_mut()
        .get_or_compile_body(&binding.body);
    LambdaBinding {
        origin_kind: CallableOriginKind::DefinedNameCallable,
        params: Rc::from(
            binding
                .params
                .iter()
                .map(|name| LambdaParam {
                    name: name.clone(),
                    optional: binding.optional_parameter_names.contains(name),
                    slot: None,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        body,
        closure: binding
            .closure
            .iter()
            .map(|(name, binding)| {
                let helper_binding = match binding {
                    DefinedNameBinding::Value(value) => {
                        HelperBinding::Arg(CallArgValue::Eval(value.clone()))
                    }
                    DefinedNameBinding::Reference(reference) => {
                        HelperBinding::Arg(CallArgValue::Reference(reference.clone()))
                    }
                    // Preserve nested callable closure entries so a callable can
                    // invoke another captured callable (composition / mutual
                    // reference). Register the nested callable in the shared
                    // registry and carry it as a lambda value, mirroring how a
                    // top-level `DefinedNameBinding::Callable` is surfaced.
                    DefinedNameBinding::Callable(nested) => {
                        // Build the nested binding (which itself borrows the
                        // registry for its cached body) before borrowing the
                        // registry to register it, to avoid a RefCell double-borrow.
                        let nested_binding =
                            lambda_binding_from_defined_name_binding(nested, callable_registry);
                        HelperBinding::Arg(CallArgValue::Eval(EvalValue::Lambda(
                            callable_registry.borrow_mut().register(nested_binding),
                        )))
                    }
                };
                (name.clone(), helper_binding)
            })
            .collect(),
    }
}

fn lambda_value_summary_from_binding(binding: &LambdaBinding) -> String {
    lambda_value_summary_from_captures(
        &binding.params,
        binding.closure.keys().cloned().collect(),
        &binding.body,
    )
}

fn lambda_value_summary_from_captures(
    params: &[LambdaParam],
    mut captures: Vec<String>,
    body: &CompiledExpr,
) -> String {
    captures.sort();
    let captures = if captures.is_empty() {
        "-".to_string()
    } else {
        captures.join("|")
    };
    let parameter_names = lambda_param_names(params);
    let optional_parameter_names = params
        .iter()
        .filter(|param| param.optional)
        .map(|param| param.name.clone())
        .collect::<Vec<_>>();
    let optional_parameter_names = if optional_parameter_names.is_empty() {
        "-".to_string()
    } else {
        optional_parameter_names.join(",")
    };
    format!(
        "arity={};required_arity={};params={};optional_params={};captures={};body={}",
        parameter_names.len(),
        lambda_required_arity(params),
        parameter_names.join(","),
        optional_parameter_names,
        captures,
        lambda_body_kind(body)
    )
}

fn lambda_body_kind(body: &CompiledExpr) -> &'static str {
    match body {
        CompiledExpr::NumberLiteral { .. } => "NumberLiteral",
        CompiledExpr::StringLiteral(_) => "StringLiteral",
        CompiledExpr::LogicalLiteral(_) => "LogicalLiteral",
        CompiledExpr::PrecomputedValue { source, .. } => lambda_body_kind(source),
        CompiledExpr::ArrayLiteral(_) => "ArrayLiteral",
        CompiledExpr::OmittedArgument => "OmittedArgument",
        CompiledExpr::HelperParameterName { .. }
        | CompiledExpr::HelperOptionalParameterName { .. } => "HelperParameter",
        CompiledExpr::Binary { .. } => "Binary",
        CompiledExpr::Unary { .. } => "Unary",
        CompiledExpr::FunctionCall { .. } => "FunctionCall",
        CompiledExpr::ResolvedFunctionCall { .. } => "ResolvedFunctionCall",
        CompiledExpr::Let { .. } => "Let",
        CompiledExpr::LambdaLiteral { .. } => "LambdaLiteral",
        CompiledExpr::If { .. } => "If",
        CompiledExpr::IfError { .. } => "IfError",
        CompiledExpr::BuiltinCallable(_) => "BuiltinCallable",
        CompiledExpr::Invocation { .. } => "Invocation",
        CompiledExpr::Reference(_) => "Reference",
        CompiledExpr::ImplicitIntersection { .. } => "ImplicitIntersection",
    }
}

fn helper_capture_names(
    body: &CompiledExpr,
    parameter_names: &[String],
    helper_bindings: &HelperBindingFrame,
) -> BTreeSet<String> {
    let mut bound_names = parameter_names
        .iter()
        .map(|name| helper_name_key(name))
        .collect::<BTreeSet<_>>();
    helper_free_names_in_expr(body, &mut bound_names, helper_bindings)
}

fn lambda_param_names(params: &[LambdaParam]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}

fn lambda_required_arity(params: &[LambdaParam]) -> usize {
    params.iter().filter(|param| !param.optional).count()
}

fn helper_parameter_name(expr: &CompiledExpr) -> Option<String> {
    match expr {
        CompiledExpr::HelperParameterName { name, .. }
        | CompiledExpr::HelperOptionalParameterName { name, .. } => Some(name.clone()),
        _ => None,
    }
}

fn helper_free_names_in_expr(
    expr: &CompiledExpr,
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &HelperBindingFrame,
) -> BTreeSet<String> {
    match expr {
        CompiledExpr::NumberLiteral { .. }
        | CompiledExpr::StringLiteral(_)
        | CompiledExpr::LogicalLiteral(_)
        | CompiledExpr::OmittedArgument
        | CompiledExpr::HelperParameterName { .. }
        | CompiledExpr::HelperOptionalParameterName { .. }
        | CompiledExpr::BuiltinCallable(_) => BTreeSet::new(),
        CompiledExpr::PrecomputedValue { source, .. } => {
            helper_free_names_in_expr(source, bound_names, helper_bindings)
        }
        CompiledExpr::ArrayLiteral(rows) => {
            let mut names = BTreeSet::new();
            for row in rows {
                for cell in row {
                    names.extend(helper_free_names_in_expr(
                        cell,
                        bound_names,
                        helper_bindings,
                    ));
                }
            }
            names
        }
        CompiledExpr::Unary { expr, .. } => {
            helper_free_names_in_expr(expr, bound_names, helper_bindings)
        }
        CompiledExpr::Binary { left, right, .. } => {
            let mut names = helper_free_names_in_expr(left, bound_names, helper_bindings);
            names.extend(helper_free_names_in_expr(
                right,
                bound_names,
                helper_bindings,
            ));
            names
        }
        CompiledExpr::Let { args, .. } => {
            helper_free_names_in_let(args, bound_names, helper_bindings)
        }
        CompiledExpr::LambdaLiteral { args, .. } => {
            helper_free_names_in_lambda(args, bound_names, helper_bindings)
        }
        CompiledExpr::FunctionCall { args, .. }
        | CompiledExpr::ResolvedFunctionCall { args, .. }
        | CompiledExpr::If { args }
        | CompiledExpr::IfError { args } => {
            let mut names = BTreeSet::new();
            for arg in args {
                names.extend(helper_free_names_in_expr(arg, bound_names, helper_bindings));
            }
            names
        }
        CompiledExpr::Invocation { callee, args } => {
            let mut names = helper_free_names_in_expr(callee, bound_names, helper_bindings);
            for arg in args {
                names.extend(helper_free_names_in_expr(arg, bound_names, helper_bindings));
            }
            names
        }
        CompiledExpr::Reference(reference) => {
            helper_free_names_in_reference(reference, bound_names, helper_bindings)
        }
        CompiledExpr::ImplicitIntersection { expr, .. } => {
            helper_free_names_in_expr(expr, bound_names, helper_bindings)
        }
    }
}

fn helper_free_names_in_let(
    args: &[CompiledExpr],
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &HelperBindingFrame,
) -> BTreeSet<String> {
    if args.is_empty() {
        return BTreeSet::new();
    }

    let mut names = BTreeSet::new();
    let mut local_bound = bound_names.clone();
    let last_index = args.len() - 1;
    let mut index = 0usize;
    while index < last_index {
        if index + 1 >= args.len() {
            break;
        }
        names.extend(helper_free_names_in_expr(
            &args[index + 1],
            &mut local_bound,
            helper_bindings,
        ));
        if let Some(name) = helper_parameter_name(&args[index]) {
            local_bound.insert(helper_name_key(&name));
        }
        index += 2;
    }
    names.extend(helper_free_names_in_expr(
        &args[last_index],
        &mut local_bound,
        helper_bindings,
    ));
    names
}

fn helper_free_names_in_lambda(
    args: &[CompiledExpr],
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &HelperBindingFrame,
) -> BTreeSet<String> {
    if args.is_empty() {
        return BTreeSet::new();
    }

    let body_index = args.len() - 1;
    let mut nested_bound = bound_names.clone();
    for arg in &args[..body_index] {
        if let Some(name) = helper_parameter_name(arg) {
            nested_bound.insert(helper_name_key(&name));
        }
    }
    helper_free_names_in_expr(&args[body_index], &mut nested_bound, helper_bindings)
}

fn helper_free_names_in_reference(
    reference: &CompiledReferenceExpr,
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &HelperBindingFrame,
) -> BTreeSet<String> {
    match reference {
        CompiledReferenceExpr::Atom(NormalizedReference::Name(name))
            if matches!(name.kind, crate::binding::NameKind::HelperLocal)
                && !bound_names.contains(&helper_name_key(&name.name))
                && helper_binding_contains(helper_bindings, &name.name) =>
        {
            BTreeSet::from([name.name.clone()])
        }
        CompiledReferenceExpr::HelperLocalSlot { name, .. }
            if !bound_names.contains(&helper_name_key(&name.name))
                && helper_binding_contains(helper_bindings, &name.name) =>
        {
            BTreeSet::from([name.name.clone()])
        }
        CompiledReferenceExpr::HelperLocalSlot { .. } => BTreeSet::new(),
        CompiledReferenceExpr::Atom(_) => BTreeSet::new(),
        CompiledReferenceExpr::Spill { anchor, .. } => {
            helper_free_names_in_reference(anchor, bound_names, helper_bindings)
        }
        CompiledReferenceExpr::Range { start, end, .. }
        | CompiledReferenceExpr::Union {
            left: start,
            right: end,
            ..
        }
        | CompiledReferenceExpr::Intersection {
            left: start,
            right: end,
            ..
        } => {
            let mut names = helper_free_names_in_reference(start, bound_names, helper_bindings);
            names.extend(helper_free_names_in_reference(
                end,
                bound_names,
                helper_bindings,
            ));
            names
        }
    }
}

fn helper_closure_from_names(
    helper_bindings: &HelperBindingFrame,
    capture_names: &BTreeSet<String>,
) -> HelperBindingFrame {
    let mut closure = HelperBindingFrame {
        layers: Vec::new(),
        slots: empty_slot_cells(helper_bindings.slots.len()),
    };
    for entry in helper_bindings.entries() {
        if capture_names
            .iter()
            .any(|capture_name| capture_name.eq_ignore_ascii_case(&entry.display_name))
        {
            let binding = entry
                .slot
                .and_then(|slot| helper_bindings.get_slot_clone(slot))
                .unwrap_or_else(|| entry.binding.clone());
            insert_helper_slot_binding(
                &mut closure,
                entry.display_name.clone(),
                entry.slot,
                binding,
            );
        }
    }
    closure
}

fn materialize_call_arg(
    arg: CallArgValue,
    resolver: &mut LocalReferenceResolver<'_>,
) -> Result<EvalValue, EvaluationError> {
    match arg {
        CallArgValue::Eval(value) => Ok(value),
        CallArgValue::MissingArg => Ok(EvalValue::Error(WorksheetErrorCode::Value)),
        CallArgValue::EmptyCell => Ok(EvalValue::Number(0.0)),
        CallArgValue::Reference(reference) => {
            resolve_oxfunc_eval_value(resolver, &reference).map_err(map_resolution_error)
        }
    }
}

fn prepared_argument_for_call_arg(
    ordinal: usize,
    expr: &CompiledExpr,
    arg: &CallArgValue,
    _preserve_reference: bool,
) -> PreparedArgument {
    let source_class = prepared_source_class(expr);
    match arg {
        CallArgValue::Reference(reference) => PreparedArgument {
            ordinal,
            structure_class: PreparedStructureClass::ReferenceVisible,
            source_class,
            evaluation_mode: PreparedEvaluationMode::ReferencePreserved,
            blankness_class: PreparedBlanknessClass::NonBlank,
            caller_context_sensitive: matches!(expr, CompiledExpr::ImplicitIntersection { .. }),
            reference_target: Some(reference.target.clone()),
            opaque_reason: prepared_argument_opaque_reason(expr),
            resolved_value: None,
        },
        CallArgValue::MissingArg => PreparedArgument {
            ordinal,
            structure_class: PreparedStructureClass::Omitted,
            source_class,
            evaluation_mode: PreparedEvaluationMode::EagerValue,
            blankness_class: PreparedBlanknessClass::Omitted,
            caller_context_sensitive: false,
            reference_target: None,
            opaque_reason: prepared_argument_opaque_reason(expr),
            resolved_value: None,
        },
        CallArgValue::EmptyCell => PreparedArgument {
            ordinal,
            structure_class: PreparedStructureClass::DirectScalar,
            source_class,
            evaluation_mode: PreparedEvaluationMode::EagerValue,
            blankness_class: PreparedBlanknessClass::EmptyCell,
            caller_context_sensitive: false,
            reference_target: None,
            opaque_reason: prepared_argument_opaque_reason(expr),
            resolved_value: None,
        },
        CallArgValue::Eval(value) => PreparedArgument {
            ordinal,
            structure_class: match value {
                EvalValue::Array(_) => PreparedStructureClass::ArrayLike,
                _ => PreparedStructureClass::DirectScalar,
            },
            source_class,
            evaluation_mode: if matches!(expr, CompiledExpr::ImplicitIntersection { .. }) {
                PreparedEvaluationMode::CallerContextScalarized
            } else {
                PreparedEvaluationMode::EagerValue
            },
            blankness_class: blankness_class_for_eval_value(value),
            caller_context_sensitive: matches!(expr, CompiledExpr::ImplicitIntersection { .. }),
            reference_target: None,
            opaque_reason: prepared_argument_opaque_reason(expr),
            resolved_value: Some(value.clone()),
        },
    }
}

fn prepared_source_class(expr: &CompiledExpr) -> PreparedSourceClass {
    match expr {
        CompiledExpr::NumberLiteral { .. }
        | CompiledExpr::StringLiteral(_)
        | CompiledExpr::LogicalLiteral(_)
        | CompiledExpr::ArrayLiteral(_)
        | CompiledExpr::OmittedArgument => PreparedSourceClass::Literal,
        CompiledExpr::PrecomputedValue { source, .. } => prepared_source_class(source),
        CompiledExpr::HelperParameterName { .. }
        | CompiledExpr::HelperOptionalParameterName { .. } => PreparedSourceClass::HelperParameter,
        CompiledExpr::FunctionCall { .. }
        | CompiledExpr::ResolvedFunctionCall { .. }
        | CompiledExpr::Let { .. }
        | CompiledExpr::LambdaLiteral { .. }
        | CompiledExpr::If { .. }
        | CompiledExpr::IfError { .. }
        | CompiledExpr::BuiltinCallable(_)
        | CompiledExpr::Invocation { .. } => PreparedSourceClass::FunctionCall,
        CompiledExpr::Binary { .. } | CompiledExpr::Unary { .. } => {
            PreparedSourceClass::BinaryExpression
        }
        CompiledExpr::ImplicitIntersection { .. } => PreparedSourceClass::ImplicitIntersection,
        CompiledExpr::Reference(reference) => match reference {
            CompiledReferenceExpr::Atom(NormalizedReference::Cell(_)) => {
                PreparedSourceClass::CellReference
            }
            CompiledReferenceExpr::Atom(NormalizedReference::Area(_)) => {
                PreparedSourceClass::AreaReference
            }
            CompiledReferenceExpr::Atom(NormalizedReference::WholeRow(_)) => {
                PreparedSourceClass::WholeRowReference
            }
            CompiledReferenceExpr::Atom(NormalizedReference::WholeColumn(_)) => {
                PreparedSourceClass::WholeColumnReference
            }
            CompiledReferenceExpr::Atom(NormalizedReference::Name(_)) => {
                PreparedSourceClass::NameReference
            }
            CompiledReferenceExpr::HelperLocalSlot { .. } => PreparedSourceClass::NameReference,
            CompiledReferenceExpr::Atom(NormalizedReference::Structured(structured)) => {
                match structured.resolved_reference {
                    StructuredResolvedRef::Cell(_) => PreparedSourceClass::CellReference,
                    StructuredResolvedRef::Area(_) | StructuredResolvedRef::EmptyArea(_) => {
                        PreparedSourceClass::AreaReference
                    }
                }
            }
            CompiledReferenceExpr::Atom(NormalizedReference::External(_)) => {
                PreparedSourceClass::ExternalReference
            }
            CompiledReferenceExpr::Spill { .. } => PreparedSourceClass::SpillReference,
            _ => PreparedSourceClass::FunctionCall,
        },
    }
}

fn lazy_skipped_prepared_argument(
    ordinal: usize,
    expr: &CompiledExpr,
    reason: &str,
) -> PreparedArgument {
    PreparedArgument {
        ordinal,
        structure_class: PreparedStructureClass::Omitted,
        source_class: prepared_source_class(expr),
        evaluation_mode: PreparedEvaluationMode::EagerValue,
        blankness_class: PreparedBlanknessClass::Omitted,
        caller_context_sensitive: false,
        reference_target: None,
        opaque_reason: Some(reason.to_string()),
        resolved_value: None,
    }
}

fn prepared_argument_opaque_reason(expr: &CompiledExpr) -> Option<String> {
    match expr {
        CompiledExpr::Reference(CompiledReferenceExpr::Atom(NormalizedReference::External(_))) => {
            Some("external_reference_deferred".to_string())
        }
        _ => None,
    }
}

fn push_special_prepared_call(
    trace: &mut EvaluationTrace,
    function_name: &str,
    function_id: &'static str,
    arg_preparation_profile: ArgPreparationProfile,
    prepared_arguments: Vec<PreparedArgument>,
    context: &EvaluationContext<'_>,
) -> Option<usize> {
    if !context.records_prepared_calls() {
        return None;
    }
    let index = trace.prepared_calls.len();
    trace.prepared_calls.push(PreparedCall {
        function_name: function_name.to_string(),
        function_id,
        arg_preparation_profile,
        prepared_arguments,
        register_id_request: None,
        registered_external_call_request: None,
        locale_profile_id: context
            .locale_ctx
            .map(|ctx| format!("{:?}", ctx.profile.id)),
        date_system: context
            .locale_ctx
            .map(|ctx| format!("{:?}", ctx.date_system)),
        host_query_enabled: context.host_info.is_some(),
        returned_value: None,
    });
    Some(index)
}

fn push_prepared_call_unchecked(trace: &mut EvaluationTrace, call: PreparedCall) -> Option<usize> {
    let index = trace.prepared_calls.len();
    trace.prepared_calls.push(call);
    Some(index)
}

/// Stamp a value onto a `PreparedCall` previously pushed by
/// `push_special_prepared_call` (or the main eval path). Designed for
/// the trace.prepared_calls.push-then-call-then-record idiom: capture
/// the index at push time, evaluate, then record the returned value.
fn record_prepared_call_returned_value(
    trace: &mut EvaluationTrace,
    index: Option<usize>,
    value: &EvalValue,
) {
    let Some(index) = index else {
        return;
    };
    if let Some(call) = trace.prepared_calls.get_mut(index) {
        call.returned_value = Some(value.clone());
    }
}

fn prepared_result_from_eval_value(value: &EvalValue, plan: &SemanticPlan) -> PreparedResult {
    let format_hint = if plan.execution_profile.requires_locale {
        Some("locale_format_semantics".to_string())
    } else {
        None
    };
    let publication_hint = if plan.execution_profile.requires_host_query {
        Some("host_query_surface".to_string())
    } else {
        None
    };
    let capability_dependencies = prepared_result_capability_dependencies(plan);
    let deferred_reason = if matches!(value, EvalValue::Error(WorksheetErrorCode::Ref))
        && plan
            .capability_requirements
            .iter()
            .any(|item| item == "external_reference")
    {
        Some("external_reference_deferred".to_string())
    } else {
        None
    };

    match value {
        EvalValue::Number(number) => PreparedResult {
            result_class: PreparedResultClass::Scalar,
            structure_class: PreparedStructureClass::DirectScalar,
            payload_summary: format!("Number({number})"),
            blankness_class: PreparedBlanknessClass::NonBlank,
            reference_target: None,
            callable_carrier: None,
            callable_profile: None,
            callable_profile_detail: None,
            deferred_reason: deferred_reason.clone(),
            format_hint,
            publication_hint,
            capability_dependencies,
        },
        EvalValue::Text(text) => PreparedResult {
            result_class: PreparedResultClass::Scalar,
            structure_class: PreparedStructureClass::DirectScalar,
            payload_summary: format!("Text({})", text.to_string_lossy()),
            blankness_class: blankness_class_for_eval_value(value),
            reference_target: None,
            callable_carrier: None,
            callable_profile: None,
            callable_profile_detail: None,
            deferred_reason: deferred_reason.clone(),
            format_hint,
            publication_hint,
            capability_dependencies,
        },
        EvalValue::Logical(value) => PreparedResult {
            result_class: PreparedResultClass::Scalar,
            structure_class: PreparedStructureClass::DirectScalar,
            payload_summary: format!("Logical({value})"),
            blankness_class: PreparedBlanknessClass::NonBlank,
            reference_target: None,
            callable_carrier: None,
            callable_profile: None,
            callable_profile_detail: None,
            deferred_reason: deferred_reason.clone(),
            format_hint,
            publication_hint,
            capability_dependencies,
        },
        EvalValue::Error(code) => PreparedResult {
            result_class: PreparedResultClass::Error,
            structure_class: PreparedStructureClass::DirectScalar,
            payload_summary: format!("Error({code:?})"),
            blankness_class: PreparedBlanknessClass::NonBlank,
            reference_target: None,
            callable_carrier: None,
            callable_profile: None,
            callable_profile_detail: None,
            deferred_reason,
            format_hint,
            publication_hint,
            capability_dependencies,
        },
        EvalValue::Array(array) => PreparedResult {
            result_class: PreparedResultClass::Array,
            structure_class: PreparedStructureClass::ArrayLike,
            payload_summary: format!("Array({}x{})", array.shape().rows, array.shape().cols),
            blankness_class: PreparedBlanknessClass::NonBlank,
            reference_target: None,
            callable_carrier: None,
            callable_profile: None,
            callable_profile_detail: None,
            deferred_reason: deferred_reason.clone(),
            format_hint,
            publication_hint,
            capability_dependencies,
        },
        EvalValue::Reference(reference) => PreparedResult {
            result_class: PreparedResultClass::Reference,
            structure_class: PreparedStructureClass::ReferenceVisible,
            payload_summary: format!("Reference({:?})", reference.kind),
            blankness_class: PreparedBlanknessClass::NonBlank,
            reference_target: Some(reference.target.clone()),
            callable_carrier: None,
            callable_profile: None,
            callable_profile_detail: None,
            deferred_reason: deferred_reason.clone(),
            format_hint,
            publication_hint,
            capability_dependencies,
        },
        EvalValue::Lambda(name) => PreparedResult {
            result_class: PreparedResultClass::Scalar,
            structure_class: PreparedStructureClass::DirectScalar,
            payload_summary: format!("Lambda({})", lambda_summary(name)),
            blankness_class: PreparedBlanknessClass::NonBlank,
            reference_target: None,
            callable_carrier: callable_carrier_from_lambda_value(name),
            callable_profile: Some(lambda_summary(name).to_string()),
            callable_profile_detail: callable_profile_detail_from_lambda_value(name),
            deferred_reason: deferred_reason.clone(),
            format_hint,
            publication_hint,
            capability_dependencies,
        },
    }
}

fn blankness_class_for_eval_value(value: &EvalValue) -> PreparedBlanknessClass {
    match value {
        EvalValue::Text(text) if text.to_string_lossy().is_empty() => {
            PreparedBlanknessClass::EmptyText
        }
        _ => PreparedBlanknessClass::NonBlank,
    }
}

fn prepared_result_capability_dependencies(plan: &SemanticPlan) -> Vec<String> {
    let mut dependencies = plan
        .capability_requirements
        .iter()
        .filter(|requirement| {
            matches!(
                requirement.as_str(),
                "caller_context"
                    | "host_query"
                    | "locale_format_context"
                    | "time_provider"
                    | "random_provider"
                    | "helper_environment"
                    | "legacy_single_compat"
                    | "external_reference"
                    | "spill_reference"
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    dependencies.sort();
    dependencies.dedup();
    dependencies
}

fn callable_profile_detail_from_summary(summary: &str) -> Option<CallableValueProfile> {
    let mut arity = None;
    let mut required_arity = None;
    let mut parameter_names = None;
    let mut optional_parameter_names = None;
    let mut capture_names = None;
    let mut body_kind = None;

    for part in summary.split(';') {
        let (key, value) = part.split_once('=')?;
        match key {
            "arity" => {
                arity = value.parse::<usize>().ok();
            }
            "required_arity" => {
                required_arity = value.parse::<usize>().ok();
            }
            "params" => {
                parameter_names = Some(split_profile_list(value));
            }
            "optional_params" => {
                optional_parameter_names = Some(split_profile_list(value));
            }
            "captures" => {
                capture_names = Some(split_profile_list(value));
            }
            "body" => {
                body_kind = Some(value.to_string());
            }
            _ => {}
        }
    }

    let arity = arity?;
    Some(CallableValueProfile {
        arity,
        parameter_names: parameter_names.unwrap_or_default(),
        optional_parameter_names: optional_parameter_names.unwrap_or_default(),
        capture_names: capture_names.unwrap_or_default(),
        required_arity: required_arity.unwrap_or(arity),
        body_kind: body_kind?,
    })
}

fn callable_profile_detail_from_lambda_value(
    lambda: &OxLambdaValue,
) -> Option<CallableValueProfile> {
    callable_profile_detail_from_summary(lambda_summary(lambda))
}

fn callable_carrier_from_lambda_value(lambda: &OxLambdaValue) -> Option<CallableValueCarrier> {
    Some(CallableValueCarrier {
        origin_kind: callable_origin_kind_from_oxfunc(lambda.origin_kind),
        invocation_model: CallableInvocationModel::TypedInvocationOnly,
        capture_mode: callable_capture_mode_from_oxfunc(lambda.capture_mode),
        arity: lambda.arity_shape.max,
    })
}

fn callable_origin_kind_from_oxfunc(origin_kind: OxCallableOriginKind) -> CallableOriginKind {
    match origin_kind {
        OxCallableOriginKind::HelperLambda => CallableOriginKind::HelperLambda,
        OxCallableOriginKind::DefinedNameCallable => CallableOriginKind::DefinedNameCallable,
        OxCallableOriginKind::BuiltInCallable
        | OxCallableOriginKind::ExternalRegisteredCallable => CallableOriginKind::HelperLambda,
    }
}

fn callable_capture_mode_from_oxfunc(capture_mode: OxCallableCaptureMode) -> CallableCaptureMode {
    match capture_mode {
        OxCallableCaptureMode::NoCapture => CallableCaptureMode::NoCapture,
        OxCallableCaptureMode::LexicalCapture => CallableCaptureMode::LexicalCapture,
    }
}

fn oxfunc_origin_kind_from_local(origin_kind: CallableOriginKind) -> OxCallableOriginKind {
    match origin_kind {
        CallableOriginKind::HelperLambda => OxCallableOriginKind::HelperLambda,
        CallableOriginKind::DefinedNameCallable => OxCallableOriginKind::DefinedNameCallable,
    }
}

fn lambda_summary(lambda: &OxLambdaValue) -> &str {
    lambda
        .callable_token
        .split_once("::")
        .map(|(_, summary)| summary)
        .unwrap_or(lambda.callable_token.as_str())
}

fn split_profile_list(value: &str) -> Vec<String> {
    if value == "-" || value.is_empty() {
        Vec::new()
    } else if value.contains('|') {
        value.split('|').map(|item| item.to_string()).collect()
    } else {
        value.split(',').map(|item| item.to_string()).collect()
    }
}

fn decode_string_literal(text: &str) -> String {
    text.trim_matches('"').replace("\"\"", "\"")
}

fn map_resolution_error(error: RefResolutionError) -> EvaluationError {
    EvaluationError {
        message: format!("reference resolution failed: {error:?}"),
    }
}

fn error_code_for_error_ref(error: &ErrorRef) -> WorksheetErrorCode {
    match error.error_class.to_ascii_uppercase().as_str() {
        "#NULL!" => WorksheetErrorCode::Null,
        "#DIV/0!" => WorksheetErrorCode::Div0,
        "#VALUE!" => WorksheetErrorCode::Value,
        "#REF!" => WorksheetErrorCode::Ref,
        "#NAME?" => WorksheetErrorCode::Name,
        "#NUM!" => WorksheetErrorCode::Num,
        "#N/A" => WorksheetErrorCode::NA,
        "#BUSY!" => WorksheetErrorCode::Busy,
        "#GETTING_DATA" => WorksheetErrorCode::GettingData,
        "#SPILL!" => WorksheetErrorCode::Spill,
        "#CALC!" => WorksheetErrorCode::Calc,
        "#FIELD!" => WorksheetErrorCode::Field,
        "#BLOCKED!" => WorksheetErrorCode::Blocked,
        "#CONNECT!" => WorksheetErrorCode::Connect,
        _ => WorksheetErrorCode::Value,
    }
}

fn a1_for_cell(cell: &CellRef) -> String {
    qualified_reference_target(
        &cell.sheet_id,
        format!("{}{}", column_letters(cell.coord.col), cell.coord.row),
    )
}

fn a1_for_area(area: &AreaRef) -> String {
    let start = format!("{}{}", column_letters(area.top_left.col), area.top_left.row);
    let end_col = area.top_left.col + area.width - 1;
    let end_row = area.top_left.row + area.height - 1;
    let end = format!("{}{}", column_letters(end_col), end_row);
    qualified_reference_target(&area.sheet_id, format!("{start}:{end}"))
}

fn whole_row_target(rows: &crate::binding::WholeRowRef) -> String {
    let row_end = rows.row_start + rows.row_count - 1;
    qualified_reference_target(&rows.sheet_id, format!("{}:{}", rows.row_start, row_end))
}

fn whole_column_target(columns: &crate::binding::WholeColumnRef) -> String {
    let end_col = columns.col_start + columns.col_count - 1;
    qualified_reference_target(
        &columns.sheet_id,
        format!(
            "{}:{}",
            column_letters(columns.col_start),
            column_letters(end_col)
        ),
    )
}

fn reference_like_for_cell(cell: &CellRef) -> ReferenceLike {
    ReferenceLike {
        kind: ReferenceKind::A1,
        target: a1_for_cell(cell),
    }
}

fn reference_like_for_area(area: &AreaRef) -> ReferenceLike {
    ReferenceLike {
        kind: ReferenceKind::Area,
        target: a1_for_area(area),
    }
}

fn reference_like_for_structured(structured: &crate::binding::StructuredRef) -> ReferenceLike {
    match &structured.resolved_reference {
        StructuredResolvedRef::Cell(cell) => reference_like_for_cell(cell),
        StructuredResolvedRef::Area(area) => reference_like_for_area(area),
        StructuredResolvedRef::EmptyArea(empty) => ReferenceLike {
            kind: ReferenceKind::Structured,
            target: format!(
                "empty-structured:{}:{}:{}:{}",
                empty.sheet_id,
                match empty.section_kind {
                    StructuredSectionKind::All => "All",
                    StructuredSectionKind::Data => "Data",
                    StructuredSectionKind::Headers => "Headers",
                    StructuredSectionKind::Totals => "Totals",
                    StructuredSectionKind::ThisRow => "ThisRow",
                },
                empty.selected_column_ids.join("|"),
                empty.column_count
            ),
        },
    }
}

fn column_letters(mut col: u32) -> String {
    let mut letters = String::new();
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        letters.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    letters
}

fn qualified_reference_target(sheet_id: &str, local_target: String) -> String {
    if should_emit_sheet_prefix(sheet_id) {
        format!("{sheet_id}!{local_target}")
    } else {
        local_target
    }
}

fn should_emit_sheet_prefix(sheet_id: &str) -> bool {
    !(sheet_id.is_empty() || sheet_id.starts_with("sheet:"))
}

struct LocalReferenceResolver<'a> {
    cell_values: &'a BTreeMap<String, EvalValue>,
    defined_names: &'a BTreeMap<String, DefinedNameBinding>,
    sparse_reference_values: &'a BTreeMap<String, SparseReferenceValuesBinding>,
    caller_row: usize,
    caller_col: usize,
    callable_registry: &'a RefCell<CallableRegistry>,
}

impl ReferenceResolver for LocalReferenceResolver<'_> {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities::permissive_local()
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        if let Some(value) = self.cell_values.get(&reference.target) {
            return Ok(value.clone());
        }

        if let Some(array) = resolve_local_area_reference(self.cell_values, reference) {
            return Ok(EvalValue::Array(array));
        }

        if let Some(binding) = self.defined_names.get(&reference.target) {
            return match binding {
                DefinedNameBinding::Value(value) => Ok(value.clone()),
                DefinedNameBinding::Reference(reference_like) => {
                    self.resolve_reference(reference_like)
                }
                DefinedNameBinding::Callable(binding) => {
                    let lambda = lambda_binding_from_defined_name_binding(
                        binding,
                        self.callable_registry,
                    );
                    Ok(EvalValue::Lambda(
                        self.callable_registry.borrow_mut().register(lambda),
                    ))
                }
            };
        }

        if is_absent_single_cell_reference(reference) {
            return Ok(blank_single_cell_eval_value());
        }

        Err(RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        })
    }

    fn resolve_reference_values(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Option<ResolvedReferenceValues>, RefResolutionError> {
        Ok(self
            .sparse_reference_values
            .get(&reference.target)
            .filter(|binding| binding.reference == *reference)
            .map(|binding| binding.values.clone()))
    }

    fn caller_context(&self) -> Option<OxFuncCallerContext> {
        Some(OxFuncCallerContext {
            prefix: None,
            row: self.caller_row,
            col: self.caller_col,
        })
    }
}

struct OxFmlCallableInvoker<'a, 'b> {
    context: &'a EvaluationContext<'b>,
    callable_registry: &'a RefCell<CallableRegistry>,
}

impl CallableInvoker for OxFmlCallableInvoker<'_, '_> {
    fn invoke(
        &self,
        callable: &OxLambdaValue,
        args: &[PreparedArgValue],
    ) -> Result<PreparedArgValue, CallableInvocationError> {
        if callable.origin_kind == OxCallableOriginKind::BuiltInCallable {
            let call_target = self
                .callable_registry
                .borrow()
                .builtin_call_target(&callable.callable_token)
                .ok_or_else(|| {
                    CallableInvocationError::UnsupportedCallableToken(
                        callable.callable_token.clone(),
                    )
                })?;
            let call_args = args
                .iter()
                .map(|arg| call_arg_from_prepared(arg, self.callable_registry))
                .collect::<Vec<_>>();
            let resolver = LocalReferenceResolver {
                cell_values: &self.context.cell_values,
                defined_names: &self.context.defined_names,
                sparse_reference_values: &self.context.sparse_reference_values,
                caller_row: self.context.caller_row,
                caller_col: self.context.caller_col,
                callable_registry: self.callable_registry,
            };
            let mut fec = FunctionExecutionContextBundle::new(&resolver);
            fec.now_serial = self.context.now_serial;
            fec.random_provider = self.context.random_provider;
            fec.locale_ctx = self.context.locale_ctx;
            fec.host_info = self.context.host_info;
            fec.callable_invoker = Some(self);
            fec.rtd_provider = self.context.rtd_provider;
            fec.registered_external_provider = self.context.registered_external_provider;
            fec.reference_text_resolver = self.context.reference_text_resolver;
            let value = call_target
                .invoke(&call_args, &mut fec)
                .map_err(|_| CallableInvocationError::Worksheet(WorksheetErrorCode::Value))?;
            return Ok(prepared_arg_from_eval_value(value));
        }

        let binding = self
            .callable_registry
            .borrow()
            .get(&callable.callable_token)
            .cloned()
            .ok_or_else(|| {
                CallableInvocationError::UnsupportedCallableToken(callable.callable_token.clone())
            })?;
        let local_slot_count = lambda_slot_count(&binding.lambda.params, &binding.lambda.body);
        let mut local_bindings = binding.lambda.closure.with_min_slot_count(local_slot_count);
        for (param, arg) in binding.lambda.params.iter().zip(args.iter()) {
            insert_helper_slot_binding(
                &mut local_bindings,
                param.name.clone(),
                param.slot,
                HelperBinding::Arg(call_arg_from_prepared(arg, self.callable_registry)),
            );
        }
        for param in binding.lambda.params.iter().skip(args.len()) {
            insert_helper_slot_binding(
                &mut local_bindings,
                param.name.clone(),
                param.slot,
                HelperBinding::Arg(CallArgValue::MissingArg),
            );
        }
        let recursion_cost_units = LOCAL_CALLABLE_RECURSION_BASE_COST_UNITS
            + args
                .iter()
                .filter(|arg| prepared_arg_contains_lambda(arg, self.callable_registry))
                .count()
                * LOCAL_CALLABLE_RECURSION_LAMBDA_ARG_COST_UNITS;
        let Some(_recursion_guard) = try_enter_callable_recursion(
            &self.context.frame_state.callable_recursion_state,
            recursion_cost_units,
        ) else {
            return Ok(prepared_arg_from_eval_value(EvalValue::Error(
                WorksheetErrorCode::Num,
            )));
        };

        let mut trace = EvaluationTrace {
            prepared_calls: Vec::new(),
        };
        let mut resolver = LocalReferenceResolver {
            cell_values: &self.context.cell_values,
            defined_names: &self.context.defined_names,
            sparse_reference_values: &self.context.sparse_reference_values,
            caller_row: self.context.caller_row,
            caller_col: self.context.caller_col,
            callable_registry: self.callable_registry,
        };
        let value = with_callable_stack_guard(self.context, || {
            evaluate_expr_value(
                &binding.lambda.body,
                self.context,
                &mut resolver,
                &local_bindings,
                self.callable_registry,
                &mut trace,
            )
        })
        .map_err(|_| CallableInvocationError::Worksheet(WorksheetErrorCode::Value))?;
        Ok(prepared_arg_from_eval_value(value))
    }

    fn invoke_many(
        &self,
        callable: &OxLambdaValue,
        batch: &mut dyn CallableInvocationBatch,
    ) -> Result<(), CallableInvocationError> {
        let _mode = batch.mode();
        if callable.origin_kind == OxCallableOriginKind::BuiltInCallable {
            return self.invoke_many_builtin_callable(callable, batch);
        }

        let binding = self
            .callable_registry
            .borrow()
            .get(&callable.callable_token)
            .cloned()
            .ok_or_else(|| {
                CallableInvocationError::UnsupportedCallableToken(callable.callable_token.clone())
            })?;
        let param_names = binding
            .lambda
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<Vec<_>>();
        let param_keys = param_names
            .iter()
            .map(|name| helper_name_key(name))
            .collect::<Vec<_>>();
        let local_slot_count = lambda_slot_count(&binding.lambda.params, &binding.lambda.body);
        let mut local_bindings = binding.lambda.closure.with_min_slot_count(local_slot_count);
        prime_local_callable_param_slots(&mut local_bindings, &binding.lambda.params, &param_keys);
        let mut trace = EvaluationTrace {
            prepared_calls: Vec::new(),
        };
        let mut resolver = LocalReferenceResolver {
            cell_values: &self.context.cell_values,
            defined_names: &self.context.defined_names,
            sparse_reference_values: &self.context.sparse_reference_values,
            caller_row: self.context.caller_row,
            caller_col: self.context.caller_col,
            callable_registry: self.callable_registry,
        };
        let mut args = Vec::new();

        with_callable_stack_guard(self.context, || {
            while {
                args.clear();
                batch.prepare_next_args(&mut args)
            } {
                let argc = args.len();
                if !callable.arity_shape.accepts(argc) {
                    return Err(CallableInvocationError::ArityMismatch {
                        expected_min: callable.arity_shape.min,
                        expected_max: callable.arity_shape.max,
                        actual: argc,
                    });
                }
                let lambda_arg_count = set_local_callable_arg_slots(
                    &mut local_bindings,
                    &binding.lambda.params,
                    &param_keys,
                    &args,
                    self.callable_registry,
                );
                let recursion_cost_units = LOCAL_CALLABLE_RECURSION_BASE_COST_UNITS
                    + lambda_arg_count * LOCAL_CALLABLE_RECURSION_LAMBDA_ARG_COST_UNITS;
                let Some(_recursion_guard) = try_enter_callable_recursion(
                    &self.context.frame_state.callable_recursion_state,
                    recursion_cost_units,
                ) else {
                    batch.accept_result(prepared_arg_from_eval_value(EvalValue::Error(
                        WorksheetErrorCode::Num,
                    )))?;
                    continue;
                };

                trace.prepared_calls.clear();
                let value = evaluate_expr_value(
                    &binding.lambda.body,
                    self.context,
                    &mut resolver,
                    &local_bindings,
                    self.callable_registry,
                    &mut trace,
                )
                .map_err(|_| CallableInvocationError::Worksheet(WorksheetErrorCode::Value))?;
                batch.accept_result(prepared_arg_from_eval_value(value))?;
            }
            Ok(())
        })
    }
}

impl OxFmlCallableInvoker<'_, '_> {
    fn invoke_many_builtin_callable(
        &self,
        callable: &OxLambdaValue,
        batch: &mut dyn CallableInvocationBatch,
    ) -> Result<(), CallableInvocationError> {
        let call_target = self
            .callable_registry
            .borrow()
            .builtin_call_target(&callable.callable_token)
            .ok_or_else(|| {
                CallableInvocationError::UnsupportedCallableToken(callable.callable_token.clone())
            })?;
        let resolver = LocalReferenceResolver {
            cell_values: &self.context.cell_values,
            defined_names: &self.context.defined_names,
            sparse_reference_values: &self.context.sparse_reference_values,
            caller_row: self.context.caller_row,
            caller_col: self.context.caller_col,
            callable_registry: self.callable_registry,
        };
        let mut fec = FunctionExecutionContextBundle::new(&resolver);
        fec.now_serial = self.context.now_serial;
        fec.random_provider = self.context.random_provider;
        fec.locale_ctx = self.context.locale_ctx;
        fec.host_info = self.context.host_info;
        fec.callable_invoker = Some(self);
        fec.rtd_provider = self.context.rtd_provider;
        fec.registered_external_provider = self.context.registered_external_provider;
        fec.reference_text_resolver = self.context.reference_text_resolver;

        let mut scratch = call_target.new_scratch();
        let mut args = Vec::new();
        while {
            args.clear();
            batch.prepare_next_args(&mut args)
        } {
            let argc = args.len();
            if !callable.arity_shape.accepts(argc) {
                return Err(CallableInvocationError::ArityMismatch {
                    expected_min: callable.arity_shape.min,
                    expected_max: callable.arity_shape.max,
                    actual: argc,
                });
            }
            build_scratch_from_prepared_args(&mut scratch, &args, self.callable_registry);
            let value = call_target
                .invoke_scratch(&scratch, &mut fec)
                .map_err(|_| CallableInvocationError::Worksheet(WorksheetErrorCode::Value))?;
            batch.accept_result(prepared_arg_from_eval_value(value))?;
        }
        Ok(())
    }
}

fn build_scratch_from_prepared_args(
    scratch: &mut FunctionCallScratch,
    args: &[PreparedArgValue],
    callable_registry: &RefCell<CallableRegistry>,
) {
    scratch.clear();
    for arg in args {
        scratch.push_arg(call_arg_from_prepared(arg, callable_registry));
    }
}

fn prime_local_callable_param_slots(
    local_bindings: &mut HelperBindingFrame,
    params: &[LambdaParam],
    param_keys: &[String],
) {
    for (param, param_key) in params.iter().zip(param_keys.iter()) {
        local_bindings.insert_key(
            param_key.clone(),
            param.name.clone(),
            param.slot,
            HelperBinding::Arg(CallArgValue::MissingArg),
        );
    }
}

fn set_local_callable_arg_slots(
    local_bindings: &mut HelperBindingFrame,
    params: &[LambdaParam],
    param_keys: &[String],
    args: &[PreparedArgValue],
    callable_registry: &RefCell<CallableRegistry>,
) -> usize {
    let mut lambda_arg_count = 0usize;
    for ((param, param_key), arg) in params.iter().zip(param_keys.iter()).zip(args.iter()) {
        let call_arg = call_arg_from_prepared(arg, callable_registry);
        if matches!(call_arg, CallArgValue::Eval(EvalValue::Lambda(_))) {
            lambda_arg_count += 1;
        }
        let binding = HelperBinding::Arg(call_arg);
        if let Some(slot) = param.slot {
            local_bindings.set_slot(slot, binding);
        } else {
            local_bindings.set_key_binding(param_key, None, binding);
        }
    }
    for (param, param_key) in params.iter().zip(param_keys.iter()).skip(args.len()) {
        let binding = HelperBinding::Arg(CallArgValue::MissingArg);
        if let Some(slot) = param.slot {
            local_bindings.set_slot(slot, binding);
        } else {
            local_bindings.set_key_binding(param_key, None, binding);
        }
    }
    lambda_arg_count
}

fn call_arg_from_prepared(
    prepared: &PreparedArgValue,
    callable_registry: &RefCell<CallableRegistry>,
) -> CallArgValue {
    match prepared {
        PreparedArgValue::Eval(value) => CallArgValue::Eval(decode_callable_carrier_scalar(
            value.clone(),
            callable_registry,
        )),
        PreparedArgValue::MissingArg => CallArgValue::MissingArg,
        PreparedArgValue::EmptyCell => CallArgValue::EmptyCell,
    }
}

fn prepared_arg_contains_lambda(
    prepared: &PreparedArgValue,
    callable_registry: &RefCell<CallableRegistry>,
) -> bool {
    matches!(
        call_arg_from_prepared(prepared, callable_registry),
        CallArgValue::Eval(EvalValue::Lambda(_))
    )
}

fn prepared_arg_from_eval_value(value: EvalValue) -> PreparedArgValue {
    PreparedArgValue::Eval(encode_callable_array_transport_value(value))
}

fn encode_callable_array_transport_value(value: EvalValue) -> EvalValue {
    match value {
        EvalValue::Lambda(lambda) => {
            EvalValue::Text(callable_array_carrier_text(&lambda.callable_token))
        }
        other => other,
    }
}

fn decode_callable_carrier_function_result(
    function_id: &str,
    value: EvalValue,
    callable_registry: &RefCell<CallableRegistry>,
) -> EvalValue {
    match function_id {
        FUNC_ID_INDEX | FUNC_ID_XLOOKUP => decode_callable_carrier_scalar(value, callable_registry),
        _ => value,
    }
}

fn decode_callable_carrier_scalar(
    value: EvalValue,
    callable_registry: &RefCell<CallableRegistry>,
) -> EvalValue {
    match value {
        EvalValue::Text(text) => {
            decode_callable_carrier_text(&text, callable_registry).unwrap_or(EvalValue::Text(text))
        }
        other => other,
    }
}

fn sanitize_final_output_value(
    value: EvalValue,
    callable_registry: &RefCell<CallableRegistry>,
) -> EvalValue {
    match value {
        EvalValue::Text(text) => {
            decode_callable_carrier_text(&text, callable_registry).unwrap_or(EvalValue::Text(text))
        }
        EvalValue::Array(array) => {
            let shape = array.shape();
            let mut cells = Vec::with_capacity(shape.rows * shape.cols);
            for row in 0..shape.rows {
                for col in 0..shape.cols {
                    let cell = match array.get(row, col) {
                        Some(ArrayCellValue::Text(text))
                            if decode_callable_carrier_text(text, callable_registry).is_some() =>
                        {
                            ArrayCellValue::Error(WorksheetErrorCode::Calc)
                        }
                        Some(cell) => cell.clone(),
                        None => ArrayCellValue::EmptyCell,
                    };
                    cells.push(cell);
                }
            }
            EvalArray::new(shape, cells)
                .map(EvalValue::Array)
                .unwrap_or(EvalValue::Error(WorksheetErrorCode::Value))
        }
        other => other,
    }
}

fn callable_array_carrier_text(token: &str) -> ExcelText {
    ExcelText::from_interop_assignment(&format!("{}{}", CALLABLE_ARRAY_CARRIER_PREFIX, token))
}

fn decode_callable_carrier_text(
    text: &ExcelText,
    callable_registry: &RefCell<CallableRegistry>,
) -> Option<EvalValue> {
    let token = text
        .to_string_lossy()
        .strip_prefix(CALLABLE_ARRAY_CARRIER_PREFIX)?
        .to_string();
    callable_registry
        .borrow()
        .value(&token)
        .map(EvalValue::Lambda)
}

fn resolve_local_area_reference(
    cell_values: &BTreeMap<String, EvalValue>,
    reference: &ReferenceLike,
) -> Option<EvalArray> {
    if !matches!(reference.kind, ReferenceKind::Area) {
        return None;
    }

    let (start, end) = reference.target.split_once(':')?;
    let (start_sheet, start_row, start_col) = parse_a1_target(start)?;
    let (end_sheet, end_row, end_col) =
        parse_a1_target_with_default_sheet(end, start_sheet.as_deref())?;
    if start_sheet != end_sheet {
        return None;
    }

    let top = start_row.min(end_row);
    let bottom = start_row.max(end_row);
    let left = start_col.min(end_col);
    let right = start_col.max(end_col);
    let prefix = start_sheet.as_deref();

    let rows = (top..=bottom)
        .map(|row| {
            (left..=right)
                .map(|col| {
                    let a1 = format!("{}{}", column_letters(col), row);
                    let target = prefix.map(|sheet| format!("{sheet}!{a1}")).unwrap_or(a1);
                    array_cell_value_from_eval_value(cell_values.get(&target).cloned())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    EvalArray::from_rows(rows)
}

fn array_cell_value_from_eval_value(value: Option<EvalValue>) -> ArrayCellValue {
    match value {
        Some(EvalValue::Number(number)) => ArrayCellValue::Number(number),
        Some(EvalValue::Text(text)) => ArrayCellValue::Text(text),
        Some(EvalValue::Logical(value)) => ArrayCellValue::Logical(value),
        Some(EvalValue::Error(code)) => ArrayCellValue::Error(code),
        Some(EvalValue::Lambda(lambda)) => {
            ArrayCellValue::Text(callable_array_carrier_text(&lambda.callable_token))
        }
        Some(EvalValue::Array(_)) | Some(EvalValue::Reference(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
        None => ArrayCellValue::EmptyCell,
    }
}

fn call_arg_from_resolved_reference_value(value: EvalValue) -> CallArgValue {
    if is_scalar_empty_cell_array(&value) {
        CallArgValue::EmptyCell
    } else {
        CallArgValue::Eval(value)
    }
}

fn is_scalar_empty_cell_array(value: &EvalValue) -> bool {
    match value {
        EvalValue::Array(array) if array.shape().rows == 1 && array.shape().cols == 1 => {
            matches!(array.get(0, 0), Some(ArrayCellValue::EmptyCell))
        }
        _ => false,
    }
}

fn blank_single_cell_eval_value() -> EvalValue {
    EvalValue::Array(EvalArray::from_scalar(ArrayCellValue::EmptyCell))
}

fn is_absent_single_cell_reference(reference: &ReferenceLike) -> bool {
    matches!(reference.kind, ReferenceKind::A1) && parse_a1_target(&reference.target).is_some()
}

fn parse_a1_target(text: &str) -> Option<(Option<String>, u32, u32)> {
    parse_a1_target_with_default_sheet(text, None)
}

fn parse_a1_target_with_default_sheet(
    text: &str,
    default_sheet: Option<&str>,
) -> Option<(Option<String>, u32, u32)> {
    let (sheet, address) = match text.rsplit_once('!') {
        Some((sheet, address)) => (Some(sheet.to_string()), address),
        None => (default_sheet.map(|sheet| sheet.to_string()), text),
    };

    let col_len = address
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .count();
    if col_len == 0 || col_len == address.len() {
        return None;
    }

    let (col_text, row_text) = address.split_at(col_len);
    let row = row_text.parse::<u32>().ok()?;
    let col = column_number(col_text)?;
    Some((sheet, row, col))
}

fn column_number(text: &str) -> Option<u32> {
    let mut value = 0u32;
    for ch in text.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        value = value.checked_mul(26)?;
        value = value.checked_add((ch.to_ascii_uppercase() as u32) - ('A' as u32) + 1)?;
    }
    Some(value)
}

const CALLABLE_ARRAY_CARRIER_PREFIX: &str = "oxfml.callable-array::";

fn callable_token(id: usize, summary: &str) -> String {
    format!("oxfml.callable.{id}::{summary}")
}

#[cfg(test)]
mod compiled_body_cache_tests {
    use super::*;
    use crate::binding::{BinaryOp, BoundExpr};

    fn add_body() -> BoundExpr {
        BoundExpr::Binary {
            op: BinaryOp::Add,
            left: Box::new(BoundExpr::HelperParameterName("x".to_string())),
            right: Box::new(BoundExpr::NumberLiteral("3".to_string())),
        }
    }

    #[test]
    fn compiled_body_cache_reuses_same_shape_and_separates_distinct_shapes() {
        let registry = RefCell::new(CallableRegistry::default());

        // The same body shape compiles once: a second request returns the SAME
        // Rc (a cache hit, not a recompile). This is the property that keeps
        // B(x) inside MAP(arr, LAMBDA(x, B(x))) from recompiling per element.
        let first = registry.borrow_mut().get_or_compile_body(&add_body());
        let second = registry.borrow_mut().get_or_compile_body(&add_body());
        assert!(
            Rc::ptr_eq(&first, &second),
            "same body shape should reuse one compiled body"
        );

        // An equal-by-value clone hits the same cache entry (structural identity).
        let clone = add_body();
        let third = registry.borrow_mut().get_or_compile_body(&clone);
        assert!(Rc::ptr_eq(&first, &third));

        // A different shape compiles separately.
        let mul_body = BoundExpr::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(BoundExpr::HelperParameterName("x".to_string())),
            right: Box::new(BoundExpr::NumberLiteral("3".to_string())),
        };
        let other = registry.borrow_mut().get_or_compile_body(&mul_body);
        assert!(!Rc::ptr_eq(&first, &other));

        assert_eq!(registry.borrow().compiled_body_cache.len(), 2);
    }
}
