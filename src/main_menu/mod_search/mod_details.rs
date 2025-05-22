use crate::prelude::*;
use thunderstore::VersionIdent;

use crate::enum_select;

enum_select! {
    #[derive(Clone)]
    pub enum ModDetailsResult {
        Back = "Back",
        Install = "Install Mod",
    }
}

pub fn view(to_view: &super::SearchablePackage) -> anyhow::Result<ModDetailsResult> {
    clearscreen::clear()?;

    let latest_version = to_view
        .0
        .versions
        .last()
        .ok_or(anyhow::anyhow!("This mod has no versions"))?;

    println!("Mod Name: {}", to_view.0.name);
    println!("Mod Author: {}", to_view.0.namespace);
    println!("Version: {}", latest_version.ident.version());
    println!("Description: {}", latest_version.description);
    println!("--- Dependencies ---");

    for dep in &latest_version.dependencies {
        print_dependencies(dep, "  ");
        println!("  --------------------");
    }

    let return_result =
        inquire::Select::new("Options", ModDetailsResult::VARIANTS.to_vec()).prompt()?;

    Ok(return_result)
}

fn print_dependencies(package: &VersionIdent, prefix: &str) {
    println!("{}Mod Name: {}", prefix, package.package_id().name());
    println!("{}Mod Author: {}", prefix, package.namespace());
    println!("{}Version: {}", prefix, package.version());
}
