#Requires -Version 5.1
<#
.SYNOPSIS
  Install Mochi unstable from GitHub Releases (Windows).

.DESCRIPTION
  Downloads the latest unstable prerelease (or a specific tag) and installs
  the MSI or NSIS installer silently when possible.

.PARAMETER ReleaseTag
  Optional GitHub release tag. Defaults to latest prerelease.

.PARAMETER Package
  msi, exe, or auto (default: auto prefers MSI).

.ENV
  MOCHI_VERSION, MOCHI_PACKAGE, GITHUB_TOKEN
#>
param(
  [string]$ReleaseTag = $env:MOCHI_VERSION,
  [ValidateSet('auto', 'msi', 'exe')]
  [string]$Package = $(if ($env:MOCHI_PACKAGE) { $env:MOCHI_PACKAGE } else { 'auto' })
)

$ErrorActionPreference = 'Stop'
$Repo = if ($env:MOCHI_GITHUB_REPO) { $env:MOCHI_GITHUB_REPO } else { 'BrainerVirus/mochi' }
$ApiBase = "https://api.github.com/repos/$Repo"

function Get-GitHubJson {
  param([string]$Url)
  $headers = @{ Accept = 'application/vnd.github+json' }
  if ($env:GITHUB_TOKEN) {
    $headers.Authorization = "Bearer $($env:GITHUB_TOKEN)"
  }
  return Invoke-RestMethod -Uri $Url -Headers $headers
}

function Resolve-ReleaseTag {
  if ($ReleaseTag) { return $ReleaseTag }

  $releases = Get-GitHubJson "$ApiBase/releases?per_page=30"
  $pre = $releases | Where-Object { $_.prerelease -and -not $_.draft } | Select-Object -First 1
  if ($pre) { return $pre.tag_name }

  $latest = $releases | Where-Object { -not $_.draft } | Select-Object -First 1
  if ($latest) { return $latest.tag_name }

  throw "No GitHub release found for $Repo. Set MOCHI_VERSION or pass -ReleaseTag."
}

function Select-Asset {
  param(
    [object]$Release,
    [string[]]$Patterns
  )
  foreach ($pattern in $Patterns) {
    $asset = $Release.assets | Where-Object { $_.name -match $pattern } | Select-Object -First 1
    if ($asset) { return $asset }
  }
  return $null
}

$tag = Resolve-ReleaseTag
Write-Host "Using release tag: $tag"
$release = Get-GitHubJson "$ApiBase/releases/tags/$tag"

$pkg = $Package
if ($pkg -eq 'auto') { $pkg = 'msi' }

$asset = $null
if ($pkg -eq 'msi') {
  $asset = Select-Asset -Release $release -Patterns @('\.msi$', 'x64.*\.msi$')
  if (-not $asset) { $pkg = 'exe' }
}
if ($pkg -eq 'exe') {
  $asset = Select-Asset -Release $release -Patterns @('-setup\.exe$', 'x64.*\.exe$')
}

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
  Write-Host "Installed Mochi unstable $tag"
}
finally {
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
