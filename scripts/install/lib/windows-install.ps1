# Shared helpers for Mochi Windows install scripts.
# Dot-sourced from install-windows.ps1 and exercised by install-windows.test.mjs.

function Get-MochiGitHubJson {
  param([string]$Url)

  if ($env:MOCHI_TEST_RELEASES_JSON -and $Url -match '/releases(\?|$|/tags/)') {
    if ($Url -match '/releases/tags/([^/?]+)$') {
      $tag = $Matches[1]
      $releases = Get-Content -Raw -Path $env:MOCHI_TEST_RELEASES_JSON | ConvertFrom-Json
      $match = $releases | Where-Object { $_.tag_name -eq $tag } | Select-Object -First 1
      if (-not $match) {
        throw "No release fixture for tag $tag"
      }
      return $match
    }

    return Get-Content -Raw -Path $env:MOCHI_TEST_RELEASES_JSON | ConvertFrom-Json
  }

  $headers = @{ Accept = 'application/vnd.github+json' }
  if ($env:GITHUB_TOKEN) {
    $headers.Authorization = "Bearer $($env:GITHUB_TOKEN)"
  }
  return Invoke-RestMethod -Uri $Url -Headers $headers
}

function Resolve-MochiReleaseTag {
  param(
    [string]$ReleaseTag,
    [string]$ApiBase
  )

  if ($ReleaseTag) { return $ReleaseTag }

  $releases = Get-MochiGitHubJson "$ApiBase/releases?per_page=30"

  $stable = $releases | Where-Object { -not $_.prerelease -and -not $_.draft } | Select-Object -First 1
  if ($stable) { return $stable.tag_name }

  throw "No stable GitHub release found for $ApiBase. Set MOCHI_VERSION."
}

function Select-MochiAsset {
  param(
    [object]$Release,
    [string[]]$Patterns
  )

  foreach ($pattern in $Patterns) {
    $asset = $Release.assets |
      Where-Object { $_.name -match $pattern } |
      Sort-Object { [datetime]$_.updated_at } -Descending |
      Select-Object -First 1
    if ($asset) { return $asset }
  }

  return $null
}

function Get-MochiWindowsPackageKind {
  param([string]$Package)

  if ($Package -eq 'auto') { return 'msi' }
  return $Package
}

function Resolve-MochiWindowsAsset {
  param(
    [object]$Release,
    [string]$Package
  )

  $pkg = Get-MochiWindowsPackageKind -Package $Package
  $asset = $null

  if ($pkg -eq 'msi') {
    $asset = Select-MochiAsset -Release $Release -Patterns @('\.msi$', 'x64.*\.msi$')
    if (-not $asset) { $pkg = 'exe' }
  }

  if ($pkg -eq 'exe') {
    $asset = Select-MochiAsset -Release $Release -Patterns @('-setup\.exe$', 'x64.*\.exe$')
  }

  return [PSCustomObject]@{
    PackageKind = $pkg
    Asset       = $asset
  }
}
