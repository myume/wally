use std::{
    fs,
    path::PathBuf,
    process::{Command, ExitCode},
};

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
    Random {
        #[arg(long)]
        set_wallpaper: bool,
    },
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

    eprintln!("pulling wallpapers from {:?}", source);

    let provider: Box<dyn WallpaperProvider> = match source {
        WallpaperSource::Wallhaven => Box::new(Wallhaven::new()),
        WallpaperSource::Pixiv => Box::new(Pixiv::new()),
    };

    let wallpaper_urls = match args.mode {
        Mode::Random { .. } => match provider.random().await {
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

    let output_dir = config.general.output_dir.value;
    if args.save {
        if !output_dir.exists() {
            eprintln!("wallpaper output dir does not exist, creating dir...");
            if let Err(e) = fs::create_dir(&output_dir) {
                eprintln!("Failed to create output dir: {e}");
            }
        }
        eprintln!("saving wallpapers to {}", output_dir.display());
    }

    for url in wallpaper_urls {
        if args.save {
            eprintln!("downloading wallpaper from {url}");

            match provider.download(&url, &output_dir).await {
                Ok(path) => {
                    if let Mode::Random { set_wallpaper } = args.mode
                        && set_wallpaper
                    {
                        let command = config.general.set_command.command.replace(
                            "{{path}}",
                            path.to_str().expect("path should be valid string"),
                        );
                        let parts: Vec<&str> = command.split(" ").collect();

                        let Some(program) = parts.first() else {
                            eprintln!("Missing program in set command");
                            return ExitCode::FAILURE;
                        };

                        eprintln!("Setting wallpaper with command \"{command}\"");
                        if let Err(e) = Command::new(program).args(&parts[1..]).output() {
                            eprintln!("Failed to set wallpaper: {e}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
                Err(e) => eprintln!("Failed to download wallpaper from {url}: {e}"),
            }
        } else {
            println!("{}", url);
        };
    }

    ExitCode::SUCCESS
}
