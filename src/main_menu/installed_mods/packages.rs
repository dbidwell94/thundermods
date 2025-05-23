use crate::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default, Debug)]
pub struct Requirements(HashMap<NamespacedPackage, semver::VersionReq>);

impl Requirements {
    pub fn get_latest_version(
        &self,
        packages: &HashMap<NamespacedPackage, SearchablePackage>,
        for_mod: &NamespacedPackage,
    ) -> Option<thunderstore::models::PackageVersionV1> {
        let remote_versions = packages.get(for_mod).map(|pkgs| &pkgs.versions)?;
        let requested_version = self.0.get(for_mod)?;

        let mut matched_versions = remote_versions
            .iter()
            .filter(|version| requested_version.matches(&version.ident.parsed_version()))
            .collect::<Vec<_>>();

        matched_versions.sort_by_key(|version| version.ident.parsed_version());
        matched_versions.last().cloned().cloned()
    }
}
