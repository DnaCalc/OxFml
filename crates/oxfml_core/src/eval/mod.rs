use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use oxfunc_core::function::ArgPreparationProfile;
use oxfunc_core::functions::adapters::PreparedArgValue;
use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalCallRequest, RegisteredExternalProvider,
    parse_call_request, parse_register_id_request,
};
use oxfunc_core::functions::callable_helpers::{CallableInvocationError, CallableInvoker};
use oxfunc_core::functions::cell::{CellEvalError, eval_cell_surface};
use oxfunc_core::functions::info_fn::{InfoEvalError, eval_info_surface};
use oxfunc_core::functions::op_implicit_intersection::{
    eval_op_implicit_intersection_surface, map_op_implicit_intersection_error_to_ws,
};
use oxfunc_core::functions::rtd_fn::RtdProvider;
use oxfunc_core::functions::surface_dispatch::{
    eval_surface_extended_call, eval_surface_value_call_with_callable,
    FUNC_ID_OP_ADD, FUNC_ID_OP_CONCAT, FUNC_ID_OP_DIVIDE, FUNC_ID_OP_EQUAL,
    FUNC_ID_OP_GREATER_EQUAL, FUNC_ID_OP_GREATER_THAN, FUNC_ID_OP_LESS_EQUAL,
    FUNC_ID_OP_INTERSECTION_REF, FUNC_ID_OP_LESS_THAN, FUNC_ID_OP_MULTIPLY, FUNC_ID_OP_NEGATE,
    FUNC_ID_OP_NOT_EQUAL, FUNC_ID_OP_PERCENT, FUNC_ID_OP_POWER, FUNC_ID_OP_RANGE_REF,
    FUNC_ID_OP_SPILL_REF, FUNC_ID_OP_SUBTRACT, FUNC_ID_OP_UNARY_PLUS, FUNC_ID_OP_UNION_REF,
};
use oxfunc_core::host_info::HostInfoProvider;
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::resolver::resolve_eval_value as resolve_oxfunc_eval_value;
use oxfunc_core::resolver::{
    CallerContext as OxFuncCallerContext, RefResolutionError, ReferenceResolver,
    ResolverCapabilities,
};
use oxfunc_core::value::{
    ArrayCellValue, CallArgValue, CallableArityShape as OxCallableArityShape,
    CallableCaptureMode as OxCallableCaptureMode, CallableOriginKind as OxCallableOriginKind,
    EvalArray, EvalValue, ExcelText, ExtendedValue, LambdaValue as OxLambdaValue, ReferenceKind,
    ReferenceLike, WorksheetErrorCode,
};

use crate::binding::{
    AreaRef, BoundExpr, BoundFormula, CellRef, ErrorRef, NameRef, NormalizedReference,
    ReferenceExpr, StructuredResolvedRef,
};
use crate::interface::{ReturnedValueSurface, TypedContextQueryBundle};
use crate::semantics::{SemanticPlan, lookup_function_meta};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedArgument {
    pub ordinal: usize,
    pub structure_class: PreparedStructureClass,
    pub source_class: PreparedSourceClass,
    pub evaluation_mode: PreparedEvaluationMode,
    pub blankness_class: PreparedBlanknessClass,
    pub caller_context_sensitive: bool,
    pub reference_target: Option<String>,
    pub opaque_reason: Option<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationTrace {
    pub prepared_calls: Vec<PreparedCall>,
}

const SPECIAL_LET_FUNCTION_ID: &str = "SPECIAL.LET";
const SPECIAL_LAMBDA_FUNCTION_ID: &str = "SPECIAL.LAMBDA";
const SPECIAL_LEGACY_SINGLE_FUNCTION_ID: &str = "SPECIAL.LEGACY_SINGLE";
const SPECIAL_EXTERNAL_REFERENCE_DEFERRED_FUNCTION_ID: &str = "SPECIAL.EXTERNAL_REFERENCE_DEFERRED";
const HELPER_LAMBDA_INVOCATION_CONTRACT_REF: &str = "oxfml.helper_lambda.invoke.v1";
const BUILTIN_CALLABLE_INVOCATION_CONTRACT_REF: &str = "oxfml.builtin_callable.invoke.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationBackend {
    LocalBootstrap,
    OxFuncBacked,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationOutput {
    pub result: PreparedResult,
    pub oxfunc_value: EvalValue,
    pub returned_value_surface: ReturnedValueSurface,
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
enum HelperBinding {
    Arg(CallArgValue),
    Lambda {
        params: Vec<LambdaParam>,
        body: BoundExpr,
        closure: BTreeMap<String, HelperBinding>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LambdaParam {
    name: String,
    optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct LambdaBinding {
    origin_kind: CallableOriginKind,
    params: Vec<LambdaParam>,
    body: BoundExpr,
    closure: BTreeMap<String, HelperBinding>,
}

#[derive(Debug, Clone, PartialEq)]
struct RegisteredCallableBinding {
    lambda: LambdaBinding,
}

#[derive(Debug, Default)]
struct CallableRegistry {
    next_id: usize,
    bindings: BTreeMap<String, RegisteredCallableBinding>,
}

impl CallableRegistry {
    fn register(&mut self, lambda: LambdaBinding) -> OxLambdaValue {
        self.next_id += 1;
        let token = callable_token(self.next_id, &lambda_value_summary_from_binding(&lambda));
        let oxfunc_value = OxLambdaValue::new(
            token.clone(),
            oxfunc_origin_kind_from_local(lambda.origin_kind),
            OxCallableArityShape::range(
                lambda_required_arity(&lambda.params),
                lambda.params.len(),
            ),
            if lambda.closure.is_empty() {
                OxCallableCaptureMode::NoCapture
            } else {
                OxCallableCaptureMode::LexicalCapture
            },
            HELPER_LAMBDA_INVOCATION_CONTRACT_REF,
        );
        self.bindings
            .insert(token, RegisteredCallableBinding { lambda });
        oxfunc_value
    }

    fn get(&self, token: &str) -> Option<&RegisteredCallableBinding> {
        self.bindings.get(token)
    }
}

pub struct EvaluationContext<'a> {
    pub bind_formula: &'a BoundFormula,
    pub plan: &'a SemanticPlan,
    pub backend: EvaluationBackend,
    pub caller_row: usize,
    pub caller_col: usize,
    pub cell_values: BTreeMap<String, EvalValue>,
    pub defined_names: BTreeMap<String, DefinedNameBinding>,
    pub locale_ctx: Option<&'a LocaleFormatContext<'a>>,
    pub host_info: Option<&'a dyn HostInfoProvider>,
    pub rtd_provider: Option<&'a dyn RtdProvider>,
    pub registered_external_provider: Option<&'a dyn RegisteredExternalProvider>,
    pub now_serial: Option<f64>,
    pub random_value: Option<f64>,
}

impl<'a> EvaluationContext<'a> {
    pub fn new(bind_formula: &'a BoundFormula, plan: &'a SemanticPlan) -> Self {
        Self {
            bind_formula,
            plan,
            backend: EvaluationBackend::OxFuncBacked,
            caller_row: 1,
            caller_col: 1,
            cell_values: BTreeMap::new(),
            defined_names: BTreeMap::new(),
            locale_ctx: None,
            host_info: None,
            rtd_provider: None,
            registered_external_provider: None,
            now_serial: None,
            random_value: None,
        }
    }

    pub fn typed_context_query_bundle(&self) -> TypedContextQueryBundle<'a> {
        TypedContextQueryBundle::new(
            self.host_info,
            self.rtd_provider,
            self.locale_ctx,
            self.now_serial,
            self.random_value,
        )
        .with_registered_external_provider(self.registered_external_provider)
    }

    pub fn apply_typed_context_query_bundle(&mut self, bundle: TypedContextQueryBundle<'a>) {
        self.host_info = bundle.host_info;
        self.rtd_provider = bundle.rtd_provider;
        self.registered_external_provider = bundle.registered_external_provider;
        self.locale_ctx = bundle.locale_ctx;
        self.now_serial = bundle.now_serial;
        self.random_value = bundle.random_value;
    }
}

pub fn evaluate_formula(
    context: EvaluationContext<'_>,
) -> Result<EvaluationOutput, EvaluationError> {
    let mut trace = EvaluationTrace {
        prepared_calls: Vec::new(),
    };
    let callable_registry = RefCell::new(CallableRegistry::default());
    let mut resolver = LocalReferenceResolver {
        cell_values: &context.cell_values,
        defined_names: &context.defined_names,
        caller_row: context.caller_row,
        caller_col: context.caller_col,
        callable_registry: &callable_registry,
    };
    let helper_bindings = BTreeMap::new();

    let value = evaluate_expr_value(
        &context.bind_formula.root,
        &context,
        &mut resolver,
        &helper_bindings,
        &callable_registry,
        &mut trace,
    )?;

    Ok(EvaluationOutput {
        result: prepared_result_from_eval_value(&value, context.plan),
        returned_value_surface: returned_value_surface_for_output(
            &context.bind_formula.root,
            &value,
            &context,
        ),
        oxfunc_value: value,
        trace,
    })
}

fn returned_value_surface_for_output(
    root: &BoundExpr,
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

fn typed_surface_for_top_level_host_or_provider_call(
    root: &BoundExpr,
    context: &EvaluationContext<'_>,
) -> Option<ReturnedValueSurface> {
    match root {
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if function_name == "RTD" && context.rtd_provider.is_some() => {
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
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
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if function_name == "INFO" && context.host_info.is_some() => {
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
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
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if function_name == "CELL" && context.host_info.is_some() => {
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
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
    root: &BoundExpr,
    context: &EvaluationContext<'_>,
) -> Option<ReturnedValueSurface> {
    match root {
        BoundExpr::FunctionCall {
            function_name,
            args,
        } => {
            let meta = lookup_function_meta(function_name)?;
            let call_args = build_top_level_call_args(args, context, true).ok()?;
            let callable_registry = RefCell::new(CallableRegistry::default());
            let resolver = LocalReferenceResolver {
                cell_values: &context.cell_values,
                defined_names: &context.defined_names,
                caller_row: context.caller_row,
                caller_col: context.caller_col,
                callable_registry: &callable_registry,
            };
            let extended = eval_surface_extended_call(
                meta.function_id,
                &call_args,
                &resolver,
                context.now_serial,
                context.random_value,
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
    args: &[BoundExpr],
    context: &EvaluationContext<'_>,
    preserve_reference: bool,
) -> Result<Vec<CallArgValue>, EvaluationError> {
    let callable_registry = RefCell::new(CallableRegistry::default());
    let mut resolver = LocalReferenceResolver {
        cell_values: &context.cell_values,
        defined_names: &context.defined_names,
        caller_row: context.caller_row,
        caller_col: context.caller_col,
        callable_registry: &callable_registry,
    };
    let helper_bindings = BTreeMap::new();
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

fn evaluate_expr_value(
    expr: &BoundExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    match expr {
        BoundExpr::NumberLiteral(text) => {
            text.parse::<f64>()
                .map(EvalValue::Number)
                .map_err(|_| EvaluationError {
                    message: format!("failed to parse numeric literal {text}"),
                })
        }
        BoundExpr::LogicalLiteral(value) => Ok(EvalValue::Logical(*value)),
        BoundExpr::StringLiteral(text) => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
            decode_string_literal(text).encode_utf16().collect(),
        ))),
        BoundExpr::ArrayLiteral(rows) => evaluate_array_literal(
            rows,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        BoundExpr::OmittedArgument => Ok(EvalValue::Error(WorksheetErrorCode::Value)),
        BoundExpr::HelperParameterName(name) | BoundExpr::HelperOptionalParameterName(name) => Err(EvaluationError {
            message: format!(
                "helper parameter {name} cannot be evaluated without helper-form environment support"
            ),
        }),
        BoundExpr::Binary { op, left, right } => {
            evaluate_binary_operator_call(
                *op,
                left,
                right,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            )
        }
        BoundExpr::Unary { op, expr } => evaluate_unary_operator_call(
            *op,
            expr,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        BoundExpr::FunctionCall {
            function_name,
            args,
        } => evaluate_function_call(
            function_name,
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        BoundExpr::Invocation { callee, args } => evaluate_invocation(
            callee,
            args,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            trace,
        ),
        BoundExpr::Reference(reference) => {
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
        BoundExpr::ImplicitIntersection(inner) => {
            let arg = evaluate_expr_as_call_arg(
                inner,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                true,
                false,
                trace,
            )?;
            Ok(
                eval_op_implicit_intersection_surface(&[arg], resolver).unwrap_or_else(|error| {
                    EvalValue::Error(map_op_implicit_intersection_error_to_ws(&error))
                }),
            )
        }
    }
}

fn evaluate_function_call(
    function_name: &str,
    args: &[BoundExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    match function_name {
        "LET" => {
            return evaluate_let_call(
                args,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            );
        }
        "LAMBDA" => {
            return evaluate_lambda_call(args, helper_bindings, callable_registry, context, trace);
        }
        "_XLFN.SINGLE" | "SINGLE" => {
            return evaluate_legacy_single_call(
                args,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            );
        }
        _ => {}
    }

    let meta = lookup_function_meta(function_name).ok_or_else(|| EvaluationError {
        message: format!("no registered function metadata for {function_name}"),
    })?;

    if context.backend == EvaluationBackend::LocalBootstrap {
        return Err(EvaluationError {
            message: format!(
                "local bootstrap backend does not support function calls: {function_name}"
            ),
        });
    }

    let mut prepared_arguments = Vec::with_capacity(args.len());
    let mut call_args = Vec::with_capacity(args.len());
    for (ordinal, arg) in args.iter().enumerate() {
        let preserve_reference =
            meta.arg_preparation_profile == ArgPreparationProfile::RefsVisibleInAdapter;
        let callable_slot = is_builtin_callable_slot(function_name, ordinal);
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
        prepared_arguments.push(prepared_argument_for_call_arg(
            ordinal,
            arg,
            &call_arg,
            preserve_reference,
        ));
        call_args.push(call_arg);
    }

    // Excel trailing omitted arguments behave like absent optional arguments at the
    // function-surface boundary. Preserve interior omissions as `MissingArg`, but do not
    // force trailing omitted placeholders into OxFunc's optional-argument lanes.
    let trailing_omitted_count = args
        .iter()
        .rev()
        .take_while(|arg| matches!(arg, BoundExpr::OmittedArgument))
        .count();
    for _ in 0..trailing_omitted_count {
        if !matches!(call_args.last(), Some(CallArgValue::MissingArg)) {
            break;
        }
        prepared_arguments.pop();
        call_args.pop();
    }

    let register_id_request = if function_name == "REGISTER.ID" {
        parse_register_id_request(&call_args, resolver).ok()
    } else {
        None
    };
    let registered_external_call_request = if function_name == "CALL" {
        parse_call_request(&call_args, resolver).ok()
    } else {
        None
    };

    trace.prepared_calls.push(PreparedCall {
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
    });

    let callable_invoker = OxFmlCallableInvoker {
        context,
        callable_registry,
    };

    match eval_surface_value_call_with_callable(
        meta.function_id,
        &call_args,
        resolver,
        context.now_serial,
        context.random_value,
        context.locale_ctx,
        context.host_info,
        Some(&callable_invoker),
        context.rtd_provider,
        context.registered_external_provider,
    ) {
        Ok(value) => Ok(value),
        Err(_error)
            if allow_host_query_worksheet_error_fallback(
                function_name,
                &call_args,
                resolver,
                context.host_info,
            ) =>
        {
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        }
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

fn allow_host_query_worksheet_error_fallback(
    function_name: &str,
    call_args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
) -> bool {
    match function_name {
        "INFO" => matches!(
            eval_info_surface(call_args, resolver, host_info),
            Err(InfoEvalError::HostInfo(_))
        ),
        "CELL" => matches!(
            eval_cell_surface(call_args, resolver, host_info),
            Err(CellEvalError::HostInfo(_))
        ),
        _ => false,
    }
}

fn evaluate_expr_as_call_arg(
    expr: &BoundExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
    callable_slot: bool,
    trace: &mut EvaluationTrace,
) -> Result<CallArgValue, EvaluationError> {
    if callable_slot {
        if let Some(callable_arg) = built_in_callable_arg_for_expr(expr, context)? {
            return Ok(callable_arg);
        }
    }

    match expr {
        BoundExpr::OmittedArgument => Ok(CallArgValue::MissingArg),
        BoundExpr::Reference(reference) => evaluate_reference_as_call_arg(
            reference,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            callable_slot,
            trace,
        ),
        BoundExpr::ImplicitIntersection(inner) => {
            let arg = evaluate_expr_as_call_arg(
                inner,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                true,
                false,
                trace,
            )?;
            Ok(CallArgValue::Eval(
                eval_op_implicit_intersection_surface(&[arg], resolver).unwrap_or_else(|error| {
                    EvalValue::Error(map_op_implicit_intersection_error_to_ws(&error))
                }),
            ))
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
    expr: &BoundExpr,
    context: &EvaluationContext<'_>,
) -> Result<Option<CallArgValue>, EvaluationError> {
    match expr {
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(name)))
            if !context.defined_names.contains_key(&name.name) =>
        {
            built_in_callable_arg_for_name(name).map(Some)
        }
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(error))) => {
            built_in_callable_arg_for_function_name(&error.source_text).map(Some)
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if args.is_empty() => built_in_callable_arg_for_function_name(function_name).map(Some),
        _ => Ok(None),
    }
}

fn evaluate_reference_as_call_arg(
    reference: &ReferenceExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
    callable_slot: bool,
    trace: &mut EvaluationTrace,
) -> Result<CallArgValue, EvaluationError> {
    match reference {
        ReferenceExpr::Atom(NormalizedReference::Cell(cell)) => {
            call_arg_for_reference_like(reference_like_for_cell(cell), preserve_reference, resolver)
        }
        ReferenceExpr::Atom(NormalizedReference::Area(area)) => {
            call_arg_for_reference_like(reference_like_for_area(area), preserve_reference, resolver)
        }
        ReferenceExpr::Atom(NormalizedReference::WholeRow(rows)) => call_arg_for_reference_like(
            ReferenceLike {
                kind: ReferenceKind::Area,
                target: whole_row_target(rows),
            },
            preserve_reference,
            resolver,
        ),
        ReferenceExpr::Atom(NormalizedReference::WholeColumn(columns)) => {
            call_arg_for_reference_like(
                ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: whole_column_target(columns),
                },
                preserve_reference,
                resolver,
            )
        }
        ReferenceExpr::Atom(NormalizedReference::Name(name)) => call_arg_for_name(
            name,
            preserve_reference,
            callable_slot,
            context,
            resolver,
            helper_bindings,
            callable_registry,
        ),
        ReferenceExpr::Atom(NormalizedReference::Structured(structured)) => {
            call_arg_for_reference_like(
                reference_like_for_structured(structured),
                preserve_reference,
                resolver,
            )
        }
        ReferenceExpr::Atom(NormalizedReference::External(external)) => {
            push_special_prepared_call(
                trace,
                "EXTERNAL_REFERENCE_DEFERRED",
                SPECIAL_EXTERNAL_REFERENCE_DEFERRED_FUNCTION_ID,
                ArgPreparationProfile::RefsVisibleInAdapter,
                vec![PreparedArgument {
                    ordinal: 0,
                    structure_class: PreparedStructureClass::ReferenceVisible,
                    source_class: PreparedSourceClass::ExternalReference,
                    evaluation_mode: PreparedEvaluationMode::ReferencePreserved,
                    blankness_class: PreparedBlanknessClass::NonBlank,
                    caller_context_sensitive: false,
                    reference_target: Some(external.target_summary.clone()),
                    opaque_reason: Some("external_reference_deferred".to_string()),
                }],
                context,
            );
            Ok(CallArgValue::Eval(EvalValue::Error(
                WorksheetErrorCode::Ref,
            )))
        }
        ReferenceExpr::Atom(NormalizedReference::Error(error)) => Ok(CallArgValue::Eval(
            EvalValue::Error(error_code_for_error_ref(error)),
        )),
        ReferenceExpr::Spill { anchor } => {
            evaluate_reference_operator_call(
                "OP_SPILL_REF",
                FUNC_ID_OP_SPILL_REF,
                vec![anchor.as_ref()],
                context,
                resolver,
                helper_bindings,
                callable_registry,
                preserve_reference,
                trace,
            )
        }
        ReferenceExpr::Range { start, end } => evaluate_reference_operator_call(
            "OP_RANGE_REF",
            FUNC_ID_OP_RANGE_REF,
            vec![start.as_ref(), end.as_ref()],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            trace,
        ),
        ReferenceExpr::Union { left, right } => evaluate_reference_operator_call(
            "OP_UNION_REF",
            FUNC_ID_OP_UNION_REF,
            vec![left.as_ref(), right.as_ref()],
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            trace,
        ),
        ReferenceExpr::Intersection { left, right } => evaluate_reference_operator_call(
            "OP_INTERSECTION_REF",
            FUNC_ID_OP_INTERSECTION_REF,
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
    function_id: &'static str,
    operands: Vec<&ReferenceExpr>,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
    trace: &mut EvaluationTrace,
) -> Result<CallArgValue, EvaluationError> {
    let mut args = Vec::with_capacity(operands.len());
    let mut prepared_arguments = Vec::with_capacity(operands.len());
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
        let expr = BoundExpr::Reference(operand.clone());
        prepared_arguments.push(prepared_argument_for_call_arg(ordinal, &expr, &arg, true));
        args.push(arg);
    }

    push_special_prepared_call(
        trace,
        function_name,
        function_id,
        ArgPreparationProfile::RefsVisibleInAdapter,
        prepared_arguments,
        context,
    );

    let callable_invoker = OxFmlCallableInvoker {
        context,
        callable_registry,
    };
    let result = eval_surface_value_call_with_callable(
        function_id,
        &args,
        resolver,
        context.now_serial,
        context.random_value,
        context.locale_ctx,
        context.host_info,
        Some(&callable_invoker),
        context.rtd_provider,
        context.registered_external_provider,
    );
    let value = match result {
        Ok(value) => value,
        Err(code) => EvalValue::Error(code),
    };
    call_arg_from_reference_operator_value(value, preserve_reference, resolver)
}

fn call_arg_from_reference_operator_value(
    value: EvalValue,
    preserve_reference: bool,
    resolver: &mut LocalReferenceResolver<'_>,
) -> Result<CallArgValue, EvaluationError> {
    match value {
        EvalValue::Reference(reference) if preserve_reference => Ok(CallArgValue::Reference(reference)),
        EvalValue::Reference(reference) => resolve_oxfunc_eval_value(resolver, &reference)
            .map(call_arg_from_resolved_reference_value)
            .map_err(map_resolution_error),
        other => Ok(CallArgValue::Eval(other)),
    }
}

fn call_arg_for_reference_like(
    reference: ReferenceLike,
    preserve_reference: bool,
    resolver: &mut LocalReferenceResolver<'_>,
) -> Result<CallArgValue, EvaluationError> {
    if preserve_reference {
        Ok(CallArgValue::Reference(reference))
    } else {
        resolve_oxfunc_eval_value(resolver, &reference)
            .map(call_arg_from_resolved_reference_value)
            .map_err(map_resolution_error)
    }
}

fn evaluate_array_literal(
    rows: &[Vec<BoundExpr>],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
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
    op: crate::binding::BinaryOp,
    left: &BoundExpr,
    right: &BoundExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
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

    push_special_prepared_call(
        trace,
        function_name,
        function_id,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        vec![
            prepared_argument_for_call_arg(0, left, &lhs, false),
            prepared_argument_for_call_arg(1, right, &rhs, false),
        ],
        context,
    );

    let callable_invoker = OxFmlCallableInvoker {
        context,
        callable_registry,
    };
    let result = eval_surface_value_call_with_callable(
        function_id,
        &[lhs, rhs],
        resolver,
        context.now_serial,
        context.random_value,
        context.locale_ctx,
        context.host_info,
        Some(&callable_invoker),
        context.rtd_provider,
        context.registered_external_provider,
    );
    match result {
        Ok(value) => Ok(value),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

fn evaluate_unary_operator_call(
    op: crate::binding::UnaryOp,
    expr: &BoundExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
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

    push_special_prepared_call(
        trace,
        function_name,
        function_id,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        vec![prepared_argument_for_call_arg(0, expr, &arg, false)],
        context,
    );

    let callable_invoker = OxFmlCallableInvoker {
        context,
        callable_registry,
    };
    let result = eval_surface_value_call_with_callable(
        function_id,
        &[arg],
        resolver,
        context.now_serial,
        context.random_value,
        context.locale_ctx,
        context.host_info,
        Some(&callable_invoker),
        context.rtd_provider,
        context.registered_external_provider,
    );
    match result {
        Ok(value) => Ok(value),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

fn binary_operator_identity(op: crate::binding::BinaryOp) -> (&'static str, &'static str) {
    match op {
        crate::binding::BinaryOp::Add => ("OP_ADD", FUNC_ID_OP_ADD),
        crate::binding::BinaryOp::Subtract => ("OP_SUBTRACT", FUNC_ID_OP_SUBTRACT),
        crate::binding::BinaryOp::Power => ("OP_POWER", FUNC_ID_OP_POWER),
        crate::binding::BinaryOp::Multiply => ("OP_MULTIPLY", FUNC_ID_OP_MULTIPLY),
        crate::binding::BinaryOp::Divide => ("OP_DIVIDE", FUNC_ID_OP_DIVIDE),
        crate::binding::BinaryOp::Concat => ("OP_CONCAT", FUNC_ID_OP_CONCAT),
        crate::binding::BinaryOp::Equal => ("OP_EQUAL", FUNC_ID_OP_EQUAL),
        crate::binding::BinaryOp::NotEqual => ("OP_NOT_EQUAL", FUNC_ID_OP_NOT_EQUAL),
        crate::binding::BinaryOp::LessThan => ("OP_LESS_THAN", FUNC_ID_OP_LESS_THAN),
        crate::binding::BinaryOp::LessEqual => ("OP_LESS_EQUAL", FUNC_ID_OP_LESS_EQUAL),
        crate::binding::BinaryOp::GreaterThan => ("OP_GREATER_THAN", FUNC_ID_OP_GREATER_THAN),
        crate::binding::BinaryOp::GreaterEqual => ("OP_GREATER_EQUAL", FUNC_ID_OP_GREATER_EQUAL),
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
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
) -> Result<CallArgValue, EvaluationError> {
    if let Some(binding) = helper_bindings.get(&name.name) {
        return match binding {
            HelperBinding::Arg(CallArgValue::Reference(reference)) => {
                if preserve_reference {
                    Ok(CallArgValue::Reference(reference.clone()))
                } else {
                    resolver
                        .resolve_reference(reference)
                        .map(call_arg_from_resolved_reference_value)
                        .map_err(map_resolution_error)
                }
            }
            HelperBinding::Arg(other) => Ok(other.clone()),
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
        };
    }

    let Some(binding) = context.defined_names.get(&name.name) else {
        if callable_slot {
            return built_in_callable_arg_for_name(name);
        }
        return Err(EvaluationError {
            message: format!("no binding available for defined name {}", name.name),
        });
    };

    match binding {
        DefinedNameBinding::Value(value) => Ok(CallArgValue::Eval(value.clone())),
        DefinedNameBinding::Reference(reference) => {
            if preserve_reference {
                Ok(CallArgValue::Reference(reference.clone()))
            } else {
                resolver
                    .resolve_reference(reference)
                    .map(CallArgValue::Eval)
                    .map_err(map_resolution_error)
            }
        }
        DefinedNameBinding::Callable(binding) => Ok(CallArgValue::Eval(EvalValue::Lambda(
            callable_registry
                .borrow_mut()
                .register(lambda_binding_from_defined_name_binding(binding)),
        ))),
    }
}

fn built_in_callable_arg_for_name(name: &NameRef) -> Result<CallArgValue, EvaluationError> {
    built_in_callable_arg_for_function_name(&name.name)
}

fn built_in_callable_arg_for_function_name(
    function_name: &str,
) -> Result<CallArgValue, EvaluationError> {
    let meta = lookup_function_meta(function_name).ok_or_else(|| EvaluationError {
        message: format!("no registered built-in callable metadata for {function_name}"),
    })?;
    Ok(CallArgValue::Eval(EvalValue::Lambda(OxLambdaValue::new(
        meta.function_id,
        OxCallableOriginKind::BuiltInCallable,
        OxCallableArityShape::range(meta.arity.min, meta.arity.max),
        OxCallableCaptureMode::NoCapture,
        BUILTIN_CALLABLE_INVOCATION_CONTRACT_REF,
    ))))
}

fn is_builtin_callable_slot(function_name: &str, ordinal: usize) -> bool {
    matches!((function_name, ordinal), ("GROUPBY", 2) | ("PIVOTBY", 3))
}

fn evaluate_let_call(
    args: &[BoundExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    if args.len() < 2 {
        return Err(EvaluationError {
            message: "LET requires at least one binding pair and a final expression".to_string(),
        });
    }

    let mut local_bindings = helper_bindings.clone();
    let mut prepared_arguments = Vec::with_capacity(args.len());
    let last_index = args.len() - 1;
    let mut index = 0usize;
    while index < last_index {
        let BoundExpr::HelperParameterName(name) = &args[index] else {
            return Err(EvaluationError {
                message: "LET binding position did not contain a helper parameter".to_string(),
            });
        };
        prepared_arguments.push(PreparedArgument {
            ordinal: index,
            structure_class: PreparedStructureClass::DirectScalar,
            source_class: PreparedSourceClass::HelperParameter,
            evaluation_mode: PreparedEvaluationMode::EagerValue,
            blankness_class: PreparedBlanknessClass::NonBlank,
            caller_context_sensitive: false,
            reference_target: None,
            opaque_reason: None,
        });
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
        prepared_arguments.push(prepared_argument_for_call_arg(
            index + 1,
            &args[index + 1],
            &binding_arg,
            true,
        ));
        let helper_binding =
            helper_binding_from_expr(&args[index + 1], binding_arg, &local_bindings);
        local_bindings.insert(name.clone(), helper_binding);
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
    prepared_arguments.push(prepared_argument_for_call_arg(
        last_index,
        &args[last_index],
        &body_arg,
        false,
    ));
    push_special_prepared_call(
        trace,
        "LET",
        SPECIAL_LET_FUNCTION_ID,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        prepared_arguments,
        context,
    );

    materialize_call_arg(body_arg, resolver)
}

fn evaluate_lambda_call(
    args: &[BoundExpr],
    helper_bindings: &BTreeMap<String, HelperBinding>,
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
    let mut prepared_arguments = Vec::with_capacity(args.len());
    let params = args[..body_index]
        .iter()
        .enumerate()
        .map(|(ordinal, arg)| match arg {
            BoundExpr::HelperParameterName(name) => {
                prepared_arguments.push(PreparedArgument {
                    ordinal,
                    structure_class: PreparedStructureClass::DirectScalar,
                    source_class: PreparedSourceClass::HelperParameter,
                    evaluation_mode: PreparedEvaluationMode::EagerValue,
                    blankness_class: PreparedBlanknessClass::NonBlank,
                    caller_context_sensitive: false,
                    reference_target: None,
                    opaque_reason: None,
                });
                Ok(LambdaParam {
                    name: name.clone(),
                    optional: false,
                })
            }
            BoundExpr::HelperOptionalParameterName(name) => {
                prepared_arguments.push(PreparedArgument {
                    ordinal,
                    structure_class: PreparedStructureClass::DirectScalar,
                    source_class: PreparedSourceClass::HelperParameter,
                    evaluation_mode: PreparedEvaluationMode::EagerValue,
                    blankness_class: PreparedBlanknessClass::NonBlank,
                    caller_context_sensitive: false,
                    reference_target: None,
                    opaque_reason: None,
                });
                Ok(LambdaParam {
                    name: name.clone(),
                    optional: true,
                })
            }
            _ => Err(EvaluationError {
                message: "LAMBDA parameter did not bind as helper parameter".to_string(),
            }),
        })
        .collect::<Result<Vec<_>, _>>()?;
    prepared_arguments.push(PreparedArgument {
        ordinal: body_index,
        structure_class: PreparedStructureClass::DirectScalar,
        source_class: prepared_source_class(&args[body_index]),
        evaluation_mode: PreparedEvaluationMode::EagerValue,
        blankness_class: PreparedBlanknessClass::NonBlank,
        caller_context_sensitive: false,
        reference_target: None,
        opaque_reason: None,
    });
    push_special_prepared_call(
        trace,
        "LAMBDA",
        SPECIAL_LAMBDA_FUNCTION_ID,
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        prepared_arguments,
        context,
    );

    let parameter_names = lambda_param_names(&params);
    let capture_names = helper_capture_names(&args[body_index], &parameter_names, helper_bindings);
    Ok(EvalValue::Lambda(callable_registry.borrow_mut().register(
        LambdaBinding {
            origin_kind: CallableOriginKind::HelperLambda,
            params,
            body: args[body_index].clone(),
            closure: helper_closure_from_names(helper_bindings, &capture_names),
        },
    )))
}

fn evaluate_legacy_single_call(
    args: &[BoundExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
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
    push_special_prepared_call(
        trace,
        "_XLFN.SINGLE",
        SPECIAL_LEGACY_SINGLE_FUNCTION_ID,
        ArgPreparationProfile::RefsVisibleInAdapter,
        vec![prepared_argument_for_call_arg(0, arg, &prepared, true)],
        context,
    );
    Ok(eval_op_implicit_intersection_surface(&[prepared], resolver)
        .unwrap_or_else(|error| EvalValue::Error(map_op_implicit_intersection_error_to_ws(&error))))
}

fn evaluate_invocation(
    callee: &BoundExpr,
    args: &[BoundExpr],
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    trace: &mut EvaluationTrace,
) -> Result<EvalValue, EvaluationError> {
    let lambda = match lambda_binding_for_callee(callee, helper_bindings) {
        Some(binding) => binding,
        None => match lambda_binding_for_defined_name_callee(callee, &context.defined_names) {
            Some(binding) => binding,
            None => {
                return Err(EvaluationError {
                    message: "only immediate, helper-bound, or defined-name callable invocation is supported"
                        .to_string(),
                });
            }
        },
    };
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
    let mut prepared_arguments = Vec::with_capacity(args.len());
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
        prepared_arguments.push(prepared_argument_for_call_arg(
            ordinal, arg, &prepared, true,
        ));
        local_bindings.insert(param.name.clone(), HelperBinding::Arg(prepared));
    }
    for param in lambda.params.iter().skip(args.len()) {
        local_bindings.insert(
            param.name.clone(),
            HelperBinding::Arg(CallArgValue::MissingArg),
        );
    }
    push_special_prepared_call(
        trace,
        "LAMBDA.INVOKE",
        "SPECIAL.LAMBDA_INVOKE",
        ArgPreparationProfile::ValuesOnlyPreAdapter,
        prepared_arguments,
        context,
    );
    evaluate_expr_value(
        &lambda.body,
        context,
        resolver,
        &local_bindings,
        callable_registry,
        trace,
    )
}

fn helper_binding_from_expr(
    expr: &BoundExpr,
    fallback: CallArgValue,
    helper_bindings: &BTreeMap<String, HelperBinding>,
) -> HelperBinding {
    match expr {
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if function_name == "LAMBDA" && !args.is_empty() => {
            let body_index = args.len() - 1;
            let params = args[..body_index]
                .iter()
                .filter_map(|arg| match arg {
                    BoundExpr::HelperParameterName(name) => Some(LambdaParam {
                        name: name.clone(),
                        optional: false,
                    }),
                    BoundExpr::HelperOptionalParameterName(name) => Some(LambdaParam {
                        name: name.clone(),
                        optional: true,
                    }),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let capture_names = helper_capture_names(
                &args[body_index],
                &lambda_param_names(&params),
                helper_bindings,
            );
            HelperBinding::Lambda {
                params,
                body: args[body_index].clone(),
                closure: helper_closure_from_names(helper_bindings, &capture_names),
            }
        }
        _ => HelperBinding::Arg(fallback),
    }
}

fn lambda_binding_for_callee(
    callee: &BoundExpr,
    helper_bindings: &BTreeMap<String, HelperBinding>,
) -> Option<LambdaBinding> {
    match callee {
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if function_name == "LAMBDA" && !args.is_empty() => {
            let body_index = args.len() - 1;
            let params = args[..body_index]
                .iter()
                .map(|arg| match arg {
                    BoundExpr::HelperParameterName(name) => Some(LambdaParam {
                        name: name.clone(),
                        optional: false,
                    }),
                    BoundExpr::HelperOptionalParameterName(name) => Some(LambdaParam {
                        name: name.clone(),
                        optional: true,
                    }),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            let capture_names = helper_capture_names(
                &args[body_index],
                &lambda_param_names(&params),
                helper_bindings,
            );
            Some(LambdaBinding {
                origin_kind: CallableOriginKind::HelperLambda,
                params,
                body: args[body_index].clone(),
                closure: helper_closure_from_names(helper_bindings, &capture_names),
            })
        }
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(name)))
            if matches!(name.kind, crate::binding::NameKind::HelperLocal) =>
        {
            match helper_bindings.get(&name.name) {
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
                _ => None,
            }
        }
        _ => None,
    }
}

fn lambda_binding_for_defined_name_callee(
    callee: &BoundExpr,
    defined_names: &BTreeMap<String, DefinedNameBinding>,
) -> Option<LambdaBinding> {
    match callee {
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(name))) => {
            match defined_names.get(&name.name) {
                Some(DefinedNameBinding::Callable(binding)) => {
                    Some(lambda_binding_from_defined_name_binding(binding))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn lambda_binding_from_defined_name_binding(binding: &CallableDefinedNameBinding) -> LambdaBinding {
    LambdaBinding {
        origin_kind: CallableOriginKind::DefinedNameCallable,
        params: binding
            .params
            .iter()
            .map(|name| LambdaParam {
                name: name.clone(),
                optional: binding.optional_parameter_names.contains(name),
            })
            .collect(),
        body: binding.body.clone(),
        closure: binding
            .closure
            .iter()
            .filter_map(|(name, binding)| match binding {
                DefinedNameBinding::Value(value) => Some((
                    name.clone(),
                    HelperBinding::Arg(CallArgValue::Eval(value.clone())),
                )),
                DefinedNameBinding::Reference(reference) => Some((
                    name.clone(),
                    HelperBinding::Arg(CallArgValue::Reference(reference.clone())),
                )),
                DefinedNameBinding::Callable(_) => None,
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
    body: &BoundExpr,
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

fn lambda_body_kind(body: &BoundExpr) -> &'static str {
    match body {
        BoundExpr::NumberLiteral(_) => "NumberLiteral",
        BoundExpr::StringLiteral(_) => "StringLiteral",
        BoundExpr::LogicalLiteral(_) => "LogicalLiteral",
        BoundExpr::ArrayLiteral(_) => "ArrayLiteral",
        BoundExpr::OmittedArgument => "OmittedArgument",
        BoundExpr::HelperParameterName(_) | BoundExpr::HelperOptionalParameterName(_) => {
            "HelperParameter"
        }
        BoundExpr::Binary { .. } => "Binary",
        BoundExpr::Unary { .. } => "Unary",
        BoundExpr::FunctionCall { .. } => "FunctionCall",
        BoundExpr::Invocation { .. } => "Invocation",
        BoundExpr::Reference(_) => "Reference",
        BoundExpr::ImplicitIntersection(_) => "ImplicitIntersection",
    }
}

fn helper_capture_names(
    body: &BoundExpr,
    parameter_names: &[String],
    helper_bindings: &BTreeMap<String, HelperBinding>,
) -> BTreeSet<String> {
    let mut bound_names = parameter_names.iter().cloned().collect::<BTreeSet<_>>();
    helper_free_names_in_expr(body, &mut bound_names, helper_bindings)
}

fn lambda_param_names(params: &[LambdaParam]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}

fn lambda_required_arity(params: &[LambdaParam]) -> usize {
    params.iter().filter(|param| !param.optional).count()
}

fn helper_parameter_name(expr: &BoundExpr) -> Option<String> {
    match expr {
        BoundExpr::HelperParameterName(name) | BoundExpr::HelperOptionalParameterName(name) => {
            Some(name.clone())
        }
        _ => None,
    }
}

fn helper_free_names_in_expr(
    expr: &BoundExpr,
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
) -> BTreeSet<String> {
    match expr {
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::ArrayLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_) => BTreeSet::new(),
        BoundExpr::Unary { expr, .. } => helper_free_names_in_expr(expr, bound_names, helper_bindings),
        BoundExpr::Binary { left, right, .. } => {
            let mut names = helper_free_names_in_expr(left, bound_names, helper_bindings);
            names.extend(helper_free_names_in_expr(
                right,
                bound_names,
                helper_bindings,
            ));
            names
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if function_name == "LET" => helper_free_names_in_let(args, bound_names, helper_bindings),
        BoundExpr::FunctionCall {
            function_name,
            args,
        } if function_name == "LAMBDA" => {
            helper_free_names_in_lambda(args, bound_names, helper_bindings)
        }
        BoundExpr::FunctionCall { args, .. } => {
            let mut names = BTreeSet::new();
            for arg in args {
                names.extend(helper_free_names_in_expr(arg, bound_names, helper_bindings));
            }
            names
        }
        BoundExpr::Invocation { callee, args } => {
            let mut names = helper_free_names_in_expr(callee, bound_names, helper_bindings);
            for arg in args {
                names.extend(helper_free_names_in_expr(arg, bound_names, helper_bindings));
            }
            names
        }
        BoundExpr::Reference(reference) => {
            helper_free_names_in_reference(reference, bound_names, helper_bindings)
        }
        BoundExpr::ImplicitIntersection(inner) => {
            helper_free_names_in_expr(inner, bound_names, helper_bindings)
        }
    }
}

fn helper_free_names_in_let(
    args: &[BoundExpr],
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
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
            local_bound.insert(name);
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
    args: &[BoundExpr],
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
) -> BTreeSet<String> {
    if args.is_empty() {
        return BTreeSet::new();
    }

    let body_index = args.len() - 1;
    let mut nested_bound = bound_names.clone();
    for arg in &args[..body_index] {
        if let Some(name) = helper_parameter_name(arg) {
            nested_bound.insert(name);
        }
    }
    helper_free_names_in_expr(&args[body_index], &mut nested_bound, helper_bindings)
}

fn helper_free_names_in_reference(
    reference: &ReferenceExpr,
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
) -> BTreeSet<String> {
    match reference {
        ReferenceExpr::Atom(NormalizedReference::Name(name))
            if matches!(name.kind, crate::binding::NameKind::HelperLocal)
                && !bound_names.contains(&name.name)
                && helper_bindings.contains_key(&name.name) =>
        {
            BTreeSet::from([name.name.clone()])
        }
        ReferenceExpr::Atom(_) => BTreeSet::new(),
        ReferenceExpr::Spill { anchor } => {
            helper_free_names_in_reference(anchor, bound_names, helper_bindings)
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
    helper_bindings: &BTreeMap<String, HelperBinding>,
    capture_names: &BTreeSet<String>,
) -> BTreeMap<String, HelperBinding> {
    helper_bindings
        .iter()
        .filter_map(|(name, binding)| {
            if capture_names.contains(name) {
                Some((name.clone(), binding.clone()))
            } else {
                None
            }
        })
        .collect()
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
    expr: &BoundExpr,
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
            caller_context_sensitive: matches!(expr, BoundExpr::ImplicitIntersection(_)),
            reference_target: Some(reference.target.clone()),
            opaque_reason: prepared_argument_opaque_reason(expr),
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
        },
        CallArgValue::Eval(value) => PreparedArgument {
            ordinal,
            structure_class: match value {
                EvalValue::Array(_) => PreparedStructureClass::ArrayLike,
                _ => PreparedStructureClass::DirectScalar,
            },
            source_class,
            evaluation_mode: if matches!(expr, BoundExpr::ImplicitIntersection(_)) {
                PreparedEvaluationMode::CallerContextScalarized
            } else {
                PreparedEvaluationMode::EagerValue
            },
            blankness_class: blankness_class_for_eval_value(value),
            caller_context_sensitive: matches!(expr, BoundExpr::ImplicitIntersection(_)),
            reference_target: None,
            opaque_reason: prepared_argument_opaque_reason(expr),
        },
    }
}

fn prepared_source_class(expr: &BoundExpr) -> PreparedSourceClass {
    match expr {
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::ArrayLiteral(_)
        | BoundExpr::OmittedArgument => PreparedSourceClass::Literal,
        BoundExpr::HelperParameterName(_) | BoundExpr::HelperOptionalParameterName(_) => {
            PreparedSourceClass::HelperParameter
        }
        BoundExpr::FunctionCall { .. } | BoundExpr::Invocation { .. } => {
            PreparedSourceClass::FunctionCall
        }
        BoundExpr::Binary { .. } | BoundExpr::Unary { .. } => PreparedSourceClass::BinaryExpression,
        BoundExpr::ImplicitIntersection(_) => PreparedSourceClass::ImplicitIntersection,
        BoundExpr::Reference(reference) => match reference {
            ReferenceExpr::Atom(NormalizedReference::Cell(_)) => PreparedSourceClass::CellReference,
            ReferenceExpr::Atom(NormalizedReference::Area(_)) => PreparedSourceClass::AreaReference,
            ReferenceExpr::Atom(NormalizedReference::WholeRow(_)) => {
                PreparedSourceClass::WholeRowReference
            }
            ReferenceExpr::Atom(NormalizedReference::WholeColumn(_)) => {
                PreparedSourceClass::WholeColumnReference
            }
            ReferenceExpr::Atom(NormalizedReference::Name(_)) => PreparedSourceClass::NameReference,
            ReferenceExpr::Atom(NormalizedReference::Structured(structured)) => {
                match structured.resolved_reference {
                    StructuredResolvedRef::Cell(_) => PreparedSourceClass::CellReference,
                    StructuredResolvedRef::Area(_) => PreparedSourceClass::AreaReference,
                }
            }
            ReferenceExpr::Atom(NormalizedReference::External(_)) => {
                PreparedSourceClass::ExternalReference
            }
            ReferenceExpr::Spill { .. } => PreparedSourceClass::SpillReference,
            _ => PreparedSourceClass::FunctionCall,
        },
    }
}

fn prepared_argument_opaque_reason(expr: &BoundExpr) -> Option<String> {
    match expr {
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::External(_))) => {
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
) {
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
    });
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
    match error.error_class.as_str() {
        "#REF!" => WorksheetErrorCode::Ref,
        "#NULL!" => WorksheetErrorCode::Null,
        "#NAME?" => WorksheetErrorCode::Name,
        "#VALUE!" => WorksheetErrorCode::Value,
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
                DefinedNameBinding::Callable(binding) => Ok(EvalValue::Lambda(
                    self.callable_registry
                        .borrow_mut()
                        .register(lambda_binding_from_defined_name_binding(binding)),
                )),
            };
        }

        if is_absent_single_cell_reference(reference) {
            return Ok(blank_single_cell_eval_value());
        }

        Err(RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        })
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
            let call_args = args.iter().map(call_arg_from_prepared).collect::<Vec<_>>();
            let mut resolver = LocalReferenceResolver {
                cell_values: &self.context.cell_values,
                defined_names: &self.context.defined_names,
                caller_row: self.context.caller_row,
                caller_col: self.context.caller_col,
                callable_registry: self.callable_registry,
            };
            let value = eval_surface_value_call_with_callable(
                &callable.callable_token,
                &call_args,
                &mut resolver,
                self.context.now_serial,
                self.context.random_value,
                self.context.locale_ctx,
                self.context.host_info,
                Some(self),
                self.context.rtd_provider,
                self.context.registered_external_provider,
            )
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
        let mut local_bindings = binding.lambda.closure;
        for (param, arg) in binding.lambda.params.iter().zip(args.iter()) {
            local_bindings.insert(
                param.name.clone(),
                HelperBinding::Arg(call_arg_from_prepared(arg)),
            );
        }
        for param in binding.lambda.params.iter().skip(args.len()) {
            local_bindings.insert(
                param.name.clone(),
                HelperBinding::Arg(CallArgValue::MissingArg),
            );
        }

        let mut trace = EvaluationTrace {
            prepared_calls: Vec::new(),
        };
        let mut resolver = LocalReferenceResolver {
            cell_values: &self.context.cell_values,
            defined_names: &self.context.defined_names,
            caller_row: self.context.caller_row,
            caller_col: self.context.caller_col,
            callable_registry: self.callable_registry,
        };
        let value = evaluate_expr_value(
            &binding.lambda.body,
            self.context,
            &mut resolver,
            &local_bindings,
            self.callable_registry,
            &mut trace,
        )
        .map_err(|_| CallableInvocationError::Worksheet(WorksheetErrorCode::Value))?;
        Ok(prepared_arg_from_eval_value(value))
    }
}

fn call_arg_from_prepared(prepared: &PreparedArgValue) -> CallArgValue {
    match prepared {
        PreparedArgValue::Eval(value) => CallArgValue::Eval(value.clone()),
        PreparedArgValue::MissingArg => CallArgValue::MissingArg,
        PreparedArgValue::EmptyCell => CallArgValue::EmptyCell,
    }
}

fn prepared_arg_from_eval_value(value: EvalValue) -> PreparedArgValue {
    PreparedArgValue::Eval(value)
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
    let (end_sheet, end_row, end_col) = parse_a1_target_with_default_sheet(end, start_sheet.as_deref())?;
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
        Some(EvalValue::Array(_)) | Some(EvalValue::Reference(_)) | Some(EvalValue::Lambda(_)) => {
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

fn callable_token(id: usize, summary: &str) -> String {
    format!("oxfml.callable.{id}::{summary}")
}
