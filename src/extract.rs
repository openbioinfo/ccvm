use anyhow::{bail, Context, Result};
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub fn extract_and_verify(
    tgz_path: &Path,
    expected_shasum: &str,
    version: &str,
) -> Result<PathBuf> {
    let versions_dir = crate::config::versions_dir();
    let dest_dir = versions_dir.join(version);
    let dest_binary = dest_dir.join("claude.exe");

    // Skip if already installed
    if dest_binary.exists() {
        println!(
            "version {} is already installed at {}",
            version,
            dest_dir.display()
        );
        return Ok(dest_binary);
    }

    // Verify tarball integrity with SHA-1 (npm dist.shasum)
    let tgz_data =
        fs::read(tgz_path).with_context(|| format!("failed to read {}", tgz_path.display()))?;
    let mut hasher = Sha1::new();
    hasher.update(&tgz_data);
    let tgz_hash = hex::encode(hasher.finalize());

    if !tgz_hash.eq_ignore_ascii_case(expected_shasum) {
        fs::remove_file(tgz_path).ok();
        bail!(
            "checksum mismatch\n  expected: {}\n  got:      {}\n  cached file deleted, try again",
            expected_shasum,
            tgz_hash
        );
    }

    // Create version directory
    fs::create_dir_all(&dest_dir)
        .with_context(|| format!("failed to create version directory: {}", dest_dir.display()))?;

    // Decompress tgz and extract package/claude.exe
    let decoder = flate2::read::GzDecoder::new(&tgz_data[..]);
    let mut archive = tar::Archive::new(decoder);

    let mut binary_data: Option<Vec<u8>> = None;

    for entry in archive
        .entries()
        .with_context(|| "failed to read tar entries")?
    {
        let mut entry = entry.with_context(|| "failed to read tar entry")?;
        let path = entry
            .path()
            .with_context(|| "failed to read tar entry path")?;
        let path_str = path.to_string_lossy();

        if path_str == "package/claude.exe"
            || path_str == "package/claude"
            || path_str.ends_with("/package/claude.exe")
            || path_str.ends_with("/package/claude")
        {
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .with_context(|| "failed to read binary from tar")?;
            binary_data = Some(data);
            break;
        }
    }

    let binary_data =
        binary_data.ok_or_else(|| anyhow::anyhow!("claude binary not found in package"))?;

    // Write binary to versions directory
    fs::write(&dest_binary, &binary_data)
        .with_context(|| format!("failed to write {}", dest_binary.display()))?;

    println!(
        "installed claude-code {} to {}",
        version,
        dest_dir.display()
    );

    Ok(dest_binary)
}

pub fn extract_codex_and_verify(
    tgz_path: &Path,
    expected_shasum: &str,
    version: &str,
) -> Result<PathBuf> {
    let target_triple = crate::platform::codex_target_triple()?;
    let binary_name = crate::platform::executable_name("codex");
    let dest_dir = crate::config::codex_versions_dir().join(version);
    let dest_binary = dest_dir
        .join("vendor")
        .join(target_triple)
        .join("bin")
        .join(binary_name);

    if dest_binary.exists() {
        println!(
            "codex version {} is already installed at {}",
            version,
            dest_dir.display()
        );
        return Ok(dest_binary);
    }

    verify_shasum(tgz_path, expected_shasum)?;

    fs::create_dir_all(&dest_dir)
        .with_context(|| format!("failed to create version directory: {}", dest_dir.display()))?;

    let tgz_file = fs::File::open(tgz_path)
        .with_context(|| format!("failed to open {}", tgz_path.display()))?;
    let decoder = flate2::read::GzDecoder::new(tgz_file);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive
        .entries()
        .with_context(|| "failed to read tar entries")?
    {
        let mut entry = entry.with_context(|| "failed to read tar entry")?;
        let path = entry
            .path()
            .with_context(|| "failed to read tar entry path")?;
        let relative = match path.strip_prefix("package") {
            Ok(path) if path.as_os_str().is_empty() => continue,
            Ok(path) => path.to_path_buf(),
            Err(_) => continue,
        };
        let output_path = safe_join(&dest_dir, &relative)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }
        let entry_type = entry.header().entry_type();
        if !(entry_type.is_file() || entry_type.is_dir()) {
            continue;
        }
        entry
            .unpack(&output_path)
            .with_context(|| format!("failed to unpack {}", output_path.display()))?;
    }

    if !dest_binary.exists() {
        fs::remove_dir_all(&dest_dir).ok();
        bail!(
            "codex binary not found after extraction: {}",
            dest_binary.display()
        );
    }

    println!("installed codex {} to {}", version, dest_dir.display());

    Ok(dest_binary)
}

fn verify_shasum(tgz_path: &Path, expected_shasum: &str) -> Result<()> {
    let tgz_data =
        fs::read(tgz_path).with_context(|| format!("failed to read {}", tgz_path.display()))?;
    let mut hasher = Sha1::new();
    hasher.update(&tgz_data);
    let tgz_hash = hex::encode(hasher.finalize());

    if !tgz_hash.eq_ignore_ascii_case(expected_shasum) {
        fs::remove_file(tgz_path).ok();
        bail!(
            "checksum mismatch\n  expected: {}\n  got:      {}\n  cached file deleted, try again",
            expected_shasum,
            tgz_hash
        );
    }

    Ok(())
}

fn safe_join(base: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.components().any(|c| {
        matches!(
            c,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        )
    }) {
        bail!("refusing to unpack unsafe tar path: {}", relative.display());
    }

    Ok(base.join(relative))
}
