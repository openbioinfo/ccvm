mod config;
mod download;
mod extract;
mod registry;
mod setup;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ccvm", about = "Claude Code Version Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Override the registry URL for this command
    #[arg(long, global = true)]
    registry: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a version of Claude Code
    Install {
        /// Version to install, e.g. "2.1.126" or "latest"
        version: String,
    },
    /// Switch to an installed version
    Use {
        /// Version to use, supports fuzzy matching (e.g. "2.1")
        version: String,
    },
    /// List installed versions
    List,
    /// List available versions from the registry
    ListRemote,
    /// Show the currently active version
    Current,
    /// Uninstall a version
    Uninstall {
        /// Version to uninstall
        version: String,
    },
    /// Write .ccvmrc to pin the current or specified version
    Pin {
        /// Version to pin (defaults to current)
        version: Option<String>,
    },
    /// Initialize ccvm: create directories, install shim, configure PATH
    Setup,
    /// View or change configuration
    #[command(subcommand)]
    Config(ConfigCmd),
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Show the current registry URL
    Registry,
    /// Set the registry URL
    Set {
        /// Subcommand: set registry
        #[command(subcommand)]
        action: ConfigSetAction,
    },
}

#[derive(Subcommand)]
enum ConfigSetAction {
    /// Set the registry URL
    Registry {
        /// Registry URL, e.g. https://registry.npmmirror.com
        url: String,
    },
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run())
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let registry = if let Some(ref r) = cli.registry {
        r.clone()
    } else {
        config::get_registry()
    };

    match cli.command {
        Commands::Install { version } => {
            println!("resolving {} from {}...", version, registry);
            match registry::resolve_package(&registry, &version).await {
                Ok(pkg) => {
                    println!("resolved: claude-code {}", pkg.version);
                    println!("tarball: {}", pkg.tarball_url);
                    println!("shasum:  {}", pkg.shasum);
                    println!();
                    let filename = pkg
                        .tarball_url
                        .rsplit('/')
                        .next()
                        .unwrap_or("package.tgz");
                    let cache_path = config::cache_dir().join(filename);
                    match download::download_tarball(&pkg.tarball_url, &cache_path).await {
                        Ok(path) => {
                            match extract::extract_and_verify(
                                &path,
                                &pkg.shasum,
                                &pkg.version,
                            ) {
                                Ok(_dest) => {}
                                Err(e) => eprintln!("error: {}", e),
                            }
                        }
                        Err(e) => eprintln!("error: {}", e),
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("predates native binary") {
                        eprintln!("{}", msg);
                        eprintln!();
                        if let Err(e) = npm_fallback(&version).await {
                            eprintln!("error: {}", e);
                        }
                    } else {
                        eprintln!("error: {}", msg);
                    }
                }
            }
        }
        Commands::Use { version } => {
            match resolve_fuzzy(&version) {
                Ok(resolved) => {
                    std::fs::write(config::current_file(), &resolved)
                        .with_context(|| "failed to write current version")?;
                    println!("now using claude-code {}", resolved);
                }
                Err(e) => eprintln!("error: {}", e),
            }
        }
        Commands::List => {
            not_yet("list")
        }
        Commands::ListRemote => {
            not_yet("list-remote")
        }
        Commands::Current => {
            let path = config::current_file();
            if path.exists() {
                let v = std::fs::read_to_string(&path)
                    .with_context(|| "failed to read current version")
                    .unwrap_or_default();
                let v = v.trim();
                if v.is_empty() {
                    println!("no version selected. Run 'ccvm use <version>' first.");
                } else {
                    println!("{}", v);
                }
            } else {
                println!("no version selected. Run 'ccvm use <version>' first.");
            }
        }
        Commands::Uninstall { version } => {
            not_yet_with_arg("uninstall", &version)
        }
        Commands::Pin { version } => {
            let v = if let Some(ref v) = version {
                v.clone()
            } else {
                let path = config::current_file();
                if path.exists() {
                    std::fs::read_to_string(&path)
                        .unwrap_or_default()
                        .trim()
                        .to_string()
                } else {
                    String::new()
                }
            };
            if v.is_empty() {
                eprintln!("no version selected. Run 'ccvm use <version>' first.");
            } else {
                std::fs::write(".ccvmrc", &v)
                    .with_context(|| "failed to write .ccvmrc")?;
                println!("pinned claude-code {}", v);
            }
        }
        Commands::Setup => {
            setup::run()?;
        }
        Commands::Config(cmd) => match cmd {
            ConfigCmd::Registry => {
                println!("{}", registry);
            }
            ConfigCmd::Set { action } => match action {
                ConfigSetAction::Registry { url } => {
                    config::set_registry(&url)?;
                    println!("registry set to: {}", url);
                }
            },
        },
    }

    Ok(())
}

fn not_yet(cmd: &str) {
    println!("not yet implemented: ccvm {}", cmd);
}

fn not_yet_with_arg(cmd: &str, arg: &str) {
    println!("not yet implemented: ccvm {} {}", cmd, arg);
}

fn resolve_fuzzy(version: &str) -> anyhow::Result<String> {
    let versions_dir = config::versions_dir();

    if !versions_dir.exists() {
        anyhow::bail!(
            "no versions installed. Run 'ccvm install {}' first.",
            version
        );
    }

    let mut candidates: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&versions_dir)
        .with_context(|| "failed to read versions directory")?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(dir_name) = entry.file_name().to_str() {
                if dir_name.starts_with(&version) {
                    candidates.push(dir_name.to_string());
                }
            }
        }
    }

    if candidates.is_empty() {
        anyhow::bail!(
            "version {} is not installed. Run 'ccvm install {}' first.",
            version,
            version
        );
    }

    // Sort by semver, pick highest
    candidates.sort_by(|a, b| {
        let va = semver::Version::parse(a).ok();
        let vb = semver::Version::parse(b).ok();
        match (va, vb) {
            (Some(va), Some(vb)) => vb.cmp(&va), // descending
            _ => b.cmp(a), // fallback to string compare
        }
    });

    let resolved = candidates[0].clone();

    if candidates.len() > 1 {
        eprintln!(
            "note: multiple versions match '{}': {}, using latest ({})",
            version,
            candidates.join(", "),
            resolved
        );
    }

    Ok(resolved)
}

async fn npm_fallback(version: &str) -> Result<(), anyhow::Error> {
    // Check for Node.js
    let node_check = std::process::Command::new("node")
        .arg("--version")
        .output();
    if node_check.is_err() {
        anyhow::bail!(
            "Node.js is required for versions < 2.1.113 but was not found.\n\
             Install Node.js from https://nodejs.org or use a version >= 2.1.113."
        );
    }

    // Prompt user
    use std::io::{self, Write as _};
    print!(
        "Continue with npm install? Requires Node.js + npm. [y/N] "
    );
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim().to_lowercase();

    if input != "y" && input != "yes" {
        println!("installation cancelled");
        return Ok(());
    }

    // Create temp directory
    let temp_dir = std::env::temp_dir().join(format!("ccvm-npm-{}", version));
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }
    std::fs::create_dir_all(&temp_dir)
        .with_context(|| format!("failed to create temp directory: {}", temp_dir.display()))?;

    println!("running npm install @anthropic-ai/claude-code@{}...", version);
    let status = std::process::Command::new("npm")
        .args([
            "install",
            &format!("@anthropic-ai/claude-code@{}", version),
            "--prefix",
        ])
        .arg(&temp_dir)
        .status()
        .with_context(|| "failed to run npm. Is npm installed?")?;

    if !status.success() {
        // Clean up temp dir on failure
        std::fs::remove_dir_all(&temp_dir).ok();
        anyhow::bail!("npm install failed with exit code: {:?}", status.code());
    }

    // Locate the installed binary
    let binary_path = temp_dir
        .join("node_modules")
        .join("@anthropic-ai")
        .join("claude-code")
        .join("claude.exe");

    if !binary_path.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
        anyhow::bail!("npm install completed but claude binary not found");
    }

    // Copy to versions directory
    let dest_dir = config::versions_dir().join(version);
    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("failed to create version directory: {}", dest_dir.display()))?;
    let dest_binary = dest_dir.join("claude.exe");
    std::fs::copy(&binary_path, &dest_binary)
        .with_context(|| format!("failed to copy binary to {}", dest_binary.display()))?;

    // Clean up temp directory
    std::fs::remove_dir_all(&temp_dir).ok();

    println!(
        "installed claude-code {} to {}",
        version,
        dest_dir.display()
    );

    Ok(())
}
