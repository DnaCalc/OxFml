---- MODULE FecSessionContentionBoundary ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "Executed",
  "RejectedBusy",
  "Committed"
}

Locus == "sheet:default!r1c1"
NoOwner == "no_owner"

VARIABLES phase, busyOwner, published

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ busyOwner = NoOwner
  /\ published = [s \in SessionIds |-> FALSE]

ExecuteAcquire(s) ==
  /\ phase[s] = "Open"
  /\ busyOwner = NoOwner
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ busyOwner' = s
  /\ UNCHANGED published

RejectBusy(s) ==
  /\ phase[s] = "Open"
  /\ busyOwner # NoOwner
  /\ busyOwner # s
  /\ phase' = [phase EXCEPT ![s] = "RejectedBusy"]
  /\ UNCHANGED <<busyOwner, published>>

CommitRelease(s) ==
  /\ phase[s] = "Executed"
  /\ busyOwner = s
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ busyOwner' = NoOwner
  /\ published' = [published EXCEPT ![s] = TRUE]

Next ==
  \E s \in SessionIds :
    ExecuteAcquire(s)
    \/ RejectBusy(s)
    \/ CommitRelease(s)

PublishedRequiresExecution ==
  \A s \in SessionIds :
    published[s] => phase[s] = "Committed"

BusyRejectDoesNotPublish ==
  \A s \in SessionIds :
    phase[s] = "RejectedBusy" => ~published[s]

SingleBusyOwner ==
  busyOwner = NoOwner \/ busyOwner \in SessionIds

BusyRejectRequiresAnotherOwner ==
  \A s \in SessionIds :
    phase[s] = "RejectedBusy" => \E t \in SessionIds : t # s

Spec == Init /\ [][Next]_<<phase, busyOwner, published>>

====
