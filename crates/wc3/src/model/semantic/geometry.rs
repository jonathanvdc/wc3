//! Geometry, skin bindings, and placements use logical graph identities.
use crate::model::semantic::profiles::{Profile, Universal};
use crate::model::semantic::BindingSchema as _;
use crate::model::semantic::TextureCoordinates;
use crate::model::semantic::{Attributes, BindMatrices, IdentityBinding, SurfaceTriangleControls};
use crate::model::semantic::{
    ConnectivityId, ConnectivitySets, DrawSubset, DrawSubsetId, DrawSubsets, Error, GeometryData,
    GeometryEdit, GeometrySchema, InfluenceRows, MaterialId, MeshId, NodeId, PrimitiveRange,
    PrimitiveSelection, SkinId, UniversalGeometry, Values,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// Axis-aligned geometry bounds and a conservative bounding sphere radius.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    /// Minimum coordinates.
    pub minimum: [f32; 3],
    /// Maximum coordinates.
    pub maximum: [f32; 3],
    /// Radius about the bounds' center.
    pub sphere_radius: f32,
}
impl Bounds {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self
            .minimum
            .iter()
            .chain(&self.maximum)
            .any(|x| !x.is_finite())
            || self.minimum.iter().zip(self.maximum).any(|(a, b)| *a > b)
            || !self.sphere_radius.is_finite()
            || self.sphere_radius < 0.
        {
            Err(Error::new(
                "bounds",
                "bounds must be finite, ordered, and nonnegative",
            ))
        } else {
            Ok(())
        }
    }
}
/// A joint index within a skin and its normalized weight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Influence {
    /// Index in the associated skin's joint list.
    pub joint: u32,
    /// Nonnegative weight.
    pub weight: f32,
}
/// Draw topology retained independently of native index encoding.
#[derive(Clone, Debug)]
pub enum Topology<'a> {
    /// Ordered groups containing different primitive families.
    Mixed(Vec<Topology<'a>>),
    /// Triangle primitives.
    Triangles(Values<'a, [u32; 3]>),
    /// Polygon face corners with one face-offset boundary per polygon plus a final boundary.
    Polygons {
        /// Vertex index of each corner.
        corners: Values<'a, u32>,
        /// Face boundaries in the corner array.
        offsets: Values<'a, u32>,
    },
    /// Line primitives.
    Lines(Values<'a, [u32; 2]>),
    /// Triangle-strip indices, retaining their authored index order.
    TriangleStrip(Values<'a, u32>),
    /// Point primitives.
    Points(Values<'a, u32>),
}
/// Which logical geometry elements carry an attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeDomain {
    /// One value per geometric vertex.
    Vertex,
    /// One value per primitive corner in topology order.
    Corner,
    /// One value per primitive in topology order.
    Face,
    /// Independently indexed values selected by draw bindings.
    Pool,
    /// One value per face in an explicitly identified connectivity set.
    ConnectivityFace(ConnectivityId),
    /// One value per corner in an explicitly identified connectivity set.
    ConnectivityCorner(ConnectivityId),
    /// One value for the whole geometry.
    Mesh,
}
/// The meaning of an additional geometry attribute.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttributeSemantic<'a> {
    /// Surface normal used for lighting.
    Normal,
    /// Surface tangent and handedness.
    Tangent,
    /// Independently authored tangent vector without inferred handedness.
    TangentVector,
    /// Independently authored binormal vector.
    Binormal,
    /// Index into the owning cloth vertex domain.
    ClothVertex,
    /// Independently authored scalar control for each collision triangle.
    CollisionTriangleControl,
    /// Color stream with its authored slot.
    Color(u32),
    /// Texture-coordinate stream with its authored slot.
    TexCoord(u32),
    /// Complete typed surface behavior per primary triangle.
    SurfaceControls,
    /// Independently authored baked-light texture coordinates.
    LightmapCoordinates,
    /// Material assigned to the domain element.
    Material,
    /// Application-defined attribute meaning.
    Named(Cow<'a, str>),
}
/// Typed values of an authoring attribute.
#[derive(Clone, Debug)]
pub enum AttributeValues<'a> {
    /// Complete typed surface records, including material relationships.
    SurfaceControls(Values<'a, SurfaceTriangleControls>),
    /// Logical boolean attributes.
    Boolean(Values<'a, bool>),
    /// Four exact byte components.
    Byte4(Values<'a, [u8; 4]>),
    /// Exact unsigned sixteen-bit values.
    U16(Values<'a, u16>),
    /// Four exact unsigned integer components.
    Unsigned4(Values<'a, [u32; 4]>),
    /// Quaternion components, distinguished from arbitrary four-vectors.
    Quaternion(Values<'a, [f32; 4]>),
    /// Scalar floating-point values.
    Scalar(Values<'a, f32>),
    /// Two-component floating-point vectors.
    Vector2(Values<'a, [f32; 2]>),
    /// Three-component floating-point vectors.
    Vector3(Values<'a, [f32; 3]>),
    /// Four-component values without implied quaternion meaning.
    Vector4(Values<'a, [f32; 4]>),
    /// Exact unsigned integers.
    Unsigned(Values<'a, u32>),
    /// Exact signed integers.
    Signed(Values<'a, i32>),
    /// References to authored material definitions.
    Materials(Values<'a, MaterialId>),
}
impl AttributeValues<'_> {
    /// Number of domain elements carrying a value.
    pub fn len(&self) -> usize {
        match self {
            Self::SurfaceControls(v) => v.len(),
            Self::Boolean(v) => v.len(),
            Self::Byte4(v) => v.len(),
            Self::U16(v) => v.len(),
            Self::Unsigned4(v) => v.len(),
            Self::Quaternion(v) => v.len(),
            Self::Scalar(v) => v.len(),
            Self::Vector2(v) => v.len(),
            Self::Vector3(v) => v.len(),
            Self::Vector4(v) => v.len(),
            Self::Unsigned(v) => v.len(),
            Self::Signed(v) => v.len(),
            Self::Materials(v) => v.len(),
        }
    }
    /// Whether this attribute contains no values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
/// A vertex contribution from an explicitly identified hierarchy node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeInfluence {
    /// Node whose authored rig convention interprets this contribution.
    pub node: NodeId,
    /// Nonnegative normalized contribution to the vertex.
    pub weight: f32,
}
/// One vertex's influences preserve whether they name binding slots or hierarchy nodes.
#[derive(Clone, Debug)]
pub enum VertexInfluences {
    /// Ordered slots interpreted through a skin's joint binding.
    Joints(Vec<Influence>),
    /// Explicit node identities interpreted through the hierarchy's rig convention.
    Nodes(Vec<NodeInfluence>),
}
impl VertexInfluences {
    /// Inspects contributions without discarding their reference domain.
    pub fn weights(&self) -> impl ExactSizeIterator<Item = f32> + '_ {
        let len = match self {
            Self::Joints(v) => v.len(),
            Self::Nodes(v) => v.len(),
        };
        (0..len).map(|index| match self {
            Self::Joints(v) => v[index].weight,
            Self::Nodes(v) => v[index].weight,
        })
    }
    /// Consumes a joint-slot distribution; direct-node distributions remain distinct.
    pub fn into_joint_weights(self) -> Option<Vec<Influence>> {
        match self {
            Self::Joints(v) => Some(v),
            Self::Nodes(_) => None,
        }
    }
}
/// Attribute data associated with vertices, polygon corners, faces, or the mesh as a whole.
#[derive(Clone, Debug)]
pub struct Attribute<'a> {
    /// Meaning of these values.
    pub semantic: AttributeSemantic<'a>,
    /// Geometry elements carrying the values.
    pub domain: AttributeDomain,
    /// Typed values in the domain's authored order.
    pub values: AttributeValues<'a>,
}
/// Geometry arrays and their draw topology. Insertion validates attribute sizes and indices.
#[derive(Clone, Debug)]
pub struct Mesh<'a, P: Profile = Universal> {
    /// Mesh-owned reusable selections and scoped connectivity identities.
    pub subsets: DrawSubsets,
    /// Additional owner-local behavior permitted by the vocabulary.
    pub controls: P::Mesh<'a>,
    /// Geometry name.
    pub name: Cow<'a, str>,
    /// Complete geometry vocabulary selected by the profile.
    pub geometry: P::Geometry<'a>,
    /// Conservative bounds.
    pub bounds: Bounds,
}
impl<'a, P: Profile> Mesh<'a, P> {
    /// Creates a mesh around geometry permitted by this profile.
    pub fn new(geometry: P::Geometry<'a>) -> Self {
        Self {
            subsets: DrawSubsets::default(),
            name: Cow::Borrowed(""),
            controls: Default::default(),
            geometry,
            bounds: Bounds::default(),
        }
    }
    pub(crate) fn resolve_draw<'view>(
        &'view self,
        draw: &'view Draw<P>,
    ) -> Result<ResolvedDraw<'view, P>, Error> {
        let subset = self.subset(draw.subset).ok_or_else(|| {
            Error::new(
                "instance.draw.subset",
                "draw subset belongs to another mesh or is absent",
            )
        })?;
        let PrimitiveRange::Primitives { first, count } = subset.selection.range else {
            return Err(Error::new(
                "instance.draw.selection",
                "native adapter requires a primitive selection",
            ));
        };
        if subset.selection.connectivity != self.subsets.primary() {
            return Err(Error::new(
                "instance.draw.connectivity",
                "native adapter requires primary connectivity",
            ));
        }
        Ok(ResolvedDraw {
            first,
            count,
            controls: &draw.controls,
            subset,
        })
    }
    /// Adds a profile-supported connectivity set and returns its mesh-scoped identity.
    pub fn add_connectivity(&mut self, topology: Topology<'a>) -> Result<ConnectivityId, Error> {
        let index = self.geometry.add_connectivity(topology)?;
        Ok(self.subsets.connectivity_id(index))
    }
    /// Applies geometry edits atomically while checking mesh-owned selections.
    pub fn edited(&self, edit: GeometryEdit<'a>) -> Result<Self, Error> {
        if edit.remap.is_some()
            && self.subsets.iter().any(|(_, subset)| {
                subset.strip.is_some()
                    || subset.tangents.is_some()
                    || subset.vertex_window.is_some()
            })
        {
            return Err(Error::new(
                "mesh.subsets.remap",
                "authored windows and pool associations require explicit correspondence updates",
            ));
        }
        let mut value = self.clone();
        value.geometry = value.geometry.edited(edit)?;
        value.validate()?;
        Ok(value)
    }
    /// Adds a reusable range in the primary topology.
    pub fn add_primary_subset(&mut self, first: u32, count: u32) -> DrawSubsetId {
        self.subsets.insert(DrawSubset::new(PrimitiveSelection {
            connectivity: self.subsets.primary(),
            range: PrimitiveRange::Primitives { first, count },
        }))
    }
    /// Inspects a connectivity set by its mesh-scoped identity.
    pub fn connectivity(&self, id: ConnectivityId) -> Option<Topology<'a>> {
        self.subsets.resolve(id, &self.geometry)
    }
    /// Inspects the geometry selected by an instance draw.
    pub fn subset(&self, id: DrawSubsetId) -> Option<&DrawSubset> {
        self.subsets.get(id)
    }
    /// Adds an explicitly identified subset; publication checks all its bindings.
    pub fn add_subset(&mut self, subset: DrawSubset) -> DrawSubsetId {
        self.subsets.insert(subset)
    }
    /// Identifies a connectivity set in the geometry's schema.
    pub fn connectivity_id(&mut self, index: usize) -> Result<ConnectivityId, Error> {
        self.geometry
            .connectivity(index)
            .ok_or_else(|| Error::new("mesh.connectivity", "connectivity set is absent"))?;
        Ok(self.subsets.connectivity_id(index))
    }
    /// Returns the binding shared by all placements of this geometry.
    pub fn skin(&self) -> Option<SkinId> {
        P::mesh_skin(&self.controls)
    }
    /// Borrows model-space vertex positions without decoding the entire stream.
    pub fn positions(&self) -> &Values<'a, [f32; 3]> {
        self.geometry.positions()
    }
    /// Borrows authored vertex normals when present.
    pub fn normals(&self) -> Option<&Values<'a, [f32; 3]>> {
        self.geometry.normals()
    }
    /// Borrows authored vertex tangents when present.
    pub fn tangents(&self) -> Option<&Values<'a, [f32; 4]>> {
        self.geometry.tangents()
    }
    /// Borrows ordered texture-coordinate streams.
    pub fn texture_coordinates(&self) -> TextureCoordinates<'_, 'a> {
        self.geometry.texture_coordinates()
    }
    /// Borrows authored vertex colors when present.
    pub fn colors(&self) -> Option<&Values<'a, [f32; 4]>> {
        self.geometry.colors()
    }
    /// Borrows additional attributes permitted by this geometry vocabulary.
    pub fn attributes(&self) -> Attributes<'_, 'a> {
        self.geometry.attributes()
    }
    /// Borrows primitive connectivity without projecting it into another vocabulary.
    pub fn topology(&self) -> &Topology<'a> {
        self.geometry.topology()
    }
    /// Borrows joint distributions in their authored structural form.
    pub fn influences(&self) -> InfluenceRows<'_, 'a> {
        self.geometry.influences()
    }
    /// Supplies geometry behavior from this graph's vocabulary.
    pub fn with_controls(mut self, controls: impl Into<P::Mesh<'a>>) -> Self {
        self.controls = controls.into();
        self
    }
    /// Names this geometry.
    pub fn named(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    pub(crate) fn validate(&self) -> Result<(), Error> {
        self.bounds.validate()?;
        self.geometry.validate()?;
        let mut domains = BTreeMap::new();
        for attribute in self.attributes() {
            let domain = match attribute.domain {
                AttributeDomain::Face => (0, Some(self.subsets.primary())),
                AttributeDomain::ConnectivityFace(id) => (0, Some(id)),
                AttributeDomain::Corner => (1, Some(self.subsets.primary())),
                AttributeDomain::ConnectivityCorner(id) => (1, Some(id)),
                AttributeDomain::Vertex => (2, None),
                AttributeDomain::Pool => (3, None),
                AttributeDomain::Mesh => (4, None),
            };
            if domains
                .insert((&attribute.semantic, domain), attribute.domain)
                .is_some_and(|previous| previous != attribute.domain)
            {
                return Err(Error::new(
                    "mesh.attributes",
                    "duplicate semantic and connectivity domain",
                ));
            }

            let expected = match attribute.domain {
                AttributeDomain::ConnectivityFace(id) => Some(
                    self.connectivity(id)
                        .ok_or_else(|| {
                            Error::new(
                                "mesh.attributes.connectivity",
                                "face connectivity belongs to another mesh or is absent",
                            )
                        })?
                        .primitive_count(),
                ),
                AttributeDomain::ConnectivityCorner(id) => Some(
                    self.connectivity(id)
                        .ok_or_else(|| {
                            Error::new(
                                "mesh.attributes.connectivity",
                                "corner connectivity belongs to another mesh or is absent",
                            )
                        })?
                        .corner_count(),
                ),
                _ => None,
            };
            if expected.is_some_and(|count| attribute.values.len() != count) {
                return Err(Error::new(
                    "mesh.attributes",
                    "attribute cardinality differs from its connectivity domain",
                ));
            }
        }
        self.subsets.validate(&self.geometry)
    }
}
/// Joint nodes and their inverse bind transforms.
#[derive(Clone, Debug)]
pub struct Skin<'a, P: Profile = Universal> {
    /// Ordered joints used by mesh influences.
    pub joints: Vec<NodeId>,
    /// Authored binding interpretation permitted by this profile.
    pub binding: P::Binding<'a>,
}
impl<'a, P: Profile> Skin<'a, P> {
    /// Constructs a skin with its profile's binding interpretation.
    pub fn new(joints: Vec<NodeId>, binding: P::Binding<'a>) -> Self {
        Self { joints, binding }
    }
    /// Inspects inverse bind transforms without allocating implicit identity arrays.
    pub fn inverse_bind(&self) -> BindMatrices<'_, 'a> {
        self.binding.matrices(self.joints.len())
    }
}
impl<'a, P: Profile<Binding<'a> = IdentityBinding>> Skin<'a, P> {
    /// Binds each joint directly in model coordinates.
    pub fn identity(joints: Vec<NodeId>) -> Self {
        Self::new(joints, IdentityBinding)
    }
}
/// One material draw range over a geometry object.
#[derive(Clone, Debug)]
pub struct Draw<P: Profile = Universal> {
    /// Behavioral controls permitted by this vocabulary.
    pub controls: P::Draw,
    /// Mesh-owned geometry selection rendered by this binding.
    pub subset: DrawSubsetId,
    /// Material used by this draw.
    pub material: MaterialId,
}
/// Geometry attached to a node, with independent skin and draw bindings.
#[derive(Clone, Debug)]
pub struct MeshInstance<'a, P: Profile = Universal> {
    /// Additional owner-local behavior permitted by the vocabulary.
    pub controls: P::Instance<'a>,
    /// Placement node.
    pub node: NodeId,
    /// Shared geometry.
    pub mesh: MeshId,
    /// Optional joint binding.
    pub skin: Option<SkinId>,
    /// Material draw ranges.
    pub draws: Vec<Draw<P>>,
    /// Authored LOD membership; none means shared across levels.
    pub lod: Option<u32>,
    /// Logical selection group.
    pub selection_group: u32,
}

impl Topology<'_> {
    /// Number of primitives in logical ordering, including mixed groups.
    pub fn primitive_count(&self) -> usize {
        match self {
            Self::TriangleStrip(v) => v.len().saturating_sub(2),
            Self::Triangles(v) => v.len(),
            Self::Lines(v) => v.len(),
            Self::Points(v) => v.len(),
            Self::Polygons { offsets, .. } => offsets.len().saturating_sub(1),
            Self::Mixed(groups) => groups.iter().map(Self::primitive_count).sum(),
        }
    }
    /// Number of primitive corners in logical ordering.
    pub fn corner_count(&self) -> usize {
        match self {
            Self::TriangleStrip(v) => v.len().saturating_sub(2) * 3,
            Self::Triangles(v) => v.len() * 3,
            Self::Lines(v) => v.len() * 2,
            Self::Points(v) => v.len(),
            Self::Polygons { corners, .. } => corners.len(),
            Self::Mixed(groups) => groups.iter().map(Self::corner_count).sum(),
        }
    }
}

impl<'a, P: Profile> MeshInstance<'a, P> {
    /// Places shared geometry at a node with no initial material draws or skin.
    pub fn new(node: NodeId, mesh: MeshId) -> Self {
        Self {
            node,
            mesh,
            skin: None,
            draws: Vec::new(),
            lod: None,
            selection_group: 0,
            controls: Default::default(),
        }
    }
    /// Supplies placement behavior from this graph's vocabulary.
    pub fn with_controls(mut self, controls: impl Into<P::Instance<'a>>) -> Self {
        self.controls = controls.into();
        self
    }
}

impl<'a> Mesh<'a> {
    /// Creates an unrestricted triangle mesh; publication validates the arrays.
    pub fn triangles(
        positions: impl Into<Values<'a, [f32; 3]>>,
        triangles: impl Into<Values<'a, [u32; 3]>>,
    ) -> Self {
        Self::new(UniversalGeometry::Data(GeometryData {
            positions: positions.into(),
            connectivity: ConnectivitySets::new(Topology::Triangles(triangles.into()), Vec::new()),
            ..GeometryData::default()
        }))
    }
}

/// Borrowed adapter view; ranges come from mesh-owned selections, never instance storage.
pub(crate) struct ResolvedDraw<'view, P: Profile> {
    pub first: u32,
    pub count: u32,
    pub controls: &'view P::Draw,
    pub subset: &'view DrawSubset,
}
