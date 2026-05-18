use crate::config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    config::ensure_dirs()?;

    // Write default config if it doesn't exist
    let config_path = config::config_path();
    if !config_path.exists() {
        config::save_config(&config::Config::default())?;
        println!("created {}", config_path.display());
    }

    // Copy shim binary to ~/.ccvm/bin/
    let shim_dest = config::bin_dir().join("claude.exe");
    if !shim_dest.exists() {
        match copy_shim(&shim_dest) {
            Ok(_) => println!("installed shim to {}", shim_dest.display()),
            Err(e) => eprintln!("warning: could not install shim: {}", e),
        }
    } else {
        println!("shim already installed at {}", shim_dest.display());
    }

    println!();
    update_path();

    Ok(())
}

fn update_path() {
    let bin_dir = config::bin_dir();
    let bin_str = bin_dir.to_string_lossy().to_string();

    if try_update_path(&bin_dir, &bin_str) {
        return;
    }

    // Fallback: print manual instructions
    println!("Add the following to your PATH to use the shim:");
    println!("  {}", bin_dir.display());
    println!();
    println!("On Windows (PowerShell):");
    println!(
        "  [Environment]::SetEnvironmentVariable('PATH', '{};' + [Environment]::GetEnvironmentVariable('PATH', 'User'), 'User')",
        bin_dir.display()
    );
    println!();
    println!("Then restart your terminal, or run:");
    println!("  $env:PATH = \"{};$env:PATH\"", bin_dir.display());
}

#[cfg(target_os = "windows")]
fn try_update_path(bin_dir: &PathBuf, bin_str: &str) -> bool {
    // Query current user PATH
    let current = match std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", "[Environment]::GetEnvironmentVariable('PATH', 'User')"])
        .output()
    {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => return false,
    };

    // Case-insensitive dedup on Windows
    if current.to_lowercase().split(';').any(|p| p == bin_str.to_lowercase()) {
        println!("PATH already contains {}", bin_dir.display());
        return true;
    }

    let new_path = if current.is_empty() {
        bin_str.to_string()
    } else {
        format!("{};{}", bin_str, current)
    };

    let escaped = new_path.replace('\'', "''");
    let script = format!(
        "[Environment]::SetEnvironmentVariable('PATH', '{}', 'User')",
        escaped
    );

    match std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .status()
    {
        Ok(s) if s.success() => {
            println!("added {} to user PATH (restart terminal to take effect)", bin_dir.display());
            true
        }
        _ => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn try_update_path(bin_dir: &PathBuf, bin_str: &str) -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    let shell = std::env::var("SHELL").unwrap_or_default();
    let shell_name = std::path::Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let (profile, line): (PathBuf, String) = match shell_name {
        "fish" => (
            home.join(".config/fish/config.fish"),
            format!("fish_add_path \"{}\"", bin_str),
        ),
        "zsh" => (
            home.join(".zshrc"),
            format!("export PATH=\"{}:$PATH\"", bin_str),
        ),
        "bash" => (
            home.join(".bashrc"),
            format!("export PATH=\"{}:$PATH\"", bin_str),
        ),
        _ => (
            home.join(".profile"),
            format!("export PATH=\"{}:$PATH\"", bin_str),
        ),
    };

    // Check if already present
    let existing = std::fs::read_to_string(&profile).unwrap_or_default();
    if existing.contains(bin_str) {
        println!("PATH entry already exists in {}", profile.display());
        return true;
    }

    // Create parent dir for fish config if needed
    if let Some(parent) = profile.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    // Append
    let mut content = existing;
    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }
    content.push_str(&format!("\n# Added by ccvm\n{}\n", line));

    match std::fs::write(&profile, content) {
        Ok(_) => {
            println!(
                "added to {} (restart terminal or 'source {}' to take effect)",
                profile.display(),
                profile.display()
            );
            true
        }
        Err(_) => false,
    }
}

fn copy_shim(dest: &PathBuf) -> Result<()> {
    // Find the shim binary next to the running ccvm binary
    let current_exe =
        std::env::current_exe().with_context(|| "could not determine ccvm binary location")?;
    let exe_dir = current_exe
        .parent()
        .with_context(|| "could not determine ccvm directory")?;

    let shim_src = exe_dir.join("ccvm-shim.exe");
    if shim_src.exists() {
        fs::copy(&shim_src, dest)
            .with_context(|| format!("failed to copy shim from {}", shim_src.display()))?;
    } else {
        // Create a placeholder script
        let placeholder = "@echo off\r\necho ccvm shim not yet installed. Run 'ccvm setup' again after building.\r\n";
        fs::write(dest, placeholder)
            .with_context(|| format!("failed to write shim placeholder to {}", dest.display()))?;
    }

    Ok(())
}
