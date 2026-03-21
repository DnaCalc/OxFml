---- MODULE FecPlacementDeferralExpiryBoundary ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "PlacementDeferred",
  "Executed",
  "Committed",
  "Rejected",
  "Expired"
}

NoLocus == "no_locus"
NoSession == "no_session"

VARIABLES
  phase,
  requestedLocus,
  assignedLocus,
  localOwner,
  publishable,
  expiryEligible,
  lastAction,
  lastSession,
  lastLocus

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ requestedLocus = [s \in SessionIds |->
        IF s = "session:1" THEN "locus:A" ELSE "locus:B"]
  /\ assignedLocus = [s \in SessionIds |-> NoLocus]
  /\ localOwner = [l \in {"locus:A", "locus:B"} |-> NoSession]
  /\ publishable = [s \in SessionIds |-> FALSE]
  /\ expiryEligible = [s \in SessionIds |-> FALSE]
  /\ lastAction = "Init"
  /\ lastSession = NoSession
  /\ lastLocus = NoLocus

DeferPlacement(s) ==
  LET l == requestedLocus[s] IN
  /\ phase[s] = "Open"
  /\ phase' = [phase EXCEPT ![s] = "PlacementDeferred"]
  /\ expiryEligible' = [expiryEligible EXCEPT ![s] = TRUE]
  /\ UNCHANGED <<requestedLocus, assignedLocus, localOwner, publishable>>
  /\ lastAction' = "DeferPlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

ResolvePlacementLocally(s) ==
  LET l == requestedLocus[s] IN
  /\ phase[s] = "PlacementDeferred"
  /\ localOwner[l] = NoSession
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ assignedLocus' = [assignedLocus EXCEPT ![s] = l]
  /\ localOwner' = [localOwner EXCEPT ![l] = s]
  /\ publishable' = [publishable EXCEPT ![s] = FALSE]
  /\ expiryEligible' = [expiryEligible EXCEPT ![s] = FALSE]
  /\ UNCHANGED requestedLocus
  /\ lastAction' = "ResolvePlacementLocally"
  /\ lastSession' = s
  /\ lastLocus' = l

ExpireDeferredPlacement(s) ==
  LET l == requestedLocus[s] IN
  /\ phase[s] = "PlacementDeferred"
  /\ expiryEligible[s]
  /\ phase' = [phase EXCEPT ![s] = "Expired"]
  /\ expiryEligible' = [expiryEligible EXCEPT ![s] = FALSE]
  /\ assignedLocus' = [assignedLocus EXCEPT ![s] = NoLocus]
  /\ publishable' = [publishable EXCEPT ![s] = FALSE]
  /\ UNCHANGED <<requestedLocus, localOwner>>
  /\ lastAction' = "ExpireDeferredPlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

RejectDeferredPlacement(s) ==
  LET l == requestedLocus[s] IN
  /\ phase[s] = "PlacementDeferred"
  /\ phase' = [phase EXCEPT ![s] = "Rejected"]
  /\ expiryEligible' = [expiryEligible EXCEPT ![s] = FALSE]
  /\ assignedLocus' = [assignedLocus EXCEPT ![s] = NoLocus]
  /\ publishable' = [publishable EXCEPT ![s] = FALSE]
  /\ UNCHANGED <<requestedLocus, localOwner>>
  /\ lastAction' = "RejectDeferredPlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

CommitExecutedPlacement(s) ==
  LET l == assignedLocus[s] IN
  /\ phase[s] = "Executed"
  /\ l # NoLocus
  /\ localOwner[l] = s
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ publishable' = [publishable EXCEPT ![s] = TRUE]
  /\ localOwner' = [localOwner EXCEPT ![l] = NoSession]
  /\ UNCHANGED <<requestedLocus, assignedLocus, expiryEligible>>
  /\ lastAction' = "CommitExecutedPlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

AbortExecutedPlacement(s) ==
  LET l == assignedLocus[s] IN
  /\ phase[s] = "Executed"
  /\ l # NoLocus
  /\ localOwner[l] = s
  /\ phase' = [phase EXCEPT ![s] = "Rejected"]
  /\ localOwner' = [localOwner EXCEPT ![l] = NoSession]
  /\ assignedLocus' = [assignedLocus EXCEPT ![s] = NoLocus]
  /\ publishable' = [publishable EXCEPT ![s] = FALSE]
  /\ UNCHANGED <<requestedLocus, expiryEligible>>
  /\ lastAction' = "AbortExecutedPlacement"
  /\ lastSession' = s
  /\ lastLocus' = l

Next ==
  \E s \in SessionIds :
    DeferPlacement(s)
    \/ ResolvePlacementLocally(s)
    \/ ExpireDeferredPlacement(s)
    \/ RejectDeferredPlacement(s)
    \/ CommitExecutedPlacement(s)
    \/ AbortExecutedPlacement(s)

DeferredPlacementIsNotPublishable ==
  \A s \in SessionIds :
    phase[s] = "PlacementDeferred" =>
      /\ ~publishable[s]
      /\ assignedLocus[s] = NoLocus
      /\ expiryEligible[s]

ExpiredDeferredPlacementHasNoClaim ==
  \A s \in SessionIds :
    phase[s] = "Expired" =>
      /\ ~publishable[s]
      /\ assignedLocus[s] = NoLocus
      /\ ~expiryEligible[s]

RejectedDeferredPlacementHasNoClaim ==
  \A s \in SessionIds :
    phase[s] = "Rejected" =>
      /\ ~publishable[s]
      /\ assignedLocus[s] = NoLocus
      /\ ~expiryEligible[s]

LocalOwnershipOnlyForExecuted ==
  \A s \in SessionIds :
    assignedLocus[s] # NoLocus =>
      phase[s] = "Executed" \/ phase[s] = "Committed"

CommitRequiresLocalResolution ==
  lastAction = "CommitExecutedPlacement" =>
    /\ lastSession # NoSession
    /\ publishable[lastSession]
    /\ assignedLocus[lastSession] = requestedLocus[lastSession]

Spec ==
  Init /\ [][Next]_<<phase, requestedLocus, assignedLocus, localOwner, publishable, expiryEligible, lastAction, lastSession, lastLocus>>

====
