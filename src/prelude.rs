use inquire::Select;
use serde::Deserialize;
use thunderstore::VersionIdent;

pub trait EnumSelectable
where
    Self: std::marker::Sized + 'static,
{
    /// All the variants of this enum packaged in a static array slice
    const VARIANTS: &'static [Self];

    /// Creates an `inquire::Select` struct from the variants of this enum
    fn selectable(message: &str) -> Select<Self>;
}

/// Allows for quick creation of an enum with
/// a pre-defined `std::fmt::Display` impl, as well
/// as a `T::VARIANTS` field impl which contains all the
/// fields of the struct
/// # Example
/// ```rust
/// use crate::enum_select;
///
/// enum_select! {
///     pub enum MyEnum {
///         One = "1",
///         Two = "2",
///         Three = "3",
///     }
/// }
///
/// fn get_variants() -> Vec<MyEnum> {
///     MyEnum::VARIANTS.to_vec()
/// }
/// ```
macro_rules! enum_select {
    (
    $(#[doc = $documentation:expr])?
    $(#[derive($($derive_target:ident),*)])?
    $vis:vis enum $enum_name:ident {
    $(
        $variant:ident = $display:expr,
    )*
    }) => {
        $(#[doc = $documentation])?
        $(#[derive($($derive_target),*)])?
        $vis enum $enum_name {
            $($variant),*
        }

        impl $crate::prelude::EnumSelectable for $enum_name {
            const VARIANTS: &'static [$enum_name] = &[$($enum_name::$variant),*];

            fn selectable(message: &str) -> inquire::Select<Self> {
                inquire::Select::new(message, Self::VARIANTS.to_vec())
            }

        }


        impl std::fmt::Display for $enum_name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                match self {
                    $(
                        Self::$variant => write!(fmt, $display)?,
                    )*
                };
                Ok(())
            }
        }
    };
}

pub(crate) use enum_select;

#[derive(Deserialize, Debug)]
pub struct ModManifest {
    pub name: String,
    #[serde(rename = "version_number")]
    pub version: semver::Version,
    pub description: String,
    pub dependencies: Vec<VersionIdent>,
}
