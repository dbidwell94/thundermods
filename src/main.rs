mod back_dialog;
mod clean;
mod main_menu;
pub mod prelude;
mod uninstall;
mod update;
pub mod utils;

use anyhow::anyhow;
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Global state for this program's session
pub struct ProgramState {
    /// The local directory pointing to the install location of mods
    mods_dir: PathBuf,
    /// Which game this session is managing
    managed_game: String,
    /// cached packages from Thunderstore
    packages: HashMap<NamespacedPackage, SearchablePackage>,
    /// The mod requirements for this session
    requirements: Requirements,
    /// The last time the package cache was updated
    last_updated: Option<DateTime<Local>>,
}

impl ProgramState {
    fn cache_path(managed_game: &str) -> Option<PathBuf> {
        if let Ok(path) = std::fs::read_dir(CACHE_DIR.to_path_buf()) {
            for path in path {
                let Ok(path) = path else {
                    continue;
                };

                let Ok(file_name) = path.file_name().into_string() else {
                    continue;
                };

                let splits = file_name.split('_').collect::<Vec<_>>();

                let game_name = *splits.first()?;

                if game_name != managed_game {
                    continue;
                }

                return Some(path.path());
            }
        }

        None
    }

    fn config_path(managed_game: &str) -> PathBuf {
        CONFIG_DIR.join(format!("requirements_{}.json", managed_game))
    }

    fn get_last_updated_from_path(path: &Path) -> Option<DateTime<Local>> {
        let timestamp = path
            .file_name()?
            .to_str()?
            .strip_suffix(".bin")?
            .split('_')
            .collect::<Vec<_>>()
            .last()?
            .parse::<i64>()
            .ok()?;

        Some(DateTime::from_timestamp(timestamp, 0)?.with_timezone(&Local))
    }

    /// Attempts to pull thunderstore mod data from the cache if it exists.
    fn from_cache(args: ProgramArgs) -> Self {
        let cache_file_name = Self::cache_path(&args.managed_game);
        let packages = cache_file_name
            .clone()
            .and_then(|path| File::open(path).ok())
            .map(BufReader::new)
            .and_then(|reader| {
                bincode::decode_from_reader::<Vec<SearchablePackage>, _, _>(
                    reader,
                    bincode::config::standard(),
                )
                .ok()
            })
            .map(|pkgs| {
                let mut map = HashMap::new();

                for pkg in pkgs {
                    map.insert(NamespacedPackage::from(&pkg), pkg);
                }
                map
            })
            .unwrap_or_default();

        let requirements: Option<Requirements> = File::open(Self::config_path(&args.managed_game))
            .ok()
            .map(BufReader::new)
            .and_then(|reader| serde_json::from_reader(reader).ok());

        Self {
            mods_dir: args.mods_dir,
            managed_game: args.managed_game,
            packages,
            requirements: requirements.unwrap_or_default(),
            last_updated: cache_file_name.and_then(|path| Self::get_last_updated_from_path(&path)),
        }
    }

    /// Attempts to save the program state to cache
    fn cache(&self) -> anyhow::Result<()> {
        let Some(last_updated) = self.last_updated else {
            return Err(anyhow!(
                "Unable to save cache with non-existant last_updated field"
            ));
        };

        if !std::fs::exists(CACHE_DIR.as_path())? {
            std::fs::create_dir_all(CACHE_DIR.as_path())?;
        }

        if let Some(previous_cached_file) = Self::cache_path(&self.managed_game) {
            std::fs::remove_file(&previous_cached_file)?;
        }

        let cache_path = CACHE_DIR.join(format!(
            "{}_{}.bin",
            self.managed_game,
            last_updated.timestamp()
        ));

        let mut writer = BufWriter::new(File::create(cache_path)?);

        bincode::encode_into_std_write(
            self.packages.values().cloned().collect::<Vec<_>>(),
            &mut writer,
            bincode::config::standard(),
        )?;

        Ok(())
    }

    async fn refresh_packages(&mut self, api: &thunderstore::Client) -> anyhow::Result<()> {
        let packages: Vec<SearchablePackage> = api
            .list_packages_v1(&self.managed_game)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        let mut map = HashMap::new();
        for pkg in packages {
            map.insert(NamespacedPackage::from(&pkg), pkg);
        }

        self.packages = map;
        let last_updated = Local::now();

        self.last_updated = Some(last_updated);
        self.cache()?;
        Ok(())
    }
}

/// A command line utility to help manager server mods using the Thunderstore api. Aids in the
/// installation, updating, and overall management of mods on a server.
#[derive(Parser)]
#[command(version, about)]
struct ProgramArgs {
    /// The directory where your mods should be deployed
    #[arg(short = 'd', long, env)]
    mods_dir: PathBuf,
    /// The game to be managed. This should match exactly with what is in the Thunderstore website
    #[arg(short = 'g', long, env)]
    managed_game: String,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Update mods for a specified game
    Update {
        /// Flag which tells the program to update the Thunderstore mod cache before updating any
        /// mods
        #[arg(short = 'c', long)]
        update_cache: bool,
        /// Flag which tells the program not to commit any updates, but instead to display which
        /// mods would be updated and which version it would be upated to
        #[arg(long)]
        dry_run: bool,
        /// Which mod should be updated. If not provided, all mods will be updated according to the
        /// requirements set in the config file
        #[arg(short = 'm', long, value_parser = NamespacedPackage::value_parser)]
        mod_name: Option<NamespacedPackage>,
    },
    /// Clears the local mod cache from thunderstore. Does NOT remove locally installed mods
    Clean,
    /// Uninstalls a specified mod
    Uninstall {
        /// The name of a mod with the namespace as a prefix followed by a '/'. Ex.
        /// ModAuthor/ModName
        #[arg(short = 'm', long, value_parser = NamespacedPackage::value_parser)]
        mod_name: NamespacedPackage,
    },
    /// Get the locations for the files the program uses for caching and config
    Files {
        #[command(subcommand)]
        file_name: FileName,
    },
}

#[derive(Subcommand, Clone)]
enum FileName {
    /// File name for the managed game's config file.
    Config,
    /// File name for the managed game's Thunderstore cache.
    Cache,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = ProgramArgs::parse();
    let thunderstore_api = thunderstore::Client::new();
    args.mods_dir = std::path::absolute(args.mods_dir)?;
    let command = args.command.clone();

    if let Some(command) = command {
        use Commands::*;
        match command {
            Update {
                update_cache,
                dry_run,
                mod_name,
            } => {
                let program_state = ProgramState::from_cache(args);
                update::perform_update(program_state, mod_name, update_cache, dry_run).await?;
            }
            Clean => {}
            Uninstall { mod_name } => {}
            Files { file_name } => {
                use FileName::*;
                match file_name {
                    Cache => {
                        if let Some(cache_path) = ProgramState::cache_path(&args.managed_game) {
                            println!("{}", cache_path.display())
                        } else {
                            return Err(anyhow!(format!(
                                "Unable to locate a Thunderstore cache for {}",
                                args.managed_game
                            )));
                        }
                    }
                    Config => {
                        let config_path = ProgramState::config_path(&args.managed_game);
                        println!("{}", config_path.display());
                    }
                }
            }
        }
    } else {
        let mut program_state = ProgramState::from_cache(args);
        main_menu::view(&thunderstore_api, &mut program_state).await?;
    };

    Ok(())
}
