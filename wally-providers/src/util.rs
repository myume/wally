use std::path::{Path, PathBuf};

use anyhow::Context;
use reqwest::Url;
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn download_wallpaper(source: &Url, dest: &Path) -> anyhow::Result<PathBuf> {
    let image_bytes = reqwest::get(source.clone()).await?.bytes().await?;

    let filename = source
        .path_segments()
        .context("Could not get filename")?
        .next_back()
        .context("Missing filename")?;

    let output_path = dest.join(filename);
    save_wallpaper(&image_bytes, &output_path).await?;
    Ok(output_path)
}

pub async fn save_wallpaper(image: &[u8], dest: &Path) -> anyhow::Result<()> {
    let mut file = File::create(dest)
        .await
        .context("Failed to create output file")?;

    file.write_all(image)
        .await
        .context("Failed to download wallpaper to file")?;

    Ok(())
}
