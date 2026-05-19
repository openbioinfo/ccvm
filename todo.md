# TODO — ccvm Windows 安装包

目标：用户下载 .exe 安装向导，双击完成安装，无需手动配置。

## 前置依赖

- [x] 安装 [Inno Setup](https://jrsoftware.org/isinfo.php)（免费）

## 任务清单

- [x] **创建 `installer/ccvm.iss`** — Inno Setup 安装脚本
  - 默认安装目录：`%LOCALAPPDATA%\ccvm`
  - 安装内容：`ccvm.exe` + `ccvm-shim.exe`
  - 添加安装目录到用户 PATH（注册表 `HKCU\Environment`）
  - 安装后自动运行 `ccvm setup`
  - 支持卸载（标准添加/删除程序入口）

- [x] **创建 `installer/build.ps1`** — 一键构建脚本
  1. `cargo build --release`
  2. 读取 `Cargo.toml` 版本号
  3. 调用 `ISCC.exe` 生成安装包
  4. 输出：`target/release/ccvm-setup-X.Y.Z.exe`

## 验证

1. 运行 `.\installer\build.ps1`
2. 执行生成的安装包
3. 新终端验证 `ccvm --help` 和 `claude` 命令
4. 验证"添加/删除程序"中可卸载
