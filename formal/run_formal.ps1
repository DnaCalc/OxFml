param()

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$leanFiles = @(
    Join-Path $PSScriptRoot "lean/OxFmlSessionLifecycle.lean"
    Join-Path $PSScriptRoot "lean/OxFmlExternalReferenceDeferred.lean"
)
$tlaJobs = @(
    @{
        File = Join-Path $PSScriptRoot "tla/FecSessionLifecycle.tla"
        Config = Join-Path $PSScriptRoot "tla/FecSessionLifecycle.cfg"
    },
    @{
        File = Join-Path $PSScriptRoot "tla/FecExternalCapabilityGate.tla"
        Config = Join-Path $PSScriptRoot "tla/FecExternalCapabilityGate.cfg"
    }
)
$tlaJar = Join-Path $PSScriptRoot "tools/tla2tools.jar"

foreach ($leanFile in $leanFiles) {
    lean $leanFile
}

foreach ($job in $tlaJobs) {
    java -cp $tlaJar tlc2.TLC -cleanup -deadlock -config $job.Config $job.File
}
