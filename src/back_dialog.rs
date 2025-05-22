use crate::prelude::*;

enum_select! {
    #[derive(Clone)]
    enum BackOptions {
        PreviousPage = "Previous Page",
        Continue = "Continue",
    }
}

/// if `true`, the user has decided to stay on the current page. If false,
/// the user has decided to go to the previous page.
pub fn view() -> anyhow::Result<bool> {
    use BackOptions::*;
    clearscreen::clear()?;
    Ok(
        match inquire::Select::new("What would you like to do?", BackOptions::VARIANTS.to_vec())
            .prompt()?
        {
            PreviousPage => false,
            Continue => true,
        },
    )
}
