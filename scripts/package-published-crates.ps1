$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path $PSScriptRoot -Parent
Push-Location $repositoryRoot

try {
    $metadata = cargo metadata --format-version 1 --locked --no-deps | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed"
    }

    $sdk = $metadata.packages | Where-Object name -eq "samp-client-sdk"
    $protocol = $metadata.packages | Where-Object name -eq "samp-protocol"
    $protocolPath = (Split-Path $protocol.manifest_path -Parent).Replace("\", "/")
    $patch = "patch.crates-io.samp-protocol.path='$protocolPath'"

    cargo package -p samp-protocol --allow-dirty --locked
    if ($LASTEXITCODE -ne 0) {
        throw "samp-protocol packaging failed"
    }

    cargo package -p samp-client-sdk --allow-dirty --locked --config $patch
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

    $packagedLock = tar -xOf $archive "$archiveRoot/Cargo.lock"
    if ($LASTEXITCODE -ne 0) {
        throw "Could not inspect the packaged SDK lockfile"
    }
    $protocolPackages = [regex]::Matches(($packagedLock -join "`n"), '(?m)^name = "samp-protocol"$')
    if ($protocolPackages.Count -ne 1) {
        throw "The packaged SDK does not resolve exactly one Protocol crate"
    }

    Write-Host "Published crate package checks passed."
}
finally {
    Pop-Location
}
