#Requires -Version 7.0

<#
.SYNOPSIS
Renders documentation screenshots from actual Rust CLI preview output.
.DESCRIPTION
Requires PowerShell 7 on Windows, System.Drawing and a locally built CLI.
Runs offline from any working directory. Copies doc/examples/convert.xml to an
isolated directory under target/doc-screenshots and creates an empty JAR only for
--test's existence check. Never starts Java or performs a conversion.
Overwrites doc/snapshoot-1.png and doc/snapshoot-2.png by default. Raw stdout and
stderr remain in target/doc-screenshots; temporary XML/JAR files are removed.
Throws on missing inputs, a command exceeding 10 seconds or a nonzero exit.
.PARAMETER BinaryPath
CLI executable; defaults to the repository's target/debug/xresconv-cli.exe.
.PARAMETER OutputDirectory
Screenshot destination; defaults to the repository's doc directory.
.EXAMPLE
cargo build --locked
pwsh -NoLogo -NoProfile -NonInteractive -File scripts/export-doc-screenshots.ps1
#>
[CmdletBinding()]
param([string]$BinaryPath, [string]$OutputDirectory)

$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'Screenshot export requires PowerShell 7 on Windows.' }
Add-Type -AssemblyName System.Drawing
$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $BinaryPath) { $BinaryPath = Join-Path $repoRoot 'target/debug/xresconv-cli.exe' }
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $repoRoot 'doc' }
$BinaryPath = (Resolve-Path -LiteralPath $BinaryPath -ErrorAction Stop).Path
$examplePath = Join-Path $repoRoot 'doc/examples/convert.xml'
if (-not (Test-Path -LiteralPath $examplePath -PathType Leaf)) { throw 'Documentation XML example is missing.' }
$logRoot = Join-Path $repoRoot 'target/doc-screenshots'
$previewRoot = Join-Path $logRoot ('preview-' + [guid]::NewGuid().ToString('N'))

function Invoke-PreviewCli {
    param([string[]]$CliArguments, [string]$LogName)
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new($BinaryPath)
    $startInfo.WorkingDirectory = $previewRoot
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.StandardOutputEncoding = [System.Text.Encoding]::UTF8
    $startInfo.StandardErrorEncoding = [System.Text.Encoding]::UTF8
    $startInfo.Environment['CPRINTF_MODE'] = 'none'
    $null = $startInfo.Environment.Remove('JAVA_HOME')
    foreach ($argument in $CliArguments) { $startInfo.ArgumentList.Add($argument) }
    $process = [System.Diagnostics.Process]::new()
    try {
        $process.StartInfo = $startInfo
        $null = $process.Start()
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $timedOut = -not $process.WaitForExit(10000)
        if ($timedOut) {
            $process.Kill($true)
            $process.WaitForExit()
        }
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        [System.IO.File]::WriteAllText((Join-Path $logRoot "$LogName.stdout.txt"), $stdout)
        [System.IO.File]::WriteAllText((Join-Path $logRoot "$LogName.stderr.txt"), $stderr)
        if ($timedOut) { throw "CLI command timed out: $LogName; see $logRoot" }
        if ($process.ExitCode -ne 0 -or $stderr.Length -ne 0) {
            throw "CLI command failed: $LogName (exit $($process.ExitCode)); see $logRoot"
        }
        # Only the absolute working directory is abbreviated in the rendered view.
        return ($stdout -replace '(?m)^\[NOTICE\] start to run conv cmds on dir: [^\r\n]+',
            '[NOTICE] start to run conv cmds on dir: <demo directory>').TrimEnd()
    } finally {
        $process.Dispose()
    }
}

function Export-Transcript {
    param([string]$Title, [string]$Transcript, [string]$Path)
    $columns = 104
    $rows = [System.Collections.Generic.List[object]]::new()
    foreach ($line in ($Transcript -split '\r?\n')) {
        $color = if ($line.StartsWith('$ ')) { '#42C9D8' }
            elseif ($line.StartsWith('[NOTICE]')) { '#EFCB75' }
            elseif ($line.StartsWith('[INFO] all jobs')) { '#D7A8FF' }
            else { '#F2F5F7' }
        $displayLine = $line.Replace("`t", '    ')
        if ($displayLine.Length -eq 0) { $rows.Add(@{ Text = ''; Color = $color; X = 24 }) }
        for ($offset = 0; $offset -lt $displayLine.Length;) {
            $length = [Math]::Min($columns, $displayLine.Length - $offset)
            if ($offset + $length -lt $displayLine.Length) {
                $space = $displayLine.LastIndexOf(' ', $offset + $length - 1, $length)
                if ($space -gt $offset) { $length = $space - $offset + 1 }
            }
            $rows.Add(@{ Text = $displayLine.Substring($offset, $length); Color = $color; X = $(if ($offset -eq 0) { 24 } else { 67 }) })
            $offset += $length
        }
    }
    $font = [System.Drawing.Font]::new('Consolas', 18, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
    $lineHeight = 26
    $bitmap = [System.Drawing.Bitmap]::new(1220, (112 + $rows.Count * $lineHeight))
    $graphics = $null
    $format = [System.Drawing.StringFormat]::GenericTypographic
    try {
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.Clear([System.Drawing.ColorTranslator]::FromHtml('#101D2F'))
        $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
        $headerBrush = [System.Drawing.SolidBrush]::new([System.Drawing.ColorTranslator]::FromHtml('#1C3048'))
        try { $graphics.FillRectangle($headerBrush, 0, 0, $bitmap.Width, 48) } finally { $headerBrush.Dispose() }
        $textBrush = [System.Drawing.SolidBrush]::new([System.Drawing.ColorTranslator]::FromHtml('#A5B6C7'))
        try {
            $graphics.DrawString($Title, $font, $textBrush, [single]24, [single]14, $format)
            $graphics.DrawString('Preview only - Java is not started.', $font, $textBrush,
                [single]24, [single]($bitmap.Height - 32), $format)
        } finally { $textBrush.Dispose() }
        $rowY = 68
        foreach ($row in $rows) {
            $brush = [System.Drawing.SolidBrush]::new([System.Drawing.ColorTranslator]::FromHtml($row.Color))
            try { $graphics.DrawString($row.Text, $font, $brush, [single]$row.X, [single]$rowY, $format) }
            finally { $brush.Dispose() }
            $rowY += $lineHeight
        }
        $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        if ($graphics) { $graphics.Dispose() }
        $format.Dispose()
        $font.Dispose()
        $bitmap.Dispose()
    }
}

try {
    $null = New-Item -ItemType Directory -Path $previewRoot -Force -ErrorAction Stop
    Copy-Item -LiteralPath $examplePath -Destination (Join-Path $previewRoot 'convert.xml') -ErrorAction Stop
    [System.IO.File]::WriteAllBytes((Join-Path $previewRoot 'xresloader.jar'), [byte[]]::new(0))
    $version = Invoke-PreviewCli -CliArguments @('--version') -LogName 'version'
    $all = Invoke-PreviewCli -CliArguments @('--test', '-p', '1', 'convert.xml') -LogName 'all'
    $filtered = Invoke-PreviewCli -CliArguments @('--test', '-p', '1', '-s', 'scheme_upgrade', '-a', 'release-demo',
        '-j', 'Xmx512m', 'convert.xml', '--', '--pretty', '2') -LogName 'filtered'
    $allTranscript = '$ xresconv-cli --version' + "`n$version`n`n" + '$ xresconv-cli --test -p 1 convert.xml' + "`n$all"
    $filteredTranscript = '$ xresconv-cli --test -p 1 -s scheme_upgrade -a release-demo -j Xmx512m convert.xml -- --pretty 2' + "`n$filtered"
    $null = New-Item -ItemType Directory -Path $OutputDirectory -Force -ErrorAction Stop
    Export-Transcript "xresconv-cli $version / all schemes / command preview" $allTranscript (Join-Path $OutputDirectory 'snapshoot-1.png')
    Export-Transcript "xresconv-cli $version / scheme_upgrade / command preview" $filteredTranscript (Join-Path $OutputDirectory 'snapshoot-2.png')
    Write-Output "Exported two preview screenshots to $OutputDirectory; raw output: $logRoot"
} finally {
    # Remove only the two files and the unique directory created by this invocation.
    foreach ($name in @('convert.xml', 'xresloader.jar')) {
        $temporaryPath = Join-Path $previewRoot $name
        if (Test-Path -LiteralPath $temporaryPath) { Remove-Item -LiteralPath $temporaryPath -ErrorAction Stop }
    }
    if (Test-Path -LiteralPath $previewRoot) { Remove-Item -LiteralPath $previewRoot -ErrorAction Stop }
}
