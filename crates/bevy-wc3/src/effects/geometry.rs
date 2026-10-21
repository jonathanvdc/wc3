//! Procedural geometry participates in native visibility without mesh preprocessing.
use bevy::asset::{uuid_handle, AssetId, RenderAssetUsages};
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::{add_visibility_class, NoFrustumCulling, VisibilityClass};
use bevy::ecs::query::QueryItem;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::sync_component::SyncComponent;

const QUAD: Handle<Mesh> = uuid_handle!("af766dc3-5661-40ac-a77f-4349fa47837c");

/// One immutable geometry asset is shared across models and emitter families.
pub(crate) fn effect_quad(meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    meshes
        .get_or_insert_with(QUAD.id(), || Mesh::from(Rectangle::new(2., 2.)))
        .expect("UUID asset identifiers cannot have stale generations");
    QUAD.clone()
}

const CAPSULE: Handle<Mesh> = uuid_handle!("b3128672-1d94-4eda-98cc-698bda387462");
// x selects +/-width, y selects the longitudinal end. z identifies flat-normal
// face vertices:0 core, +/-1 cap base, +/-2 tip. The shader expands world records.
const CAPSULE_CORNERS: [[f32; 3]; 10] = [
    [1., 1., 0.],
    [1., -1., 0.],
    [-1., -1., 0.],
    [-1., 1., 0.],
    [1., 1., 1.],
    [-1., 1., 1.],
    [0., 1., 2.],
    [-1., -1., -1.],
    [1., -1., -1.],
    [0., -1., -2.],
];
const CAPSULE_INDICES: [u16; 12] = [0, 1, 2, 0, 2, 3, 4, 5, 6, 7, 8, 9];

/// Four triangles retain separate core/cap normals without per-particle geometry.
pub(crate) fn effect_capsule(meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    meshes
        .get_or_insert_with(CAPSULE.id(), || {
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, CAPSULE_CORNERS.to_vec())
            .with_inserted_indices(Indices::U16(CAPSULE_INDICES.to_vec()))
        })
        .expect("UUID asset identifiers cannot have stale generations");
    CAPSULE.clone()
}

fn quad_bounds() -> Aabb {
    Aabb::from_min_max(Vec3::new(-1., -1., 0.), Vec3::new(1., 1., 0.))
}

/// Immutable selectors only choose shader vertices; resident records define world geometry.
/// Its bounds must not cull detached particles or world-space trails.
#[derive(Component, Clone)]
#[require(Transform, Visibility, VisibilityClass, NoFrustumCulling, Aabb = quad_bounds())]
#[component(on_add = add_visibility_class::<EffectGeometry>)]
pub(crate) struct EffectGeometry(pub Handle<Mesh>);

#[derive(Component, Clone, Copy)]
pub(crate) struct ExtractedEffectGeometry {
    pub(crate) mesh: AssetId<Mesh>,
    pub(crate) world_center: Vec3,
}
impl SyncComponent for EffectGeometry {
    type Target = ExtractedEffectGeometry;
}
impl ExtractComponent for EffectGeometry {
    type QueryData = (&'static Self, &'static GlobalTransform);
    type QueryFilter = ();
    type Out = ExtractedEffectGeometry;
    fn extract_component(
        (geometry, transform): QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some(ExtractedEffectGeometry {
            mesh: geometry.0.id(),
            world_center: transform.translation(),
        })
    }
}

pub(crate) struct EffectGeometryPlugin;
impl Plugin for EffectGeometryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<EffectGeometry>::default());
    }
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
