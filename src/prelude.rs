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
#[macro_export]
macro_rules! enum_select {
    (
    $(#[derive($($derive_target:ident),*)])?
    $vis:vis enum $enum_name:ident {
    $(
        $variant:ident = $display:expr,
    )*
    }) => {
        $(#[derive($($derive_target),*)])?
        $vis enum $enum_name {
            $($variant),*
        }

        impl $enum_name {
            /// All of the variants for this enum neatly packed into a `const &'static []`
            const VARIANTS: &'static [$enum_name] = &[
                $($enum_name::$variant),*
            ];
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
