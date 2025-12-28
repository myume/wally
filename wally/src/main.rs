use anyhow::{Context, anyhow};
use reqwest::Url;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Parser, Subcommand, ValueEnum};
use wally_providers::providers::{
    WallpaperProvider, konachan::Konachan, pixiv::Pixiv, wallhaven::Wallhaven,
};

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

    /// Set the wallpaper. If there are multiple wallpapers, randomly choose one.
    #[arg(long)]
    set_wallpaper: bool,
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
    Konachan,
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

    eprintln!("pulling wallpapers from {:?}", source);

    let provider: Box<dyn WallpaperProvider> = match source {
        WallpaperSource::Wallhaven => Box::new(Wallhaven::new(config.wallhaven)),
        WallpaperSource::Pixiv => Box::new(Pixiv::new()),
        WallpaperSource::Konachan => Box::new(Konachan::new(config.konachan)),
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

    let output_dir = args.output_path.unwrap_or(config.general.output_dir.value);
    if args.save {
        if !output_dir.exists() {
            eprintln!("wallpaper output dir does not exist, creating dir...");
            if let Err(e) = fs::create_dir(&output_dir) {
                eprintln!("Failed to create output dir: {e}");
            }
        }
        eprintln!("saving wallpapers to {}", output_dir.display());
    }

    if args.save {
        let image_paths = download_wallpapers(wallpaper_urls, provider, &output_dir).await;
        if args.set_wallpaper {
            let selected_image = &image_paths[rand::random_range(..image_paths.len())];
            if let Err(e) = set_wallpaper(&config.general.set_command.command, selected_image) {
                println!("Failed to set wallpaper: {e}");
            }
        }
    } else {
        wallpaper_urls.iter().for_each(|url| println!("{url}"));
    }

    ExitCode::SUCCESS
}

async fn download_wallpapers(
    wallpaper_urls: Vec<Url>,
    provider: Box<dyn WallpaperProvider>,
    output_dir: &Path,
) -> Vec<PathBuf> {
    let mut downloaded_images = Vec::new();
    for url in wallpaper_urls {
        eprintln!("downloading wallpaper from {url}");
        match provider.download(&url, output_dir).await {
            Ok(path) => downloaded_images.push(path),
            Err(e) => eprintln!("Failed to download wallpaper from {url}: {e}"),
        }
    }
    downloaded_images
}

fn set_wallpaper(command: &str, img_path: &Path) -> anyhow::Result<()> {
    let command = command.replace(
        "{{path}}",
        img_path.to_str().expect("path should be valid string"),
    );
    let parts: Vec<&str> = command.split(" ").collect();

    let Some(program) = parts.first() else {
        return Err(anyhow!("Missing program in set command"));
    };

    eprintln!("Setting wallpaper with command \"{command}\"");
    Command::new(program)
        .args(&parts[1..])
        .output()
        .context("Failed to set wallpaper")?;
    Ok(())
}
