---- MODULE FecRetryOrderingBoundary ----
EXTENDS Naturals, Sequences, FiniteSets

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "PendingRetry",
  "Executed",
  "Committed",
  "Rejected"
}

NoOwner == "no_owner"
NoOrder == 0

VARIABLES
  phase,
  locusOwner,
  retryAdmissible,
  retryOrder,
  nextOrder,
  lastAction,
  lastSession,
  lastGrantedOrder

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ locusOwner = NoOwner
  /\ retryAdmissible = [s \in SessionIds |-> FALSE]
  /\ retryOrder = [s \in SessionIds |-> NoOrder]
  /\ nextOrder = 1
  /\ lastAction = "Init"
  /\ lastSession = "none"
  /\ lastGrantedOrder = NoOrder

OccupyInitial(s) ==
  /\ phase[s] = "Open"
  /\ locusOwner = NoOwner
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ locusOwner' = s
  /\ UNCHANGED <<retryAdmissible, retryOrder, nextOrder>>
  /\ lastAction' = "OccupyInitial"
  /\ lastSession' = s
  /\ lastGrantedOrder' = NoOrder

BecomePendingRetry(s) ==
  /\ phase[s] = "Open"
  /\ locusOwner # NoOwner
  /\ phase' = [phase EXCEPT ![s] = "PendingRetry"]
  /\ retryAdmissible' = [retryAdmissible EXCEPT ![s] = TRUE]
  /\ retryOrder' = [retryOrder EXCEPT ![s] = nextOrder]
  /\ nextOrder' = nextOrder + 1
  /\ UNCHANGED locusOwner
  /\ lastAction' = "BecomePendingRetry"
  /\ lastSession' = s
  /\ lastGrantedOrder' = NoOrder

ReleaseExecuted(s) ==
  /\ phase[s] = "Executed"
  /\ locusOwner = s
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ locusOwner' = NoOwner
  /\ UNCHANGED <<retryAdmissible, retryOrder, nextOrder>>
  /\ lastAction' = "ReleaseExecuted"
  /\ lastSession' = s
  /\ lastGrantedOrder' = NoOrder

GrantNextRetry(s) ==
  /\ phase[s] = "PendingRetry"
  /\ retryAdmissible[s]
  /\ retryOrder[s] # NoOrder
  /\ locusOwner = NoOwner
  /\ \A t \in SessionIds :
       phase[t] = "PendingRetry" /\ retryAdmissible[t] =>
         retryOrder[s] <= retryOrder[t]
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ locusOwner' = s
  /\ retryAdmissible' = [retryAdmissible EXCEPT ![s] = FALSE]
  /\ retryOrder' = [retryOrder EXCEPT ![s] = NoOrder]
  /\ UNCHANGED nextOrder
  /\ lastAction' = "GrantNextRetry"
  /\ lastSession' = s
  /\ lastGrantedOrder' = retryOrder[s]

RejectPendingRetry(s) ==
  /\ phase[s] = "PendingRetry"
  /\ phase' = [phase EXCEPT ![s] = "Rejected"]
  /\ retryAdmissible' = [retryAdmissible EXCEPT ![s] = FALSE]
  /\ retryOrder' = [retryOrder EXCEPT ![s] = NoOrder]
  /\ UNCHANGED <<locusOwner, nextOrder>>
  /\ lastAction' = "RejectPendingRetry"
  /\ lastSession' = s
  /\ lastGrantedOrder' = NoOrder

Next ==
  \E s \in SessionIds :
    OccupyInitial(s)
    \/ BecomePendingRetry(s)
    \/ ReleaseExecuted(s)
    \/ GrantNextRetry(s)
    \/ RejectPendingRetry(s)

PendingRetryRequiresOrder ==
  \A s \in SessionIds :
    phase[s] = "PendingRetry" =>
      /\ retryAdmissible[s]
      /\ retryOrder[s] # NoOrder

NonPendingRetryHasNoOrder ==
  \A s \in SessionIds :
    phase[s] # "PendingRetry" /\ phase[s] # "Executed" =>
      retryOrder[s] = NoOrder

GrantedRetryUsesLowestOrder ==
  lastAction = "GrantNextRetry" =>
    \A s \in SessionIds :
      phase[s] = "PendingRetry" /\ retryAdmissible[s] =>
        retryOrder[s] > lastGrantedOrder

RejectedRetryIsNotAdmissible ==
  \A s \in SessionIds :
    phase[s] = "Rejected" =>
      /\ ~retryAdmissible[s]
      /\ retryOrder[s] = NoOrder

ExclusiveExecutedOwnership ==
  Cardinality({s \in SessionIds : phase[s] = "Executed"}) <= 1

Spec ==
  Init /\ [][Next]_<<phase, locusOwner, retryAdmissible, retryOrder, nextOrder, lastAction, lastSession, lastGrantedOrder>>

====
