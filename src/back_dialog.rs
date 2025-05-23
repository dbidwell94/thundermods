use crate::prelude::*;

enum_select! {
    /// The options for the back menu
    #[derive(Clone)]
    enum BackOptions {
        Continue = "Continue",
        PreviousPage = "Previous Page",
    }
}

/// if `true`, the user has decided to stay on the current page. If false,
/// the user has decided to go to the previous page.
pub fn view() -> anyhow::Result<bool> {
    use BackOptions::*;
    clearscreen::clear()?;
    Ok(
        match BackOptions::selectable("What would you like to do?").prompt()? {
            PreviousPage => false,
            Continue => true,
        },
    )
}
