---- MODULE FecHigherOrderCallableBoundary ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Prepared",
  "CatalogAdmitted",
  "Executed",
  "CallableInvokerRejected"
}

VARIABLES phase, catalogAdmitted, callableInvokerRequired

Init ==
  /\ phase = [s \in SessionIds |-> "Prepared"]
  /\ catalogAdmitted = [s \in SessionIds |-> FALSE]
  /\ callableInvokerRequired = [s \in SessionIds |-> FALSE]

AdmitHigherOrderLane(s, requiresInvoker) ==
  /\ phase[s] = "Prepared"
  /\ phase' = [phase EXCEPT ![s] = "CatalogAdmitted"]
  /\ catalogAdmitted' = [catalogAdmitted EXCEPT ![s] = TRUE]
  /\ callableInvokerRequired' = [callableInvokerRequired EXCEPT ![s] = requiresInvoker]

ExecuteWithoutInvokerGap(s) ==
  /\ phase[s] = "CatalogAdmitted"
  /\ catalogAdmitted[s]
  /\ ~callableInvokerRequired[s]
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ UNCHANGED <<catalogAdmitted, callableInvokerRequired>>

RejectAtCallableInvoker(s) ==
  /\ phase[s] = "CatalogAdmitted"
  /\ catalogAdmitted[s]
  /\ callableInvokerRequired[s]
  /\ phase' = [phase EXCEPT ![s] = "CallableInvokerRejected"]
  /\ UNCHANGED <<catalogAdmitted, callableInvokerRequired>>

Next ==
  \E s \in SessionIds :
    AdmitHigherOrderLane(s, TRUE)
    \/ AdmitHigherOrderLane(s, FALSE)
    \/ ExecuteWithoutInvokerGap(s)
    \/ RejectAtCallableInvoker(s)

ExecutedRequiresCatalogAdmission ==
  \A s \in SessionIds :
    phase[s] = "Executed" => catalogAdmitted[s]

CallableBoundaryRejectRequiresCatalogAdmission ==
  \A s \in SessionIds :
    phase[s] = "CallableInvokerRejected" => catalogAdmitted[s]

CallableBoundaryRejectRequiresInvokerGap ==
  \A s \in SessionIds :
    phase[s] = "CallableInvokerRejected" => callableInvokerRequired[s]

Spec == Init /\ [][Next]_<<phase, catalogAdmitted, callableInvokerRequired>>

====
