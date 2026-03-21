---- MODULE FecRetryAfterReleaseBoundary ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "Executed",
  "RejectedBusy",
  "Committed"
}

NoOwner == "no_owner"

VARIABLES phase, busyOwner, published, retryEligible, hadBusyReject

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ busyOwner = NoOwner
  /\ published = [s \in SessionIds |-> FALSE]
  /\ retryEligible = [s \in SessionIds |-> FALSE]
  /\ hadBusyReject = [s \in SessionIds |-> FALSE]

ExecuteAcquire(s) ==
  /\ phase[s] = "Open"
  /\ busyOwner = NoOwner
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ busyOwner' = s
  /\ UNCHANGED <<published, retryEligible, hadBusyReject>>

RejectBusy(s) ==
  /\ phase[s] = "Open"
  /\ busyOwner # NoOwner
  /\ busyOwner # s
  /\ phase' = [phase EXCEPT ![s] = "RejectedBusy"]
  /\ retryEligible' = [retryEligible EXCEPT ![s] = TRUE]
  /\ hadBusyReject' = [hadBusyReject EXCEPT ![s] = TRUE]
  /\ UNCHANGED <<busyOwner, published>>

RetryAfterRelease(s) ==
  /\ phase[s] = "RejectedBusy"
  /\ retryEligible[s]
  /\ hadBusyReject[s]
  /\ busyOwner = NoOwner
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ busyOwner' = s
  /\ retryEligible' = [retryEligible EXCEPT ![s] = FALSE]
  /\ UNCHANGED <<published, hadBusyReject>>

CommitRelease(s) ==
  /\ phase[s] = "Executed"
  /\ busyOwner = s
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ busyOwner' = NoOwner
  /\ published' = [published EXCEPT ![s] = TRUE]
  /\ UNCHANGED <<retryEligible, hadBusyReject>>

Next ==
  \E s \in SessionIds :
    ExecuteAcquire(s)
    \/ RejectBusy(s)
    \/ RetryAfterRelease(s)
    \/ CommitRelease(s)

PublishedImpliesCommitted ==
  \A s \in SessionIds :
    published[s] => phase[s] = "Committed"

RetryEligibilityRequiresBusyRejectHistory ==
  \A s \in SessionIds :
    retryEligible[s] => hadBusyReject[s]

RejectedBusyImpliesRetryEligible ==
  \A s \in SessionIds :
    phase[s] = "RejectedBusy" => retryEligible[s]

RetriedExecutionRequiresBusyRejectHistory ==
  \A s \in SessionIds :
    /\ phase[s] = "Executed"
    /\ hadBusyReject[s]
    => busyOwner = s

Spec == Init /\ [][Next]_<<phase, busyOwner, published, retryEligible, hadBusyReject>>

====
