use crate::enum_select;
use crate::prelude::*;
use std::{collections::VecDeque, path::PathBuf};

#[derive(Debug)]
struct ModDirWithMetadata {
    dir: PathBuf,
    metadata: ModManifest,
}

pub async fn view(
    program_state: &mut crate::ProgramState,
    api: &thunderstore::Client,
) -> anyhow::Result<()> {
    if program_state.packages.is_empty() {
        api.list_packages_v1(crate::COMMUNITY_NAME).await?;
    }

    clearscreen::clear()?;
    let mut installed_mods = Vec::new();
    let mut dirs = VecDeque::new();
    dirs.push_back(program_state.args.mods_dir.clone());

    while let Some(dir) = dirs.pop_front() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = std::fs::metadata(entry.path())?;

            if metadata.is_dir() {
                dirs.push_back(entry.path());
            } else if entry.path().extension().unwrap_or_default() == "json" {
                let file_str = std::fs::read_to_string(entry.path())?;

                if let Ok(mod_manifest) = serde_json::from_str::<ModManifest>(&file_str) {
                    installed_mods.push(ModDirWithMetadata {
                        metadata: mod_manifest,
                        dir: entry
                            .path()
                            .parent()
                            .ok_or(anyhow::anyhow!(
                                "Unable to get parent folder for installed mod"
                            ))?
                            .into(),
                    });
                }
            }
        }
    }

    println!("{installed_mods:?}");

    inquire::prompt_confirmation("Continue?")?;

    Ok(())
}
