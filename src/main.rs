mod back_dialog;
mod main_menu;
pub mod prelude;

use clap::Parser;
use prelude::*;
use std::path::PathBuf;

/// Global state for this program's session
pub struct ProgramState {
    /// The program args this session was started with
    args: ProgramArgs,
    /// cached packages from Thunderstore
    packages: Vec<SearchablePackage>,
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

    main_menu::view(
        thunderstore_api,
        ProgramState {
            packages: Vec::new(),
            args,
        },
    )
    .await?;
    Ok(())
}
