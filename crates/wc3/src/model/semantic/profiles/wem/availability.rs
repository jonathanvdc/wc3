//! Producer-defined family availability is distinct from actor and material payload kinds.
use bitfield::bitfield;
bitfield! {
    /// Authored availability mask; individual producer-assigned bits remain uninterpreted.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct Availability(u32);
    /// Returns the complete availability selection.
    pub bits,_:31,0;
}
impl Availability {
    /// Retains producer-assigned selections without inventing family-bit meanings.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
