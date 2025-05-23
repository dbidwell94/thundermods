pub use crate::main_menu::Requirements;
use directories::ProjectDirs;
use inquire::Select;
use pad::PadStr;
use serde::{Deserialize, de::Visitor};
use std::{ops::Deref, path::PathBuf, sync::LazyLock};
use thunderstore::{VersionIdent, models::PackageV1};

pub static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    ProjectDirs::from("com", "biddydev", env!("CARGO_BIN_NAME"))
        .unwrap()
        .cache_dir()
        .to_path_buf()
});
pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    ProjectDirs::from("com", "biddydev", env!("CARGO_BIN_NAME"))
        .unwrap()
        .config_dir()
        .to_path_buf()
});

/// Represents a namespaced package as used as a key for the requirements.json file.
///
/// ## Example
/// "MyNamespace/PackageName"
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct NamespacedPackage(String, String);

impl NamespacedPackage {
    /// For use with `clap` to parse command line arguments
    pub fn value_parser(value: &str) -> Result<Self, String> {
        todo!()
    }

    pub fn new(namespace: &str, name: &str) -> Self {
        Self(namespace.to_owned(), name.to_owned())
    }
    /// Gets the namespace of this package
    pub fn namespace(&self) -> &str {
        &self.0
    }
    /// Gets the name of this package
    pub fn name(&self) -> &str {
        &self.1
    }
}

impl<'de> serde::Deserialize<'de> for NamespacedPackage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NamespacedPackageVisitor;

        impl Visitor<'_> for NamespacedPackageVisitor {
            type Value = NamespacedPackage;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A string seperated by a forward slash. Ex: One/Two")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut splits = v.split('/');
                let namespace = splits
                    .next()
                    .ok_or(E::custom("Missing package namespace"))?;
                let name = splits.next().ok_or(E::custom("Missing package name"))?;

                if splits.next().is_some() {
                    return Err(E::custom("Package namespace is malformed"));
                }

                Ok(NamespacedPackage(namespace.to_owned(), name.to_owned()))
            }
        }

        deserializer.deserialize_string(NamespacedPackageVisitor)
    }
}

impl From<&PackageV1> for NamespacedPackage {
    fn from(value: &PackageV1) -> Self {
        Self(value.namespace.clone(), value.name.clone())
    }
}

impl From<&SearchablePackage> for NamespacedPackage {
    fn from(value: &SearchablePackage) -> Self {
        NamespacedPackage(value.namespace.clone(), value.name.clone())
    }
}

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

#[derive(Deserialize, Debug)]
pub struct ModManifest {
    pub name: String,
    #[serde(rename = "version_number")]
    pub version: semver::Version,
    pub description: String,
    pub dependencies: Vec<VersionIdent>,
}

#[derive(Clone, bincode::Encode, bincode::Decode)]
pub struct SearchablePackage(#[bincode(with_serde)] pub PackageV1);

impl Deref for SearchablePackage {
    type Target = PackageV1;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

pub(crate) use enum_select;
