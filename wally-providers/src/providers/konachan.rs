use std::path::{Path, PathBuf};

use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{providers::WallpaperProvider, util::download_wallpaper};

const KONACHAN_BASE_URL: &str = "https://konachan.net";

#[derive(Debug, Serialize, Deserialize)]
struct KonachanItem {
    id: u32,
    author: String,
    file_url: Url,
    width: u32,
    height: u32,
}

pub struct Konachan {}

impl Konachan {
    pub fn new() -> Self {
        Self {}
    }

    async fn fetch_list(&self, limit: u32) -> anyhow::Result<Vec<KonachanItem>> {
        Ok(
            reqwest::get(format!("{KONACHAN_BASE_URL}/post.json?limit={limit}"))
                .await?
                .json()
                .await?,
        )
    }
}

impl Default for Konachan {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WallpaperProvider for Konachan {
    async fn list(&self, limit: u32) -> anyhow::Result<Vec<Url>> {
        Ok(self
            .fetch_list(limit)
            .await?
            .into_iter()
            .map(|item| item.file_url)
            .collect())
    }

    async fn random(&self) -> anyhow::Result<Url> {
        let wallpapers = self.fetch_list(100).await?;
        Ok(wallpapers[rand::random_range(..wallpapers.len())]
            .file_url
            .clone())
    }

    async fn download(&self, source: &Url, dest: &Path) -> anyhow::Result<PathBuf> {
        download_wallpaper(source, dest).await
    }
}
