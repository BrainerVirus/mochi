#Requires -Version 5.1
<#
.SYNOPSIS
  Install Mochi from GitHub Releases (Windows).

.DESCRIPTION
  Downloads the latest stable release by default, or the unstable prerelease
  when -Unstable is set. Pass an explicit tag or set MOCHI_VERSION to pin.

.PARAMETER ReleaseTag
  Optional GitHub release tag. Overrides automatic channel resolution.

.PARAMETER Unstable
  Install the unstable channel (latest prerelease) instead of stable.

.PARAMETER Package
  msi, exe, or auto (default: auto prefers MSI).

.ENV
  MOCHI_VERSION, MOCHI_UNSTABLE=1, MOCHI_PACKAGE, GITHUB_TOKEN
#>
param(
  [Alias('i')]
  [switch]$Unstable,
  [string]$ReleaseTag = $env:MOCHI_VERSION,
  [ValidateSet('auto', 'msi', 'exe')]
  [string]$Package = $(if ($env:MOCHI_PACKAGE) { $env:MOCHI_PACKAGE } else { 'auto' })
)

$ErrorActionPreference = 'Stop'
$Repo = if ($env:MOCHI_GITHUB_REPO) { $env:MOCHI_GITHUB_REPO } else { 'BrainerVirus/mochi' }
$ApiBase = "https://api.github.com/repos/$Repo"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $ScriptDir 'lib/windows-install.ps1')

if (-not $Unstable -and $env:MOCHI_UNSTABLE -eq '1') {
  $Unstable = $true
}

function Test-WebView2Installed {
  $regPaths = @(
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
    'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
  )
  foreach ($path in $regPaths) {
    if (Test-Path $path) {
      return $true
    }
  }
  return $false
}

function Ensure-MochiRuntimeDependencies {
  if ($env:MOCHI_SKIP_DEPS -eq '1') {
    Write-Host 'Skipping Windows runtime dependencies (MOCHI_SKIP_DEPS=1)'
    return
  }

  if (Test-WebView2Installed) {
    Write-Host 'ok (already installed): Microsoft Edge WebView2 Runtime'
    return
  }

  Write-Host 'Installing Microsoft Edge WebView2 Runtime...'
  if (Get-Command winget -ErrorAction SilentlyContinue) {
    $winget = Start-Process -FilePath winget -ArgumentList @(
      'install', '--id', 'Microsoft.EdgeWebView2Runtime', '-e',
      '--accept-package-agreements', '--accept-source-agreements'
    ) -Wait -PassThru -NoNewWindow
    if ($winget.ExitCode -eq 0 -or (Test-WebView2Installed)) {
      Write-Host 'WebView2 installed via winget'
      return
    }
    Write-Warning "winget WebView2 install exited $($winget.ExitCode); trying bootstrapper"
  }

  $bootstrapper = Join-Path $env:TEMP 'MicrosoftEdgeWebview2Setup.exe'
  $bootstrapUrl = 'https://go.microsoft.com/fwlink/p/?LinkId=2124703'
  Invoke-WebRequest -Uri $bootstrapUrl -OutFile $bootstrapper
  $proc = Start-Process -FilePath $bootstrapper -ArgumentList @('/silent', '/install') -Wait -PassThru
  Remove-Item -Force $bootstrapper -ErrorAction SilentlyContinue
  if ($proc.ExitCode -ne 0 -and -not (Test-WebView2Installed)) {
    throw "WebView2 bootstrapper exited with code $($proc.ExitCode)"
  }
  Write-Host 'WebView2 runtime installed'
}

$channel = if ($Unstable) { 'unstable' } else { 'stable' }
$tag = Resolve-MochiReleaseTag -ReleaseTag $ReleaseTag -Unstable:$Unstable -ApiBase $ApiBase
Write-Host "Installing Mochi ($channel channel, release $tag)"
Ensure-MochiRuntimeDependencies
$release = Get-MochiGitHubJson "$ApiBase/releases/tags/$tag"

$resolved = Resolve-MochiWindowsAsset -Release $release -Package $Package
$asset = $resolved.Asset

if (-not $asset) {
  throw "No Windows installer asset found in release $tag"
}

$tmp = Join-Path $env:TEMP ("mochi-install-" + [guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tmp | Out-Null
$installer = Join-Path $tmp $asset.name

Write-Host "Downloading $($asset.browser_download_url)"
$headers = @{}
if ($env:GITHUB_TOKEN) {
  $headers.Authorization = "Bearer $($env:GITHUB_TOKEN)"
}
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $installer -Headers $headers

try {
  if ($installer -match '\.msi$') {
    Write-Host 'Running MSI installer (per-machine)...'
    $args = @('/i', $installer, '/qn', '/norestart')
    $proc = Start-Process -FilePath 'msiexec.exe' -ArgumentList $args -Wait -PassThru
    if ($proc.ExitCode -ne 0) {
      throw "msiexec exited with code $($proc.ExitCode)"
    }
  }
  else {
    Write-Host 'Running NSIS installer silently...'
    $proc = Start-Process -FilePath $installer -ArgumentList @('/S') -Wait -PassThru
    if ($proc.ExitCode -ne 0) {
      throw "Installer exited with code $($proc.ExitCode)"
    }
  }
  Write-Host "Installed Mochi $tag ($channel)"
}
finally {
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
