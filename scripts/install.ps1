$ErrorActionPreference = "Stop"

function Get-ConjureTarget {
  switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { return "x86_64-pc-windows-msvc" }
    default { throw "Unsupported Windows architecture: $env:PROCESSOR_ARCHITECTURE" }
  }
}

function Get-ReleaseBaseUrl {
  param(
    [string] $Repo,
    [string] $Version
  )

  if ($Version -eq "latest") {
    return "https://github.com/$Repo/releases/latest/download"
  }

  return "https://github.com/$Repo/releases/download/$Version"
}

function Get-FrontendDataDir {
  if ($env:CONJURE_DATA_DIR) {
    return $env:CONJURE_DATA_DIR
  }

  return Join-Path $env:APPDATA "Conjure\frontend"
}

function Add-BinDirToUserPath {
  param([string] $BinDir)

  $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
  $paths = @()
  if ($currentPath) {
    $paths = $currentPath -split ";"
  }

  if ($paths -contains $BinDir) {
    return
  }

  $nextPath = if ($currentPath) { "$currentPath;$BinDir" } else { $BinDir }
  [Environment]::SetEnvironmentVariable("Path", $nextPath, "User")
  $env:Path = "$env:Path;$BinDir"
}

$repo = if ($env:CONJURE_REPO) { $env:CONJURE_REPO } else { "bariskau/conjure" }
$version = if ($env:CONJURE_VERSION) { $env:CONJURE_VERSION } else { "latest" }
$binDir = if ($env:CONJURE_BIN_DIR) { $env:CONJURE_BIN_DIR } else { Join-Path $env:USERPROFILE ".conjure\bin" }
$dataDir = Get-FrontendDataDir
$target = Get-ConjureTarget
$asset = "conjure-$target.tar.gz"
$baseUrl = Get-ReleaseBaseUrl -Repo $repo -Version $version
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "conjure-install-$([System.Guid]::NewGuid())"
$archive = Join-Path $tempDir $asset
$packageDir = Join-Path $tempDir "conjure-$target"

New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

try {
  Invoke-WebRequest -Uri "$baseUrl/$asset" -OutFile $archive
  tar -xzf $archive -C $tempDir

  New-Item -ItemType Directory -Force -Path $binDir | Out-Null
  Copy-Item -Force -Path (Join-Path $packageDir "bin\conjure.exe") -Destination (Join-Path $binDir "conjure.exe")

  if (Test-Path $dataDir) {
    Remove-Item -Recurse -Force $dataDir
  }

  New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
  Copy-Item -Recurse -Force -Path (Join-Path $packageDir "frontend\*") -Destination $dataDir

  Add-BinDirToUserPath -BinDir $binDir

  Write-Host "Installed Conjure to $(Join-Path $binDir "conjure.exe")"
  Write-Host "Installed UI assets to $dataDir"
  Write-Host "Run: conjure"
}
finally {
  Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
}
