//! Complete graph import for the supported native layout and envelope combinations.
use super::*;
use crate::model::profiles::M2;
use native::Flat;

mod sealed {
    pub trait ImportDialect {}
}

/// A native layout and envelope with a complete semantic import implementation.
///
/// Implemented for the flat and chunked forms of every supported M2 layout.
/// Generic callers can use this capability instead of individual record-family bounds.
pub trait ImportDialect: Dialect + sealed::ImportDialect {
    /// Imports the complete checked graph for this native dialect.
    fn import<'a, 'data: 'a>(
        source: &'a Bundle<'data, Self>,
        options: &ImportOptions,
    ) -> Result<Conversion<Model<'a, M2>>, Error>;
}

macro_rules! import_dialects {
    ($($layout:ty),* $(,)?) => { $(
        import_dialects!(@impl Flat<$layout>);
        import_dialects!(@impl Chunked<$layout>);
    )* };
    (@impl $dialect:ty) => {
        impl sealed::ImportDialect for $dialect {}
        impl ImportDialect for $dialect {
            // Code generation for each layout belongs to this crate, even for unoptimized callers.
            #[inline(never)]
            fn import<'a, 'data: 'a>(
                source: &'a Bundle<'data, Self>,
                options: &ImportOptions,
            ) -> Result<Conversion<Model<'a, M2>>, Error> {
                graph::import(source, options)
            }
        }
    };
}
import_dialects!(
    V256,
    V257,
    V260,
    V263,
    V264,
    V265,
    V272,
    V272PreSplit,
    V273,
    V274
);

/// Imports an M2 bundle without a canonical intermediate encoding.
pub fn import<'a, 'data: 'a, D: ImportDialect>(
    source: &'a Bundle<'data, D>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'a, M2>>, Error> {
    D::import(source, options)
}
