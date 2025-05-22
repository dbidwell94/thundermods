mod back_dialog;
mod main_menu;
pub mod prelude;

use clap::Parser;
use std::path::PathBuf;

/// Global state for this program's session
pub struct ProgramState {
    /// The program args this session was started with
    args: ProgramArgs,
    /// cached packages from Thunderstore
    packages: Vec<main_menu::SearchablePackage>,
}

#[derive(Parser)]
#[command(version)]
struct ProgramArgs {
    // The directory where your mods should be deployed
    #[arg(short = 'd', long, env)]
    mods_dir: PathBuf,
    #[arg(short, long, env)]
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
