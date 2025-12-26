use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use wally_providers::providers::{WallpaperProvider, pixiv::Pixiv, wallhaven::Wallhaven};

/// Wally the wallpaper picker
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Choose a random wallpaper from the source
    #[arg(long)]
    random: bool,

    /// Save the wallpaper to the the specific output path, otherwise the wallpaper is saved in the
    /// default location
    #[arg(long)]
    save: bool,

    /// The path to save wallpapers to
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// The source to choose a wallpaper from. If unspecified, a random source is chosen
    #[arg(short, long)]
    source: Option<WallpaperSource>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum WallpaperSource {
    Wallhaven,
    Pixiv,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let all_sources = WallpaperSource::value_variants();
    let source = args
        .source
        .unwrap_or(all_sources[rand::random_range(..all_sources.len())]);

    let provider: Box<dyn WallpaperProvider> = match source {
        WallpaperSource::Wallhaven => Box::new(Wallhaven::new()),
        WallpaperSource::Pixiv => Box::new(Pixiv::new()),
    };

    if args.random {
        let random = provider.random().await;
        match random {
            Ok(url) => println!("{url}"),
            Err(e) => eprintln!("Failed to fetch random wallpaper: {e}"),
        }
    }
}
