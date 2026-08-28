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
    $protocolDependency = $sdk.dependencies | Where-Object name -eq "samp-protocol"

    if ($sdk.version -ne $protocol.version) {
        throw "Published crate versions are not synchronized"
    }
    if ($protocolDependency.req -ne "=$($protocol.version)") {
        throw "The SDK does not pin the exact Protocol version"
    }
    if (-not $protocolDependency.path) {
        throw "The SDK Protocol dependency lost its workspace path"
    }

    if (Select-String -Quiet -Path "README.md" -Pattern "samp_client_sdk::raknet") {
        throw "README uses the removed SDK Protocol path"
    }
    if (Test-Path "sdk/src/events/rpc/incoming/fixed.rs") {
        throw "SDK production taxonomy still contains incoming/fixed.rs"
    }
    if (-not (Test-Path "sdk/src/events/rpc/incoming/common.rs")) {
        throw "SDK production taxonomy does not contain incoming/common.rs"
    }
    if (Select-String -Quiet -Path "sdk/src/events/core.rs" -Pattern "^pub type Packet<") {
        throw "The migration-only public Packet alias remains"
    }
    if (Select-String -Quiet -Path "crates/samp-protocol/src/rpc/incoming/mod.rs" -Pattern "pub\s+use\s+(common|r1)::\*") {
        throw "Protocol incoming RPC ownership is hidden by a broad re-export"
    }

    $repositoryFiles = git ls-files --cached --others --exclude-standard
    if ($LASTEXITCODE -ne 0) {
        throw "Could not enumerate repository files"
    }
    $textExtensions = ".cpp", ".h", ".md", ".ps1", ".rs", ".toml", ".yaml", ".yml"
    $repositoryTextFiles = $repositoryFiles | Where-Object {
        (Test-Path $_ -PathType Leaf) -and ($textExtensions -contains [IO.Path]::GetExtension($_))
    }
    $contextualAuditAllowlist = @(
        "docs/evidence/p0-architecture-gate.md",
        "docs/evidence/protocol-sdk-boundary-completion.md",
        "docs/structural-split-plan.md",
        "scripts/check-release-hygiene.ps1"
    )
    $currentRepositoryFiles = $repositoryTextFiles | Where-Object {
        $contextualAuditAllowlist -notcontains $_.Replace("\", "/")
    }

    $migrationVocabulary = $currentRepositoryFiles |
        Select-String -Pattern "phase15|on_[a-z_]*protocol_"
    if ($migrationVocabulary) {
        throw "Repository contains migration-history API vocabulary outside its contextual allowlist"
    }
    $migrationTaxonomy = $currentRepositoryFiles |
        Select-String -Pattern "incoming[\\/:]fixed|incoming::fixed|mod fixed;|fixed::\*"
    if ($migrationTaxonomy) {
        throw "Repository contains migration taxonomy outside historical records"
    }
    $publicImplementationTypes = Get-ChildItem "sdk/src" -Recurse -File -Filter "*.rs" |
        Select-String -Pattern "^pub\s+(struct|enum|type|trait)\s+(HostApi|RpcEncoder|PayloadWriter|EncodedPayload|Packet)\b"
    if ($publicImplementationTypes) {
        throw "SDK source exposes a forbidden implementation type"
    }

    $target = "i686-pc-windows-msvc"
    $publicIndex = Join-Path $metadata.target_directory "$target/doc/samp_client_sdk/all.html"
    if (-not (Test-Path $publicIndex)) {
        throw "Public SDK documentation index is missing: $publicIndex"
    }

    $forbiddenPublicItems = "HostApi", "Rpc", "RpcEncoder", "PayloadWriter", "EncodedPayload", "Packet"
    $publicIndexText = Get-Content -Raw $publicIndex
    foreach ($item in $forbiddenPublicItems) {
        if ($publicIndexText -match ">(?:[^<]*::)?$([regex]::Escape($item))</a>") {
            throw "Generated public SDK index exposes $item"
        }
    }

    Write-Host "Release hygiene audit passed."
}
finally {
    Pop-Location
}
