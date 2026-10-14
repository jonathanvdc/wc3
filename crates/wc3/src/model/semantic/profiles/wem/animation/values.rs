//! Playback and recorded simulation conventions independent of native records.

/// ReadRule authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReadRule {
    #[default]
    /// Wc3.
    Wc3,
    /// Sc2.
    Sc2,
    /// Wow.
    Wow,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// BakePlays authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BakePlays {
    #[default]
    /// FromClip.
    FromClip,
    /// Loops.
    Loops,
    /// Once.
    Once,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// BakeFrom authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BakeFrom {
    #[default]
    /// Auto.
    Auto,
    /// Itself.
    Itself,
    /// FirstFrame.
    FirstFrame,
    /// Clip.
    Clip,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// BakeTo authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BakeTo {
    #[default]
    /// Auto.
    Auto,
    /// Nothing.
    Nothing,
    /// Clip.
    Clip,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// LoopMatch authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoopMatch {
    #[default]
    /// Crossfade.
    Crossfade,
    /// Offset.
    Offset,
    /// Off.
    Off,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// MatchWhere authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MatchWhere {
    #[default]
    /// End.
    End,
    /// Start.
    Start,
    /// Both.
    Both,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// MatchEase authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MatchEase {
    #[default]
    /// Smooth.
    Smooth,
    /// Linear.
    Linear,
    /// EaseIn.
    EaseIn,
    /// EaseOut.
    EaseOut,
    /// Custom.
    Custom,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// BakeFloor authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BakeFloor {
    #[default]
    /// Grid.
    Grid,
    /// Off.
    Off,
    /// FollowsRoot.
    FollowsRoot,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// BakeWorldForceKind authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BakeWorldForceKind {
    #[default]
    /// Wind.
    Wind,
    /// Blast.
    Blast,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}
