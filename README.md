# ccvm

Node-free version manager for Claude Code and OpenAI Codex, backed by the npm registry.

ccvm downloads platform-specific native packages directly from npm-compatible registries. For modern Claude Code and Codex versions, Node.js is not required.

The default registry is `https://registry.npmmirror.com` for faster downloads in China. You can switch to the official npm registry at any time.

## Quick Start

### Linux / macOS one-line install

```bash
curl -fsSL https://raw.githubusercontent.com/openbioinfo/ccvm/master/install.sh | sh
```

Install a specific version:

```bash
VERSION=0.2.0 curl -fsSL https://raw.githubusercontent.com/openbioinfo/ccvm/master/install.sh | sh
```

Restart your terminal after installation.

### Windows one-line install

```powershell
irm https://raw.githubusercontent.com/openbioinfo/ccvm/master/install.ps1 | iex
```

Install a specific ccvm release:

```powershell
irm https://raw.githubusercontent.com/openbioinfo/ccvm/master/install.ps1 | iex -args "-Version 0.2.0"
```

Restart your terminal after installation.

### Build from source

```bash
git clone https://github.com/openbioinfo/ccvm.git
cd ccvm
cargo build --release
cargo install --path ./
```

### Setup

```bash
ccvm setup
```

`ccvm setup` creates `~/.ccvm/`, installs the `claude` and `codex` shims into `~/.ccvm/bin/`, and prints PATH instructions when automatic PATH setup is not possible.

## Claude Code

```bash
ccvm install latest       # install and switch to the latest Claude Code version
ccvm install 2.1.126      # install a specific version
ccvm use 2.1.126          # switch to that version
ccvm current              # show the active Claude Code version
claude --version          # shim resolves to the active version
```

Project-local pin:

```bash
ccvm pin 2.1.126          # writes .ccvmrc
```

The `claude` shim checks `.ccvmrc` from the current directory up to the filesystem root, then falls back to `~/.ccvm/current`.

## OpenAI Codex

```bash
ccvm codex install latest  # install and switch to the latest Codex version
ccvm codex install 0.134.0 # install a specific Codex version
ccvm codex use 0.134       # switch to that version with fuzzy matching
ccvm codex current         # show the active Codex version
codex --version            # shim resolves to the active version
```

Project-local pin:

```bash
ccvm codex pin 0.134.0     # writes .codexvmrc
```

The `codex` shim checks `.codexvmrc` from the current directory up to the filesystem root, then falls back to `~/.ccvm/codex/current`.

## Commands

| Command | Description |
|---------|-------------|
| `ccvm setup` | Initialize directories, install shims, and configure PATH |
| `ccvm install <version>` | Install Claude Code and switch to it (`latest` or a version like `2.1.126`; add `--no-use` to install without switching) |
| `ccvm use <version>` | Switch to an installed Claude Code version |
| `ccvm current` | Show the active Claude Code version |
| `ccvm list` | List installed Claude Code versions |
| `ccvm list-remote` | List Claude Code versions available from the registry |
| `ccvm uninstall <version>` | Remove an installed Claude Code version |
| `ccvm pin [version]` | Write `.ccvmrc` for Claude Code |
| `ccvm codex install <version>` | Install Codex and switch to it (`latest` or a version like `0.134.0`; add `--no-use` to install without switching) |
| `ccvm codex use <version>` | Switch to an installed Codex version |
| `ccvm codex current` | Show the active Codex version |
| `ccvm codex list` | List installed Codex versions |
| `ccvm codex list-remote` | List Codex versions available from the registry |
| `ccvm codex uninstall <version>` | Remove an installed Codex version |
| `ccvm codex pin [version]` | Write `.codexvmrc` for Codex |
| `ccvm config registry` | Show the current registry URL |
| `ccvm config set registry <url>` | Set the registry URL |

Global option:

```bash
ccvm --registry https://registry.npmjs.org list-remote
```

## How It Works

### Claude Code

Claude Code distributes native binaries through npm `optionalDependencies` starting from v2.1.113.

```text
ccvm install 2.1.126
  GET registry/@anthropic-ai/claude-code/2.1.126
  parse optionalDependencies
  find @anthropic-ai/claude-code-win32-x64
  GET platform package metadata
  download tarball to ~/.ccvm/cache/
  verify npm dist.shasum
  extract package/claude.exe to ~/.ccvm/versions/2.1.126/
```

For Claude Code versions older than v2.1.113, ccvm falls back to `npm install` and requires Node.js.

### OpenAI Codex

Codex publishes platform-specific native packages through npm alias dependencies such as:

```json
{
  "@openai/codex-win32-x64": "npm:@openai/codex@0.134.0-win32-x64",
  "@openai/codex-linux-x64": "npm:@openai/codex@0.134.0-linux-x64",
  "@openai/codex-darwin-arm64": "npm:@openai/codex@0.134.0-darwin-arm64"
}
```

ccvm downloads the matching platform tarball, verifies `dist.shasum`, and extracts the bundled `vendor/` directory. Codex needs the full vendor directory because it includes the native `codex` binary plus supporting tools such as `rg` and sandbox resources.

## Directory Structure

```text
~/.ccvm/
  versions/
    2.1.126/
      claude.exe
  current
  codex/
    versions/
      0.134.0/
        vendor/
          x86_64-pc-windows-msvc/
            bin/
              codex.exe
            codex-path/
              rg.exe
            codex-resources/
    current
    cache/
  bin/
    claude.exe
    codex.exe
  cache/
  config.toml
```

On macOS and Linux the shim and binary names do not use `.exe`.

## Registry Mirrors

| Registry | URL |
|----------|-----|
| npmmirror (default) | `https://registry.npmmirror.com` |
| npm official | `https://registry.npmjs.org` |

```bash
ccvm config set registry https://registry.npmjs.org
ccvm config set registry https://registry.npmmirror.com
```

## Windows Installer

Build the release binaries and installer:

```powershell
powershell.exe -ExecutionPolicy Bypass -File installer\build.ps1
```

Outputs:

```text
target\release\ccvm-setup-X.Y.Z.exe
target\release\ccvm-setup.exe
```

The installer includes:

```text
ccvm.exe
ccvm-shim.exe
ccvm-codex-shim.exe
```

It installs to `%LOCALAPPDATA%\ccvm`, adds the install directory to the user PATH, and runs `ccvm setup` after installation.

## Platform Support

| Platform | Status |
|----------|--------|
| Windows x64 | Primary target |
| Windows arm64 | Supported when upstream packages are available |
| macOS arm64/x64 | Supported |
| Linux arm64/x64 | Supported |

## Build Requirements

- Rust 1.70+
- Inno Setup 6 for Windows installer builds
- Node.js only for installing old Claude Code versions before v2.1.113

## License

MIT
