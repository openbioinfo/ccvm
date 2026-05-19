# ccvm Windows 安装包

## 一条命令安装（推荐）

在 PowerShell 中运行：

```powershell
irm https://raw.githubusercontent.com/kongdeju/ccvm/master/install.ps1 | iex
```

默认安装最新版本。如需指定版本：

```powershell
irm https://raw.githubusercontent.com/kongdeju/ccvm/master/install.ps1 | iex -args "-Version 0.2.0"
```

安装程序会引导你完成安装，包括自动配置 PATH 和初始化 ccvm。

## 从源码构建

### 前置条件

- [Rust](https://rustup.rs) toolchain
- [Inno Setup 6](https://jrsoftware.org/isinfo.php)（安装到默认路径即可）

### 构建

```bash
powershell.exe -ExecutionPolicy Bypass -File installer/build.ps1
```

输出文件：`target/release/ccvm-setup-X.Y.Z.exe`

### 手动安装

双击 `ccvm-setup-X.Y.Z.exe`，按向导完成安装。

安装程序会：
- 将 `ccvm.exe` + `ccvm-shim.exe` 放到 `%LOCALAPPDATA%\ccvm`
- 将安装目录添加到用户 PATH
- 安装完成后自动运行 `ccvm setup`（初始化 ~/.ccvm 目录、部署 shim）

## 验证

打开新终端，运行：

```
ccvm --help
claude
```

## 卸载

Windows 设置 → 应用 → 搜索 `ccvm` → 卸载。

卸载时会自动清理 PATH 中的安装目录条目。`~/.ccvm` 目录（版本缓存）不会被自动删除，如需清理可手动执行 `rm -r ~/.ccvm`。
