use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use wally_providers::providers::{WallpaperProvider, pixiv::Pixiv, wallhaven::Wallhaven};

/// Wally the wallpaper picker
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Choose a random wallpaper from the source
    #[arg(long)]
    mode: Mode,

    /// Save the wallpaper to the the specific output path, otherwise the wallpaper is saved in the
    /// default location
    #[arg(long)]
    save: bool,

    /// The location of the config file
    #[arg(long, default_value = "./wally.kdl")]
    config: PathBuf,

    /// The path to save wallpapers to
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// The source to choose a wallpaper from. If unspecified, a random source is chosen
    #[arg(short, long)]
    source: Option<WallpaperSource>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Random,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum WallpaperSource {
    Wallhaven,
    Pixiv,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let config = match wally_config::read_config(&args.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{:?}", e.wrap_err("Failed to read config"));
            return;
        }
    };

    let all_sources = WallpaperSource::value_variants();
    let source = args
        .source
        .unwrap_or(all_sources[rand::random_range(..all_sources.len())]);

    let provider: Box<dyn WallpaperProvider> = match source {
        WallpaperSource::Wallhaven => Box::new(Wallhaven::new()),
        WallpaperSource::Pixiv => Box::new(Pixiv::new()),
    };

    let wallpaper_urls = match args.mode {
        Mode::Random => {
            let random = provider.random().await;
            match random {
                Ok(url) => vec![url],
                Err(e) => {
                    eprintln!("Failed to fetch random wallpaper: {e}");
                    return;
                }
            }
        }
    };
}
