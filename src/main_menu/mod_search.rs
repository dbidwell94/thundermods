mod mod_details;
mod mod_install;

use crate::prelude::*;

pub async fn view(
    state: &mut crate::ProgramState,
    api: &thunderstore::Client,
) -> anyhow::Result<()> {
    if state.packages.is_empty() {
        clearscreen::clear()?;
        println!("Please wait, fetching downloadable packages from Thunderstore...");
        let mut packages: Vec<SearchablePackage> = api
            .list_packages_v1(&state.args.managed_game)
            .await?
            .into_iter()
            .filter(|f| !f.is_deprecated && !f.is_modpack())
            .map(Into::into)
            .filter(|searchable: &SearchablePackage| searchable.is_server_mod())
            .collect();

        packages.sort_by_key(|item| item.total_downloads());

        state.packages = packages;
    }

    loop {
        clearscreen::clear()?;
        let (_, height) = term_size::dimensions().unwrap_or((60, 60));

        let Some(selected_option) =
            inquire::Select::new("Online mods. Press <esc> to cancel", state.packages.clone())
                .with_page_size(height - 2)
                .with_help_message(" |       Name       |    Downloads    |    Rating    | ")
                .prompt_skippable()?
        else {
            if !crate::back_dialog::view()? {
                break;
            } else {
                continue;
            }
        };

        match mod_details::view(&selected_option)? {
            mod_details::ModDetailsResult::Install => {
                mod_install::view(&selected_option, api).await?;
            }
            mod_details::ModDetailsResult::Back => {}
        }
    }

    Ok(())
}
