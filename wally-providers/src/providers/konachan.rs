use std::path::{Path, PathBuf};

use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use wally_config::konachan::KonachanConfig;

use crate::{providers::WallpaperProvider, util::download_wallpaper};

const KONACHAN_BASE_URL: &str = "https://konachan.net";

#[derive(Debug, Serialize, Deserialize)]
struct KonachanItem {
    id: u32,
    author: String,
    file_url: Url,
    width: u32,
    height: u32,
    rating: String,
}

pub struct Konachan {
    config: KonachanConfig,
}

impl Konachan {
    pub fn new(config: KonachanConfig) -> Self {
        Self { config }
    }

    async fn fetch_list(&self, limit: u32) -> anyhow::Result<Vec<KonachanItem>> {
        let mut wallpapers = Vec::new();
        let mut page = 1;
        while wallpapers.len() < limit as usize {
            let mut response: Vec<KonachanItem> = reqwest::get(format!(
                "{KONACHAN_BASE_URL}/post.json?limit={}&page={page}",
                limit.min(100)
            ))
            .await?
            .error_for_status()?
            .json()
            .await?;

            if !self.config.explicit.value {
                response.retain(|item| item.rating == "s");
            }

            wallpapers.extend(response);
            page += 1;
        }

        Ok(wallpapers.into_iter().take(limit as usize).collect())
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
        let wallpapers = self.fetch_list(500).await?;
        Ok(wallpapers[rand::random_range(..wallpapers.len())]
            .file_url
            .clone())
    }

    async fn download(&self, source: &Url, dest: &Path) -> anyhow::Result<PathBuf> {
        download_wallpaper(source, dest).await
    }
}
