namespace OxFml

inductive SessionPhase where
  | open
  | capabilityViewEstablished
  | executed
  | committed
  | rejected
  | aborted
  | expired
deriving DecidableEq, Repr

inductive RejectCode where
  | fenceMismatch
  | capabilityDenied
  | sessionTerminated
  | bindMismatch
  | structuralConflict
  | dynamicReferenceFailure
  | resourceInvariantFailure
deriving DecidableEq, Repr

structure FenceSnapshot where
  formulaToken : String
  snapshotEpoch : String
  bindHash : String
  profileVersion : String
  capabilityViewKey : Option String
deriving DecidableEq, Repr

structure CandidateResult where
  formulaStableId : String
  sessionId : Option String
  candidateResultId : String
  fenceSnapshot : FenceSnapshot
deriving DecidableEq, Repr

structure CommitBundle where
  formulaStableId : String
  commitAttemptId : String
  candidateResultId : String
  fenceSnapshot : FenceSnapshot
deriving DecidableEq, Repr

structure RejectRecord where
  formulaStableId : String
  sessionId : Option String
  commitAttemptId : Option String
  rejectCode : RejectCode
deriving DecidableEq, Repr

def FenceCompatible (expected observed : FenceSnapshot) : Prop :=
  expected = observed

def CandidatePublishesAs
    (candidate : CandidateResult)
    (commitAttemptId : String)
    (bundle : CommitBundle) : Prop :=
  bundle.formulaStableId = candidate.formulaStableId
    /\ bundle.commitAttemptId = commitAttemptId
    /\ bundle.candidateResultId = candidate.candidateResultId
    /\ bundle.fenceSnapshot = candidate.fenceSnapshot

theorem reject_is_no_publish
    (candidate : CandidateResult)
    (reject : RejectRecord)
    (bundle : CommitBundle)
    (hReject : reject.rejectCode = RejectCode.fenceMismatch)
    (hBundle : CandidatePublishesAs candidate "commit" bundle) :
    candidate.fenceSnapshot = bundle.fenceSnapshot := by
  simp [CandidatePublishesAs] at hBundle
  exact hBundle.right.right.right

theorem incompatible_fence_cannot_publish
    (candidate : CandidateResult)
    (observed : FenceSnapshot)
    (bundle : CommitBundle)
    (hIncompatible : ¬ FenceCompatible candidate.fenceSnapshot observed)
    (hPublishes : CandidatePublishesAs candidate bundle.commitAttemptId bundle) :
    observed ≠ bundle.fenceSnapshot := by
  intro hEq
  apply hIncompatible
  simp [FenceCompatible]
  simpa [CandidatePublishesAs] using hPublishes.right.right.right.symm.trans hEq

end OxFml
