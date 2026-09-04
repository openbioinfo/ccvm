# ccvm install 自动启用（auto-use）设计

日期：2026-09-04
状态：已批准

## 背景

当前 `ccvm install <version>` 只负责下载并解压到 `~/.ccvm/versions/`，不会写
`~/.ccvm/current`；用户必须再执行一次 `ccvm use <version>` 才能让 shim 解析到新
版本。`ccvm codex install` 同理。目标：install 成功后自动启用该版本，省去手动
`use` 一步。

## 需求

1. `ccvm install <version>` 与 `ccvm codex install <version>` 安装成功后
   **总是自动启用**该版本（写 current 文件），与 `use` 输出一致。
2. 提供 `--no-use` 退出标志：只安装不启用。
3. 失败语义：解压 / 复制失败则**不启用**；npm-fallback 安装失败同样不启用。
4. 覆盖安装已存在的版本：直接覆盖 current，无提示。
5. npm-fallback 路径（< 2.1.113，需 Node.js）成功后同样自动启用。
6. 启用写入的版本号使用 `pkg.version`（registry 解析后的精确版本，如
   `latest` → `2.1.126`），不使用用户输入的原始字符串。

## 设计

改动全部集中在 `src/main.rs`。

### 新增辅助函数

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

放在 `resolve_fuzzy_in_dir` 附近。

### 调用点（4 处）

| 位置 | 现状 | 改动 |
|------|------|------|
| `Commands::Use`（main.rs:162-169） | 内联写 current + println | 改为调用 `activate_version(&resolved, &config::current_file(), "claude-code")` |
| `Commands::Install` 原生路径（`extract_and_verify` Ok 分支） | 什么都不做 | `if !no_use { activate_version(&pkg.version, ...) }` |
| `install_codex` 成功处 | 什么都不做 | 同上，用 `config::codex_current_file()`、`"codex"` |
| `npm_fallback` 成功处（打印 installed 之后） | 仅打印 installed | `if !no_use { activate_version(version, ...) }` |

### --no-use 标志

- `Install` 与 `CodexCommands::Install` 各加 `#[arg(long)] no_use: bool`。
- `npm_fallback(version)` 签名改为 `npm_fallback(version, no_use)`，由 install
  分支传入。

### 已知边界：npm-fallback 的版本号来源

npm-fallback 收到的 `version` 是用户原始输入。但 fallback 的唯一触发条件是错误消息
含 "predates native binary"，这只在用户指定了 < 2.1.113 的精确版本时发生——此时
`version` 就是精确版本，目录名与 `npm install @...@{version}` 安装的版本一致。
用户输入 `latest` 时 `resolve_latest` 返回当前 latest（≥ 2.1.113，有平台包），
不会触发 fallback。故 fallback 路径无需额外处理，激活写 `version` 即正确。

## 错误处理

- `activate_version` 写文件失败 → `Err` 传播（`?`），与现有 `use` 分支行为一致。
- 解压失败 → 保持现状（`eprintln!` 后不启用）。
- `--no-use` 时激活被跳过，不产生激活错误。

## 测试

项目无测试框架。验证方式为手工验证：

```bash
cargo run -- install 2.1.126                        # 打印 "now using claude-code 2.1.126"
cargo run -- install 2.1.124 --no-use               # 安装但不切换
cargo run -- codex install latest --no-use          # Codex 同理
cargo run -- install 2.1.126 --registry https://registry.npmjs.org
                                                    # 确认 --no-use 与全局 --registry 不冲突
```

## 文档

更新 README：

- 命令表中 `install` 描述改为「安装并启用（`latest` 或版本号）」。
- Claude Code / Codex 快速上手示例补充一句「install 后即自动启用」。
- 注明 `--no-use` 存在。
