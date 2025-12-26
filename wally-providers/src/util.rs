use std::path::Path;

use anyhow::Context;
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn save_wallpaper(image: &[u8], dest: &Path) -> anyhow::Result<()> {
    let mut file = File::create(dest)
        .await
        .context("Failed to create output file")?;

    file.write_all(image)
        .await
        .context("Failed to download wallpaper to file")?;

    Ok(())
}
