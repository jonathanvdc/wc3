//! Soft deformation definitions own actual pivots, rigid riders, and independently selectable effects.
//! Geometry and hierarchy associations refer to the checked model. Object identities
//! remain stable when definitions move; preparation data never backs their editable values.
mod values;
pub use values::*;
mod settings;
pub use settings::*;
mod body;
pub use body::*;
mod links;
