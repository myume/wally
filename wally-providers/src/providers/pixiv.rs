use std::path::{Path, PathBuf};

use anyhow::Context;
use async_trait::async_trait;
use regex::Regex;
use reqwest::{Client, Url, header::REFERER};
use serde::{Deserialize, Serialize};
use tokio::{join, task::JoinSet};

use crate::{providers::WallpaperProvider, util::save_wallpaper};

const PIXIV_BASE_URL: &str = "https://www.pixiv.net/ranking.php";
const ITEMS_PER_PAGE: usize = 50;

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
        limit: usize,
        query_string: &str,
    ) -> anyhow::Result<Vec<PixivContent>> {
        let mut handles = JoinSet::new();

        let client = Client::new();
        for page in 1..=limit.div_ceil(ITEMS_PER_PAGE) {
            let query_string = query_string.to_owned();
            let client = client.clone();
            handles.spawn(async move {
                client
                    .get(format!("{PIXIV_BASE_URL}?{query_string}&p={page}"))
                    .send()
                    .await?
                    .json::<PixivResponse>()
                    .await
                    .context("Unable to parse pixiv response into json")
            });
        }

        let mut wallpapers = Vec::new();
        for handle in handles.join_all().await {
            wallpapers.extend(handle?.contents);
        }

        wallpapers.truncate(limit);
        Ok(wallpapers)
    }
}

impl Default for Pixiv {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WallpaperProvider for Pixiv {
    async fn list(&self, limit: usize) -> anyhow::Result<Vec<reqwest::Url>> {
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

    async fn download(&self, source: &Url, dest: &Path) -> anyhow::Result<PathBuf> {
        let regex = Regex::new(r"c/\d+x\d+/img-master").expect("invalid regex");
        let url = regex
            .replace(source.as_str(), "img-original")
            .replace("_master1200", "");

        let client = Client::new();

        let (jpg, png) = join!(
            client.get(&url).header(REFERER, PIXIV_BASE_URL).send(),
            client
                .get(&url)
                .header(REFERER, PIXIV_BASE_URL.replace(".jpg", ".png"))
                .send()
        );

        let response = jpg?
            .error_for_status()
            .or(png?.error_for_status())
            .context("Failed to fetch image")?;
        let image_bytes = response.bytes().await?;
        let filename = url
            .split("/")
            .last()
            .context("unable to extract filename from url")?;

        let output_path = dest.join(filename);
        save_wallpaper(&image_bytes, &output_path).await?;
        Ok(output_path)
    }
}
