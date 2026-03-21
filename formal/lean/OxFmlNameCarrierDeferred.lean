namespace OxFml

inductive NameCarrierKind where
  | plain
  | mixedOrDeferred
deriving DecidableEq, Repr

def DeferredEvaluationRequirement : String := "NameCarrierDeferred"
def NameFormulaCarrierCapability : String := "name_formula_carrier"

structure NameCarrierPlan where
  nameKind : NameCarrierKind
  evaluationRequirements : List String
  capabilityRequirements : List String
deriving DecidableEq, Repr

def RequiresDeferredNameCarrier (plan : NameCarrierPlan) : Prop :=
  plan.nameKind = NameCarrierKind.mixedOrDeferred

def WellFormedDeferredNameCarrierPlan (plan : NameCarrierPlan) : Prop :=
  RequiresDeferredNameCarrier plan ->
    DeferredEvaluationRequirement ∈ plan.evaluationRequirements
      /\ NameFormulaCarrierCapability ∈ plan.capabilityRequirements

theorem mixed_or_deferred_requires_explicit_markers
    (plan : NameCarrierPlan)
    (hKind : RequiresDeferredNameCarrier plan)
    (hWellFormed : WellFormedDeferredNameCarrierPlan plan) :
    DeferredEvaluationRequirement ∈ plan.evaluationRequirements
      /\ NameFormulaCarrierCapability ∈ plan.capabilityRequirements := by
  exact hWellFormed hKind

theorem explicit_markers_make_mixed_or_deferred_plan_well_formed
    (evalReqs capabilityReqs : List String)
    (hEval : DeferredEvaluationRequirement ∈ evalReqs)
    (hCap : NameFormulaCarrierCapability ∈ capabilityReqs) :
    WellFormedDeferredNameCarrierPlan {
      nameKind := NameCarrierKind.mixedOrDeferred
      evaluationRequirements := evalReqs
      capabilityRequirements := capabilityReqs
    } := by
  intro _hKind
  exact ⟨hEval, hCap⟩

theorem plain_name_does_not_force_deferred_name_carrier
    (evalReqs capabilityReqs : List String) :
    ¬ RequiresDeferredNameCarrier {
      nameKind := NameCarrierKind.plain
      evaluationRequirements := evalReqs
      capabilityRequirements := capabilityReqs
    } := by
  simp [RequiresDeferredNameCarrier]

theorem mixed_or_deferred_missing_markers_is_not_well_formed :
    ¬ WellFormedDeferredNameCarrierPlan {
      nameKind := NameCarrierKind.mixedOrDeferred
      evaluationRequirements := []
      capabilityRequirements := []
    } := by
  intro hWellFormed
  have hMarkers := hWellFormed rfl
  simp [DeferredEvaluationRequirement, NameFormulaCarrierCapability] at hMarkers

end OxFml
