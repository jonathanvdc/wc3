//! Named triangle behavior mapped independently of native flag storage.
use crate::model::conversion::Error;
use crate::model::formats::wmo::records::TriangleFlags2;
use crate::model::{MaterialId, SurfaceTriangleControls};
pub(in crate::model::io::wmo::graph) fn face(
    flags: TriangleFlags2,
    selector: u16,
    collision_only: bool,
    materials: &[MaterialId],
) -> Result<SurfaceTriangleControls, Error> {
    let material = if collision_only || flags.ground_type_entry() {
        None
    } else {
        Some(
            *materials
                .get(usize::from(selector))
                .ok_or_else(|| Error::new("world.face.material", "face material does not exist"))?,
        )
    };
    let mut unknown = flags;
    unknown.set_transition(false);
    unknown.set_no_camera_collision(false);
    unknown.set_detail(false);
    unknown.set_collision(false);
    unknown.set_hint(false);
    unknown.set_render(false);
    unknown.set_cull_objects(false);
    unknown.set_collide_hit(false);
    unknown.set_ground_type_entry(false);
    Ok(SurfaceTriangleControls {
        spatial_queries: false,
        material,
        ground_type_selector: flags.ground_type_entry().then_some(selector),
        ground_type: None,
        lightmap_selector: None,
        alignment: 0,
        transition: flags.transition(),
        no_camera_collision: flags.no_camera_collision(),
        detail: flags.detail(),
        collision: flags.collision(),
        hint: flags.hint(),
        render: flags.render(),
        cull_objects: flags.cull_objects(),
        collide_hit: flags.collide_hit(),
        unclassified: unknown.bits().into(),
    })
}
