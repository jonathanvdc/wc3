//! Diablo III authoring vocabulary, independent of native layout.
pub use crate::model::materials::diablo3::*;

pub use crate::model::controls::diablo3::*;
pub use crate::model::geometry::diablo3::*;
mod vocabulary;
pub use vocabulary::D3;

pub use crate::model::appearance::diablo3::*;

/// The native appearance embeds one external shader programme.
pub type Material<'a> = MaterialStep<'a>;
