---- MODULE FecSessionLifecycle ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "CapabilityViewEstablished",
  "Executed",
  "Committed",
  "Rejected",
  "Aborted",
  "Expired"
}

VARIABLES phase, candidateBuilt, rejectCode

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ candidateBuilt = [s \in SessionIds |-> FALSE]
  /\ rejectCode = [s \in SessionIds |-> "None"]

EstablishCapability(s) ==
  /\ phase[s] = "Open"
  /\ phase' = [phase EXCEPT ![s] = "CapabilityViewEstablished"]
  /\ UNCHANGED <<candidateBuilt, rejectCode>>

Execute(s) ==
  /\ phase[s] = "CapabilityViewEstablished"
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ candidateBuilt' = [candidateBuilt EXCEPT ![s] = TRUE]
  /\ UNCHANGED rejectCode

CommitAccepted(s) ==
  /\ phase[s] = "Executed"
  /\ candidateBuilt[s] = TRUE
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ UNCHANGED <<candidateBuilt, rejectCode>>

Reject(s, code) ==
  /\ phase[s] \in {"Open", "CapabilityViewEstablished", "Executed"}
  /\ phase' = [phase EXCEPT ![s] = "Rejected"]
  /\ rejectCode' = [rejectCode EXCEPT ![s] = code]
  /\ UNCHANGED candidateBuilt

Abort(s) ==
  /\ phase[s] \in {"Open", "CapabilityViewEstablished", "Executed"}
  /\ phase' = [phase EXCEPT ![s] = "Aborted"]
  /\ rejectCode' = [rejectCode EXCEPT ![s] = "SessionTerminated"]
  /\ UNCHANGED candidateBuilt

Expire(s) ==
  /\ phase[s] \in {"Open", "CapabilityViewEstablished", "Executed"}
  /\ phase' = [phase EXCEPT ![s] = "Expired"]
  /\ rejectCode' = [rejectCode EXCEPT ![s] = "SessionTerminated"]
  /\ UNCHANGED candidateBuilt

Next ==
  \E s \in SessionIds :
    EstablishCapability(s)
    \/ Execute(s)
    \/ CommitAccepted(s)
    \/ Reject(s, "CapabilityDenied")
    \/ Reject(s, "FenceMismatch")
    \/ Abort(s)
    \/ Expire(s)

NoPublishAfterReject ==
  \A s \in SessionIds :
    phase[s] \in {"Rejected", "Aborted", "Expired"} => phase[s] # "Committed"

CommitRequiresCandidate ==
  \A s \in SessionIds :
    phase[s] = "Committed" => candidateBuilt[s]

Spec == Init /\ [][Next]_<<phase, candidateBuilt, rejectCode>>

====
