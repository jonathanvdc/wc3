//! Explicit producer shader choices retain unresolved categories.
use bitfield::bitfield;
/// Producer-defined address mode selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AddressMode(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined blend factor selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendFactor(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined blend operation selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendOperation(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined comparison selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Comparison(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined cull mode selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CullMode(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined fill mode selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FillMode(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined parameter type selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParameterType(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined stencil operation selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StencilOperation(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined switch selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Switch {
    /// Disabled operation.
    #[default]
    Disabled,
    /// Enabled operation.
    Enabled,
    /// Category whose interpretation is not established.
    Unknown(i32),
}

/// Producer-defined texture filter selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextureFilter(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Shader-defined texture role selection, retaining unresolved codes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextureType(
    /// Producer code whose interpretation is not established.
    pub i32,
);
/// Producer-defined uv transform mode selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UvTransformMode(
    /// Producer code whose interpretation is not established.
    pub i32,
);
bitfield! { /// Producer-defined MaterialFlags options; every bit is retained.
 #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
 pub struct MaterialFlags(i32);
 /// Full authored option mask.
 pub bits,_:31,0;
}
impl MaterialFlags {
    /// Constructs the complete authored option mask.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
}
bitfield! { /// Producer-defined UvFlags options; every bit is retained.
 #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
 pub struct UvFlags(i32);
 /// Full authored option mask.
 pub bits,_:31,0;
}
impl UvFlags {
    /// Constructs the complete authored option mask.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
}
bitfield! { /// Producer-defined PassFlags options; every bit is retained.
 #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
 pub struct PassFlags(i32);
 /// Full authored option mask.
 pub bits,_:31,0;
}
impl PassFlags {
    /// Constructs the complete authored option mask.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
}
/// External SNO resource namespace.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Group {
    /// Unspecified resource namespace.
    #[default]
    Unspecified,
    /// Actor resource namespace.
    Actor,
    /// Adventure resource namespace.
    Adventure,
    /// AIBehavior resource namespace.
    AIBehavior,
    /// AIState resource namespace.
    AIState,
    /// AmbientSound resource namespace.
    AmbientSound,
    /// Anim resource namespace.
    Anim,
    /// Anim2D resource namespace.
    Anim2D,
    /// AnimSet resource namespace.
    AnimSet,
    /// Appearance resource namespace.
    Appearance,
    /// Hero resource namespace.
    Hero,
    /// Cloth resource namespace.
    Cloth,
    /// Conversation resource namespace.
    Conversation,
    /// ConversationList resource namespace.
    ConversationList,
    /// EffectGroup resource namespace.
    EffectGroup,
    /// Encounter resource namespace.
    Encounter,
    /// Explosion resource namespace.
    Explosion,
    /// FlagSet resource namespace.
    FlagSet,
    /// Font resource namespace.
    Font,
    /// GameBalance resource namespace.
    GameBalance,
    /// Globals resource namespace.
    Globals,
    /// LevelArea resource namespace.
    LevelArea,
    /// Light resource namespace.
    Light,
    /// MarkerSet resource namespace.
    MarkerSet,
    /// Monster resource namespace.
    Monster,
    /// Observer resource namespace.
    Observer,
    /// Particle resource namespace.
    Particle,
    /// Physics resource namespace.
    Physics,
    /// Power resource namespace.
    Power,
    /// Quest resource namespace.
    Quest,
    /// Rope resource namespace.
    Rope,
    /// Scene resource namespace.
    Scene,
    /// SceneGroup resource namespace.
    SceneGroup,
    /// Script resource namespace.
    Script,
    /// ShaderMap resource namespace.
    ShaderMap,
    /// Shaders resource namespace.
    Shaders,
    /// Shakes resource namespace.
    Shakes,
    /// SkillKit resource namespace.
    SkillKit,
    /// Sound resource namespace.
    Sound,
    /// SoundBank resource namespace.
    SoundBank,
    /// StringList resource namespace.
    StringList,
    /// Surface resource namespace.
    Surface,
    /// Textures resource namespace.
    Textures,
    /// Trail resource namespace.
    Trail,
    /// UI resource namespace.
    UI,
    /// Weather resource namespace.
    Weather,
    /// Worlds resource namespace.
    Worlds,
    /// Recipe resource namespace.
    Recipe,
    /// Condition resource namespace.
    Condition,
    /// TreasureClass resource namespace.
    TreasureClass,
    /// Account resource namespace.
    Account,
    /// Conductor resource namespace.
    Conductor,
    /// TimedEvent resource namespace.
    TimedEvent,
    /// Act resource namespace.
    Act,
    /// Material resource namespace.
    Material,
    /// QuestRange resource namespace.
    QuestRange,
    /// Lore resource namespace.
    Lore,
    /// Reverb resource namespace.
    Reverb,
    /// PhysMesh resource namespace.
    PhysMesh,
    /// Music resource namespace.
    Music,
    /// Tutorial resource namespace.
    Tutorial,
    /// BossEncounter resource namespace.
    BossEncounter,
    /// ControlScheme resource namespace.
    ControlScheme,
    /// Accolade resource namespace.
    Accolade,
    /// AnimTree resource namespace.
    AnimTree,
    /// Vibrations resource namespace.
    Vibrations,
    /// DungeonFinder resource namespace.
    DungeonFinder,
    /// Unrecognized resource namespace.
    Unknown(i32),
}
