use crate::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default, Debug)]
pub struct Requirements(HashMap<NamespacedPackage, semver::VersionReq>);

impl Requirements {
    pub fn get_latest_version(
        &self,
        packages: &[SearchablePackage],
        for_mod: &str,
    ) -> Option<thunderstore::models::PackageVersion> {
        todo!()
    }
}
