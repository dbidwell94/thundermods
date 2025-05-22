mod installed_mods;
mod mod_search;
use crate::enum_select;
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
        let selected =
            inquire::Select::new("Main Menu", MainMenuSelection::VARIANTS.to_vec()).prompt()?;

        match selected {
            ViewInstalledMods => {
                installed_mods::view(&mut program_args, &api).await?;
            }
            ModSearch => {
                mod_search::view(&api, &mut program_args).await?;
            }
            Quit => {
                break;
            }
        };
    }

    Ok(())
}
