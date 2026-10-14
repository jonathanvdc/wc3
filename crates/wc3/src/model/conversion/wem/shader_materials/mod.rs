//! Game shader declarations remain independent from the common material authority.
mod warcraft;
mod wow;
pub(super) use warcraft::{read_warcraft, write_warcraft};
pub(super) use wow::{read_wow, write_wow};

mod starcraft;
pub(super) use starcraft::{read as read_starcraft, write as write_starcraft};

mod diablo;
pub(super) use diablo::{read as read_diablo, write as write_diablo};
