mod back_dialog;
mod main_menu;
pub mod prelude;

use chrono::{DateTime, Local};
use clap::Parser;
use prelude::*;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Global state for this program's session
pub struct ProgramState {
    /// The program args this session was started with
    args: ProgramArgs,
    /// cached packages from Thunderstore
    packages: Vec<SearchablePackage>,
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
            .unwrap_or_default();

        Self {
            args,
            packages,
            last_updated: cache_file_name.and_then(|path| Self::get_last_updated_from_path(&path)),
        }
    }

    /// Attempts to save the program state to cache
    fn cache(&self) -> anyhow::Result<()> {
        let Some(last_updated) = self.last_updated else {
            return Err(anyhow::anyhow!(
                "Unable to save cache with non-existant last_updated field"
            ));
        };

        if !std::fs::exists(CACHE_DIR.as_path())? {
            std::fs::create_dir_all(CACHE_DIR.as_path())?;
        }

        if let Some(previous_cached_file) = Self::cache_path(&self.args.managed_game) {
            std::fs::remove_file(&previous_cached_file)?;
        }

        let cache_path = CACHE_DIR.join(format!(
            "{}_{}.bin",
            self.args.managed_game,
            last_updated.timestamp()
        ));

        let mut writer = BufWriter::new(File::create(cache_path)?);

        bincode::encode_into_std_write(&self.packages, &mut writer, bincode::config::standard())?;

        Ok(())
    }

    async fn refresh_packages(&mut self, api: &thunderstore::Client) -> anyhow::Result<()> {
        let packages = api.list_packages_v1(&self.args.managed_game).await?;

        self.packages = packages.into_iter().map(Into::into).collect();
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = ProgramArgs::parse();
    let thunderstore_api = thunderstore::Client::new();
    args.mods_dir = std::path::absolute(args.mods_dir)?;

    let mut program_state = ProgramState::from_cache(args);

    main_menu::view(&thunderstore_api, &mut program_state).await?;
    program_state.cache()?;

    Ok(())
}
