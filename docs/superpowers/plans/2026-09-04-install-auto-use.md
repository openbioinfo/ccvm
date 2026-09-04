# ccvm install 自动启用（auto-use）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `ccvm install` / `ccvm codex install` 安装成功后自动启用该版本（写 current 文件），并提供 `--no-use` 退出标志。

**Architecture:** 在 `src/main.rs` 新增一个 `activate_version()` 辅助函数封装「写 current + 打印 now using」，`Use` 分支重构为复用它，三个安装成功点（Claude 原生、Codex 原生、npm-fallback）在 `--no-use` 未设置时调用它。全部改动限于 `src/main.rs` 与 README。

**Tech Stack:** Rust（edition 2021），clap 4 derive，anyhow。

## Global Constraints

- Rust 1.70+，不新增任何依赖。
- 本项目无测试框架，不引入；每个任务的验证是 `cargo build` 成功 + 手工命令检查输出。每个任务结束时树必须可编译。
- 启用写入的版本号一律使用 `pkg.version`（registry 解析后的精确版本）。
- 激活失败的语义与现有 `use` 一致：`Err` 经 `?` 传播，命令失败并报错。
- 解压失败不激活（保持现状：`eprintln!` 后继续，不写 current）。

---

### Task 1: 新增 `activate_version` 辅助函数并重构 `Use` 分支

**Files:**
- Modify: `src/main.rs:162-169`（Use 分支）、`src/main.rs:372-374`（resolve_fuzzy 之后插入新函数）

**Interfaces:**
- Produces: `fn activate_version(version: &str, current_file: &std::path::Path, tool_name: &str) -> anyhow::Result<()>` — 写 `current_file` 内容为 `version`，打印 `now using {tool_name} {version}`。后续 Task 2/3/4 依赖此签名。

- [ ] **Step 1: 在 `resolve_fuzzy` 函数之后插入辅助函数**

在 `src/main.rs` 第 374 行（`resolve_fuzzy` 函数结束的 `}` 之后、`resolve_fuzzy_in_dir` 之前）插入：

```rust
fn activate_version(
    version: &str,
    current_file: &std::path::Path,
    tool_name: &str,
) -> anyhow::Result<()> {
    std::fs::write(current_file, version)
        .with_context(|| format!("failed to write current {} version", tool_name))?;
    println!("now using {} {}", tool_name, version);
    Ok(())
}
```

- [ ] **Step 2: 重构 `Use` 分支使用新函数**

把 `src/main.rs:162-169` 的：

```rust
        Commands::Use { version } => match resolve_fuzzy(&version) {
            Ok(resolved) => {
                std::fs::write(config::current_file(), &resolved)
                    .with_context(|| "failed to write current version")?;
                println!("now using claude-code {}", resolved);
            }
            Err(e) => eprintln!("error: {}", e),
        },
```

替换为：

```rust
        Commands::Use { version } => match resolve_fuzzy(&version) {
            Ok(resolved) => {
                activate_version(&resolved, &config::current_file(), "claude-code")?;
            }
            Err(e) => eprintln!("error: {}", e),
        },
```

- [ ] **Step 3: 编译并手工验证**

Run: `cargo build`
Expected: 编译成功，无警告新增。

Run: `cargo run -- use <已安装版本>`
（先 `cargo run -- list` 看一个已装版本号，没有就先 `cargo run -- install 2.1.126` 安装。）
Expected: 输出 `now using claude-code <版本>`，且 `cargo run -- current` 显示该版本。行为与改动前完全一致。

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "refactor: extract activate_version helper, reuse in use command"
```

---

### Task 2: Claude `install` 自动启用 + `--no-use` 标志

**Files:**
- Modify: `src/main.rs:23-28`（Install 子命令定义）、`src/main.rs:128-161`（Install 分支）

**Interfaces:**
- Consumes: `activate_version`（Task 1）。
- Produces: `Install { version: String, no_use: bool }` 子命令结构。

- [ ] **Step 1: 给 `Install` 子命令加 `--no-use` 标志**

把 `src/main.rs:23-28` 的：

```rust
    /// Install a version of Claude Code
    Install {
        /// Version to install, e.g. "2.1.126" or "latest"
        version: String,
    },
```

替换为：

```rust
    /// Install a version of Claude Code
    Install {
        /// Version to install, e.g. "2.1.126" or "latest"
        version: String,
        /// Install without switching to the new version
        #[arg(long)]
        no_use: bool,
    },
```

- [ ] **Step 2: Install 分支解构并激活**

把 `src/main.rs:128` 的：

```rust
        Commands::Install { version } => {
```

替换为：

```rust
        Commands::Install { version, no_use } => {
```

把 `src/main.rs:140-143` 的：

```rust
                            match extract::extract_and_verify(&path, &pkg.shasum, &pkg.version) {
                                Ok(_dest) => {}
                                Err(e) => eprintln!("error: {}", e),
                            }
```

替换为：

```rust
                            match extract::extract_and_verify(&path, &pkg.shasum, &pkg.version) {
                                Ok(_dest) => {
                                    if !no_use {
                                        activate_version(
                                            &pkg.version,
                                            &config::current_file(),
                                            "claude-code",
                                        )?;
                                    }
                                }
                                Err(e) => eprintln!("error: {}", e),
                            }
```

注意：`npm_fallback(&version)` 的调用（约 `src/main.rs:153`）本任务**不动**——它的签名在 Task 4 才改。

- [ ] **Step 3: 编译并手工验证**

Run: `cargo build`
Expected: 编译成功。

Run: `cargo run -- install 2.1.126 --no-use`
Expected: 安装输出照常，但**没有** `now using ...`，且 `cargo run -- current` 不是 2.1.126（若之前启用的是别的版本）。

Run: `cargo run -- install 2.1.126`
Expected: 安装输出照常，且输出 `now using claude-code 2.1.126`，`cargo run -- current` 显示 `2.1.126`。

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: auto-activate version after claude-code install, add --no-use"
```

---

### Task 3: Codex `install` 自动启用 + `--no-use` 标志

**Files:**
- Modify: `src/main.rs:63-67`（CodexCommands::Install 定义）、`src/main.rs:300-302`（handle_codex Install 分支）、`src/main.rs:347-370`（install_codex 函数）

**Interfaces:**
- Consumes: `activate_version`（Task 1）。
- Produces: `async fn install_codex(registry: &str, version: &str, no_use: bool) -> Result<()>`。

- [ ] **Step 1: 给 `CodexCommands::Install` 加 `--no-use` 标志**

把 `src/main.rs:63-67` 的：

```rust
    /// Install a version of Codex
    Install {
        /// Version to install, e.g. "0.134.0" or "latest"
        version: String,
    },
```

替换为：

```rust
    /// Install a version of Codex
    Install {
        /// Version to install, e.g. "0.134.0" or "latest"
        version: String,
        /// Install without switching to the new version
        #[arg(long)]
        no_use: bool,
    },
```

- [ ] **Step 2: handle_codex 传递 no_use**

把 `src/main.rs:300-302` 的：

```rust
        CodexCommands::Install { version } => {
            install_codex(registry, &version).await;
        }
```

替换为：

```rust
        CodexCommands::Install { version, no_use } => {
            install_codex(registry, &version, no_use).await?;
        }
```

- [ ] **Step 3: install_codex 签名改为带 no_use 并返回 Result**

把 `src/main.rs:347` 的：

```rust
async fn install_codex(registry: &str, version: &str) {
```

替换为：

```rust
async fn install_codex(registry: &str, version: &str, no_use: bool) -> Result<()> {
```

- [ ] **Step 4: install_codex 成功处激活**

把 `src/main.rs:357-366` 的：

```rust
            match download::download_tarball(&pkg.tarball_url, &cache_path).await {
                Ok(path) => {
                    if let Err(e) =
                        extract::extract_codex_and_verify(&path, &pkg.shasum, &pkg.version)
                    {
                        eprintln!("error: {}", e);
                    }
                }
                Err(e) => eprintln!("error: {}", e),
            }
        }
        Err(e) => eprintln!("error: {}", e),
    }
}
```

替换为：

```rust
            match download::download_tarball(&pkg.tarball_url, &cache_path).await {
                Ok(path) => {
                    match extract::extract_codex_and_verify(&path, &pkg.shasum, &pkg.version) {
                        Ok(_dest) => {
                            if !no_use {
                                activate_version(
                                    &pkg.version,
                                    &config::codex_current_file(),
                                    "codex",
                                )?;
                            }
                        }
                        Err(e) => eprintln!("error: {}", e),
                    }
                }
                Err(e) => eprintln!("error: {}", e),
            }
        }
        Err(e) => eprintln!("error: {}", e),
    }

    Ok(())
}
```

（函数末尾补 `Ok(())`，因为签名现在是 `-> Result<()>`。）

- [ ] **Step 5: 编译并手工验证**

Run: `cargo build`
Expected: 编译成功。

Run: `cargo run -- codex install 0.134.0 --no-use`
Expected: 安装输出照常，无 `now using codex ...`，`cargo run -- codex current` 不是 0.134.0（若之前启用的是别的版本）。

Run: `cargo run -- codex install 0.134.0`
Expected: 输出 `now using codex 0.134.0`，`cargo run -- codex current` 显示 `0.134.0`。

- [ ] **Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat: auto-activate version after codex install, add --no-use"
```

---

### Task 4: npm-fallback 安装成功也自动启用

**Files:**
- Modify: `src/main.rs:153`（fallback 调用点）、`src/main.rs:606`（npm_fallback 签名）、`src/main.rs:683-689`（函数结尾）

**Interfaces:**
- Consumes: `activate_version`（Task 1）。
- Produces: `async fn npm_fallback(version: &str, no_use: bool) -> Result<(), anyhow::Error>`。

- [ ] **Step 1: 修改调用点传入 no_use**

把 `src/main.rs:153` 的：

```rust
                        if let Err(e) = npm_fallback(&version).await {
```

替换为：

```rust
                        if let Err(e) = npm_fallback(&version, no_use).await {
```

- [ ] **Step 2: 修改 npm_fallback 签名**

把 `src/main.rs:606` 的：

```rust
async fn npm_fallback(version: &str) -> Result<(), anyhow::Error> {
```

替换为：

```rust
async fn npm_fallback(version: &str, no_use: bool) -> Result<(), anyhow::Error> {
```

- [ ] **Step 3: 函数末尾激活**

把 `src/main.rs:683-689` 的：

```rust
    println!(
        "installed claude-code {} to {}",
        version,
        dest_dir.display()
    );

    Ok(())
}
```

替换为：

```rust
    println!(
        "installed claude-code {} to {}",
        version,
        dest_dir.display()
    );

    if !no_use {
        activate_version(version, &config::current_file(), "claude-code")?;
    }

    Ok(())
}
```

- [ ] **Step 4: 编译验证**

Run: `cargo build`
Expected: 编译成功。

（fallback 路径需 Node.js 且只对 < 2.1.113 版本触发；无 Node.js 环境时以编译通过 + 逻辑审查为准。有环境则可试 `cargo run -- install 2.1.100 --no-use` 走 y 确认后观察无 `now using`。）

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: auto-activate after npm-fallback install"
```

---

### Task 5: 更新 README

**Files:**
- Modify: `README.md`（命令表 + 两处快速上手注释）

- [ ] **Step 1: 更新命令表两行**

把 README 命令表中的：

```markdown
| `ccvm install <version>` | Install Claude Code (`latest` or a version like `2.1.126`) |
```

替换为：

```markdown
| `ccvm install <version>` | Install Claude Code and switch to it (`latest` or a version like `2.1.126`; add `--no-use` to install without switching) |
```

把：

```markdown
| `ccvm codex install <version>` | Install Codex (`latest` or a version like `0.134.0`) |
```

替换为：

```markdown
| `ccvm codex install <version>` | Install Codex and switch to it (`latest` or a version like `0.134.0`; add `--no-use` to install without switching) |
```

- [ ] **Step 2: 更新快速上手注释**

把 Claude Code 示例中的：

```markdown
ccvm install latest       # install the latest Claude Code version
```

替换为：

```markdown
ccvm install latest       # install and switch to the latest Claude Code version
```

把 Codex 示例中的：

```markdown
ccvm codex install latest  # install the latest Codex version
```

替换为：

```markdown
ccvm codex install latest  # install and switch to the latest Codex version
```

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: document install auto-switch and --no-use"
```

---

### 收尾验证（全部任务完成后）

- [ ] `cargo build --release` 通过
- [ ] `cargo run -- install 2.1.126` → 输出含 `now using claude-code 2.1.126`
- [ ] `cargo run -- install 2.1.126 --no-use` → 无 `now using`
- [ ] `cargo run -- codex install 0.134.0` → 输出含 `now using codex 0.134.0`
- [ ] `cargo run -- codex install 0.134.0 --no-use` → 无 `now using`
