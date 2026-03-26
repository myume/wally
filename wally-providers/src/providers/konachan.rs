use std::path::{Path, PathBuf};

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use wally_config::konachan::KonachanConfig;

use crate::{providers::WallpaperProvider, util::download_wallpaper};

const KONACHAN_BASE_URL: &str = "https://konachan.net";
const MAX_PAGE_LIMIT: usize = 100;

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

    async fn fetch_list(&self, limit: usize) -> anyhow::Result<Vec<KonachanItem>> {
        let mut handles = JoinSet::new();

        let mut base_url = Url::parse(&format!(
            "{KONACHAN_BASE_URL}/post.json?limit={}",
            limit.min(MAX_PAGE_LIMIT)
        ))?;
        if !self.config.explicit.value {
            base_url.query_pairs_mut().append_pair("tags", "rating:s");
        }

        let client = Client::new();
        for page in 1..=limit.div_ceil(limit.min(MAX_PAGE_LIMIT)) {
            let mut url = base_url.clone();
            url.query_pairs_mut().append_pair("page", &page.to_string());
            let client = client.clone();
            handles.spawn(async move {
                client
                    .get(url)
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<Vec<KonachanItem>>()
                    .await
            });
        }

        let mut wallpapers = Vec::new();
        for handle in handles.join_all().await {
            wallpapers.extend(handle?);
        }

        wallpapers.truncate(limit);
        Ok(wallpapers)
    }
}

#[async_trait]
impl WallpaperProvider for Konachan {
    async fn list(&self, limit: usize) -> anyhow::Result<Vec<Url>> {
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
