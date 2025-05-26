use crate::{ProgramState, prelude::NamespacedPackage};

pub async fn perform_update(
    program_state: ProgramState,
    mod_name: Option<NamespacedPackage>,
    update_cache: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    todo!()
}
