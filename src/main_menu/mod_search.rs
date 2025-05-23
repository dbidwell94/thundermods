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
        state.refresh_packages(api).await?;
    }

    loop {
        clearscreen::clear()?;
        let (_, height) = term_size::dimensions().unwrap_or((60, 60));

        let mut to_display = state.packages.values().cloned().collect::<Vec<_>>();
        to_display.sort_by_key(|item| std::cmp::Reverse(item.total_downloads()));

        let Some(selected_option) =
            inquire::Select::new("Online mods. Press <esc> to cancel", to_display)
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
