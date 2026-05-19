# ccvm — Claude Code Version Manager

A fast, Node-free version manager for [Claude Code](https://claude.ai/code), using the npm registry as the download source.

## Why ccvm?

Claude Code distributes native binaries via npm optionalDependencies starting from v2.1.113. ccvm downloads the platform-specific tarball directly from the npm registry — no Node.js required for modern versions. Uses the npmmirror mirror by default for fast downloads in China.

## Quick Start

### Windows 一条命令安装

```powershell
irm https://raw.githubusercontent.com/openbioinfo/ccvm/master/install.ps1 | iex
```

安装后重启终端即可使用。

### 从源码构建

```bash
git clone https://github.com/openbioinfo/ccvm.git
cd ccvm
cargo build --release
cargo install --path ./
```

### Setup

```bash
# One command to create directories, install the shim, and print PATH instructions
ccvm setup
```

will add `~/.ccvm/bin/` to your PATH, then restart your terminal.

### Install and use

```bash
ccvm install latest       # install the latest version
ccvm install 2.1.126      # install a specific version
ccvm use 2.1.126           # switch to that version
claude --version           # shim resolves to the active version
```

## Commands

| Command | Description |
|---------|-------------|
| `ccvm setup` | Initialize ccvm: create directories, install shim, print PATH instructions |
| `ccvm install <version>` | Install a version (`latest` or e.g. `2.1.126`) |
| `ccvm use <version>` | Switch to an installed version (supports fuzzy matching, e.g. `2.1`) |
| `ccvm current` | Show the currently active version |
| `ccvm list` | List installed versions (`*` marks active) |
| `ccvm list-remote` | List all versions available from the registry |
| `ccvm uninstall <version>` | Remove an installed version |
| `ccvm pin [version]` | Write `.ccvmrc` to pin the current or specified version |
| `ccvm config registry` | Show the current registry URL |
| `ccvm config set registry <url>` | Switch the registry mirror |

### Options

| Flag | Description |
|------|-------------|
| `--registry <url>` | Override registry for a single command |

## How It Works

### For versions >= 2.1.113 (Node-free)

```
ccvm install 2.1.126
  ├─ GET registry/@anthropic-ai/claude-code/2.1.126
  │   → Parse optionalDependencies
  │   → Find @anthropic-ai/claude-code-win32-x64
  ├─ GET platform package metadata → Get tarball URL + shasum
  ├─ Stream download with progress bar → ~/.ccvm/cache/
  ├─ SHA-1 verify against npm dist.shasum
  └─ Extract package/claude.exe → ~/.ccvm/versions/2.1.126/
```

### For versions < 2.1.113 (npm fallback)

Older versions require Node.js. ccvm prompts for confirmation, then runs `npm install` in a temp directory and copies the binary.

### Shim

The `ccvm-shim` executable resolves `claude` to the right binary:
1. Check `.ccvmrc` in the current directory (cascading up to root)
2. Fall back to `~/.ccvm/current`
3. Exec `~/.ccvm/versions/{version}/claude.exe` with all arguments forwarded

## Directory Structure

```
~/.ccvm/
├── versions/           # Installed versions
│   ├── 2.1.126/
│   │   └── claude.exe
│   └── 2.1.143/
│       └── claude.exe
├── bin/
│   └── ccvm-shim.exe   # Shim binary (add to PATH)
├── cache/              # Downloaded tarballs
├── config.toml         # Configuration
└── current             # Active version (plain text)
```

## Registry Mirrors

| Registry | URL |
|----------|-----|
| npmmirror (default) | `https://registry.npmmirror.com` |
| npm official | `https://registry.npmjs.org` |

```bash
ccvm config set registry https://registry.npmjs.org    # switch to official
ccvm config set registry https://registry.npmmirror.com # switch to mirror
```

## Platform Support

| Platform | Status |
|----------|--------|
| Windows x64 | Primary target |
| macOS arm64/x64 | Supported |
| Linux x64/arm64 | Supported |

## Build Requirements

- [Rust](https://rustup.rs) 1.70+
- Node.js (optional, only for installing versions < 2.1.113)

## License

MIT
