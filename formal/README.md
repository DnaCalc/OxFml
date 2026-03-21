# OxFml Local Formal Artifacts

This directory holds the first local formal-model skeletons for OxFml.

Working rules:
1. these files are local implementation-start artifacts, not final Green-owned proof or model authority,
2. they exist to keep the formal shape coupled to exercised code and replay fixtures,
3. promotion to shared Green-owned locations remains a later step.

Current local artifacts:
1. `formal/lean/OxFmlSessionLifecycle.lean`
2. `formal/lean/OxFmlExternalReferenceDeferred.lean`
3. `formal/lean/OxFmlNameCarrierDeferred.lean`
4. `formal/lean/OxFmlFailureStageSplit.lean`
5. `formal/lean/OxFmlExternalNameCarrier.lean`
6. `formal/tla/FecSessionLifecycle.tla`
7. `formal/tla/FecExternalCapabilityGate.tla`
8. `formal/tla/FecHigherOrderCallableBoundary.tla`
9. `formal/tla/FecSessionContentionBoundary.tla`
10. `formal/tla/FecRetryAfterReleaseBoundary.tla`
11. `formal/tla/FecOverlayCleanupBoundary.tla`
12. `formal/tla/FecPinnedEpochOverlayBoundary.tla`
13. `formal/tla/FecDistributedPlacementBoundary.tla`
14. `formal/tla/FecRetryOrderingBoundary.tla`
15. `formal/tla/FecPlacementDeferralExpiryBoundary.tla`
16. `formal/run_formal.ps1`
