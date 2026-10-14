//! Deformation and effect choices are independent authored concepts.

/// Authored EffectKind interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EffectKind {
    /// Inflate.
    #[default]
    Inflate,
    /// Bulge.
    Bulge,
    /// Jiggle.
    Jiggle,
    /// Wobble.
    Wobble,
    /// Blobify.
    Blobify,
    /// Drip.
    Drip,
    /// Squash Stretch.
    SquashStretch,
    /// Melt.
    Melt,
    /// Ripple.
    Ripple,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl EffectKind {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Inflate,
            1 => Self::Bulge,
            2 => Self::Jiggle,
            3 => Self::Wobble,
            4 => Self::Blobify,
            5 => Self::Drip,
            6 => Self::SquashStretch,
            7 => Self::Melt,
            8 => Self::Ripple,
            9 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Inflate => 0,
            Self::Bulge => 1,
            Self::Jiggle => 2,
            Self::Wobble => 3,
            Self::Blobify => 4,
            Self::Drip => 5,
            Self::SquashStretch => 6,
            Self::Melt => 7,
            Self::Ripple => 8,
            Self::Count => 9,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored LumpKind interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LumpKind {
    /// Bone.
    #[default]
    Bone,
    /// Copy.
    Copy,
    /// Surface.
    Surface,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl LumpKind {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Bone,
            1 => Self::Copy,
            2 => Self::Surface,
            3 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Bone => 0,
            Self::Copy => 1,
            Self::Surface => 2,
            Self::Count => 3,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored Along interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Along {
    /// Surface.
    #[default]
    Surface,
    /// Bones.
    Bones,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl Along {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Surface,
            1 => Self::Bones,
            2 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Surface => 0,
            Self::Bones => 1,
            Self::Count => 2,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored Axis interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    /// Auto.
    #[default]
    Auto,
    /// Up.
    Up,
    /// X.
    X,
    /// Y.
    Y,
    /// Z.
    Z,
    /// Bone.
    Bone,
    /// Motion.
    Motion,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl Axis {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Auto,
            1 => Self::Up,
            2 => Self::X,
            3 => Self::Y,
            4 => Self::Z,
            5 => Self::Bone,
            6 => Self::Motion,
            7 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Up => 1,
            Self::X => 2,
            Self::Y => 3,
            Self::Z => 4,
            Self::Bone => 5,
            Self::Motion => 6,
            Self::Count => 7,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored ComesFrom interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComesFrom {
    /// Spans.
    #[default]
    Spans,
    /// Times.
    Times,
    /// Events.
    Events,
    /// Contact.
    Contact,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl ComesFrom {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Spans,
            1 => Self::Times,
            2 => Self::Events,
            3 => Self::Contact,
            4 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Spans => 0,
            Self::Times => 1,
            Self::Events => 2,
            Self::Contact => 3,
            Self::Count => 4,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored Pulse interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pulse {
    /// None.
    #[default]
    None,
    /// Breathe.
    Breathe,
    /// Heartbeat.
    Heartbeat,
    /// Rate.
    Rate,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl Pulse {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Breathe,
            2 => Self::Heartbeat,
            3 => Self::Rate,
            4 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Breathe => 1,
            Self::Heartbeat => 2,
            Self::Rate => 3,
            Self::Count => 4,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored BulgeShape interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BulgeShape {
    /// Round.
    #[default]
    Round,
    /// Ring.
    Ring,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl BulgeShape {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Round,
            1 => Self::Ring,
            2 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Round => 0,
            Self::Ring => 1,
            Self::Count => 2,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored JigglePreset interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JigglePreset {
    /// Custom.
    #[default]
    Custom,
    /// Jelly.
    Jelly,
    /// Fat.
    Fat,
    /// Muscle.
    Muscle,
    /// Rubber.
    Rubber,
    /// Water Balloon.
    WaterBalloon,
    /// Water.
    Water,
    /// Slime.
    Slime,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl JigglePreset {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Custom,
            1 => Self::Jelly,
            2 => Self::Fat,
            3 => Self::Muscle,
            4 => Self::Rubber,
            5 => Self::WaterBalloon,
            6 => Self::Water,
            7 => Self::Slime,
            8 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Custom => 0,
            Self::Jelly => 1,
            Self::Fat => 2,
            Self::Muscle => 3,
            Self::Rubber => 4,
            Self::WaterBalloon => 5,
            Self::Water => 6,
            Self::Slime => 7,
            Self::Count => 8,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored WobblePattern interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WobblePattern {
    /// Jelly.
    #[default]
    Jelly,
    /// Sway.
    Sway,
    /// Roll.
    Roll,
    /// Random.
    Random,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl WobblePattern {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Jelly,
            1 => Self::Sway,
            2 => Self::Roll,
            3 => Self::Random,
            4 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Jelly => 0,
            Self::Sway => 1,
            Self::Roll => 2,
            Self::Random => 3,
            Self::Count => 4,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored DripPreset interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DripPreset {
    /// Custom.
    #[default]
    Custom,
    /// Wax.
    Wax,
    /// Slime.
    Slime,
    /// Blood.
    Blood,
    /// Paint.
    Paint,
    /// Chocolate.
    Chocolate,
    /// Honey.
    Honey,
    /// Water.
    Water,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl DripPreset {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Custom,
            1 => Self::Wax,
            2 => Self::Slime,
            3 => Self::Blood,
            4 => Self::Paint,
            5 => Self::Chocolate,
            6 => Self::Honey,
            7 => Self::Water,
            8 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Custom => 0,
            Self::Wax => 1,
            Self::Slime => 2,
            Self::Blood => 3,
            Self::Paint => 4,
            Self::Chocolate => 5,
            Self::Honey => 6,
            Self::Water => 7,
            Self::Count => 8,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored NeckLaw interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NeckLaw {
    /// Linear.
    #[default]
    Linear,
    /// Snap.
    Snap,
    /// Thread.
    Thread,
    /// Sets.
    Sets,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl NeckLaw {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Linear,
            1 => Self::Snap,
            2 => Self::Thread,
            3 => Self::Sets,
            4 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Linear => 0,
            Self::Snap => 1,
            Self::Thread => 2,
            Self::Sets => 3,
            Self::Count => 4,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored SquashMode interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SquashMode {
    /// Auto.
    #[default]
    Auto,
    /// Keyed.
    Keyed,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl SquashMode {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Auto,
            1 => Self::Keyed,
            2 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Keyed => 1,
            Self::Count => 2,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored Anchor interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Anchor {
    /// Bottom.
    #[default]
    Bottom,
    /// Centre.
    Centre,
    /// Pivot.
    Pivot,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl Anchor {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Bottom,
            1 => Self::Centre,
            2 => Self::Pivot,
            3 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Bottom => 0,
            Self::Centre => 1,
            Self::Pivot => 2,
            Self::Count => 3,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored MeltOrder interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeltOrder {
    /// Top.
    #[default]
    Top,
    /// Bottom.
    Bottom,
    /// All At Once.
    AllAtOnce,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl MeltOrder {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Top,
            1 => Self::Bottom,
            2 => Self::AllAtOnce,
            3 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Top => 0,
            Self::Bottom => 1,
            Self::AllAtOnce => 2,
            Self::Count => 3,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored RipplePlays interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RipplePlays {
    /// Once.
    #[default]
    Once,
    /// Loops.
    Loops,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl RipplePlays {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Once,
            1 => Self::Loops,
            2 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Once => 0,
            Self::Loops => 1,
            Self::Count => 2,
            Self::Unknown(value) => value,
        }
    }
}

/// Authored Through interpretation, including producer-defined choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Through {
    /// Skin.
    #[default]
    Skin,
    /// Body.
    Body,
    /// Count.
    Count,
    /// A producer-defined interpretation.
    Unknown(u8),
}
impl Through {
    /// Converts a native selector without discarding unnamed choices.
    pub fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Skin,
            1 => Self::Body,
            2 => Self::Count,
            value => Self::Unknown(value),
        }
    }
    /// Selector code for an explicitly chosen interpretation.
    pub fn raw(self) -> u8 {
        match self {
            Self::Skin => 0,
            Self::Body => 1,
            Self::Count => 2,
            Self::Unknown(value) => value,
        }
    }
}
