//! Direct conversion between native indexed M3 documents and checked M3 graph vocabulary.
use crate::model::semantic::Model;
mod logical;
use super::{Conversion, Error, ExportOptions, ImportOptions, Plan};
use crate::model::m3::{Bundle, Envelope};
use crate::model::semantic::profiles::M3;
pub use logical::profiles::ExportProfile;

/// Imports the native scene and its complete external animation closure.
/// Returned array and string views borrow the caller's native documents.
/// Known layouts whose authored meaning remains unresolved receive explicit loss-policy
/// diagnostics rather than being presented as editable semantic features.
pub fn import<'view, E: Envelope + Send + Sync>(
    source: &'view Bundle<'_, E>,
    options: &ImportOptions,
) -> Result<Conversion<Model<'view, M3>>, Error> {
    logical::read(source, options)
}
/// Builds a relocated native draft from the concrete M3 graph vocabulary.
/// Semantic vocabulary translation is a separate operation from native encoding.
pub fn plan<'view, 'source: 'view, P: ExportProfile>(
    asset: &'view Model<'source, M3>,
    options: &ExportOptions,
) -> Result<Plan<Bundle<'view, P::Envelope>>, Error> {
    logical::plan::<P>(asset, options)
}
