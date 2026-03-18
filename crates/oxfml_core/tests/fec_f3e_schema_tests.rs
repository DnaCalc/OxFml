use oxfml_core::ExecutionProfileSummary;
use oxfml_core::seam::{
    AcceptDecision, AcceptedCandidateResult, CapabilityEffectFact, CommitRequest,
    DynamicReferenceFact, Extent, FenceSnapshot, Locus, RejectCode, RejectContext, ShapeDelta,
    ShapeOutcomeClass, SpillEvent, SpillEventKind, SpillFact, TopologyDelta, ValueDelta,
    ValuePayload, WorksheetValueClass, commit_candidate,
};

#[test]
fn commit_candidate_accepts_matching_fence_and_preserves_candidate_payload() {
    let candidate = sample_candidate();
    let observed_fence = candidate.fence_snapshot.clone();

    let decision = commit_candidate(CommitRequest {
        candidate_result: candidate.clone(),
        commit_attempt_id: "commit-001".to_string(),
        observed_fence,
    });

    let AcceptDecision::Accepted(bundle) = decision else {
        panic!("expected accepted commit bundle");
    };

    assert_eq!(bundle.formula_stable_id, candidate.formula_stable_id);
    assert_eq!(bundle.candidate_result_id, candidate.candidate_result_id);
    assert_eq!(
        bundle.value_delta.published_payload,
        ValuePayload::Number("42".to_string())
    );
    assert_eq!(
        bundle.shape_delta.shape_outcome_class,
        ShapeOutcomeClass::Established
    );
    assert_eq!(bundle.spill_events.len(), 1);
    assert!(bundle.execution_profile.is_some());
}

#[test]
fn commit_candidate_rejects_formula_token_mismatch_without_publication() {
    let candidate = sample_candidate();
    let mut observed_fence = candidate.fence_snapshot.clone();
    observed_fence.formula_token = "token:v2".to_string();

    let decision = commit_candidate(CommitRequest {
        candidate_result: candidate,
        commit_attempt_id: "commit-002".to_string(),
        observed_fence,
    });

    let AcceptDecision::Rejected(reject) = decision else {
        panic!("expected fence mismatch reject");
    };

    assert_eq!(reject.reject_code, RejectCode::FenceMismatch);
    let RejectContext::FenceMismatch(context) = reject.context else {
        panic!("expected typed fence mismatch context");
    };
    assert_eq!(context.mismatch_member_kind, "formula_token");
    assert_eq!(context.mismatch_class, "stale");
}

#[test]
fn commit_candidate_rejects_capability_view_mismatch_as_typed_no_publish_result() {
    let candidate = sample_candidate();
    let mut observed_fence = candidate.fence_snapshot.clone();
    observed_fence.capability_view_key = Some("cap:view:v2".to_string());

    let decision = commit_candidate(CommitRequest {
        candidate_result: candidate,
        commit_attempt_id: "commit-003".to_string(),
        observed_fence,
    });

    let AcceptDecision::Rejected(reject) = decision else {
        panic!("expected capability-view fence reject");
    };

    assert_eq!(reject.reject_code, RejectCode::FenceMismatch);
    let RejectContext::FenceMismatch(context) = reject.context else {
        panic!("expected typed fence mismatch context");
    };
    assert_eq!(context.mismatch_member_kind, "capability_view_key");
    assert_eq!(context.mismatch_class, "incompatible");
}

fn sample_candidate() -> AcceptedCandidateResult {
    let primary_locus = Locus {
        sheet_id: "sheet:default".to_string(),
        row: 1,
        col: 1,
    };

    AcceptedCandidateResult {
        formula_stable_id: "formula:001".to_string(),
        session_id: Some("session:001".to_string()),
        candidate_result_id: "candidate:001".to_string(),
        fence_snapshot: FenceSnapshot {
            formula_token: "token:v1".to_string(),
            snapshot_epoch: "epoch:10".to_string(),
            bind_hash: "bind:abc".to_string(),
            profile_version: "profile:v1".to_string(),
            capability_view_key: Some("cap:view:v1".to_string()),
        },
        value_delta: ValueDelta {
            formula_stable_id: "formula:001".to_string(),
            primary_locus: primary_locus.clone(),
            affected_value_loci: vec![primary_locus.clone()],
            published_value_class: WorksheetValueClass::Scalar,
            published_payload: ValuePayload::Number("42".to_string()),
            result_extent: Some(Extent { rows: 1, cols: 1 }),
            candidate_result_id: Some("candidate:001".to_string()),
        },
        shape_delta: ShapeDelta {
            formula_stable_id: "formula:001".to_string(),
            anchor_locus: primary_locus.clone(),
            intended_extent: Extent { rows: 1, cols: 1 },
            published_extent: Some(Extent { rows: 1, cols: 1 }),
            blocked_loci: Vec::new(),
            shape_outcome_class: ShapeOutcomeClass::Established,
            candidate_result_id: Some("candidate:001".to_string()),
        },
        topology_delta: TopologyDelta {
            formula_stable_id: "formula:001".to_string(),
            dependency_additions: vec!["name:InputA".to_string()],
            dependency_removals: Vec::new(),
            dependency_reclassifications: Vec::new(),
            dependency_consequence_facts: Vec::new(),
            dynamic_reference_facts: vec![DynamicReferenceFact {
                formula_stable_id: "formula:001".to_string(),
                discovery_site: "OFFSET".to_string(),
                reference_identity: Some("ref:A1:B2".to_string()),
                target_extent: Some(Extent { rows: 2, cols: 2 }),
                resolution_failure_class: None,
            }],
            spill_facts: vec![SpillFact {
                formula_stable_id: "formula:001".to_string(),
                anchor_locus: primary_locus.clone(),
                intended_extent: Extent { rows: 1, cols: 1 },
                published_extent: Some(Extent { rows: 1, cols: 1 }),
                blocked_loci: Vec::new(),
                blocked_reason_class: None,
            }],
            format_dependency_facts: Vec::new(),
            capability_effect_facts: vec![CapabilityEffectFact {
                formula_stable_id: "formula:001".to_string(),
                capability_kind: "host_query".to_string(),
                phase_kind: "execute".to_string(),
                effect_class: "admitted".to_string(),
                fallback_class: None,
            }],
            candidate_result_id: Some("candidate:001".to_string()),
        },
        format_delta: None,
        display_delta: None,
        spill_events: vec![SpillEvent {
            spill_event_kind: SpillEventKind::SpillTakeover,
            formula_stable_id: "formula:001".to_string(),
            anchor_locus: primary_locus,
            intended_extent: Extent { rows: 1, cols: 1 },
            affected_extent: Some(Extent { rows: 1, cols: 1 }),
            blocking_loci: Vec::new(),
            blocking_reason_class: None,
            correlation_id: "candidate:001".to_string(),
        }],
        execution_profile: Some(ExecutionProfileSummary::default()),
        trace_correlation_id: "trace:001".to_string(),
    }
}
