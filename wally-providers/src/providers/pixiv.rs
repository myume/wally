use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::providers::WallpaperProvider;

const PIXIV_BASE_URL: &str = "https://www.pixiv.net/ranking.php";
const ITEMS_PER_PAGE: u32 = 50;

#[derive(Debug, Serialize, Deserialize)]
struct PixivResponse {
    contents: Vec<PixivContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PixivContent {
    title: String,
    url: Url,
    user_name: String,
    width: u32,
    height: u32,
}

pub struct Pixiv {}

impl Pixiv {
    pub fn new() -> Self {
        Self {}
    }

    async fn fetch_list(
        &self,
        limit: u32,
        query_string: &str,
    ) -> anyhow::Result<Vec<PixivContent>> {
        let mut handles = Vec::new();

        for page in 1..=limit.div_ceil(ITEMS_PER_PAGE) {
            let query_string = query_string.to_owned();
            let handle: JoinHandle<anyhow::Result<PixivResponse>> = tokio::spawn(async move {
                reqwest::get(format!("{PIXIV_BASE_URL}?{query_string}&p={page}"))
                    .await?
                    .json()
                    .await
                    .context("Unable to parse pixiv response into json")
            });
            handles.push(handle);
        }

        let mut wallpaper_list = Vec::new();
        for handle in handles {
            let response = handle.await.context("Failed to fire request")??;
            wallpaper_list.extend(response.contents)
        }

        Ok(wallpaper_list.into_iter().take(limit as usize).collect())
    }
}

impl Default for Pixiv {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperProvider for Pixiv {
    async fn list(&self, limit: u32) -> anyhow::Result<Vec<reqwest::Url>> {
        let wallpaper_list = self
            .fetch_list(limit, "mode=monthly&content=illust&format=json")
            .await?;

        Ok(wallpaper_list.into_iter().map(|item| item.url).collect())
    }

    async fn random(&self) -> anyhow::Result<reqwest::Url> {
        let wallpaper_list = self
            .fetch_list(50, "mode=daily&content=illust&format=json")
            .await?;

        let selected = rand::random_range(0..wallpaper_list.len());

        Ok(wallpaper_list
            .get(selected)
            .map(|data| data.url.clone())
            .context("selected item does not exist")?
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_list_wallpapers() {
        let provider = Pixiv::new();
        let limit = 100;
        let wallpapers = provider.list(limit).await;
        assert!(wallpapers.is_ok(), "{:?}", wallpapers);
        assert!(wallpapers.unwrap().len() == limit as usize);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_random_wallpaper() {
        let provider = Pixiv::new();
        let url = provider.random().await;
        assert!(url.is_ok(), "{:?}", url);
        dbg!(url.unwrap().to_string());
    }
}
