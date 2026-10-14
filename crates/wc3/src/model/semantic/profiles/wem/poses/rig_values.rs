//! Rig classification and joint choices are independent of encoded records.

/// RigRole authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RigRole {
    #[default]
    /// None.
    None,
    /// Body.
    Body,
    /// Spine.
    Spine,
    /// Neck.
    Neck,
    /// Head.
    Head,
    /// Clavicle.
    Clavicle,
    /// Upper.
    Upper,
    /// Lower.
    Lower,
    /// Hock.
    Hock,
    /// End.
    End,
    /// Toe.
    Toe,
    /// Twist.
    Twist,
    /// Pad.
    Pad,
    /// Prop.
    Prop,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}

/// RigSide authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RigSide {
    #[default]
    /// Centre.
    Centre,
    /// Left.
    Left,
    /// Right.
    Right,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}

/// RigLimb authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RigLimb {
    #[default]
    /// Other.
    Other,
    /// Arm.
    Arm,
    /// Leg.
    Leg,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}

/// RigPlant authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RigPlant {
    #[default]
    /// ByRole.
    ByRole,
    /// OnGround.
    OnGround,
    /// Always.
    Always,
    /// Free.
    Free,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}

/// RigSource authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RigSource {
    #[default]
    /// None.
    None,
    /// File.
    File,
    /// Name.
    Name,
    /// Attachment.
    Attachment,
    /// Shape.
    Shape,
    /// You.
    You,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}

/// JointMotion authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JointMotion {
    #[default]
    /// Bend.
    Bend,
    /// Twist.
    Twist,
    /// Spin.
    Spin,
    /// Slide.
    Slide,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}

/// JointAxis authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JointAxis {
    #[default]
    /// X.
    X,
    /// Y.
    Y,
    /// Z.
    Z,
    /// Along.
    Along,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}

/// EnvelopeFalloff authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EnvelopeFalloff {
    #[default]
    /// Hard.
    Hard,
    /// Linear.
    Linear,
    /// Smooth.
    Smooth,
    /// Gaussian.
    Gaussian,
    /// Unrecognized authored choice retained without interpretation.
    Unknown(u8),
}
