//! Bone-attached scene features with typed logical animation bindings.
use super::values::{BoneBehaviorFlags, CameraKind, LightKind};
use super::{Particle, TextureTransform};
use crate::model::semantic::{Animated, Link, MaterialId, NodeId, TextureId, ValidateLinks};

/// Skeleton controls whose meanings are specific to the game model family.
#[derive(Clone, Debug)]
pub struct BoneControls {
    /// External key-bone category.
    pub key_bone: i32,
    /// Simulation and transform behavior beyond the common hierarchy controls.
    pub flags: BoneBehaviorFlags,
    /// Associated mesh-part category.
    pub submesh: u16,
    /// External name checksum used by host-skeleton matching.
    pub name_crc: u32,
}

impl Default for BoneControls {
    fn default() -> Self {
        Self {
            key_bone: -1,
            flags: BoneBehaviorFlags::default(),
            submesh: 0,
            name_crc: 0,
        }
    }
}
/// A hierarchy node carrying a model-family feature.
#[derive(Clone, Debug)]
pub struct Actor<'a> {
    /// Feature placement in the common hierarchy.
    pub node: NodeId,
    /// Feature-specific authoring state.
    pub kind: ActorKind<'a>,
}
/// Distinct scene feature families.
#[derive(Clone, Debug)]
pub enum ActorKind<'a> {
    /// Runtime attachment role.
    Attachment(Attachment),
    /// Model light.
    Light(Light),
    /// Camera and look-at controls.
    Camera(Camera),
    /// Ribbon trail.
    Ribbon(Box<Ribbon>),
    /// Particle generator.
    Particle(Box<Particle<'a>>),
    /// Timestamp-triggered game action.
    Event(Event),
}
/// Runtime attachment role and enable control.
#[derive(Clone, Debug)]
pub struct Attachment {
    /// Attachment role, interpreted by application game data.
    pub role: u32,
    /// Animated enable toggle.
    pub enabled: Animated<bool>,
}
/// Timestamp-only action code and external game-data identifier.
#[derive(Clone, Debug)]
pub struct Event {
    /// Four-character action identifier.
    pub identifier: [u8; 4],
    /// Event-specific external game-data identity.
    pub data: u32,
}
/// Model light with independently animated ambient and diffuse components.
#[derive(Clone, Debug)]
pub struct Light {
    /// Directional or point-light behavior.
    pub kind: LightKind,
    /// Ambient color multiplier.
    pub ambient_color: Animated<[f32; 3]>,
    /// Ambient intensity multiplier.
    pub ambient_intensity: Animated<f32>,
    /// Diffuse color multiplier.
    pub diffuse_color: Animated<[f32; 3]>,
    /// Diffuse intensity multiplier.
    pub diffuse_intensity: Animated<f32>,
    /// Beginning and ending falloff distances in model units.
    pub attenuation_start: Animated<f32>,
    /// Ending falloff distance in model units.
    pub attenuation_end: Animated<f32>,
    /// Animated light enable toggle.
    pub enabled: Animated<bool>,
}
/// Camera projection and model-local look-at animation.
#[derive(Clone, Debug)]
pub struct Camera {
    /// Game camera role.
    pub kind: CameraKind,
    /// Diagonal field of view in radians.
    pub field_of_view: Animated<f32>,
    /// Near and far clipping distances in model units.
    pub clip: [f32; 2],
    /// Base look-at target in model coordinates.
    pub target_base: [f32; 3],
    /// Position displacement from the carrying node's rest translation.
    pub position: Animated<[f32; 3]>,
    /// Look-at target displacement from the base target.
    pub target: Animated<[f32; 3]>,
    /// Camera roll angle in radians.
    pub roll: Animated<f32>,
}
/// A generated ribbon trail with texture and shader graph bindings.
#[derive(Clone, Debug)]
pub struct Ribbon {
    /// Runtime effect identifier.
    pub id: u32,
    /// Ordered ribbon texture resources.
    pub textures: Vec<TextureId>,
    /// Ordered ribbon shader surfaces.
    pub materials: Vec<MaterialId>,
    /// Animated RGB multiplier.
    pub color: Animated<[f32; 3]>,
    /// Animated opacity fraction.
    pub alpha: Animated<f32>,
    /// Trail extent above its center line.
    pub height_above: Animated<f32>,
    /// Trail extent below its center line.
    pub height_below: Animated<f32>,
    /// Trail edges generated per second.
    pub edges_per_second: f32,
    /// Persistence of each edge in seconds.
    pub edge_lifetime: f32,
    /// Emission angle parameter, not gravitational acceleration.
    pub emission_angle: f32,
    /// Texture atlas row and column counts.
    pub atlas: [u16; 2],
    /// Animated atlas cell selection.
    pub texture_slot: Animated<u16>,
    /// Animated trail enable toggle.
    pub enabled: Animated<bool>,
    /// Signed pass ordering plane.
    pub priority_plane: i16,
    /// External particle-color table selection.
    pub color_index: i8,
    /// Optional animated texture-coordinate transform.
    pub texture_transform: Option<TextureTransform>,
}
impl ValidateLinks for Actor<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.node.visit_links(v);
        self.kind.visit_links(v)
    }
}
impl ValidateLinks for ActorKind<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Attachment(x) => x.enabled.visit_links(v),
            Self::Light(x) => x.visit_links(v),
            Self::Camera(x) => x.visit_links(v),
            Self::Ribbon(x) => x.visit_links(v),
            Self::Particle(x) => x.visit_links(v),
            Self::Event(_) => {}
        }
    }
}
impl ValidateLinks for Light {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.ambient_color.visit_links(v);
        self.ambient_intensity.visit_links(v);
        self.diffuse_color.visit_links(v);
        self.diffuse_intensity.visit_links(v);
        self.attenuation_start.visit_links(v);
        self.attenuation_end.visit_links(v);
        self.enabled.visit_links(v)
    }
}
impl ValidateLinks for Camera {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.field_of_view.visit_links(v);
        self.position.visit_links(v);
        self.target.visit_links(v);
        self.roll.visit_links(v)
    }
}
impl ValidateLinks for Ribbon {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.textures.visit_links(v);
        self.materials.visit_links(v);
        self.color.visit_links(v);
        self.alpha.visit_links(v);
        self.height_above.visit_links(v);
        self.height_below.visit_links(v);
        self.texture_slot.visit_links(v);
        self.enabled.visit_links(v);
        self.texture_transform.visit_links(v)
    }
}

impl ValidateLinks for BoneControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.flags.bits() & 0x7f != 0 {
            visitor(Link::InvalidValue);
        }
    }
}
