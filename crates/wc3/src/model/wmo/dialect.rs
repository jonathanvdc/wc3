//! Client layout profiles independent of the shared WMO version number.
//!
//! MVER 17 spans multiple client generations. It cannot identify whether batch
//! prefixes or split-group links use their older representations. Choose a
//! profile from client provenance; reading a header number does not convert
//! these native fields. Chunk presence is determined by its actual tag rather
//! than an inferred expansion whitelist.
use super::records::{GroupLayout, RootLayout};
use std::fmt::Debug;
mod sealed {
    pub trait Dialect {}
}

/// A supported client profile with statically selected native record families.
pub trait Dialect:
    sealed::Dialect + Clone + Copy + Debug + Default + Eq + 'static + RootLayout + GroupLayout
{
    /// Version stored by ordinary WMO root and group files.
    const VERSION: u32 = 17;
}
macro_rules! profiles{($($name:ident:$doc:literal),*$(,)?)=>{$(
 #[doc=$doc]
 #[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
 pub struct $name;
 impl sealed::Dialect for $name{}
 impl Dialect for $name{}
)*};}
profiles! {
 Classic:"Classic client profile with legacy group-batch culling boxes.",
 Legion:"Legion client profile with large material IDs in group batches.",
 Bfa:"Battle for Azeroth client profile with large material IDs in group batches.",
 ShadowlandsPre92:"Shadowlands profile before 9.2, retaining the reserved group-header word.",
 Shadowlands:"Shadowlands 9.2 profile with explicit split-group links.",
 Dragonflight:"Dragonflight profile with split-group links and large material IDs.",
 WarWithin:"The War Within profile with split-group links and large material IDs."
}
