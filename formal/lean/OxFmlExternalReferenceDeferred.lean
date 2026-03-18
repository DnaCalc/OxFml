namespace OxFml

structure CapabilityView where
  externalProviderEnabled : Bool
deriving DecidableEq, Repr

structure ExternalReferencePlan where
  requiresExternalProvider : Bool
  deferredReason : String
deriving DecidableEq, Repr

inductive ExternalLaneOutcome where
  | deferred
  | admitted
deriving DecidableEq, Repr

def ExternalLaneAdmissible (plan : ExternalReferencePlan) (view : CapabilityView) : Bool :=
  (!plan.requiresExternalProvider) || view.externalProviderEnabled

def ExternalLaneOutcomeFor
    (plan : ExternalReferencePlan)
    (view : CapabilityView) : ExternalLaneOutcome :=
  if ExternalLaneAdmissible plan view then
    ExternalLaneOutcome.admitted
  else
    ExternalLaneOutcome.deferred

def AsyncCapabilityEffectActive
    (plan : ExternalReferencePlan)
    (view : CapabilityView) : Bool :=
  plan.requiresExternalProvider && view.externalProviderEnabled

theorem missing_external_provider_forces_deferred
    (plan : ExternalReferencePlan)
    (view : CapabilityView)
    (hRequires : plan.requiresExternalProvider = true)
    (hMissing : view.externalProviderEnabled = false) :
    ExternalLaneOutcomeFor plan view = ExternalLaneOutcome.deferred := by
  simp [ExternalLaneOutcomeFor, ExternalLaneAdmissible, hRequires, hMissing]

theorem admitted_external_lane_implies_provider_or_no_requirement
    (plan : ExternalReferencePlan)
    (view : CapabilityView)
    (hAdmitted : ExternalLaneOutcomeFor plan view = ExternalLaneOutcome.admitted) :
    plan.requiresExternalProvider = false ∨ view.externalProviderEnabled = true := by
  by_cases hRequires : plan.requiresExternalProvider
  · right
    have hEnabled : view.externalProviderEnabled = true := by
      simp [ExternalLaneOutcomeFor, ExternalLaneAdmissible, hRequires] at hAdmitted
      exact hAdmitted
    exact hEnabled
  · left
    exact Bool.eq_false_iff.mpr hRequires

theorem async_capability_effect_requires_provider_enabled
    (plan : ExternalReferencePlan)
    (view : CapabilityView)
    (hAsync : AsyncCapabilityEffectActive plan view = true) :
    view.externalProviderEnabled = true := by
  simp [AsyncCapabilityEffectActive] at hAsync
  exact hAsync.right

theorem missing_provider_disables_async_capability_effect
    (plan : ExternalReferencePlan)
    (view : CapabilityView)
    (hMissing : view.externalProviderEnabled = false) :
    AsyncCapabilityEffectActive plan view = false := by
  simp [AsyncCapabilityEffectActive, hMissing]

end OxFml
