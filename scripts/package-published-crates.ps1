$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path $PSScriptRoot -Parent
Push-Location $repositoryRoot

try {
    $metadata = cargo metadata --format-version 1 --locked --no-deps | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed"
    }

    $sdk = $metadata.packages | Where-Object name -eq "samp-client-sdk"
    $abi = $metadata.packages | Where-Object name -eq "modkit-abi"
    $modkitSdk = $metadata.packages | Where-Object name -eq "modkit-sdk"
    $gta = $metadata.packages | Where-Object name -eq "gta-sa"
    $protocol = $metadata.packages | Where-Object name -eq "samp-protocol"
    $abiPath = (Split-Path $abi.manifest_path -Parent).Replace("\", "/")
    $modkitSdkPath = (Split-Path $modkitSdk.manifest_path -Parent).Replace("\", "/")
    $gtaPath = (Split-Path $gta.manifest_path -Parent).Replace("\", "/")
    $protocolPath = (Split-Path $protocol.manifest_path -Parent).Replace("\", "/")
    $abiPatch = "patch.crates-io.modkit-abi.path='$abiPath'"
    $modkitSdkPatch = "patch.crates-io.modkit-sdk.path='$modkitSdkPath'"
    $gtaPatch = "patch.crates-io.gta-sa.path='$gtaPath'"
    $protocolPatch = "patch.crates-io.samp-protocol.path='$protocolPath'"

    cargo package -p modkit-abi --allow-dirty --locked
    if ($LASTEXITCODE -ne 0) {
        throw "modkit-abi packaging failed"
    }

    cargo package -p modkit-sdk --allow-dirty --locked --config $abiPatch
    if ($LASTEXITCODE -ne 0) {
        throw "modkit-sdk packaging failed"
    }

    cargo package -p gta-sa --allow-dirty --locked --config $abiPatch --config $modkitSdkPatch
    if ($LASTEXITCODE -ne 0) {
        throw "gta-sa packaging failed"
    }

    cargo package -p samp-protocol --allow-dirty --locked
    if ($LASTEXITCODE -ne 0) {
        throw "samp-protocol packaging failed"
    }

    cargo package -p samp-client-sdk --allow-dirty --locked --config $abiPatch --config $modkitSdkPatch --config $gtaPatch --config $protocolPatch
    if ($LASTEXITCODE -ne 0) {
        throw "samp-client-sdk packaging failed"
    }

    $archive = Join-Path $metadata.target_directory "package/samp-client-sdk-$($sdk.version).crate"
    $archiveRoot = "samp-client-sdk-$($sdk.version)"
    if (-not (Test-Path $archive)) {
        throw "SDK package archive is missing: $archive"
    }

    $normalizedManifest = tar -xOf $archive "$archiveRoot/Cargo.toml"
    if ($LASTEXITCODE -ne 0) {
        throw "Could not inspect the packaged SDK manifest"
    }
    $expectedVersion = [regex]::Escape("=$($protocol.version)")
    if (($normalizedManifest -join "`n") -notmatch "(?ms)\[dependencies\.samp-protocol\].*?version = `"$expectedVersion`"") {
        throw "The packaged SDK does not pin the synchronized Protocol version"
    }
    $expectedGtaVersion = [regex]::Escape("=$($gta.version)")
    if (($normalizedManifest -join "`n") -notmatch "(?ms)\[dependencies\.gta-sa\].*?version = `"$expectedGtaVersion`"") {
        throw "The packaged SDK does not pin the synchronized GTA version"
    }

    $packagedLock = tar -xOf $archive "$archiveRoot/Cargo.lock"
    if ($LASTEXITCODE -ne 0) {
        throw "Could not inspect the packaged SDK lockfile"
    }
    $protocolPackages = [regex]::Matches(($packagedLock -join "`n"), '(?m)^name = "samp-protocol"$')
    if ($protocolPackages.Count -ne 1) {
        throw "The packaged SDK does not resolve exactly one Protocol crate"
    }
    $gtaPackages = [regex]::Matches(($packagedLock -join "`n"), '(?m)^name = "gta-sa"$')
    if ($gtaPackages.Count -ne 1) {
        throw "The packaged SDK does not resolve exactly one GTA crate"
    }

    Write-Host "Published crate package checks passed."
}
finally {
    Pop-Location
}
