mod installed_mods;
mod mod_search;

pub use installed_mods::packages::Requirements;

use crate::prelude::*;

enum_select! {
    /// Menu menu selections
    #[derive(Clone, Copy)]
    enum MainMenuSelection {
        ViewInstalledMods = "View Installed Mods",
        ModSearch = "Mod Search",
        UpdateCache = "Update Thunderstore Mod Cache",
        Quit = "Quit",
    }
}
pub async fn view(
    api: &thunderstore::Client,
    program_args: &mut super::ProgramState,
) -> anyhow::Result<()> {
    use MainMenuSelection::*;

    loop {
        clearscreen::clear()?;

        match MainMenuSelection::selectable("Main Menu")
            .with_help_message(&format!(
                "Last cache update: {}",
                program_args
                    .last_updated
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or("N/A".into())
            ))
            .prompt()?
        {
            ViewInstalledMods => {
                installed_mods::view(program_args, api).await?;
            }
            ModSearch => {
                mod_search::view(program_args, api).await?;
            }
            UpdateCache => {
                clearscreen::clear()?;
                println!("Refreshing packages...");
                program_args.refresh_packages(api).await?;
            }
            Quit => {
                break;
            }
        };
    }

    Ok(())
}
