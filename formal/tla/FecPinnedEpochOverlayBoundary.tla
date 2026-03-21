---- MODULE FecPinnedEpochOverlayBoundary ----
EXTENDS Naturals, Sequences

CONSTANT SessionIds

SessionPhase == {
  "Open",
  "Executed",
  "Committed",
  "Aborted",
  "Expired"
}

EpochValues == {
  "epoch:1",
  "epoch:2",
  "epoch:3"
}

NoEpoch == "no_epoch"
NoSession == "no_session"

NextEpoch(epoch) ==
  CASE epoch = "epoch:1" -> "epoch:2"
    [] epoch = "epoch:2" -> "epoch:3"
    [] OTHER -> epoch

VARIABLES
  phase,
  snapshotEpoch,
  overlayEpoch,
  retainedEpochs,
  pinCount,
  lastAction,
  lastSession,
  lastEpoch

Init ==
  /\ phase = [s \in SessionIds |-> "Open"]
  /\ snapshotEpoch = [s \in SessionIds |-> "epoch:1"]
  /\ overlayEpoch = [s \in SessionIds |-> NoEpoch]
  /\ retainedEpochs = {}
  /\ pinCount = [e \in EpochValues |-> 0]
  /\ lastAction = "Init"
  /\ lastSession = NoSession
  /\ lastEpoch = NoEpoch

BuildFreshOverlay(s) ==
  LET e == snapshotEpoch[s] IN
  /\ phase[s] = "Open"
  /\ overlayEpoch[s] = NoEpoch
  /\ e \notin retainedEpochs
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = e]
  /\ retainedEpochs' = retainedEpochs \cup {e}
  /\ pinCount' = [pinCount EXCEPT ![e] = @ + 1]
  /\ UNCHANGED snapshotEpoch
  /\ lastAction' = "BuildFreshOverlay"
  /\ lastSession' = s
  /\ lastEpoch' = e

ReuseRetainedOverlay(s) ==
  LET e == snapshotEpoch[s] IN
  /\ phase[s] = "Open"
  /\ overlayEpoch[s] = NoEpoch
  /\ e \in retainedEpochs
  /\ phase' = [phase EXCEPT ![s] = "Executed"]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = e]
  /\ UNCHANGED <<snapshotEpoch, retainedEpochs>>
  /\ pinCount' = [pinCount EXCEPT ![e] = @ + 1]
  /\ lastAction' = "ReuseRetainedOverlay"
  /\ lastSession' = s
  /\ lastEpoch' = e

CommitCleanup(s) ==
  LET e == overlayEpoch[s] IN
  /\ phase[s] = "Executed"
  /\ e # NoEpoch
  /\ phase' = [phase EXCEPT ![s] = "Committed"]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = NoEpoch]
  /\ pinCount' = [pinCount EXCEPT ![e] = @ - 1]
  /\ UNCHANGED <<snapshotEpoch, retainedEpochs>>
  /\ lastAction' = "CommitCleanup"
  /\ lastSession' = s
  /\ lastEpoch' = e

AbortCleanup(s) ==
  LET e == overlayEpoch[s] IN
  /\ phase[s] = "Executed"
  /\ e # NoEpoch
  /\ phase' = [phase EXCEPT ![s] = "Aborted"]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = NoEpoch]
  /\ pinCount' = [pinCount EXCEPT ![e] = @ - 1]
  /\ UNCHANGED <<snapshotEpoch, retainedEpochs>>
  /\ lastAction' = "AbortCleanup"
  /\ lastSession' = s
  /\ lastEpoch' = e

ExpireCleanup(s) ==
  LET e == overlayEpoch[s] IN
  /\ phase[s] = "Executed"
  /\ e # NoEpoch
  /\ phase' = [phase EXCEPT ![s] = "Expired"]
  /\ overlayEpoch' = [overlayEpoch EXCEPT ![s] = NoEpoch]
  /\ pinCount' = [pinCount EXCEPT ![e] = @ - 1]
  /\ UNCHANGED <<snapshotEpoch, retainedEpochs>>
  /\ lastAction' = "ExpireCleanup"
  /\ lastSession' = s
  /\ lastEpoch' = e

AdvanceOpenEpoch(s) ==
  /\ phase[s] = "Open"
  /\ overlayEpoch[s] = NoEpoch
  /\ snapshotEpoch[s] # "epoch:3"
  /\ snapshotEpoch' = [snapshotEpoch EXCEPT ![s] = NextEpoch(@)]
  /\ UNCHANGED <<phase, overlayEpoch, retainedEpochs, pinCount>>
  /\ lastAction' = "AdvanceOpenEpoch"
  /\ lastSession' = s
  /\ lastEpoch' = snapshotEpoch'[s]

EvictRetainedEpoch(e) ==
  /\ e \in retainedEpochs
  /\ pinCount[e] = 0
  /\ retainedEpochs' = retainedEpochs \ {e}
  /\ UNCHANGED <<phase, snapshotEpoch, overlayEpoch, pinCount>>
  /\ lastAction' = "EvictRetainedEpoch"
  /\ lastSession' = NoSession
  /\ lastEpoch' = e

Next ==
  \E s \in SessionIds :
    BuildFreshOverlay(s)
    \/ ReuseRetainedOverlay(s)
    \/ CommitCleanup(s)
    \/ AbortCleanup(s)
    \/ ExpireCleanup(s)
    \/ AdvanceOpenEpoch(s)
  \/ \E e \in EpochValues :
    EvictRetainedEpoch(e)

ExecutedSessionUsesPinnedCurrentEpoch ==
  \A s \in SessionIds :
    phase[s] = "Executed" =>
      /\ overlayEpoch[s] # NoEpoch
      /\ overlayEpoch[s] = snapshotEpoch[s]
      /\ pinCount[overlayEpoch[s]] > 0

PinnedEpochRemainsRetained ==
  \A e \in EpochValues :
    pinCount[e] > 0 => e \in retainedEpochs

NonExecutedSessionHasNoOverlayEpoch ==
  \A s \in SessionIds :
    phase[s] # "Executed" => overlayEpoch[s] = NoEpoch

ReuseUsesCurrentRetainedEpoch ==
  lastAction = "ReuseRetainedOverlay" =>
    /\ lastSession # NoSession
    /\ lastEpoch = snapshotEpoch[lastSession]
    /\ lastEpoch \in retainedEpochs

EvictionOnlyRemovesUnpinnedEpoch ==
  lastAction = "EvictRetainedEpoch" =>
    /\ lastEpoch \notin retainedEpochs
    /\ pinCount[lastEpoch] = 0

Spec ==
  Init /\ [][Next]_<<phase, snapshotEpoch, overlayEpoch, retainedEpochs, pinCount, lastAction, lastSession, lastEpoch>>

====
