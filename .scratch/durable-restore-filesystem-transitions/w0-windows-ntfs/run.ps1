[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location -LiteralPath $scriptRoot

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($isAdmin) {
    Write-Error 'W0 must run from an ordinary, non-elevated PowerShell session.'
    exit 90
}
if ([string]::IsNullOrWhiteSpace($env:LOCALAPPDATA)) {
    Write-Error 'W0 requires LOCALAPPDATA to select its ordinary-user local NTFS test boundary.'
    exit 91
}

$runId = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ') + '-' + ([guid]::NewGuid().ToString('N').Substring(0, 8))
$runDir = Join-Path $scriptRoot "evidence\$runId"
$testRoot = Join-Path $env:LOCALAPPDATA "RepuestosAutos-W0\$runId"
New-Item -ItemType Directory -Path $runDir -Force | Out-Null
New-Item -ItemType Directory -Path $testRoot -Force | Out-Null

$envFacts = [ordered]@{
    run_id = $runId
    captured_utc = (Get-Date).ToUniversalTime().ToString('o')
    ordinary_non_admin = -not $isAdmin
    os_caption = (Get-CimInstance Win32_OperatingSystem).Caption
    os_version = [Environment]::OSVersion.VersionString
    os_build = [Environment]::OSVersion.Version.Build
    process_architecture = [Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture.ToString()
    os_architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    powershell_version = $PSVersionTable.PSVersion.ToString()
    rustc = (& rustc -V 2>&1 | Out-String).Trim()
    cargo = (& cargo -V 2>&1 | Out-String).Trim()
    rustup_active_toolchain = (& rustup show active-toolchain 2>&1 | Out-String).Trim()
    checkout_path = '<CHECKOUT>'
    logical_test_root = '<TEST_ROOT>'
    test_root_boundary = 'LOCALAPPDATA/RepuestosAutos-W0/<run-id>'
}
$envFacts | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $runDir 'environment.json') -Encoding utf8

$exitCodes = [ordered]@{}
& cargo tree --manifest-path (Join-Path $scriptRoot 'Cargo.toml') *>&1 |
    Tee-Object -FilePath (Join-Path $runDir 'cargo-tree.txt') | Out-Host
$exitCodes.cargo_tree = $LASTEXITCODE

& cargo build --manifest-path (Join-Path $scriptRoot 'Cargo.toml') *>&1 |
    Tee-Object -FilePath (Join-Path $runDir 'build.txt') | Out-Host
$exitCodes.cargo_build = $LASTEXITCODE

$runtimeExit = 125
if ($exitCodes.cargo_build -eq 0) {
    $exe = Join-Path $scriptRoot 'target\debug\w0-windows-ntfs-feasibility.exe'
    $env:W0_ORDINARY_NON_ADMIN = 'true'
    $stdout = Join-Path $runDir 'evidence.jsonl'
    $stderr = Join-Path $runDir 'runtime-stderr.txt'
    $quotedTestRoot = '"' + $testRoot.Replace('"', '\"') + '"'
    $process = Start-Process -FilePath $exe -ArgumentList @('--root', $quotedTestRoot) -RedirectStandardOutput $stdout -RedirectStandardError $stderr -Wait -PassThru
    $runtimeExit = $process.ExitCode
    Get-Content -LiteralPath $stdout | Out-Host
} else {
    'Runtime not started because the Windows MSVC build failed.' |
        Set-Content -LiteralPath (Join-Path $runDir 'runtime-stderr.txt') -Encoding utf8
    '' | Set-Content -LiteralPath (Join-Path $runDir 'evidence.jsonl') -Encoding utf8
}
$exitCodes.runtime = $runtimeExit

$cleanup = [ordered]@{
    logical_test_root = '<TEST_ROOT>'
    status = 'FAIL'
    removed = $false
    hresult = 0
    native_error = 0
    powershell_error_id = $null
}
try {
    Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction Stop
    if (Test-Path -LiteralPath $testRoot) {
        $cleanup.powershell_error_id = 'TestRootStillExistsAfterRemoveItem'
    } else {
        $cleanup.status = 'PASS'
        $cleanup.removed = $true
    }
} catch {
    $cleanup.hresult = [int]$_.Exception.HResult
    $cleanup.native_error = [int]($_.Exception.HResult -band 0xffff)
    $cleanup.powershell_error_id = [string]$_.FullyQualifiedErrorId
}
$cleanup | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $runDir 'cleanup.json') -Encoding utf8
$exitCodes.cleanup = if ($cleanup.status -eq 'PASS') { 0 } else { 1 }

$redactionCandidates = @(
    [pscustomobject]@{ Value = $testRoot; Replacement = '<TEST_ROOT>' },
    [pscustomobject]@{ Value = $env:LOCALAPPDATA; Replacement = '<LOCAL_APP_DATA>' },
    [pscustomobject]@{ Value = $env:APPDATA; Replacement = '<ROAMING_APP_DATA>' },
    [pscustomobject]@{ Value = $env:USERPROFILE; Replacement = '<USER_PROFILE>' },
    [pscustomobject]@{ Value = [Environment]::GetFolderPath('UserProfile'); Replacement = '<USER_PROFILE>' },
    [pscustomobject]@{ Value = $env:HOME; Replacement = '<USER_PROFILE>' },
    [pscustomobject]@{ Value = "$env:HOMEDRIVE$env:HOMEPATH"; Replacement = '<USER_PROFILE>' },
    [pscustomobject]@{ Value = $scriptRoot; Replacement = '<HARNESS_ROOT>' },
    [pscustomobject]@{ Value = (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $scriptRoot))); Replacement = '<CHECKOUT>' },
    [pscustomobject]@{ Value = $identity.Name; Replacement = '<WINDOWS_IDENTITY>' },
    [pscustomobject]@{ Value = $identity.User.Value; Replacement = '<WINDOWS_SID>' },
    [pscustomobject]@{ Value = $env:USERNAME; Replacement = '<WINDOWS_USER>' }
)
$redactions = foreach ($candidate in $redactionCandidates) {
    if (-not [string]::IsNullOrWhiteSpace([string]$candidate.Value)) {
        $raw = [string]$candidate.Value
        foreach ($variant in @($raw, $raw.Replace('\', '/'), $raw.Replace('\', '\\'))) {
            if (-not [string]::IsNullOrWhiteSpace($variant)) {
                [pscustomobject]@{ Value = $variant; Replacement = $candidate.Replacement }
            }
        }
    }
}
$redactions = $redactions | Sort-Object { $_.Value.Length } -Descending

Get-ChildItem -LiteralPath $runDir -File -Recurse | ForEach-Object {
    $content = [IO.File]::ReadAllText($_.FullName)
    foreach ($redaction in $redactions) {
        $content = [regex]::Replace(
            $content,
            [regex]::Escape([string]$redaction.Value),
            [string]$redaction.Replacement,
            [Text.RegularExpressions.RegexOptions]::IgnoreCase
        )
    }
    [IO.File]::WriteAllText($_.FullName, $content, [Text.UTF8Encoding]::new($false))
}

$redactionFailures = 0
Get-ChildItem -LiteralPath $runDir -File -Recurse | ForEach-Object {
    $content = [IO.File]::ReadAllText($_.FullName)
    foreach ($redaction in $redactions) {
        if ($content.IndexOf([string]$redaction.Value, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
            $redactionFailures++
        }
    }
}
$exitCodes.redaction = if ($redactionFailures -eq 0) { 0 } else { 1 }
$exitCodes.overall = if (
    ($exitCodes.cargo_tree -eq 0) -and
    ($exitCodes.cargo_build -eq 0) -and
    ($exitCodes.runtime -eq 0) -and
    ($exitCodes.cleanup -eq 0) -and
    ($exitCodes.redaction -eq 0)
) { 0 } else { 1 }
$exitCodes | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $runDir 'exit-codes.json') -Encoding utf8

Get-ChildItem -LiteralPath $runDir -File -Recurse |
    Where-Object { $_.Name -ne 'checksums.sha256' } |
    Sort-Object FullName |
    ForEach-Object {
        $hash = Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName
        $relative = $_.FullName.Substring($runDir.Length + 1).Replace('\', '/')
        "{0}  {1}" -f $hash.Hash.ToLowerInvariant(), $relative
    } | Set-Content -LiteralPath (Join-Path $runDir 'checksums.sha256') -Encoding ascii

Write-Host "Evidence directory: evidence/$runId"
exit $exitCodes.overall
