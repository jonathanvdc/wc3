//! Additional geometry domains selected by faces and draw windows.
use super::{MaterialId, Topology, ZeroVec};
use crate::model::common::AsULE;
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
            primary: Topology::Triangles(ZeroVec::default()),
            additional: Vec::new(),
        }
    }
}

// Constrained WMO storage keeps its independently indexed arrays together.
#[derive(Clone, Debug, Default)]
pub(crate) struct GeometryPools<'data> {
    pub triangle_strip_indices: ZeroVec<'data, u32>,
    pub tangent_vectors: ZeroVec<'data, [f32; 4]>,
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

impl AsULE for TriangleFlags {
    type ULE = <u16 as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        self.bits().to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        Self::from_bits_retain(u16::from_unaligned(value))
    }
}
type SurfaceRecord = (
    (
        bool,
        Option<MaterialId>,
        Option<u16>,
        Option<u32>,
        Option<u8>,
        u8,
    ),
    (bool, bool, bool, bool, bool, bool),
    (bool, bool, TriangleFlags),
);
impl AsULE for SurfaceTriangleControls {
    type ULE = <SurfaceRecord as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        (
            (
                self.spatial_queries,
                self.material,
                self.ground_type_selector,
                self.ground_type,
                self.lightmap_selector,
                self.alignment,
            ),
            (
                self.transition,
                self.no_camera_collision,
                self.detail,
                self.collision,
                self.hint,
                self.render,
            ),
            (self.cull_objects, self.collide_hit, self.unclassified),
        )
            .to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let (
            (
                spatial_queries,
                material,
                ground_type_selector,
                ground_type,
                lightmap_selector,
                alignment,
            ),
            (transition, no_camera_collision, detail, collision, hint, render),
            (cull_objects, collide_hit, unclassified),
        ) = SurfaceRecord::from_unaligned(value);
        Self {
            spatial_queries,
            material,
            ground_type_selector,
            ground_type,
            lightmap_selector,
            alignment,
            transition,
            no_camera_collision,
            detail,
            collision,
            hint,
            render,
            cull_objects,
            collide_hit,
            unclassified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SurfaceTriangleControls, TriangleFlags};
    use crate::model::common::ZeroVec;
    use crate::model::semantic::Scope;

    #[test]
    fn packed_surface_controls_preserve_optional_zero_values_and_identity() {
        let material = Scope::default().allocate();
        let record = SurfaceTriangleControls {
            spatial_queries: true,
            material: Some(material),
            ground_type_selector: Some(0),
            ground_type: Some(0),
            lightmap_selector: Some(0),
            alignment: 255,
            transition: true,
            no_camera_collision: true,
            detail: true,
            collision: true,
            hint: true,
            render: true,
            cull_objects: true,
            collide_hit: true,
            unclassified: TriangleFlags::from_bits_retain(u16::MAX),
        };
        let packed = ZeroVec::alloc_from_slice(&[SurfaceTriangleControls::default(), record]);
        let parsed = ZeroVec::<SurfaceTriangleControls>::parse_bytes(packed.as_bytes()).unwrap();
        assert_eq!(parsed.get(0), Some(SurfaceTriangleControls::default()));
        assert_eq!(parsed.get(1), Some(record));
        let mut invalid = packed.as_bytes().to_vec();
        invalid[0] = 2;
        assert!(ZeroVec::<SurfaceTriangleControls>::parse_bytes(&invalid).is_err());
    }
}
