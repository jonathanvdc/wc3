//! Additional geometry domains selected by faces and draw windows.
use super::{MaterialId, PackedArray, Topology};
use crate::model::common::packed_identity;
use crate::model::common::AsULE;
use crate::model::common::IntoOwned;
use glam::Vec4;

/// Authored world-triangle rendering and query behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TriangleProperties {
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
}

/// Primary primitives and additional connectivity selected by draws.
#[derive(Clone, Debug, IntoOwned)]
pub struct ConnectivitySets<'data> {
    /// Primary authored primitive topology.
    pub primary: Topology<'data>,
    /// Independently addressable alternatives; each retains its own topology.
    additional: Vec<Topology<'data>>,
}
impl Default for ConnectivitySets<'_> {
    fn default() -> Self {
        Self {
            primary: Topology::Triangles(PackedArray::default()),
            additional: Vec::new(),
        }
    }
}

// Constrained WMO storage keeps its independently indexed arrays together.
#[derive(Clone, Debug, Default, IntoOwned)]
pub(crate) struct GeometryPools<'data> {
    pub triangle_strip_indices: PackedArray<'data, u32>,
    pub tangent_vectors: PackedArray<'data, Vec4>,
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
    (bool, bool),
);
impl AsULE for TriangleProperties {
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
            (self.cull_objects, self.collide_hit),
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
            (cull_objects, collide_hit),
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
        }
    }
}

packed_identity!(TriangleProperties);

#[cfg(test)]
mod tests {
    use super::TriangleProperties;
    use crate::model::common::PackedArray;
    use crate::model::Scope;

    #[test]
    fn packed_surface_controls_preserve_optional_zero_values_and_identity() {
        let material = Scope::default().allocate();
        let record = TriangleProperties {
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
        };
        let packed = PackedArray::alloc_from_slice(&[TriangleProperties::default(), record]);
        let parsed = PackedArray::<TriangleProperties>::parse_bytes(packed.as_bytes()).unwrap();
        assert_eq!(parsed.get(0), Some(TriangleProperties::default()));
        assert_eq!(parsed.get(1), Some(record));
        let mut invalid = packed.as_bytes().to_vec();
        invalid[0] = 2;
        assert!(PackedArray::<TriangleProperties>::parse_bytes(&invalid).is_err());
    }
}
