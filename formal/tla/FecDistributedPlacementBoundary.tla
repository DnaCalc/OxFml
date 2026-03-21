---- MODULE FecDistributedPlacementBoundary ----
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS SessionIds, LocusIds

SessionPhase == {
  "Open",
  "PlacementDeferred",
  "Executed",
  "Committed",
  "Rejected"
}

NoLocus == "no_locus"
NoSession == "no_session"

VARIABLES
  phase,
  requestedLocus,
  assignedLocus,
  localOwner,
  publishable,
  lastAction,
  lastSession,
  lastLocus

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ requestedLocus = [s \in SessionIds |->
        IF s = "session:1" THEN "locus:A" ELSE "locus:B"]
  /\ assignedLocus = [s \in SessionIds |-> NoLocus]
  /\ localOwner = [l \in LocusIds |-> NoSession]
  /\ publishable = [s \in SessionIds |-> FALSE]
  /\ lastAction = "Init"
  /\ lastSession = NoSession
  /\ lastLocus = NoLocus

AdmitLocalPlacement(s) ==
  LET l == requestedLocus[s] IN
  /\ phase[s] = "Open"
  /\ localOwner[l] = NoSession
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ assignedLocus' = [assignedLocus EXCEPT ![s] = l]
  /\ localOwner' = [localOwner EXCEPT ![l] = s]
  /\ publishable' = [publishable EXCEPT ![s] = FALSE]
  /\ UNCHANGED requestedLocus
  /\ lastAction' = "AdmitLocalPlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

DeferRemotePlacement(s) ==
  LET l == requestedLocus[s] IN
  /\ phase[s] = "Open"
  /\ phase' = [phase EXCEPT ![s] = "PlacementDeferred"]
  /\ UNCHANGED <<requestedLocus, assignedLocus, localOwner, publishable>>
  /\ lastAction' = "DeferRemotePlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

ResolveDeferredPlacementLocally(s) ==
  LET l == requestedLocus[s] IN
  /\ phase[s] = "PlacementDeferred"
  /\ localOwner[l] = NoSession
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ assignedLocus' = [assignedLocus EXCEPT ![s] = l]
  /\ localOwner' = [localOwner EXCEPT ![l] = s]
  /\ publishable' = [publishable EXCEPT ![s] = FALSE]
  /\ UNCHANGED requestedLocus
  /\ lastAction' = "ResolveDeferredPlacementLocally"
  /\ lastSession' = s
  /\ lastLocus' = l

CommitPublish(s) ==
  LET l == assignedLocus[s] IN
  /\ phase[s] = "Executed"
  /\ l # NoLocus
  /\ localOwner[l] = s
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ publishable' = [publishable EXCEPT ![s] = TRUE]
  /\ localOwner' = [localOwner EXCEPT ![l] = NoSession]
  /\ UNCHANGED <<requestedLocus, assignedLocus>>
  /\ lastAction' = "CommitPublish"
  /\ lastSession' = s
  /\ lastLocus' = l

RejectDeferredPlacement(s) ==
  /\ phase[s] = "PlacementDeferred"
  /\ phase' = [phase EXCEPT ![s] = "Rejected"]
  /\ UNCHANGED <<requestedLocus, assignedLocus, localOwner, publishable>>
  /\ lastAction' = "RejectDeferredPlacement"
  /\ lastSession' = s
  /\ lastLocus' = requestedLocus[s]

AbortExecutedPlacement(s) ==
  LET l == assignedLocus[s] IN
  /\ phase[s] = "Executed"
  /\ l # NoLocus
  /\ localOwner[l] = s
  /\ phase' = [phase EXCEPT ![s] = "Rejected"]
  /\ localOwner' = [localOwner EXCEPT ![l] = NoSession]
  /\ assignedLocus' = [assignedLocus EXCEPT ![s] = NoLocus]
  /\ UNCHANGED <<requestedLocus, publishable>>
  /\ lastAction' = "AbortExecutedPlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

Next ==
  \E s \in SessionIds :
    AdmitLocalPlacement(s)
    \/ DeferRemotePlacement(s)
    \/ ResolveDeferredPlacementLocally(s)
    \/ CommitPublish(s)
    \/ RejectDeferredPlacement(s)
    \/ AbortExecutedPlacement(s)

ExecutedRequiresLocalPlacement ==
  \A s \in SessionIds :
    phase[s] = "Executed" =>
      /\ assignedLocus[s] = requestedLocus[s]
      /\ localOwner[assignedLocus[s]] = s

DeferredPlacementHasNoLocalClaim ==
  \A s \in SessionIds :
    phase[s] = "PlacementDeferred" =>
      /\ assignedLocus[s] = NoLocus
      /\ publishable[s] = FALSE

RejectedWithoutCommitIsNotPublishable ==
  \A s \in SessionIds :
    phase[s] = "Rejected" => publishable[s] = FALSE

CommitRequiresExecutedLocalPlacement ==
  lastAction = "CommitPublish" =>
    /\ lastSession # NoSession
    /\ phase[lastSession] = "Committed"
    /\ publishable[lastSession]
    /\ assignedLocus[lastSession] = requestedLocus[lastSession]

LocalOwnershipIsExclusive ==
  \A l \in LocusIds :
    Cardinality({s \in SessionIds : phase[s] = "Executed" /\ assignedLocus[s] = l}) <= 1

Spec ==
  Init /\ [][Next]_<<phase, requestedLocus, assignedLocus, localOwner, publishable, lastAction, lastSession, lastLocus>>

====
