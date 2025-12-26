use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand, ValueEnum};
use wally_providers::providers::{WallpaperProvider, pixiv::Pixiv, wallhaven::Wallhaven};

/// Wally the wallpaper picker
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Choose a random wallpaper from the source
    #[command(subcommand)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand)]
enum Mode {
    Random,
    List {
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum WallpaperSource {
    Wallhaven,
    Pixiv,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Cli::parse();
    let config = match wally_config::read_config(&args.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{:?}", e.wrap_err("Failed to read config"));
            return ExitCode::FAILURE;
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
        Mode::Random => match provider.random().await {
            Ok(url) => vec![url],
            Err(e) => {
                eprintln!("Failed to fetch random wallpaper: {e}");
                return ExitCode::FAILURE;
            }
        },
        Mode::List { limit } => match provider.list(limit).await {
            Ok(list) => list,
            Err(e) => {
                eprintln!("Failed to fetch wallpapers: {e}");
                return ExitCode::FAILURE;
            }
        },
    };

    for url in wallpaper_urls {
        if args.save {
            if let Err(e) = provider
                .download(&url, &config.general.output_dir.value)
                .await
            {
                eprintln!("Failed to download wallpaper from {url}: {e}")
            }
        } else {
            println!("{}", url);
        };
    }

    ExitCode::SUCCESS
}
