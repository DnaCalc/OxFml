---- MODULE FecOverlayCleanupBoundary ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "Executed",
  "Committed",
  "Aborted",
  "Expired"
}

NoEpoch == "no_epoch"

VARIABLES phase, snapshotEpoch, overlayActive, overlayEpoch

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ snapshotEpoch = [s \in SessionIds |-> "epoch:1"]
  /\ overlayActive = [s \in SessionIds |-> FALSE]
  /\ overlayEpoch = [s \in SessionIds |-> NoEpoch]

ExecuteBuildOverlay(s) ==
  /\ phase[s] = "Open"
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ overlayActive' = [overlayActive EXCEPT ![s] = TRUE]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = snapshotEpoch[s]]
  /\ UNCHANGED snapshotEpoch

CommitCleanup(s) ==
  /\ phase[s] = "Executed"
  /\ overlayActive[s]
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ overlayActive' = [overlayActive EXCEPT ![s] = FALSE]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = NoEpoch]
  /\ UNCHANGED snapshotEpoch

AbortCleanup(s) ==
  /\ phase[s] \in {"Open", "Executed"}
  /\ phase' = [phase EXCEPT ![s] = "Aborted"]
  /\ overlayActive' = [overlayActive EXCEPT ![s] = FALSE]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = NoEpoch]
  /\ UNCHANGED snapshotEpoch

ExpireCleanup(s) ==
  /\ phase[s] \in {"Open", "Executed"}
  /\ phase' = [phase EXCEPT ![s] = "Expired"]
  /\ overlayActive' = [overlayActive EXCEPT ![s] = FALSE]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = NoEpoch]
  /\ UNCHANGED snapshotEpoch

EpochAdvanceWithoutOverlay(s) ==
  /\ phase[s] = "Open"
  /\ ~overlayActive[s]
  /\ snapshotEpoch' = [snapshotEpoch EXCEPT ![s] = "epoch:2"]
  /\ UNCHANGED <<phase, overlayActive, overlayEpoch>>

Next ==
  \E s \in SessionIds :
    ExecuteBuildOverlay(s)
    \/ CommitCleanup(s)
    \/ AbortCleanup(s)
    \/ ExpireCleanup(s)
    \/ EpochAdvanceWithoutOverlay(s)

ActiveOverlayRequiresExecutedPhase ==
  \A s \in SessionIds :
    overlayActive[s] => phase[s] = "Executed"

ActiveOverlayUsesCurrentEpoch ==
  \A s \in SessionIds :
    overlayActive[s] => overlayEpoch[s] = snapshotEpoch[s]

ReleasedOrTerminatedSessionHasNoOverlay ==
  \A s \in SessionIds :
    phase[s] \in {"Committed", "Aborted", "Expired"} => /\ ~overlayActive[s] /\ overlayEpoch[s] = NoEpoch

InactiveSessionHasNoOverlayEpoch ==
  \A s \in SessionIds :
    ~overlayActive[s] => overlayEpoch[s] = NoEpoch

Spec == Init /\ [][Next]_<<phase, snapshotEpoch, overlayActive, overlayEpoch>>

====
