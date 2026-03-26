use anyhow::{Context, anyhow};
use reqwest::Url;
use std::{
    fs::remove_file,
    path::{Path, PathBuf},
    process::Command,
};
use tokio::time::{Duration, sleep};

use clap::{Parser, Subcommand, ValueEnum};
use wally_providers::providers::{
    WallpaperProvider, konachan::Konachan, pixiv::Pixiv, wallhaven::Wallhaven,
};

macro_rules! retry {
    ($logic:expr, $num_retries:expr, $backoff:expr) => {{
        let mut retries = $num_retries;
        let mut backoff = $backoff;
        loop {
            match $logic.await {
                Ok(val) => break Ok(val),
                Err(e) => {
                    retries -= 1;
                    if retries <= 0 {
                        break Err(anyhow::anyhow!(
                            "Operation failed after {} attempts: {}",
                            $num_retries,
                            e
                        ));
                    }
                    eprintln!("Error: {}. Retrying... ({} left)", e, retries);
                    sleep(Duration::from_millis(backoff)).await;
                    backoff *= 2;
                }
            }
        }
    }};
}

/// Wally the wallpaper scraper
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
    #[arg(short, long)]
    config: PathBuf,

    /// The path to save wallpapers to
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// The source to choose a wallpaper from. If unspecified, a random source is chosen
    #[arg(short, long)]
    source: Option<WallpaperSource>,

    /// Set the wallpaper. If there are multiple wallpapers, randomly choose one. By default,
    /// setting a wallpaper will save it as well.
    #[arg(long)]
    set_wallpaper: bool,

    /// Delete the oldest wallpapers if saving a new wallpaper to the output dir would result in more
    /// files than the max_downloaded specified in the config
    #[arg(long)]
    evict_oldest: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand)]
enum Mode {
    Random,
    List {
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum WallpaperSource {
    Wallhaven,
    Pixiv,
    Konachan,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let config = match wally_config::read_config(&args.config) {
        Ok(config) => config,
        Err(e) => return Err(anyhow!("{:?}", e.wrap_err("Failed to read config"))),
    };

    let output_dir = args.output_path.unwrap_or(config.general.output_dir.value);
    if !output_dir.exists() {
        return Err(anyhow!(
            "Wallpaper output dir {} does not exist",
            output_dir.display()
        ));
    }

    let all_sources = WallpaperSource::value_variants();
    let source = args
        .source
        .unwrap_or(all_sources[rand::random_range(..all_sources.len())]);

    eprintln!("Pulling wallpapers from {:?}", source);

    let provider: Box<dyn WallpaperProvider> = match source {
        WallpaperSource::Wallhaven => Box::new(Wallhaven::new(config.wallhaven)),
        WallpaperSource::Pixiv => Box::new(Pixiv::new()),
        WallpaperSource::Konachan => Box::new(Konachan::new(config.konachan)),
    };

    let wallpaper_urls = match args.mode {
        Mode::Random => vec![retry!(provider.random(), 5, 1000)?],
        Mode::List { limit } => retry!(provider.list(limit), 5, 1000)?,
    };

    if args.save || args.set_wallpaper {
        eprintln!("Saving wallpapers to {}", output_dir.display());
        let image_paths = download_wallpapers(wallpaper_urls, provider, &output_dir).await;
        if args.set_wallpaper {
            let selected_image = &image_paths[rand::random_range(..image_paths.len())];
            set_wallpaper(&config.general.set_command.command, selected_image)
                .context("Failed to set wallpaper")?
        }
    } else {
        wallpaper_urls.iter().for_each(|url| println!("{url}"));
    }

    if args.evict_oldest {
        evict_oldest(&output_dir, config.general.max_downloaded.value as usize)
            .context("Failed to evict wallpapers")?;
    }

    Ok(())
}

fn evict_oldest(output_dir: &Path, max_downloaded: usize) -> anyhow::Result<()> {
    let mut wallpaper_files = Vec::new();
    for entry in output_dir.read_dir().context("failed to read output dir")? {
        let entry = entry.context("failed to read wallpaper file")?;
        let metadata = entry.metadata().context("could not read file metadata")?;
        wallpaper_files.push((
            entry.path(),
            metadata
                .modified()
                .context("could not read wallpaper modified time")?,
        ));
    }

    if wallpaper_files.len() > max_downloaded {
        eprintln!("Evicting extra files");
        wallpaper_files.sort_by(|a, b| a.1.cmp(&b.1));
        let files_to_evict = &wallpaper_files[..wallpaper_files.len() - max_downloaded];
        for (path, _) in files_to_evict {
            eprintln!("removing file {}", path.display());
            remove_file(path).context("failed to remove file")?;
        }
    }

    Ok(())
}

async fn download_wallpapers(
    wallpaper_urls: Vec<Url>,
    provider: Box<dyn WallpaperProvider>,
    output_dir: &Path,
) -> Vec<PathBuf> {
    let mut downloaded_images = Vec::new();
    for url in wallpaper_urls {
        eprintln!("Downloading wallpaper from {url}");
        match retry!(provider.download(&url, output_dir), 5, 1000) {
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
