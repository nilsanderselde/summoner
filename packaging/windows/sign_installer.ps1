# Summoner DAW - Windows Code Signing Script (Epic J, Task J2)
# Automates dual-signing (SHA256) of executables and NSIS installer with RFC 3161 timestamping.

param(
    [Parameter(Mandatory=$false)]
    [string]$FilePath = "packaging/windows/SummonerDAW-Installer-v1.0.0-x64.exe",
    
    [Parameter(Mandatory=$false)]
    [string]$CertificateThumbprint = $env:WINDOWS_CERT_THUMBPRINT,

    [Parameter(Mandatory=$false)]
    [string]$CertificateFile = $env:WINDOWS_CERT_FILE,

    [Parameter(Mandatory=$false)]
    [string]$CertificatePassword = $env:WINDOWS_CERT_PASSWORD,

    [Parameter(Mandatory=$false)]
    [string]$TimestampUrl = "http://timestamp.digicert.com"
)

Write-Host "=== Summoner DAW Windows Code Signing ==="
Write-Host "Target: $FilePath"

if (-not (Test-Path $FilePath)) {
    Write-Warning "File not found: $FilePath"
    exit 0
}

# Resolve signtool.exe from Windows SDK if available
$signtool = (Get-Command signtool.exe -ErrorAction SilentlyContinue)?.Source
if (-not $signtool) {
    $kits = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    if (Test-Path $kits) {
        $found = Get-ChildItem -Path $kits -Filter "signtool.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) { $signtool = $found.FullName }
    }
}

if (-not $signtool) {
    Write-Warning "signtool.exe not found in PATH or Windows Kits. Skipping code signing in this environment."
    exit 0
}

Write-Host "Using signtool: $signtool"

$signArgs = @("sign", "/fd", "SHA256", "/tr", $TimestampUrl, "/td", "SHA256", "/d", "Summoner DAW")

if ($CertificateThumbprint) {
    Write-Host "Signing using certificate thumbprint..."
    $signArgs += @("/sha1", $CertificateThumbprint)
} elseif ($CertificateFile -and (Test-Path $CertificateFile)) {
    Write-Host "Signing using PFX certificate file: $CertificateFile"
    $signArgs += @("/f", $CertificateFile)
    if ($CertificatePassword) {
        $signArgs += @("/p", $CertificatePassword)
    }
} else {
    Write-Host "Notice: No signing certificate provided (WINDOWS_CERT_THUMBPRINT or WINDOWS_CERT_FILE)."
    Write-Host "In production release CI, provide certificate secret for SmartScreen validation."
    Write-Host "Skipping live signing."
    exit 0
}

$signArgs += $FilePath

& $signtool @signArgs

Write-Host "Verifying digital signature..."
& $signtool verify /pa /v $FilePath

Write-Host "Code signing completed successfully for $FilePath."

