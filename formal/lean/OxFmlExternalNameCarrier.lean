namespace OxFml

inductive ExternalNameOutcome where
  | admitted
  | sameExternalBookRejected
  | runtimeCapabilityDenied
  | postDispatchProviderUnavailable
deriving DecidableEq, Repr

structure ExternalNameCarrierPlan where
  currentExternalBook : Option String
  referencedExternalBook : Option String
  sameExternalBookRequired : Bool
  providerCapabilityEnabled : Bool
  providerAvailableAfterDispatch : Bool
deriving DecidableEq, Repr

def ExplicitExternalBookIdentityPresent (plan : ExternalNameCarrierPlan) : Bool :=
  plan.referencedExternalBook.isSome

def SameExternalBookSatisfied (plan : ExternalNameCarrierPlan) : Bool :=
  if plan.sameExternalBookRequired then
    plan.currentExternalBook == plan.referencedExternalBook
  else
    true

def ExternalNameOutcomeFor (plan : ExternalNameCarrierPlan) : ExternalNameOutcome :=
  if !ExplicitExternalBookIdentityPresent plan then
    ExternalNameOutcome.sameExternalBookRejected
  else if !SameExternalBookSatisfied plan then
    ExternalNameOutcome.sameExternalBookRejected
  else if !plan.providerCapabilityEnabled then
    ExternalNameOutcome.runtimeCapabilityDenied
  else if !plan.providerAvailableAfterDispatch then
    ExternalNameOutcome.postDispatchProviderUnavailable
  else
    ExternalNameOutcome.admitted

theorem admitted_external_name_requires_explicit_book_identity
    (plan : ExternalNameCarrierPlan)
    (hAdmitted : ExternalNameOutcomeFor plan = ExternalNameOutcome.admitted) :
    ExplicitExternalBookIdentityPresent plan = true := by
  by_cases hIdentity : ExplicitExternalBookIdentityPresent plan = true
  · exact hIdentity
  · simp [ExternalNameOutcomeFor, hIdentity] at hAdmitted

theorem same_external_book_restriction_blocks_mismatch
    (currentBook referencedBook : String)
    (hMismatch : currentBook ≠ referencedBook) :
    ExternalNameOutcomeFor {
      currentExternalBook := some currentBook
      referencedExternalBook := some referencedBook
      sameExternalBookRequired := true
      providerCapabilityEnabled := true
      providerAvailableAfterDispatch := true
    } = ExternalNameOutcome.sameExternalBookRejected := by
  simp [ExternalNameOutcomeFor, ExplicitExternalBookIdentityPresent, SameExternalBookSatisfied, hMismatch]

theorem missing_provider_capability_is_distinct_from_provider_unavailable :
    ExternalNameOutcomeFor {
      currentExternalBook := some "Book.xlsx"
      referencedExternalBook := some "Book.xlsx"
      sameExternalBookRequired := true
      providerCapabilityEnabled := false
      providerAvailableAfterDispatch := false
    } = ExternalNameOutcome.runtimeCapabilityDenied := by
  simp [ExternalNameOutcomeFor, ExplicitExternalBookIdentityPresent, SameExternalBookSatisfied]

theorem provider_unavailable_requires_capability_but_blocks_admission :
    ExternalNameOutcomeFor {
      currentExternalBook := some "Book.xlsx"
      referencedExternalBook := some "Book.xlsx"
      sameExternalBookRequired := true
      providerCapabilityEnabled := true
      providerAvailableAfterDispatch := false
    } = ExternalNameOutcome.postDispatchProviderUnavailable := by
  simp [ExternalNameOutcomeFor, ExplicitExternalBookIdentityPresent, SameExternalBookSatisfied]

end OxFml
