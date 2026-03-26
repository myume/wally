use std::path::{Path, PathBuf};

use async_trait::async_trait;
use reqwest::Url;

pub mod konachan;
pub mod pixiv;
pub mod wallhaven;

#[async_trait]
pub trait WallpaperProvider {
    /// Query the wallpaper provider for a list of wallpaper source urls
    async fn list(&self, limit: usize) -> anyhow::Result<Vec<Url>>;

    /// Retreive a random wallpaper url
    async fn random(&self) -> anyhow::Result<Url>;

    /// Download the wallpaper from the url to the specified destination folder.
    async fn download(&self, source: &Url, dest: &Path) -> anyhow::Result<PathBuf>;
}
