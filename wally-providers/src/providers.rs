use reqwest::Url;

pub mod wallhaven;

pub trait WallpaperProvider {
    /// Query the wallpaper provider for a list of wallpaper source urls
    fn list(&self) -> impl Future<Output = Vec<Url>>;

    /// Retreive a random wallpaper url
    fn random(&self) -> impl Future<Output = anyhow::Result<Url>>;
}
