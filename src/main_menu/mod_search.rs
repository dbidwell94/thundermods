mod mod_details;
mod mod_install;

use pad::PadStr;
use thunderstore::models::PackageV1;

#[derive(Clone)]
pub struct SearchablePackage(PackageV1);

impl SearchablePackage {
    pub fn is_server_mod(&self) -> bool {
        self.0.categories.contains("Server-side")
    }
}

impl std::fmt::Display for SearchablePackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.0.name.clone();
        name.truncate(16);
        write!(
            f,
            "|{}|{}|{}|",
            name.pad_to_width_with_alignment(18, pad::Alignment::Middle),
            self.0
                .total_downloads()
                .to_string()
                .pad_to_width_with_alignment(17, pad::Alignment::Middle),
            self.0
                .rating_score
                .to_string()
                .pad_to_width_with_alignment(14, pad::Alignment::Middle)
        )
    }
}

impl From<PackageV1> for SearchablePackage {
    fn from(value: PackageV1) -> Self {
        Self(value)
    }
}

pub async fn view(
    api: &thunderstore::Client,
    state: &mut crate::ProgramState,
) -> anyhow::Result<()> {
    if state.packages.is_empty() {
        clearscreen::clear()?;
        println!("Please wait, fetching downloadable packages from Thunderstore...");
        let mut packages: Vec<SearchablePackage> = api
            .list_packages_v1(crate::COMMUNITY_NAME)
            .await?
            .into_iter()
            .filter(|f| !f.is_deprecated && !f.is_modpack())
            .map(Into::into)
            .filter(|searchable: &SearchablePackage| searchable.is_server_mod())
            .collect();

        packages
            .sort_by(|current, next| next.0.total_downloads().cmp(&current.0.total_downloads()));

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
