use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

pub async fn download_tarball(url: &str, dest: &Path) -> Result<PathBuf> {
    // Skip if already cached
    if dest.exists() {
        return Ok(dest.to_path_buf());
    }

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache directory: {}", parent.display()))?;
    }

    let client = reqwest::Client::new();
    let mut response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to connect to {}", url))?
        .error_for_status()
        .with_context(|| format!("download failed for {}", url))?;

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{bytes}/{total_bytes} ({bytes_per_sec}) [{elapsed_precise}] {wide_bar:.cyan/blue}",
            )
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut file =
        std::fs::File::create(dest).with_context(|| format!("failed to create {}", dest.display()))?;
    let mut downloaded: u64 = 0;

    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                std::io::Write::write_all(&mut file, &chunk)
                    .with_context(|| format!("failed to write to {}", dest.display()))?;
                downloaded += chunk.len() as u64;
                pb.set_position(downloaded);
            }
            Ok(None) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("download interrupted: {}", e));
            }
        }
    }

    pb.finish_with_message("done");
    Ok(dest.to_path_buf())
}
