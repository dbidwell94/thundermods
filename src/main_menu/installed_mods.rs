pub mod packages;

use crate::prelude::*;
use anyhow::anyhow;
use colored::Colorize;
use pad::{Alignment, PadStr};
use std::{cmp::Ordering, collections::VecDeque, path::PathBuf};
use thunderstore::models::PackageVersionV1;

#[derive(Debug)]
struct ModDirWithMetadata {
    dir: PathBuf,
    metadata: ModManifest,
    namespaced: NamespacedPackage,
    updated_version: Option<PackageVersionV1>,
}

impl std::fmt::Display for ModDirWithMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let update_version = match &self.updated_version {
            Some(updated) => updated
                .ident
                .parsed_version()
                .to_string()
                .pad(20, ' ', Alignment::Middle, true)
                .red(),
            None => "N/A".pad(20, ' ', Alignment::Middle, true).green(),
        };

        write!(
            f,
            "|{}|{}|{}|",
            self.metadata.name.pad(20, ' ', Alignment::Middle, true),
            self.metadata
                .version
                .to_string()
                .pad(20, ' ', Alignment::Middle, true),
            update_version
        )
    }
}

pub async fn view(
    program_state: &mut crate::ProgramState,
    api: &thunderstore::Client,
) -> anyhow::Result<()> {
    if program_state.packages.is_empty() {
        api.list_packages_v1(&program_state.managed_game).await?;
    }

    loop {
        clearscreen::clear()?;
        let mut installed_mods = Vec::new();
        let mut dirs = VecDeque::new();
        dirs.push_back(program_state.mods_dir.clone());

        while let Some(dir) = dirs.pop_front() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let metadata = std::fs::metadata(entry.path())?;

                if metadata.is_dir() {
                    dirs.push_back(entry.path());
                } else if entry.path().extension().unwrap_or_default() == "json" {
                    let file_str = std::fs::read_to_string(entry.path())?;

                    let parent_folder = entry
                        .path()
                        .parent()
                        .ok_or(anyhow!("Unable to get parent folder for installed mod"))?
                        .to_path_buf();

                    let folder_name = parent_folder
                        .iter()
                        .next_back()
                        .ok_or(anyhow!("invalid dir"))?
                        .to_str()
                        .ok_or(anyhow!("Invalid characters in dir name"))?;

                    let mut splits = folder_name.split('-');
                    let namespace = splits.next().ok_or(anyhow!("Invalid directory name"))?;
                    let name = splits.next().ok_or(anyhow!("Invalid directory name"))?;

                    if let Ok(mod_manifest) = serde_json::from_str::<ModManifest>(&file_str) {
                        installed_mods.push(ModDirWithMetadata {
                            metadata: mod_manifest,
                            namespaced: NamespacedPackage::new(namespace, name),
                            dir: entry
                                .path()
                                .parent()
                                .ok_or(anyhow::anyhow!(
                                    "Unable to get parent folder for installed mod"
                                ))?
                                .into(),
                            updated_version: None,
                        });
                    }
                }
            }
        }

        for installed in &mut installed_mods {
            let Some(latest) = program_state
                .requirements
                .get_latest_version(&program_state.packages, &installed.namespaced)
            else {
                continue;
            };

            if latest.ident.parsed_version() > installed.metadata.version {
                installed.updated_version = Some(latest);
            }
        }

        installed_mods.sort_by(|_, next| {
            if next.updated_version.is_some() {
                return Ordering::Greater;
            }
            Ordering::Less
        });

        let (_, height) = term_size::dimensions().unwrap_or((20, 20));

        let selected_option = inquire::Select::new("Installed mods...", installed_mods)
            .with_help_message(&format!(
                " |{}|{}|{}| ",
                "Name".pad_to_width_with_alignment(20, Alignment::Middle),
                "Installed Version".pad_to_width_with_alignment(20, Alignment::Middle),
                "Update Version".pad_to_width_with_alignment(20, Alignment::Middle)
            ))
            .with_page_size(height - 2)
            .prompt_skippable()?;

        let Some(selected_option) = selected_option else {
            if !crate::back_dialog::view()? {
                break;
            } else {
                continue;
            }
        };
    }

    Ok(())
}
