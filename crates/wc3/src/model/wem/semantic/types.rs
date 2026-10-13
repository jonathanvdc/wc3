//! Decoded authoring values and borrowed attribute views for WEM model graphs.
use crate::model::wem::records::{
    BlendMode, ClipFlags, CoordSpace, Extent, MaterialFlags, NodeFlags, ProfileId, TextureFlags,
};
use crate::model::wem::{Document, V3};
use std::borrow::Cow;

/// Editable WEM model values, independent of physical section placement.
#[derive(Clone, Debug, PartialEq)]
pub struct Model<'data> {
    /// Display name.
    pub name: Cow<'data, str>,
    /// Declared coordinate convention; changing it does not transform coordinates.
    pub space: CoordSpace,
    /// Authored unit scale metadata shared by the WEM document.
    pub unit_scale: f32,
    /// Stored geometry bounds.
    pub bounds: Extent,
    /// Shared meshes.
    pub meshes: Vec<Mesh<'data>>,
    /// Parent-linked nodes; all references use this vector's indices.
    pub nodes: Vec<Node<'data>>,
    /// Material slots.
    pub materials: Vec<Material<'data>>,
    /// Texture resources.
    pub textures: Vec<Texture<'data>>,
    /// Animation clips.
    pub clips: Vec<Clip<'data>>,
    /// Native profile represented by the materials and effects.
    pub profile: ProfileId,
    /// Original WEM graph for accessing native effects and profile records.
    pub native: Option<NativeModel<'data>>,
}
impl Default for Model<'_> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            space: CoordSpace::Blizzard,
            unit_scale: 1.,
            bounds: Extent::default(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            materials: Vec::new(),
            textures: Vec::new(),
            clips: Vec::new(),
            profile: ProfileId::default(),
            native: None,
        }
    }
}
/// Borrowed native WEM graph associated with a semantic model.
#[derive(Clone, Debug)]
pub struct NativeModel<'data> {
    /// Original record graph, including native materials, effects, and physics.
    pub document: Document<'data, V3>,
    /// Selected index in the root model table.
    pub model: u32,
    /// Selected look's physical material index for each logical model slot.
    pub slot_materials: Vec<u32>,
    /// Logical slots of source meshes, retained while editing a selected look.
    pub mesh_slots: Vec<u32>,
    /// Draw/vertex provenance of the geometry exported by this view.
    pub mesh_projections: Vec<MeshProjection>,
}
impl NativeModel<'_> {
    /// Whether export geometry changes source mesh or vertex addresses.
    pub fn requires_geometry_remap(&self) -> bool {
        self.mesh_projections
            .iter()
            .enumerate()
            .any(|(index, projection)| {
                projection.mesh as usize != index
                    || projection.section != 0
                    || projection.vertices.len() != projection.source_vertex_count as usize
                    || projection
                        .vertices
                        .iter()
                        .enumerate()
                        .any(|(index, source)| source.vertex as usize != index)
            })
    }
}
impl PartialEq for NativeModel<'_> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.document.raw();
        let b = other.document.raw();
        self.model == other.model
            && self.slot_materials == other.slot_materials
            && self.mesh_slots == other.mesh_slots
            && self.mesh_projections == other.mesh_projections
            && a.root_reference() == b.root_reference()
            && a.profile_mask() == b.profile_mask()
            && a.sections().len() == b.sections().len()
            && a.sections().zip(b.sections()).all(|((_, a), (_, b))| {
                a.tag() == b.tag()
                    && a.revision() == b.revision()
                    && a.count() == b.count()
                    && (a.tag() == *b"WOEM" || a.bytes() == b.bytes())
            })
    }
}
/// Original polygon-domain addresses for one projected vertex.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexSource {
    /// Source topological vertex.
    pub vertex: u32,
    /// Source face corner; `u32::MAX` when no corner expansion was needed.
    pub corner: u32,
    /// Source face; `u32::MAX` when no face expansion was needed.
    pub face: u32,
}
/// Addresses a draw section and the original domains behind its export vertices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshProjection {
    /// Original model-local mesh index.
    pub mesh: u32,
    /// Original draw-section index.
    pub section: u32,
    /// Original topological vertex count, including vertices unused by this draw.
    pub source_vertex_count: u32,
    /// Domain address of each exported vertex.
    pub vertices: Vec<VertexSource>,
}
/// One draw section within a triangle mesh.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh<'data> {
    /// Mesh name.
    pub name: Cow<'data, str>,
    /// Vertex positions.
    pub positions: Cow<'data, [[f32; 3]]>,
    /// Vertex normals; empty when absent.
    pub normals: Cow<'data, [[f32; 3]]>,
    /// Tangent direction and handedness.
    pub tangents: Cow<'data, [[f32; 4]]>,
    /// Texture coordinates by set.
    pub uv_sets: Vec<Cow<'data, [[f32; 2]]>>,
    /// Normalized vertex colors; empty when absent.
    pub colors: Cow<'data, [[f32; 4]]>,
    /// Triangle vertex indices.
    pub triangles: Cow<'data, [[u32; 3]]>,
    /// Normalized bone influences by vertex; empty means rigid geometry.
    pub influences: Vec<Vec<Influence>>,
    /// Material slot.
    pub material_slot: u32,
    /// Selection group.
    pub selection_group: u32,
    /// Level of detail.
    pub lod: u32,
    /// Geometry bounds.
    pub bounds: Extent,
}
/// A node index and its skin weight.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Influence {
    /// Index in the model node vector.
    pub node: u32,
    /// Influence weight.
    pub weight: f32,
}
/// Node transform and hierarchy common to supported model profiles.
#[derive(Clone, Debug, PartialEq)]
pub struct Node<'data> {
    /// Node name.
    pub name: Cow<'data, str>,
    /// Parent node index.
    pub parent: Option<u32>,
    /// Node pivot in model space.
    pub pivot: [f32; 3],
    /// Rest translation.
    pub translation: [f32; 3],
    /// Rest quaternion in x,y,z,w order.
    pub rotation: [f32; 4],
    /// Rest scale.
    pub scale: [f32; 3],
    /// Inverse bind matrix, when explicitly supplied.
    pub inverse_bind: Option<[[f32; 4]; 4]>,
    /// Common node transformation and billboard flags.
    pub flags: NodeFlags,
    /// Whether this node participates in skinning.
    pub bone: bool,
}
impl Default for Node<'_> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            parent: None,
            pivot: [0.; 3],
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
            scale: [1.; 3],
            inverse_bind: None,
            flags: NodeFlags::default(),
            bone: false,
        }
    }
}
/// A material slot with common render state and optional encoded native metadata.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material<'data> {
    /// Material name.
    pub name: Cow<'data, str>,
    /// Referenced texture index.
    pub texture: Option<u32>,
    /// Common material blending mode.
    pub blend: BlendMode,
    /// Common material rendering flags.
    pub flags: MaterialFlags,
}
/// A filename or numeric resource identity.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Texture<'data> {
    /// Filename, when present.
    pub path: Cow<'data, str>,
    /// Numeric identifier when available.
    pub id: Option<u32>,
    /// Resource namespace.
    pub group: u32,
    /// Replaceable texture category.
    pub replaceable: u32,
    /// Native texture flags.
    pub flags: TextureFlags,
}
/// An animation clip with channels in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Clip<'data> {
    /// Clip name.
    pub name: Cow<'data, str>,
    /// Clip duration in seconds.
    pub duration: f32,
    /// Native sequence identifier.
    pub id: u32,
    /// Clip playback and evaluation flags.
    pub flags: ClipFlags,
    /// Whether playback repeats after the clip duration.
    pub looping: bool,
    /// Channel tracks.
    pub tracks: Vec<Track>,
    /// Integer-valued channels retained without floating-point narrowing.
    pub integer_tracks: Vec<IntegerTrack>,
}
impl Default for Clip<'_> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            duration: 0.,
            id: 0,
            flags: ClipFlags::default(),
            looping: true,
            tracks: Vec::new(),
            integer_tracks: Vec::new(),
        }
    }
}
/// An animation track's destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    /// Node translation.
    Translation(u32),
    /// Node rotation.
    Rotation(u32),
    /// Node scale.
    Scale(u32),
    /// A profile-specific animation identity whose field is resolved by its native adapter.
    NativeProperty {
        /// Node or material owner in the selected native profile.
        owner: u32,
        /// Native animation identifier.
        id: u32,
    },
    /// Native property channel, with owner and channel codes.
    Property {
        /// Native owner index or material owner namespace.
        owner: u32,
        /// Native property identifier, stored without narrowing.
        channel: u32,
    },
}
/// Interpolation retained independently of key payload width.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Interpolation {
    /// Step interpolation.
    Step,
    /// Linear interpolation.
    #[default]
    Linear,
    /// Hermite interpolation.
    Hermite,
    /// Bezier interpolation.
    Bezier,
}
/// One typed animation key, including explicit spline tangents.
#[derive(Clone, Debug, PartialEq)]
pub struct Key<T = f32> {
    /// Time in seconds.
    pub time: f32,
    /// Value components.
    pub value: Vec<T>,
    /// Incoming tangent or control point.
    pub incoming: Option<Vec<T>>,
    /// Outgoing tangent or control point.
    pub outgoing: Option<Vec<T>>,
}
/// A channel's keys and optional shared clock.
#[derive(Clone, Debug, PartialEq)]
pub struct Track<T = f32> {
    /// Controller identity, ordering, and concurrency for these keys.
    pub layer: TrackLayer,
    /// Animated destination.
    pub target: Target,
    /// Interpolation convention.
    pub interpolation: Interpolation,
    /// Global clock duration in seconds when independent of the clip.
    pub global_duration: Option<f32>,
    /// Ordered keys.
    pub keys: Vec<Key<T>>,
}

/// An integer channel using the same key and track structures as floating-point channels.
#[derive(Clone, Debug, PartialEq)]
pub enum IntegerTrack {
    /// Unsigned 32-bit control values.
    Unsigned(Track<u32>),
    /// Signed 32-bit control values.
    Signed(Track<i32>),
    /// Unsigned 16-bit control values.
    Unsigned16(Track<u16>),
    /// Four unsigned byte components per key.
    Bytes4(Track<u8>),
    /// Boolean bytes, retaining their stored nonzero encoding.
    Boolean(Track<u8>),
    /// Four unsigned 32-bit components per key.
    Unsigned4(Track<u32>),
}
impl IntegerTrack {
    /// Controller settings shared by the integer keys.
    pub fn layer(&self) -> &TrackLayer {
        match self {
            Self::Unsigned(t) | Self::Unsigned4(t) => &t.layer,
            Self::Signed(t) => &t.layer,
            Self::Unsigned16(t) => &t.layer,
            Self::Bytes4(t) | Self::Boolean(t) => &t.layer,
        }
    }
    /// Animated destination shared by all keys in this channel.
    pub fn target(&self) -> Target {
        match self {
            Self::Unsigned(t) | Self::Unsigned4(t) => t.target,
            Self::Signed(t) => t.target,
            Self::Unsigned16(t) => t.target,
            Self::Bytes4(t) | Self::Boolean(t) => t.target,
        }
    }
}

/// Animation controller settings shared by its channels.
#[derive(Clone, Debug, Default)]
pub struct TrackLayer {
    /// Controller name.
    pub name: String,
    /// Ordering priority within the clip.
    pub priority: i32,
    /// Whether the controller runs concurrently with other layers.
    pub concurrent: bool,
    /// Original controller ordinal in the clip, used while editing its native graph.
    pub source: Option<u32>,
}

impl PartialEq for TrackLayer {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.priority == other.priority
            && self.concurrent == other.concurrent
    }
}
