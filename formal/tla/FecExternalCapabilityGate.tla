---- MODULE FecExternalCapabilityGate ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "CapabilityViewEstablished",
  "Executed",
  "Rejected"
}

VARIABLES phase, externalProviderEnabled, asyncCoupled

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ externalProviderEnabled = [s \in SessionIds |-> FALSE]
  /\ asyncCoupled = [s \in SessionIds |-> FALSE]

EstablishCapability(s, enabled) ==
  /\ phase[s] = "Open"
  /\ phase' = [phase EXCEPT ![s] = "CapabilityViewEstablished"]
  /\ externalProviderEnabled' = [externalProviderEnabled EXCEPT ![s] = enabled]
  /\ UNCHANGED asyncCoupled

Execute(s) ==
  /\ phase[s] = "CapabilityViewEstablished"
  /\ externalProviderEnabled[s]
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ asyncCoupled' = [asyncCoupled EXCEPT ![s] = TRUE]
  /\ UNCHANGED externalProviderEnabled

RejectMissingExternalProvider(s) ==
  /\ phase[s] = "CapabilityViewEstablished"
  /\ ~externalProviderEnabled[s]
  /\ phase' = [phase EXCEPT ![s] = "Rejected"]
  /\ UNCHANGED <<externalProviderEnabled, asyncCoupled>>

Next ==
  \E s \in SessionIds :
    EstablishCapability(s, TRUE)
    \/ EstablishCapability(s, FALSE)
    \/ Execute(s)
    \/ RejectMissingExternalProvider(s)

ExecutedImpliesProvider ==
  \A s \in SessionIds :
    phase[s] = "Executed" => externalProviderEnabled[s]

RejectedWhenProviderMissing ==
  \A s \in SessionIds :
    /\ phase[s] = "Rejected"
    => ~externalProviderEnabled[s]

AsyncConsequenceImpliesProvider ==
  \A s \in SessionIds :
    asyncCoupled[s] => externalProviderEnabled[s]

Spec == Init /\ [][Next]_<<phase, externalProviderEnabled, asyncCoupled>>

====
