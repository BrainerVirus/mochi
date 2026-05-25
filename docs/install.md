# Installing Mochi (Unstable)

Install the latest **unstable** build from [GitHub Releases](https://github.com/BrainerVirus/mochi/releases). Scripts default to the newest **prerelease**; pass an explicit tag or set `MOCHI_VERSION` to pin a version.

## One-liners

### macOS (DMG → `/Applications`)

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos.sh | bash
```

Install to `~/Applications` instead:

```bash
MOCHI_INSTALL_DIR="$HOME/Applications" curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos.sh | bash
```

### macOS (Homebrew cask)

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos-brew.sh | bash
```

Requires [Homebrew](https://brew.sh/). The script generates a temporary cask from the latest unstable release (correct URL + sha256).

### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash
```

Package selection:

| Env | Values | Default |
| --- | --- | --- |
| `MOCHI_PACKAGE` | `appimage`, `deb`, `rpm`, `auto` | `auto` (deb on Debian/Ubuntu, rpm on Fedora/RHEL, else AppImage) |

AppImage installs to `~/.local/bin/mochi`.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-windows.ps1 | iex
```

Or download and run:

```powershell
.\scripts\install\install-windows.ps1
```

Prefer MSI (default); set `$env:MOCHI_PACKAGE = "exe"` for the NSIS installer.

## Pin a release

```bash
MOCHI_VERSION=unstable-abc1234 ./scripts/install/install-macos.sh
```

```powershell
$env:MOCHI_VERSION = "unstable-abc1234"
.\scripts\install\install-windows.ps1
```

## Requirements

- **macOS / Linux scripts:** `curl`, `jq`
- **macOS:** `hdiutil`, `ditto`
- **Linux `.deb` / `.rpm`:** `sudo` and the matching package manager
- **Windows:** PowerShell 5.1+
- **Optional:** `GITHUB_TOKEN` for higher GitHub API rate limits

## Unstable release channel

Every merge to `main` publishes a GitHub **prerelease** tagged `unstable` (rolling). See [docs/releasing.md](releasing.md).

## App icon regeneration

Source: `assets/icon/mochi-app-icon.svg` (MochiChibi on full-bleed Matcha background).

```bash
./scripts/generate-icons.sh
```

See [assets/icon/README.md](../assets/icon/README.md).
