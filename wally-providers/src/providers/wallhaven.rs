use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use wally_config::wallhaven::WallhavenConfig;

use crate::{providers::WallpaperProvider, util::save_wallpaper};

const WALLHAVEN_API_URL: &str = "https://wallhaven.cc/api/v1/search";
const ITEMS_PER_PAGE: u32 = 24;

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

    async fn fetch_list(&self, limit: u32) -> anyhow::Result<Vec<WallhavenData>> {
        let mut handles = Vec::new();

        for page in 1..=limit.div_ceil(ITEMS_PER_PAGE) {
            let category = self.category_bits();
            let handle: JoinHandle<anyhow::Result<WallhavenResponse>> = tokio::spawn(async move {
                reqwest::get(format!(
                    "{WALLHAVEN_API_URL}?page={page}&categories={category}"
                ))
                .await?
                .json()
                .await
                .context("Unable to parse wallhaven response into json")
            });
            handles.push(handle);
        }

        let mut wallpaper_list = Vec::new();
        for handle in handles {
            let response = handle.await.context("Failed to fire request")??;
            wallpaper_list.extend(response.data)
        }

        Ok(wallpaper_list.into_iter().take(limit as usize).collect())
    }
}

#[async_trait]
impl WallpaperProvider for Wallhaven {
    async fn list(&self, limit: u32) -> anyhow::Result<Vec<Url>> {
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
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use wally_config::{util::KdlBool, wallhaven::WallhavenCategories};

    use super::*;

    const CONFIG: WallhavenConfig = WallhavenConfig {
        categories: WallhavenCategories {
            general: KdlBool { value: true },
            anime: KdlBool { value: true },
            people: KdlBool { value: true },
        },
    };

    #[ignore]
    #[tokio::test]
    async fn test_list_wallpapers() {
        let provider = Wallhaven::new(CONFIG);
        let limit = 50;
        let list = provider.list(limit).await;
        assert!(list.is_ok(), "{:?}", list);
        assert!(list.unwrap().len() == limit as usize);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_random_wallpaper() {
        let provider = Wallhaven::new(CONFIG);
        let url = provider.random().await;
        assert!(url.is_ok(), "{:?}", url);
    }

    #[ignore]
    #[tokio::test]
    async fn test_download() {
        let provider = Wallhaven::new(CONFIG);
        let source = provider.random().await.unwrap();
        let dir = tempdir().expect("Should create a tempdir");
        let filepath = provider.download(&source, dir.path()).await.unwrap();
        assert!(filepath.exists());
    }
}
