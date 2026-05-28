use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
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

pub async fn resolve_package(registry: &str, version: &str) -> Result<ResolvedPackage> {
    resolve_claude_package(registry, version).await
}

pub async fn resolve_claude_package(registry: &str, version: &str) -> Result<ResolvedPackage> {
    let platform_name = format!(
        "@anthropic-ai/claude-code-{}",
        crate::platform::npm_platform_key()?
    );
    resolve_native_package(
        registry,
        "@anthropic-ai/claude-code",
        &platform_name,
        version,
        "claude-code",
    )
    .await
}

pub async fn resolve_codex_package(registry: &str, version: &str) -> Result<ResolvedPackage> {
    let platform_name = format!("@openai/codex-{}", crate::platform::npm_platform_key()?);
    resolve_native_package(registry, "@openai/codex", &platform_name, version, "codex").await
}

async fn resolve_native_package(
    registry: &str,
    main_package: &str,
    platform_dependency_name: &str,
    version: &str,
    display_name: &str,
) -> Result<ResolvedPackage> {
    let client = reqwest::Client::new();

    // Resolve "latest" to actual version number
    let resolved_version = if version == "latest" {
        resolve_latest(&client, registry, main_package).await?
    } else {
        version.to_string()
    };

    // Fetch the main package metadata
    let main_url = format!(
        "{}/{}/{}",
        registry.trim_end_matches('/'),
        main_package,
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
    let platform_version = main_pkg
        .optional_dependencies
        .as_ref()
        .and_then(|deps| deps.get(platform_dependency_name))
        .cloned();

    match platform_version {
        Some(pv) => {
            let (platform_package, pv) = parse_platform_spec(platform_dependency_name, &pv);
            // Fetch platform package metadata to get tarball URL and shasum
            let plat_url = format!(
                "{}/{}/{}",
                registry.trim_end_matches('/'),
                platform_package,
                pv
            );

            let plat_pkg: PackageVersion = client
                .get(&plat_url)
                .header("Accept", "application/json")
                .send()
                .await
                .with_context(|| {
                    format!(
                        "failed to fetch platform package metadata from {}",
                        plat_url
                    )
                })?
                .error_for_status()
                .with_context(|| {
                    format!(
                        "platform package {}@{} not found in registry",
                        platform_dependency_name, pv
                    )
                })?
                .json()
                .await
                .with_context(|| "failed to parse platform package metadata JSON")?;

            let dist = plat_pkg.dist.ok_or_else(|| {
                anyhow::anyhow!(
                    "platform package {}@{} has no dist info",
                    platform_dependency_name,
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
            if display_name == "claude-code" {
                bail!(
                    "version {} predates native binary distribution and requires Node.js + npm.\n\
                     Use a version >= 2.1.113, or see 'ccvm install --help' for fallback options.",
                    resolved_version
                );
            }
            bail!(
                "{} version {} has no native package for {}",
                display_name,
                resolved_version,
                platform_dependency_name
            );
        }
    }
}

fn parse_platform_spec<'a>(
    platform_dependency_name: &'a str,
    version_spec: &'a str,
) -> (&'a str, &'a str) {
    if let Some(alias) = version_spec.strip_prefix("npm:") {
        if let Some((package_name, version)) = alias.rsplit_once('@') {
            return (package_name, version);
        }
    }

    if let Some((_, version)) = version_spec.rsplit_once('@') {
        (platform_dependency_name, version)
    } else {
        (platform_dependency_name, version_spec)
    }
}

async fn resolve_latest(
    client: &reqwest::Client,
    registry: &str,
    package_name: &str,
) -> Result<String> {
    let url = format!("{}/{}", registry.trim_end_matches('/'), package_name);

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
