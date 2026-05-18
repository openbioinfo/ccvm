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

    // Print PATH instructions
    let bin_dir = config::bin_dir();
    println!();
    println!("ccvm is ready!");
    println!();
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

    Ok(())
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
