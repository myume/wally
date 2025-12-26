use std::path::{Path, PathBuf};

use reqwest::Url;

pub mod pixiv;
pub mod wallhaven;

pub trait WallpaperProvider {
    /// Query the wallpaper provider for a list of wallpaper source urls
    fn list(&self, limit: u32) -> impl Future<Output = anyhow::Result<Vec<Url>>>;

    /// Retreive a random wallpaper url
    fn random(&self) -> impl Future<Output = anyhow::Result<Url>>;

    /// Download the wallpaper from the url to the specified destination folder.
    fn download(&self, source: &Url, dest: &Path) -> impl Future<Output = anyhow::Result<PathBuf>>;
}
