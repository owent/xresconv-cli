<#
.SYNOPSIS
Validate a release tag, package a Cargo binary, or verify a release asset directory.
.DESCRIPTION
Requires PowerShell 7 and Cargo. Writes only the supplied output directory in Package
mode. No network, Git mutations, or publishing. Throws on validation or tool failure.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('Tag', 'Package', 'Verify')][string]$Mode,
    [string]$Tag = $env:GITHUB_REF_NAME,
    [string]$Target,
    [string]$Binary,
    [string]$OutputDirectory = 'dist'
)
$ErrorActionPreference = 'Stop'
$metadata = & cargo metadata --no-deps --format-version 1 --locked
if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' }
$version = ($metadata | ConvertFrom-Json).packages[0].version
if ($Mode -eq 'Tag') {
    if ($Tag -cne $version -and $Tag -cne "v$version") { throw "Tag '$Tag' must match Cargo version '$version'" }
    Write-Output $version
    return
}
$requiredTargets = @(
    'x86_64-unknown-linux-gnu', 'x86_64-unknown-linux-musl',
    'aarch64-unknown-linux-gnu', 'aarch64-unknown-linux-musl',
    'x86_64-apple-darwin', 'aarch64-apple-darwin',
    'x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'
)
if ($Mode -eq 'Package') {
    if ($Target -notmatch '^[a-zA-Z0-9_]+(-[a-zA-Z0-9_]+)+$') { throw 'Invalid target triple' }
    $binName = if ($Target -like '*-windows-*') { 'xresconv-cli.exe' } else { 'xresconv-cli' }
    if (-not $Binary) { $Binary = "target/$Target/release/$binName" }
    if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) { throw "Missing binary: $Binary" }
    New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
    # Unique task-owned staging, safe under concurrent packaging in one checkout.
    $stage = Join-Path ([IO.Path]::GetFullPath($OutputDirectory)) ('.stage-' + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stage | Out-Null
    try {
        Copy-Item -LiteralPath $Binary -Destination (Join-Path $stage $binName)
        foreach ($name in @('README.md', 'LICENSE')) { Copy-Item -LiteralPath $name -Destination (Join-Path $stage $name) }
        $ext = if ($Target -like '*-windows-*') { 'zip' } else { 'tar.gz' }
        $file = Join-Path $OutputDirectory "xresconv-cli-$version-$Target.$ext"
        if (Test-Path -LiteralPath $file) { throw "Refusing to overwrite existing package: $file" }
        if ($ext -eq 'zip') {
            Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $file
        } else {
            & tar -czf $file -C $stage $binName README.md LICENSE
            if ($LASTEXITCODE -ne 0) { throw 'tar failed' }
        }
        $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
        [IO.File]::WriteAllText("$file.sha256", "$hash  $([IO.Path]::GetFileName($file))`n", [Text.Encoding]::ASCII)
        Write-Output $file
    } finally {
        # Stage is a unique, directly created child of the resolved output root.
        $root = [IO.Path]::GetFullPath($OutputDirectory)
        # On Unix, .stage-* is hidden; Get-Item requires -Force to see it.
        if ([IO.Path]::GetDirectoryName($stage) -ne $root -or (Get-Item -LiteralPath $stage -Force).LinkType) { throw 'Unsafe staging cleanup path' }
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
    return
}
foreach ($targetName in $requiredTargets) {
    $ext = if ($targetName -like '*-windows-*') { 'zip' } else { 'tar.gz' }
    $name = "xresconv-cli-$version-$targetName.$ext"
    if (-not (Test-Path -LiteralPath (Join-Path $OutputDirectory $name) -PathType Leaf)) { throw "Missing required asset: $name" }
}
$archives = @(Get-ChildItem -LiteralPath $OutputDirectory -File | Where-Object { $_.Name -match '\.(zip|tar\.gz)$' })
foreach ($file in $archives) {
    if (-not $file.Name.StartsWith("xresconv-cli-$version-", [StringComparison]::Ordinal)) { throw "Wrong asset version: $($file.Name)" }
    $checksumPath = $file.FullName + '.sha256'
    $parts = (Get-Content -LiteralPath $checksumPath -Raw).Trim() -split '\s+', 2
    $actual = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($parts.Count -ne 2 -or $parts[0] -cne $actual -or $parts[1] -cne $file.Name) { throw "Checksum mismatch: $($file.Name)" }
}
Write-Output "Verified $($archives.Count) archives with checksums; all required targets present."
