use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalDescriptor, RegisteredExternalOriginKind,
    RegisteredExternalProvider, RegisteredExternalProviderError, RegisteredExternalTarget,
    RegisteredProcedureSpec,
};
use oxfunc_core::value::{CoreValue, ExcelText, ReferenceKind, ReferenceLike};
use std::cell::RefCell;

use oxfml_core::EvaluationTraceMode;
use oxfml_core::TypedContextQueryBundleSpec;
use oxfml_core::interface::{
    RegisteredExternalCatalogController, RegisteredExternalCatalogMutationRequest,
    RegisteredExternalCatalogMutationResult, RegisteredExternalHostRegistrationRequest,
    RegisteredExternalRegistrationChannel, TypedContextQueryBundle, TypedContextQueryFamily,
};
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfunc_core::value::CalcValue;

#[test]
fn typed_query_bundle_includes_registered_external_family_when_provider_present() {
    let spec = TypedContextQueryBundle::default()
        .with_registered_external_provider(Some(&RecordingRegisteredExternalProvider::default()))
        .freeze_candidate_spec();

    assert!(
        spec.families
            .contains(&TypedContextQueryFamily::RegisteredExternal)
    );
}

#[test]
fn host_executes_register_id_and_call_through_registered_external_provider() {
    let provider = RecordingRegisteredExternalProvider::default();
    let mut register_host = SingleFormulaHost::new(
        "formula:register-id",
        "=REGISTER.ID(\"Kernel32\",\"GetTickCount\",\"J!\")",
    );
    register_host.set_trace_mode(EvaluationTraceMode::PreparedCalls);

    let register_output = register_host
        .recalc_with_registered_external_provider(None, Some(&provider), None, None)
        .expect("register.id host recalc");

    assert_eq!(
        register_output.published_worksheet_value,
        CalcValue::number(4242.0)
    );
    assert_eq!(
        register_output.evaluation.trace.prepared_calls[0]
            .register_id_request
            .as_ref(),
        Some(&sample_register_id_request(
            "Kernel32",
            "GetTickCount",
            Some("J!")
        ))
    );
    assert_eq!(
        provider
            .last_resolve
            .borrow()
            .as_ref()
            .expect("register request")
            .library_name
            .to_string_lossy(),
        "Kernel32"
    );
    assert_eq!(
        register_output.typed_query_bundle_spec,
        TypedContextQueryBundleSpec {
            families: vec![
                TypedContextQueryFamily::RegisteredExternal,
                TypedContextQueryFamily::NowSerial,
            ],
        }
    );

    let mut call_host = SingleFormulaHost::new(
        "formula:call-direct",
        "=CALL(\"Kernel32\",\"MulDiv\",\"JJJJ\",6,7,3)",
    );
    call_host.set_trace_mode(EvaluationTraceMode::PreparedCalls);
    let call_output = call_host
        .recalc_with_registered_external_provider(None, Some(&provider), None, None)
        .expect("call host recalc");

    assert_eq!(
        call_output.published_worksheet_value,
        CalcValue::number(14.0)
    );
    let (descriptor, args) = provider.last_invoke.borrow().clone().expect("invoke");
    assert_eq!(descriptor.register_id, 4242.0);
    assert_eq!(
        args,
        vec![
            CalcValue::number(6.0),
            CalcValue::number(7.0),
            CalcValue::number(3.0),
        ]
    );
    match call_output.evaluation.trace.prepared_calls[0]
        .registered_external_call_request
        .as_ref()
        .expect("normalized call request")
    {
        oxfml_core::RegisteredExternalCallRequest {
            target:
                RegisteredExternalTarget::Direct(RegisterIdRequest {
                    library_name,
                    procedure: RegisteredProcedureSpec::Name(procedure),
                    declared_type_text,
                }),
            invocation_args,
        } => {
            assert_eq!(library_name.to_string_lossy(), "Kernel32");
            assert_eq!(procedure.to_string_lossy(), "MulDiv");
            assert_eq!(
                declared_type_text
                    .as_ref()
                    .map(|value| value.to_string_lossy()),
                Some("JJJJ".to_string())
            );
            assert_eq!(
                *invocation_args,
                vec![
                    CalcValue::number(6.0),
                    CalcValue::number(7.0),
                    CalcValue::number(3.0),
                ]
            );
        }
        other => panic!("unexpected normalized call request: {other:?}"),
    }
}

#[test]
fn host_executes_call_by_register_id_through_lookup_lane() {
    let provider = RecordingRegisteredExternalProvider::default();
    let mut host = SingleFormulaHost::new("formula:call-register-id", "=CALL(4242,6,7,3)");
    host.set_trace_mode(EvaluationTraceMode::PreparedCalls);

    let output = host
        .recalc_with_registered_external_provider(None, Some(&provider), None, None)
        .expect("call by register id host recalc");

    assert_eq!(output.published_worksheet_value, CalcValue::number(14.0));
    assert_eq!(provider.last_lookup.borrow().as_ref(), Some(&4242.0));
    match output.evaluation.trace.prepared_calls[0]
        .registered_external_call_request
        .as_ref()
        .expect("normalized by-id call request")
    {
        oxfml_core::RegisteredExternalCallRequest {
            target: RegisteredExternalTarget::RegisterId(register_id),
            invocation_args,
        } => {
            assert_eq!(*register_id, 4242.0);
            assert_eq!(
                *invocation_args,
                vec![
                    CalcValue::number(6.0),
                    CalcValue::number(7.0),
                    CalcValue::number(3.0),
                ]
            );
        }
        other => panic!("unexpected normalized by-id call request: {other:?}"),
    }
}

#[test]
fn host_preserves_reference_visible_call_argument_for_registered_external_invocation() {
    let provider = RecordingRegisteredExternalProvider::default();
    let mut host = SingleFormulaHost::new(
        "formula:call-ref",
        "=CALL(\"Kernel32\",\"ProbeRef\",\"J\",A1)",
    );
    host.set_cell_value("A1", CalcValue::number(7.0));

    let output = host
        .recalc_with_registered_external_provider(None, Some(&provider), None, None)
        .expect("call with reference host recalc");

    assert_eq!(output.published_worksheet_value, CalcValue::number(99.0));
    let (_, args) = provider.last_invoke.borrow().clone().expect("invoke");
    assert_eq!(
        args,
        vec![CalcValue::reference(ReferenceLike::new(
            ReferenceKind::A1,
            "A1"
        ))]
    );
}

#[test]
fn host_registration_mutation_packet_preserves_channel_specific_hints() {
    let controller = RecordingCatalogController::default();
    let host = SingleFormulaHost::new("formula:registered-external-catalog", "=1");
    let worksheet_request = RegisteredExternalCatalogMutationRequest::Register(
        RegisteredExternalHostRegistrationRequest {
            registration_channel: RegisteredExternalRegistrationChannel::WorksheetRegisterId,
            register_id_request: sample_register_id_request("Kernel32", "GetTickCount", Some("J!")),
            stable_registration_id_hint: None,
            display_name_hint: None,
            help_text_hint: None,
            source_project_ref: None,
            source_module_ref: None,
            source_procedure_ref: None,
            host_execution_profile: None,
        },
    );
    let host_api_request = RegisteredExternalCatalogMutationRequest::Register(
        RegisteredExternalHostRegistrationRequest {
            registration_channel: RegisteredExternalRegistrationChannel::HostApiRegistration,
            register_id_request: sample_register_id_request("User32", "MessageBoxW", Some("JJCCJ")),
            stable_registration_id_hint: Some("REG.messagebox".to_string()),
            display_name_hint: Some("MessageBoxW".to_string()),
            help_text_hint: Some("Displays a modal message box.".to_string()),
            source_project_ref: None,
            source_module_ref: None,
            source_procedure_ref: None,
            host_execution_profile: Some("desktop-trusted".to_string()),
        },
    );
    let vba_request = RegisteredExternalCatalogMutationRequest::Register(
        RegisteredExternalHostRegistrationRequest {
            registration_channel: RegisteredExternalRegistrationChannel::VbaProjectShimRegistration,
            register_id_request: sample_register_id_request("vba:Book1", "Module1.MyFunc", None),
            stable_registration_id_hint: Some("REG.vba.book1.module1.myfunc".to_string()),
            display_name_hint: Some("MyFunc".to_string()),
            help_text_hint: Some("Workbook VBA shim".to_string()),
            source_project_ref: Some("vba:Book1".to_string()),
            source_module_ref: Some("Module1".to_string()),
            source_procedure_ref: Some("MyFunc".to_string()),
            host_execution_profile: Some("vba-shim".to_string()),
        },
    );

    let worksheet_result = host
        .apply_registered_external_catalog_mutation(&controller, &worksheet_request)
        .expect("worksheet mutation");
    let host_result = host
        .apply_registered_external_catalog_mutation(&controller, &host_api_request)
        .expect("host mutation");
    let vba_result = host
        .apply_registered_external_catalog_mutation(&controller, &vba_request)
        .expect("vba mutation");

    let recorded = controller.recorded.borrow().clone();
    assert_eq!(recorded.len(), 3);
    assert!(matches!(
        recorded[0],
        RegisteredExternalCatalogMutationRequest::Register(
            RegisteredExternalHostRegistrationRequest {
                registration_channel: RegisteredExternalRegistrationChannel::WorksheetRegisterId,
                ..
            }
        )
    ));
    assert!(matches!(
        recorded[1],
        RegisteredExternalCatalogMutationRequest::Register(
            RegisteredExternalHostRegistrationRequest {
                registration_channel: RegisteredExternalRegistrationChannel::HostApiRegistration,
                ..
            }
        )
    ));
    assert!(matches!(
        recorded[2],
        RegisteredExternalCatalogMutationRequest::Register(
            RegisteredExternalHostRegistrationRequest {
                registration_channel:
                    RegisteredExternalRegistrationChannel::VbaProjectShimRegistration,
                ..
            }
        )
    ));
    match worksheet_result {
        RegisteredExternalCatalogMutationResult::RegisterApplied {
            descriptor,
            host_execution_profile,
        } => {
            assert_eq!(
                descriptor.origin_kind,
                RegisteredExternalOriginKind::WorksheetRegisterId
            );
            assert_eq!(host_execution_profile, None);
        }
        _ => panic!("worksheet registration should return register result"),
    }
    match host_result {
        RegisteredExternalCatalogMutationResult::RegisterApplied {
            descriptor,
            host_execution_profile,
        } => {
            assert_eq!(
                descriptor.origin_kind,
                RegisteredExternalOriginKind::HostRegisteredExternal
            );
            assert_eq!(host_execution_profile.as_deref(), Some("desktop-trusted"));
        }
        _ => panic!("host registration should return register result"),
    }
    match vba_result {
        RegisteredExternalCatalogMutationResult::RegisterApplied {
            descriptor,
            host_execution_profile,
        } => {
            assert_eq!(
                descriptor.origin_kind,
                RegisteredExternalOriginKind::HostRegisteredExternal
            );
            assert_eq!(host_execution_profile.as_deref(), Some("vba-shim"));
        }
        _ => panic!("vba registration should return register result"),
    }
}

#[test]
fn host_unregistration_mutation_packet_preserves_channel_and_identity() {
    let controller = RecordingCatalogController::default();
    let host = SingleFormulaHost::new("formula:registered-external-unregister", "=1");
    let unregister_request = RegisteredExternalCatalogMutationRequest::Unregister(
        oxfml_core::RegisteredExternalUnregisterRequest {
            registration_channel: RegisteredExternalRegistrationChannel::HostApiRegistration,
            stable_registration_id: "REG.messagebox".to_string(),
            host_execution_profile: Some("desktop-trusted".to_string()),
        },
    );

    let result = host
        .apply_registered_external_catalog_mutation(&controller, &unregister_request)
        .expect("unregister mutation");

    let recorded = controller.recorded.borrow().clone();
    assert_eq!(recorded.len(), 1);
    assert!(matches!(
        &recorded[0],
        RegisteredExternalCatalogMutationRequest::Unregister(
            oxfml_core::RegisteredExternalUnregisterRequest {
                registration_channel: RegisteredExternalRegistrationChannel::HostApiRegistration,
                stable_registration_id,
                ..
            }
        ) if stable_registration_id == "REG.messagebox"
    ));
    match result {
        RegisteredExternalCatalogMutationResult::UnregisterApplied {
            stable_registration_id,
            host_execution_profile,
        } => {
            assert_eq!(stable_registration_id, "REG.messagebox");
            assert_eq!(host_execution_profile.as_deref(), Some("desktop-trusted"));
        }
        _ => panic!("unregister should return unregister result"),
    }
}

#[derive(Default)]
struct RecordingRegisteredExternalProvider {
    last_resolve: RefCell<Option<RegisterIdRequest>>,
    last_lookup: RefCell<Option<f64>>,
    last_invoke: RefCell<Option<(RegisteredExternalDescriptor, Vec<CalcValue>)>>,
}

impl RecordingRegisteredExternalProvider {
    fn descriptor_from_request(
        &self,
        request: &RegisterIdRequest,
        origin_kind: RegisteredExternalOriginKind,
    ) -> RegisteredExternalDescriptor {
        RegisteredExternalDescriptor {
            stable_registration_id: format!(
                "REG.{}",
                request.library_name.to_string_lossy().to_ascii_lowercase()
            ),
            register_id: 4242.0,
            origin_kind,
            display_name: Some(ExcelText::from_interop_assignment(
                &request.procedure_display_name(),
            )),
            library_name: request.library_name.clone(),
            procedure: request.procedure.clone(),
            declared_type_text: request.declared_type_text.clone(),
        }
    }
}

impl RegisteredExternalProvider for RecordingRegisteredExternalProvider {
    fn resolve_register_id(
        &self,
        request: &RegisterIdRequest,
    ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        self.last_resolve.replace(Some(request.clone()));
        Ok(
            self.descriptor_from_request(
                request,
                RegisteredExternalOriginKind::WorksheetRegisterId,
            ),
        )
    }

    fn lookup_registered_external(
        &self,
        register_id: f64,
    ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        self.last_lookup.replace(Some(register_id));
        Ok(RegisteredExternalDescriptor {
            stable_registration_id: "REG.by-id".to_string(),
            register_id,
            origin_kind: RegisteredExternalOriginKind::WorksheetRegisterId,
            display_name: Some(ExcelText::from_interop_assignment("LookupById")),
            library_name: ExcelText::from_interop_assignment("Kernel32"),
            procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment("MulDiv")),
            declared_type_text: Some(ExcelText::from_interop_assignment("JJJJ")),
        })
    }

    fn invoke_registered_external(
        &self,
        descriptor: &RegisteredExternalDescriptor,
        args: &[CalcValue],
    ) -> Result<CalcValue, RegisteredExternalProviderError> {
        self.last_invoke
            .replace(Some((descriptor.clone(), args.to_vec())));
        match &descriptor.procedure {
            RegisteredProcedureSpec::Name(name) if name.to_string_lossy() == "MulDiv" => match args
            {
                [a, b, c] => match (a.core(), b.core(), c.core()) {
                    (CoreValue::Number(a), CoreValue::Number(b), CoreValue::Number(c)) => {
                        Ok(CalcValue::number((a * b) / c))
                    }
                    _ => Err(RegisteredExternalProviderError::WorksheetError(
                        oxfunc_core::value::WorksheetErrorCode::Value,
                    )),
                },
                _ => Err(RegisteredExternalProviderError::WorksheetError(
                    oxfunc_core::value::WorksheetErrorCode::Value,
                )),
            },
            RegisteredProcedureSpec::Name(name) if name.to_string_lossy() == "ProbeRef" => {
                Ok(CalcValue::number(99.0))
            }
            _ => Ok(CalcValue::number(descriptor.register_id)),
        }
    }
}

#[derive(Default)]
struct RecordingCatalogController {
    recorded: RefCell<Vec<RegisteredExternalCatalogMutationRequest>>,
}

impl RegisteredExternalCatalogController for RecordingCatalogController {
    fn apply_mutation(
        &self,
        request: &RegisteredExternalCatalogMutationRequest,
    ) -> Result<RegisteredExternalCatalogMutationResult, RegisteredExternalProviderError> {
        self.recorded.borrow_mut().push(request.clone());
        let (
            origin_kind,
            register_id_request,
            host_execution_profile,
            stable_registration_id_hint,
            display_name_hint,
        ) = match request {
            RegisteredExternalCatalogMutationRequest::Register(register) => (
                match register.registration_channel {
                    RegisteredExternalRegistrationChannel::WorksheetRegisterId => {
                        RegisteredExternalOriginKind::WorksheetRegisterId
                    }
                    RegisteredExternalRegistrationChannel::HostApiRegistration
                    | RegisteredExternalRegistrationChannel::VbaProjectShimRegistration => {
                        RegisteredExternalOriginKind::HostRegisteredExternal
                    }
                },
                register.register_id_request.clone(),
                register.host_execution_profile.clone(),
                register.stable_registration_id_hint.clone(),
                register.display_name_hint.clone(),
            ),
            RegisteredExternalCatalogMutationRequest::Unregister(unregister) => {
                return Ok(RegisteredExternalCatalogMutationResult::UnregisterApplied {
                    stable_registration_id: unregister.stable_registration_id.clone(),
                    host_execution_profile: unregister.host_execution_profile.clone(),
                });
            }
        };
        Ok(RegisteredExternalCatalogMutationResult::RegisterApplied {
            descriptor: RegisteredExternalDescriptor {
                stable_registration_id: stable_registration_id_hint
                    .unwrap_or_else(|| "REG.synthetic".to_string()),
                register_id: 5000.0,
                origin_kind,
                display_name: display_name_hint
                    .map(|text| ExcelText::from_interop_assignment(&text)),
                library_name: register_id_request.library_name,
                procedure: register_id_request.procedure,
                declared_type_text: register_id_request.declared_type_text,
            },
            host_execution_profile,
        })
    }
}

trait RegisterIdRequestExt {
    fn procedure_display_name(&self) -> String;
}

impl RegisterIdRequestExt for RegisterIdRequest {
    fn procedure_display_name(&self) -> String {
        match &self.procedure {
            RegisteredProcedureSpec::Name(name) => name.to_string_lossy(),
            RegisteredProcedureSpec::Ordinal(ordinal) => ordinal.to_string(),
        }
    }
}

fn sample_register_id_request(
    library_name: &str,
    procedure_name: &str,
    type_text: Option<&str>,
) -> RegisterIdRequest {
    RegisterIdRequest {
        library_name: ExcelText::from_interop_assignment(library_name),
        procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment(
            procedure_name,
        )),
        declared_type_text: type_text.map(ExcelText::from_interop_assignment),
    }
}
