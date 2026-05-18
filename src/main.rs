mod config;
mod download;
mod registry;
mod setup;

use anyhow::Result;
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
                    // Derive cache filename from tarball URL
                    let filename = pkg
                        .tarball_url
                        .rsplit('/')
                        .next()
                        .unwrap_or("package.tgz");
                    let cache_path = config::cache_dir().join(filename);
                    match download::download_tarball(&pkg.tarball_url, &cache_path).await {
                        Ok(path) => println!("downloaded to {}", path.display()),
                        Err(e) => eprintln!("error: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                }
            }
        }
        Commands::Use { version } => {
            not_yet_with_arg("use", &version)
        }
        Commands::List => {
            not_yet("list")
        }
        Commands::ListRemote => {
            not_yet("list-remote")
        }
        Commands::Current => {
            not_yet("current")
        }
        Commands::Uninstall { version } => {
            not_yet_with_arg("uninstall", &version)
        }
        Commands::Pin { version } => {
            let v = version.unwrap_or_else(|| "current".to_string());
            not_yet_with_arg("pin", &v)
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
