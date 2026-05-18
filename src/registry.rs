use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct PackageVersion {
    #[serde(default)]
    #[allow(dead_code)]
    pub name: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub version: String,
    #[serde(default)]
    pub dist: Option<DistInfo>,
    #[serde(default)]
    pub optional_dependencies: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct DistInfo {
    pub tarball: String,
    pub shasum: String,
}

#[derive(Deserialize, Debug)]
pub struct Packument {
    #[serde(default)]
    #[allow(dead_code)]
    pub name: String,
    #[serde(default, rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
}

pub struct ResolvedPackage {
    pub version: String,
    pub tarball_url: String,
    pub shasum: String,
}

fn platform_key() -> &'static str {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (os, arch) {
        ("windows", "x86_64") | ("windows", "x86") => "win32-x64",
        ("macos", "aarch64") | ("macos", "arm64") => "darwin-arm64",
        ("macos", "x86_64") => "darwin-x64",
        ("linux", "x86_64") => "linux-x64",
        ("linux", "aarch64") => "linux-arm64",
        _ => panic!("unsupported platform: {}/{}", os, arch),
    }
}

pub async fn resolve_package(registry: &str, version: &str) -> Result<ResolvedPackage> {
    let client = reqwest::Client::new();

    // Resolve "latest" to actual version number
    let resolved_version = if version == "latest" {
        resolve_latest(&client, registry).await?
    } else {
        version.to_string()
    };

    // Fetch the main package metadata
    let main_url = format!(
        "{}/@anthropic-ai/claude-code/{}",
        registry.trim_end_matches('/'),
        &resolved_version
    );

    let main_pkg: PackageVersion = client
        .get(&main_url)
        .header("Accept", "application/json")
        .send()
        .await
        .with_context(|| format!("failed to fetch package metadata from {}", main_url))?
        .error_for_status()
        .with_context(|| {
            format!(
                "version {} not found in registry {}",
                resolved_version,
                registry.trim_end_matches('/')
            )
        })?
        .json()
        .await
        .with_context(|| "failed to parse package metadata JSON")?;

    // Find the platform-specific optional dependency
    let platform_name = format!("@anthropic-ai/claude-code-{}", platform_key());

    let platform_version = main_pkg
        .optional_dependencies
        .as_ref()
        .and_then(|deps| deps.get(&platform_name))
        .cloned();

    match platform_version {
        Some(pv) => {
            // Fetch platform package metadata to get tarball URL and shasum
            let plat_url = format!(
                "{}/{}/{}",
                registry.trim_end_matches('/'),
                platform_name,
                pv
            );

            let plat_pkg: PackageVersion = client
                .get(&plat_url)
                .header("Accept", "application/json")
                .send()
                .await
                .with_context(|| {
                    format!("failed to fetch platform package metadata from {}", plat_url)
                })?
                .error_for_status()
                .with_context(|| {
                    format!(
                        "platform package {}@{} not found in registry",
                        platform_name, pv
                    )
                })?
                .json()
                .await
                .with_context(|| "failed to parse platform package metadata JSON")?;

            let dist = plat_pkg.dist.ok_or_else(|| {
                anyhow::anyhow!(
                    "platform package {}@{} has no dist info",
                    platform_name,
                    pv
                )
            })?;

            Ok(ResolvedPackage {
                version: resolved_version,
                tarball_url: dist.tarball,
                shasum: dist.shasum,
            })
        }
        None => {
            bail!(
                "version {} predates native binary distribution and requires Node.js + npm.\n\
                 Use a version >= 2.1.113, or see 'ccvm install --help' for fallback options.",
                resolved_version
            );
        }
    }
}

async fn resolve_latest(client: &reqwest::Client, registry: &str) -> Result<String> {
    let url = format!(
        "{}/@anthropic-ai/claude-code",
        registry.trim_end_matches('/')
    );

    let packument: Packument = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .with_context(|| format!("failed to fetch packument from {}", url))?
        .error_for_status()
        .with_context(|| format!("failed to access registry at {}", registry))?
        .json()
        .await
        .with_context(|| "failed to parse packument JSON")?;

    packument
        .dist_tags
        .get("latest")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no 'latest' tag found in registry"))
}
