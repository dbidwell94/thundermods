mod installed_mods;
mod mod_search;
use crate::prelude::*;
pub use mod_search::SearchablePackage;

enum_select! {
    /// Menu menu selections
    #[derive(Clone, Copy)]
    enum MainMenuSelection {
        ViewInstalledMods = "View Installed Mods",
        ModSearch = "Mod Search",
        Quit = "Quit",
    }
}
pub async fn view(
    api: thunderstore::Client,
    mut program_args: super::ProgramState,
) -> anyhow::Result<()> {
    use MainMenuSelection::*;

    loop {
        clearscreen::clear()?;

        match MainMenuSelection::selectable("Main Menu").prompt()? {
            ViewInstalledMods => {
                installed_mods::view(&mut program_args, &api).await?;
            }
            ModSearch => {
                mod_search::view(&mut program_args, &api).await?;
            }
            Quit => {
                break;
            }
        };
    }

    Ok(())
}
