#Requires -Version 7.0
<#
.SYNOPSIS
Exports PNG sizes and a multi-resolution Windows ICO from the saved artwork.
.DESCRIPTION
Requires Windows PowerShell 7 and System.Drawing. Runs offline from any working
directory. Reads assets/source and overwrites only the generated files in
assets/icons and assets/branding. Does not regenerate or edit the source artwork.
Throws on missing/unhydrated sources or invalid dimensions (nonzero script exit).
.EXAMPLE
pwsh -NoLogo -NoProfile -NonInteractive -File scripts/export-assets.ps1
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'Asset export requires PowerShell 7 on Windows.' }
Add-Type -AssemblyName System.Drawing

$assetRoot = Join-Path (Split-Path -Parent $PSScriptRoot) 'assets'
$iconRoot = Join-Path $assetRoot 'icons'
$bannerRoot = Join-Path $assetRoot 'branding'
$icon = $null
$banner = $null

function Export-Png {
    param([System.Drawing.Image]$Source, [int]$Width, [int]$Height, [string]$Path)
    $bitmap = [System.Drawing.Bitmap]::new($Width, $Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = $null
    $attributes = $null
    try {
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $attributes = [System.Drawing.Imaging.ImageAttributes]::new()
        $attributes.SetWrapMode([System.Drawing.Drawing2D.WrapMode]::TileFlipXY)
        $graphics.DrawImage($Source, [System.Drawing.Rectangle]::new(0, 0, $Width, $Height),
            0, 0, $Source.Width, $Source.Height, [System.Drawing.GraphicsUnit]::Pixel, $attributes)
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        if ($attributes) { $attributes.Dispose() }
        if ($graphics) { $graphics.Dispose() }
        $bitmap.Dispose()
    }
}

try {
    # Load and validate both sources before creating output files.
    $icon = [System.Drawing.Bitmap]::new((Join-Path $assetRoot 'source/icon.png'))
    $banner = [System.Drawing.Bitmap]::new((Join-Path $assetRoot 'source/banner.png'))
    if ($icon.Width -ne $icon.Height -or $icon.Width -lt 1024 -or $icon.GetPixel(0, 0).A -ne 0) {
        throw 'Icon source must be square, at least 1024px, with transparent corners.'
    }
    if ($banner.Width -ne 2 * $banner.Height -or $banner.Width -lt 1280) {
        throw 'Banner source must be at least 1280px wide with a 2:1 aspect ratio.'
    }
    New-Item -ItemType Directory -Path $iconRoot, $bannerRoot -Force | Out-Null
    $sizes = @(16, 24, 32, 48, 64, 128, 256, 512, 1024)
    foreach ($size in $sizes) {
        Export-Png $icon $size $size (Join-Path $iconRoot "xresconv-cli-$size.png")
    }
    Export-Png $banner 1280 640 (Join-Path $bannerRoot 'repository-banner.png')

    # Modern Windows ICO: ICONDIR + ICONDIRENTRY records + lossless PNG frames.
    $icoSizes = @($sizes | Where-Object { $_ -le 256 })
    $frames = @($icoSizes | ForEach-Object {
        , [System.IO.File]::ReadAllBytes((Join-Path $iconRoot "xresconv-cli-$_.png"))
    })
    $stream = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($stream)
    try {
        $writer.Write([uint16]0)
        $writer.Write([uint16]1)
        $writer.Write([uint16]$frames.Count)
        $offset = 6 + 16 * $frames.Count
        for ($index = 0; $index -lt $frames.Count; $index++) {
            $dimension = if ($icoSizes[$index] -eq 256) { 0 } else { $icoSizes[$index] }
            $writer.Write([byte]$dimension)
            $writer.Write([byte]$dimension)
            $writer.Write([byte]0)
            $writer.Write([byte]0)
            $writer.Write([uint16]1)
            $writer.Write([uint16]32)
            $writer.Write([uint32]$frames[$index].Length)
            $writer.Write([uint32]$offset)
            $offset += $frames[$index].Length
        }
        foreach ($frame in $frames) { $writer.Write([byte[]]$frame) }
        $writer.Flush()
        [System.IO.File]::WriteAllBytes((Join-Path $iconRoot 'xresconv-cli.ico'), $stream.ToArray())
    } finally {
        $writer.Dispose()
        $stream.Dispose()
    }
    Write-Output "Exported $($sizes.Count) icon PNGs, a 7-frame ICO and a 1280x640 banner."
} finally {
    if ($icon) { $icon.Dispose() }
    if ($banner) { $banner.Dispose() }
}
