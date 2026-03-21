namespace OxFml

inductive FailureStage where
  | editRejected
  | acceptedUnresolvedName
  | semanticPlanGated
  | runtimeCapabilityDenied
  | postDispatchProviderUnavailable
deriving DecidableEq, Repr

structure FormulaEditDisposition where
  stage : FailureStage
  formulaAdopted : Bool
  oxfuncErrorValueRequired : Bool
deriving DecidableEq, Repr

def WellFormedFailureDisposition (disposition : FormulaEditDisposition) : Prop :=
  match disposition.stage with
  | FailureStage.editRejected =>
      disposition.formulaAdopted = false /\ disposition.oxfuncErrorValueRequired = false
  | FailureStage.acceptedUnresolvedName =>
      disposition.formulaAdopted = true /\ disposition.oxfuncErrorValueRequired = true
  | FailureStage.semanticPlanGated =>
      disposition.formulaAdopted = true /\ disposition.oxfuncErrorValueRequired = false
  | FailureStage.runtimeCapabilityDenied =>
      disposition.formulaAdopted = true /\ disposition.oxfuncErrorValueRequired = false
  | FailureStage.postDispatchProviderUnavailable =>
      disposition.formulaAdopted = true /\ disposition.oxfuncErrorValueRequired = false

theorem edit_rejection_does_not_adopt_formula :
    WellFormedFailureDisposition {
      stage := FailureStage.editRejected
      formulaAdopted := false
      oxfuncErrorValueRequired := false
    } := by
  simp [WellFormedFailureDisposition]

theorem accepted_unresolved_name_requires_adoption_and_oxfunc_value :
    WellFormedFailureDisposition {
      stage := FailureStage.acceptedUnresolvedName
      formulaAdopted := true
      oxfuncErrorValueRequired := true
    } := by
  simp [WellFormedFailureDisposition]

theorem accepted_unresolved_name_without_oxfunc_value_is_not_well_formed :
    ¬ WellFormedFailureDisposition {
      stage := FailureStage.acceptedUnresolvedName
      formulaAdopted := true
      oxfuncErrorValueRequired := false
    } := by
  simp [WellFormedFailureDisposition]

theorem runtime_capability_denial_preserves_formula_adoption :
    WellFormedFailureDisposition {
      stage := FailureStage.runtimeCapabilityDenied
      formulaAdopted := true
      oxfuncErrorValueRequired := false
    } := by
  simp [WellFormedFailureDisposition]

theorem provider_failure_preserves_formula_adoption
    (formulaAdopted oxfuncErrorValueRequired : Bool)
    (hWellFormed : WellFormedFailureDisposition {
      stage := FailureStage.postDispatchProviderUnavailable
      formulaAdopted := formulaAdopted
      oxfuncErrorValueRequired := oxfuncErrorValueRequired
    }) :
    formulaAdopted = true := by
  have h := hWellFormed
  simp [WellFormedFailureDisposition] at h
  exact h.left

theorem semantic_plan_gate_is_distinct_from_accepted_unresolved_name :
    FailureStage.semanticPlanGated ≠ FailureStage.acceptedUnresolvedName := by
  decide

end OxFml
