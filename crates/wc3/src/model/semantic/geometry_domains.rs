//! Additional geometry domains selected by faces and draw windows.
use super::{MaterialId, Topology, Values};
use bitfield::bitfield;
bitfield! {
    /// Unresolved triangle flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct TriangleFlags(u16);
    /// Complete retained option word.
    pub bits, _:15,0;
}
impl TriangleFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<u16> for TriangleFlags {
    fn from(bits: u16) -> Self {
        Self::from_bits_retain(bits)
    }
}

/// Authored world-triangle rendering and query behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceTriangleControls {
    /// Include this face in the static spatial-query structure.
    pub spatial_queries: bool,
    /// Surface material; absent for collision-only faces.
    pub material: Option<MaterialId>,
    /// Unresolved ground-type selector when the extended triangle marks a ground entry.
    pub ground_type_selector: Option<u16>,
    /// Optional authored terrain query classification.
    pub ground_type: Option<u32>,
    /// Historical baked-light selector whose table interpretation remains unresolved.
    pub lightmap_selector: Option<u8>,
    /// Historical alignment control whose meaning remains unresolved.
    pub alignment: u8,
    /// Interior/exterior lighting transition face.
    pub transition: bool,
    /// Disable camera collisions.
    pub no_camera_collision: bool,
    /// Detail geometry classification.
    pub detail: bool,
    /// Include the triangle in collision queries.
    pub collision: bool,
    /// Geometry hint classification.
    pub hint: bool,
    /// Include the triangle in rendering queries.
    pub render: bool,
    /// Cull nearby objects against this triangle.
    pub cull_objects: bool,
    /// Participate in hit queries.
    pub collide_hit: bool,
    /// Behavior bits with unresolved meanings.
    pub unclassified: TriangleFlags,
}

/// Primary primitives and additional connectivity selected by draws.
#[derive(Clone, Debug)]
pub struct ConnectivitySets<'data> {
    /// Primary authored primitive topology.
    pub primary: Topology<'data>,
    /// Independently addressable alternatives; each retains its own topology.
    additional: Vec<Topology<'data>>,
}
impl Default for ConnectivitySets<'_> {
    fn default() -> Self {
        Self {
            primary: Topology::Triangles(Values::default()),
            additional: Vec::new(),
        }
    }
}

// Constrained WMO storage keeps its independently indexed arrays together.
#[derive(Clone, Debug, Default)]
pub(crate) struct GeometryPools<'data> {
    pub triangle_strip_indices: Values<'data, u32>,
    pub tangent_vectors: Values<'data, [f32; 4]>,
}

impl<'data> ConnectivitySets<'data> {
    /// Creates primary topology with independently selectable alternatives.
    pub fn new(primary: Topology<'data>, additional: Vec<Topology<'data>>) -> Self {
        Self {
            primary,
            additional,
        }
    }
    /// Inspects alternatives in stable registration order.
    pub fn additional(&self) -> &[Topology<'data>] {
        &self.additional
    }
    /// Adds an alternative without changing existing registrations.
    pub fn add(&mut self, topology: Topology<'data>) -> usize {
        self.additional.push(topology);
        self.additional.len()
    }
}
