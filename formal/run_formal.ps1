param()

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$leanFiles = @(
    Join-Path $PSScriptRoot "lean/OxFmlSessionLifecycle.lean"
    Join-Path $PSScriptRoot "lean/OxFmlExternalReferenceDeferred.lean"
    Join-Path $PSScriptRoot "lean/OxFmlNameCarrierDeferred.lean"
    Join-Path $PSScriptRoot "lean/OxFmlFailureStageSplit.lean"
    Join-Path $PSScriptRoot "lean/OxFmlExternalNameCarrier.lean"
)
$tlaJobs = @(
    @{
        File = Join-Path $PSScriptRoot "tla/FecSessionLifecycle.tla"
        Config = Join-Path $PSScriptRoot "tla/FecSessionLifecycle.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecExternalCapabilityGate.tla"
        Config = Join-Path $PSScriptRoot "tla/FecExternalCapabilityGate.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecHigherOrderCallableBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecHigherOrderCallableBoundary.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecSessionContentionBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecSessionContentionBoundary.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecRetryAfterReleaseBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecRetryAfterReleaseBoundary.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecOverlayCleanupBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecOverlayCleanupBoundary.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecPinnedEpochOverlayBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecPinnedEpochOverlayBoundary.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecDistributedPlacementBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecDistributedPlacementBoundary.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecRetryOrderingBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecRetryOrderingBoundary.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecPlacementDeferralExpiryBoundary.tla"
        Config = Join-Path $PSScriptRoot "tla/FecPlacementDeferralExpiryBoundary.cfg"
    }
)
$tlaJar = Join-Path $PSScriptRoot "tools/tla2tools.jar"

foreach ($leanFile in $leanFiles) {
    lean $leanFile
}

foreach ($job in $tlaJobs) {
    java -cp $tlaJar tlc2.TLC -cleanup -deadlock -config $job.Config $job.File
}
