use anyhow::{bail, Result};

#[allow(dead_code)]
pub fn npm_platform_key() -> Result<&'static str> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (os, arch) {
        ("windows", "x86_64") | ("windows", "x86") => Ok("win32-x64"),
        ("windows", "aarch64") => Ok("win32-arm64"),
        ("macos", "aarch64") | ("macos", "arm64") => Ok("darwin-arm64"),
        ("macos", "x86_64") => Ok("darwin-x64"),
        ("linux", "x86_64") => Ok("linux-x64"),
        ("linux", "aarch64") => Ok("linux-arm64"),
        _ => bail!("unsupported platform: {}/{}", os, arch),
    }
}

pub fn codex_target_triple() -> Result<&'static str> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (os, arch) {
        ("windows", "x86_64") | ("windows", "x86") => Ok("x86_64-pc-windows-msvc"),
        ("windows", "aarch64") => Ok("aarch64-pc-windows-msvc"),
        ("macos", "aarch64") | ("macos", "arm64") => Ok("aarch64-apple-darwin"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-musl"),
        _ => bail!("unsupported platform: {}/{}", os, arch),
    }
}

pub fn executable_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}
