//! Native attached payloads are adapted independently of hierarchy placement.
pub(super) mod common;
pub(super) use common::{read_asset, write_asset};

mod warcraft;
pub(super) use warcraft::{
    bind_property as bind_warcraft_property, property_code as warcraft_property_code,
    read as read_warcraft, write as write_warcraft,
};

pub(super) mod wow;

pub(super) mod starcraft;

mod properties;
pub(super) use properties::bind as bind_property;
