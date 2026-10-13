//! Native physics selectors and flags preserve their encoded widths.
use crate::model::wem;
use bitfield::bitfield;

bitfield! {
    /// Native Sc2BodyUntracedFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct Sc2BodyUntracedFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
}
impl Sc2BodyUntracedFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for Sc2BodyUntracedFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<Sc2BodyUntracedFlags> for u32 {
    fn from(flags: Sc2BodyUntracedFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Selects local and world force-field channels applied to a physics body.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct ForceChannels(u32);
    /// Returns the exact channel mask.
    pub bits, _: 31, 0;
    /// Local force-field channel mask.
    pub local, set_local: 15, 0;
    /// World force-field channel mask.
    pub world, set_world: 31, 16;
}
impl ForceChannels {
    /// Preserves the entire channel mask.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for ForceChannels {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<ForceChannels> for u32 {
    fn from(flags: ForceChannels) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native LocalForceChannels mask, including unnamed channels.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct LocalForceChannels(u16);
    /// Returns the complete stored mask.
    pub bits, _: 15, 0;
    /// Returns or changes the channel 0 control.
    pub channel_0, set_channel_0: 0;
    /// Returns or changes the channel 1 control.
    pub channel_1, set_channel_1: 1;
    /// Returns or changes the channel 2 control.
    pub channel_2, set_channel_2: 2;
    /// Returns or changes the channel 3 control.
    pub channel_3, set_channel_3: 3;
    /// Returns or changes the channel 4 control.
    pub channel_4, set_channel_4: 4;
    /// Returns or changes the channel 5 control.
    pub channel_5, set_channel_5: 5;
    /// Returns or changes the channel 6 control.
    pub channel_6, set_channel_6: 6;
    /// Returns or changes the channel 7 control.
    pub channel_7, set_channel_7: 7;
    /// Returns or changes the channel 8 control.
    pub channel_8, set_channel_8: 8;
    /// Returns or changes the channel 9 control.
    pub channel_9, set_channel_9: 9;
    /// Returns or changes the channel 10 control.
    pub channel_10, set_channel_10: 10;
    /// Returns or changes the channel 11 control.
    pub channel_11, set_channel_11: 11;
    /// Returns or changes the channel 12 control.
    pub channel_12, set_channel_12: 12;
    /// Returns or changes the channel 13 control.
    pub channel_13, set_channel_13: 13;
    /// Returns or changes the channel 14 control.
    pub channel_14, set_channel_14: 14;
    /// Returns or changes the channel 15 control.
    pub channel_15, set_channel_15: 15;
}
impl LocalForceChannels {
    /// Preserves the entire stored mask.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<u16> for LocalForceChannels {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<LocalForceChannels> for u16 {
    fn from(flags: LocalForceChannels) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native WorldForceChannels mask, including unnamed channels.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct WorldForceChannels(u16);
    /// Returns the complete stored mask.
    pub bits, _: 15, 0;
    /// Returns or changes the channel 0 control.
    pub channel_0, set_channel_0: 0;
    /// Returns or changes the channel 1 control.
    pub channel_1, set_channel_1: 1;
    /// Returns or changes the channel 2 control.
    pub channel_2, set_channel_2: 2;
    /// Returns or changes the channel 3 control.
    pub channel_3, set_channel_3: 3;
    /// Returns or changes the channel 4 control.
    pub channel_4, set_channel_4: 4;
    /// Returns or changes the channel 5 control.
    pub channel_5, set_channel_5: 5;
    /// Returns or changes the channel 6 control.
    pub channel_6, set_channel_6: 6;
    /// Returns or changes the channel 7 control.
    pub channel_7, set_channel_7: 7;
    /// Returns or changes the channel 8 control.
    pub channel_8, set_channel_8: 8;
    /// Returns or changes the channel 9 control.
    pub channel_9, set_channel_9: 9;
    /// Returns or changes the channel 10 control.
    pub channel_10, set_channel_10: 10;
    /// Returns or changes the channel 11 control.
    pub channel_11, set_channel_11: 11;
    /// Returns or changes the channel 12 control.
    pub channel_12, set_channel_12: 12;
    /// Returns or changes the channel 13 control.
    pub channel_13, set_channel_13: 13;
    /// Returns or changes the channel 14 control.
    pub channel_14, set_channel_14: 14;
    /// Returns or changes the channel 15 control.
    pub channel_15, set_channel_15: 15;
}
impl WorldForceChannels {
    /// Preserves the entire stored mask.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<u16> for WorldForceChannels {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<WorldForceChannels> for u16 {
    fn from(flags: WorldForceChannels) -> Self {
        flags.bits()
    }
}
