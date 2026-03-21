use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use oxfunc_core::function::ArgPreparationProfile;
use oxfunc_core::functions::adapters::PreparedArgValue;
use oxfunc_core::functions::callable_helpers::{CallableInvocationError, CallableInvoker};
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call_with_callable;
use oxfunc_core::host_info::HostInfoProvider;
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::resolver::{
    CallerContext as OxFuncCallerContext, RefResolutionError, ReferenceResolver,
    ResolverCapabilities,
};
use oxfunc_core::value::{
    CallArgValue, CallableArityShape as OxCallableArityShape,
    CallableCaptureMode as OxCallableCaptureMode, CallableOriginKind as OxCallableOriginKind,
    EvalValue, ExcelText, LambdaValue as OxLambdaValue, ReferenceKind, ReferenceLike,
    WorksheetErrorCode,
};

use crate::binding::{
    AreaRef, BoundExpr, BoundFormula, CellRef, ErrorRef, NameRef, NormalizedReference,
    ReferenceExpr,
};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedCall {
    pub function_name: String,
    pub function_id: &'static str,
    pub arg_preparation_profile: ArgPreparationProfile,
    pub prepared_arguments: Vec<PreparedArgument>,
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
    pub parameter_names: Vec<String>,
    pub capture_names: Vec<String>,
    pub body_kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallableDefinedNameBinding {
    pub summary: String,
    pub carrier: CallableValueCarrier,
    pub profile: CallableValueProfile,
    pub params: Vec<String>,
    pub body: BoundExpr,
    pub closure: BTreeMap<String, DefinedNameBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationTrace {
    pub prepared_calls: Vec<PreparedCall>,
}

const SPECIAL_LET_FUNCTION_ID: &str = "SPECIAL.LET";
const SPECIAL_LAMBDA_FUNCTION_ID: &str = "SPECIAL.LAMBDA";
const SPECIAL_LEGACY_SINGLE_FUNCTION_ID: &str = "SPECIAL.LEGACY_SINGLE";
const SPECIAL_EXTERNAL_REFERENCE_DEFERRED_FUNCTION_ID: &str = "SPECIAL.EXTERNAL_REFERENCE_DEFERRED";
const HELPER_LAMBDA_INVOCATION_CONTRACT_REF: &str = "oxfml.helper_lambda.invoke.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationBackend {
    LocalBootstrap,
    OxFuncBacked,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationOutput {
    pub result: PreparedResult,
    pub oxfunc_value: EvalValue,
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
        params: Vec<String>,
        body: BoundExpr,
        closure: BTreeMap<String, HelperBinding>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct LambdaBinding {
    origin_kind: CallableOriginKind,
    params: Vec<String>,
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
            OxCallableArityShape::exact(lambda.params.len()),
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
            now_serial: None,
            random_value: None,
        }
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
        oxfunc_value: value,
        trace,
    })
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
        BoundExpr::StringLiteral(text) => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
            decode_string_literal(text).encode_utf16().collect(),
        ))),
        BoundExpr::HelperParameterName(name) => Err(EvaluationError {
            message: format!(
                "helper parameter {name} cannot be evaluated without helper-form environment support"
            ),
        }),
        BoundExpr::Binary { op, left, right } => {
            let lhs = coerce_to_number(evaluate_expr_value(
                left,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            )?)?;
            let rhs = coerce_to_number(evaluate_expr_value(
                right,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            )?)?;
            Ok(EvalValue::Number(match op {
                crate::binding::BinaryOp::Add => lhs + rhs,
                crate::binding::BinaryOp::Subtract => lhs - rhs,
            }))
        }
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
                trace,
            )?;
            match arg {
                CallArgValue::Reference(reference) => {
                    let resolved = resolver
                        .resolve_reference(&reference)
                        .map_err(map_resolution_error)?;
                    scalarize_eval_value(resolved)
                }
                other => scalarize_eval_value(materialize_call_arg(other, resolver)?),
            }
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
        let call_arg = evaluate_expr_as_call_arg(
            arg,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
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

    trace.prepared_calls.push(PreparedCall {
        function_name: function_name.to_string(),
        function_id: meta.function_id,
        arg_preparation_profile: meta.arg_preparation_profile,
        prepared_arguments,
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

    eval_surface_value_call_with_callable(
        meta.function_id,
        &call_args,
        resolver,
        context.now_serial,
        context.random_value,
        context.locale_ctx,
        context.host_info,
        Some(&callable_invoker),
        None,
    )
    .map_err(|error| EvaluationError {
        message: format!("OxFunc surface evaluation failed for {function_name}: {error:?}"),
    })
}

fn evaluate_expr_as_call_arg(
    expr: &BoundExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
    trace: &mut EvaluationTrace,
) -> Result<CallArgValue, EvaluationError> {
    match expr {
        BoundExpr::Reference(reference) => evaluate_reference_as_call_arg(
            reference,
            context,
            resolver,
            helper_bindings,
            callable_registry,
            preserve_reference,
            trace,
        ),
        BoundExpr::ImplicitIntersection(inner) => {
            let value = evaluate_expr_value(
                inner,
                context,
                resolver,
                helper_bindings,
                callable_registry,
                trace,
            )?;
            Ok(CallArgValue::Eval(scalarize_eval_value(value)?))
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

fn evaluate_reference_as_call_arg(
    reference: &ReferenceExpr,
    context: &EvaluationContext<'_>,
    resolver: &mut LocalReferenceResolver<'_>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
    callable_registry: &RefCell<CallableRegistry>,
    preserve_reference: bool,
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
            context,
            resolver,
            helper_bindings,
            callable_registry,
        ),
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
            let anchor_target = reference_target_string(anchor)?;
            let reference = ReferenceLike {
                kind: ReferenceKind::SpillAnchor,
                target: format!("{anchor_target}#"),
            };
            call_arg_for_reference_like(reference, preserve_reference, resolver)
        }
        ReferenceExpr::Range { start, end } => {
            let start_target = reference_target_string(start)?;
            let end_target = reference_target_string(end)?;
            call_arg_for_reference_like(
                ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: format!("{start_target}:{end_target}"),
                },
                preserve_reference,
                resolver,
            )
        }
        ReferenceExpr::Union { .. } => Err(EvaluationError {
            message: "union references are not yet evaluatable in the OxFunc bridge".to_string(),
        }),
        ReferenceExpr::Intersection { .. } => Err(EvaluationError {
            message: "intersection references are not yet evaluatable in the OxFunc bridge"
                .to_string(),
        }),
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
        resolver
            .resolve_reference(&reference)
            .map(CallArgValue::Eval)
            .map_err(map_resolution_error)
    }
}

fn call_arg_for_name(
    name: &NameRef,
    preserve_reference: bool,
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
                        .map(CallArgValue::Eval)
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

    let binding = context
        .defined_names
        .get(&name.name)
        .ok_or_else(|| EvaluationError {
            message: format!("no binding available for defined name {}", name.name),
        })?;

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
    let parameter_names = args[..body_index]
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
                Ok(name.clone())
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

    let capture_names = helper_capture_names(&args[body_index], &parameter_names, helper_bindings);
    Ok(EvalValue::Lambda(callable_registry.borrow_mut().register(
        LambdaBinding {
            origin_kind: CallableOriginKind::HelperLambda,
            params: parameter_names,
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
    match prepared {
        CallArgValue::Reference(reference) => {
            let resolved = resolver
                .resolve_reference(&reference)
                .map_err(map_resolution_error)?;
            scalarize_eval_value(resolved)
        }
        other => scalarize_eval_value(materialize_call_arg(other, resolver)?),
    }
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
    if lambda.params.len() != args.len() {
        return Err(EvaluationError {
            message: format!(
                "lambda invocation arity mismatch: expected {}, got {}",
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
            &local_bindings,
            callable_registry,
            true,
            trace,
        )?;
        prepared_arguments.push(prepared_argument_for_call_arg(
            ordinal, arg, &prepared, true,
        ));
        local_bindings.insert(param.clone(), HelperBinding::Arg(prepared));
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
                    BoundExpr::HelperParameterName(name) => Some(name.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let capture_names = helper_capture_names(&args[body_index], &params, helper_bindings);
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
                    BoundExpr::HelperParameterName(name) => Some(name.clone()),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?;
            let capture_names = helper_capture_names(&args[body_index], &params, helper_bindings);
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
        params: binding.params.clone(),
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
    parameter_names: &[String],
    mut captures: Vec<String>,
    body: &BoundExpr,
) -> String {
    captures.sort();
    let captures = if captures.is_empty() {
        "-".to_string()
    } else {
        captures.join("|")
    };
    format!(
        "arity={};params={};captures={};body={}",
        parameter_names.len(),
        parameter_names.join(","),
        captures,
        lambda_body_kind(body)
    )
}

fn lambda_body_kind(body: &BoundExpr) -> &'static str {
    match body {
        BoundExpr::NumberLiteral(_) => "NumberLiteral",
        BoundExpr::StringLiteral(_) => "StringLiteral",
        BoundExpr::HelperParameterName(_) => "HelperParameter",
        BoundExpr::Binary { .. } => "Binary",
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

fn helper_free_names_in_expr(
    expr: &BoundExpr,
    bound_names: &mut BTreeSet<String>,
    helper_bindings: &BTreeMap<String, HelperBinding>,
) -> BTreeSet<String> {
    match expr {
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::HelperParameterName(_) => BTreeSet::new(),
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
        if let BoundExpr::HelperParameterName(name) = &args[index] {
            local_bound.insert(name.clone());
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
        if let BoundExpr::HelperParameterName(name) = arg {
            nested_bound.insert(name.clone());
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
        CallArgValue::Reference(reference) => resolver
            .resolve_reference(&reference)
            .map_err(map_resolution_error),
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
        BoundExpr::NumberLiteral(_) | BoundExpr::StringLiteral(_) => PreparedSourceClass::Literal,
        BoundExpr::HelperParameterName(_) => PreparedSourceClass::HelperParameter,
        BoundExpr::FunctionCall { .. } | BoundExpr::Invocation { .. } => {
            PreparedSourceClass::FunctionCall
        }
        BoundExpr::Binary { .. } => PreparedSourceClass::BinaryExpression,
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
    let mut parameter_names = None;
    let mut capture_names = None;
    let mut body_kind = None;

    for part in summary.split(';') {
        let (key, value) = part.split_once('=')?;
        match key {
            "arity" => {
                arity = value.parse::<usize>().ok();
            }
            "params" => {
                parameter_names = Some(split_profile_list(value));
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

    Some(CallableValueProfile {
        arity: arity?,
        parameter_names: parameter_names.unwrap_or_default(),
        capture_names: capture_names.unwrap_or_default(),
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
        arity: lambda.arity_shape.min,
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

fn coerce_to_number(value: EvalValue) -> Result<f64, EvaluationError> {
    match value {
        EvalValue::Number(number) => Ok(number),
        EvalValue::Logical(value) => Ok(if value { 1.0 } else { 0.0 }),
        EvalValue::Text(text) => {
            text.to_string_lossy()
                .trim()
                .parse::<f64>()
                .map_err(|_| EvaluationError {
                    message: "text could not be coerced to number".to_string(),
                })
        }
        EvalValue::Error(code) => Err(EvaluationError {
            message: format!("encountered worksheet error {code:?}"),
        }),
        _ => Err(EvaluationError {
            message: "value could not be coerced to number".to_string(),
        }),
    }
}

fn scalarize_eval_value(value: EvalValue) -> Result<EvalValue, EvaluationError> {
    match value {
        EvalValue::Array(array) => array
            .get(0, 0)
            .and_then(|cell| cell.to_eval_value())
            .ok_or_else(|| EvaluationError {
                message: "array could not be scalarized".to_string(),
            }),
        other => Ok(other),
    }
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

fn reference_target_string(reference: &ReferenceExpr) -> Result<String, EvaluationError> {
    match reference {
        ReferenceExpr::Atom(NormalizedReference::Cell(cell)) => Ok(a1_for_cell(cell)),
        ReferenceExpr::Atom(NormalizedReference::Area(area)) => Ok(a1_for_area(area)),
        ReferenceExpr::Atom(NormalizedReference::WholeRow(rows)) => Ok(whole_row_target(rows)),
        ReferenceExpr::Atom(NormalizedReference::WholeColumn(columns)) => {
            Ok(whole_column_target(columns))
        }
        ReferenceExpr::Atom(NormalizedReference::Name(name)) => Ok(name.name.clone()),
        ReferenceExpr::Atom(NormalizedReference::External(external)) => Err(EvaluationError {
            message: format!(
                "cannot create executable reference target for {}",
                external.target_summary
            ),
        }),
        ReferenceExpr::Atom(NormalizedReference::Error(error)) => Err(EvaluationError {
            message: format!("cannot create reference target for {}", error.error_class),
        }),
        ReferenceExpr::Spill { anchor } => Ok(format!("{}#", reference_target_string(anchor)?)),
        ReferenceExpr::Range { start, end } => Ok(format!(
            "{}:{}",
            reference_target_string(start)?,
            reference_target_string(end)?
        )),
        ReferenceExpr::Union { .. } | ReferenceExpr::Intersection { .. } => Err(EvaluationError {
            message: "union and intersection reference targets are not supported".to_string(),
        }),
    }
}

fn a1_for_cell(cell: &CellRef) -> String {
    format!("{}{}", column_letters(cell.coord.col), cell.coord.row)
}

fn a1_for_area(area: &AreaRef) -> String {
    let start = format!("{}{}", column_letters(area.top_left.col), area.top_left.row);
    let end_col = area.top_left.col + area.width - 1;
    let end_row = area.top_left.row + area.height - 1;
    let end = format!("{}{}", column_letters(end_col), end_row);
    format!("{start}:{end}")
}

fn whole_row_target(rows: &crate::binding::WholeRowRef) -> String {
    let row_end = rows.row_start + rows.row_count - 1;
    format!("{}:{}", rows.row_start, row_end)
}

fn whole_column_target(columns: &crate::binding::WholeColumnRef) -> String {
    let end_col = columns.col_start + columns.col_count - 1;
    format!(
        "{}:{}",
        column_letters(columns.col_start),
        column_letters(end_col)
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

fn column_letters(mut col: u32) -> String {
    let mut letters = String::new();
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        letters.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    letters
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
                param.clone(),
                HelperBinding::Arg(call_arg_from_prepared(arg)),
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

fn callable_token(id: usize, summary: &str) -> String {
    format!("oxfml.callable.{id}::{summary}")
}
