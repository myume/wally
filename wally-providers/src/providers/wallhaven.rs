use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use wally_config::wallhaven::WallhavenConfig;

use crate::{providers::WallpaperProvider, util::download_wallpaper};

const WALLHAVEN_API_URL: &str = "https://wallhaven.cc/api/v1/search";
const ITEMS_PER_PAGE: usize = 24;

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenData {
    id: String,
    dimension_x: u32,
    dimension_y: u32,
    resolution: String,
    file_size: u32,
    file_type: String,
    path: Url,
}

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenMeta {
    current_page: u32,
    last_page: u32,
    per_page: u32,
    total: u32,
    query: Option<String>,
    seed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenResponse {
    data: Vec<WallhavenData>,
    meta: WallhavenMeta,
}

pub struct Wallhaven {
    config: WallhavenConfig,
}

impl Wallhaven {
    pub fn new(config: WallhavenConfig) -> Self {
        Self { config }
    }

    fn category_bits(&self) -> String {
        let mut bits = String::new();
        if self.config.categories.general.value {
            bits.push('1');
        } else {
            bits.push('0');
        }

        if self.config.categories.anime.value {
            bits.push('1');
        } else {
            bits.push('0');
        }

        if self.config.categories.people.value {
            bits.push('1');
        } else {
            bits.push('0');
        }

        bits
    }

    async fn fetch_list(&self, limit: usize) -> anyhow::Result<Vec<WallhavenData>> {
        let mut handles = JoinSet::new();

        let client = Client::new();
        for page in 1..=limit.div_ceil(ITEMS_PER_PAGE) {
            let category = self.category_bits();
            let client = client.clone();
            handles.spawn(async move {
                client
                    .get(format!(
                        "{WALLHAVEN_API_URL}?page={page}&categories={category}"
                    ))
                    .send()
                    .await?
                    .json::<WallhavenResponse>()
                    .await
                    .context("Unable to parse wallhaven response into json")
            });
        }

        let mut wallpapers = Vec::new();
        for handle in handles.join_all().await {
            wallpapers.extend(handle?.data)
        }

        wallpapers.truncate(limit);
        Ok(wallpapers)
    }
}

#[async_trait]
impl WallpaperProvider for Wallhaven {
    async fn list(&self, limit: usize) -> anyhow::Result<Vec<Url>> {
        Ok(self
            .fetch_list(limit)
            .await?
            .into_iter()
            .map(|data| data.path)
            .collect())
    }

    async fn random(&self) -> anyhow::Result<Url> {
        let wallpaper_list = self.fetch_list(100).await?;

        let selected = rand::random_range(0..wallpaper_list.len());

        Ok(wallpaper_list
            .get(selected)
            .map(|data| data.path.clone())
            .context("selected item does not exist")?
            .clone())
    }

    async fn download(&self, source: &Url, dest: &Path) -> anyhow::Result<PathBuf> {
        download_wallpaper(source, dest).await
    }
}
