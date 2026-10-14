//! Native section inventories are resolved before publishing the logical collection.
mod records;

mod prepare;

mod models;

mod resources;

pub(super) mod model_records;

mod maps;

mod emit;

mod import;
pub(super) use import::read;

mod export;
pub(super) use export::write;
