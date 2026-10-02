//! Decoded models and file-backed dependencies.
mod blp;
pub(crate) mod loader;
pub(crate) mod model;
mod paths;
pub(crate) mod resources;

pub use blp::BlpImageLoader;
pub use loader::{Wc3ModelAsset, Wc3ModelLoader};
pub use model::{ModelError, Wc3Model};
pub use resources::Wc3ModelResources;
