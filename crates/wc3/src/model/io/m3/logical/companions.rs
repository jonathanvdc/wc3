//! Companion documents contribute animation programs, not an implicit replacement scene.
use super::materials::{read_error, Slot};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m3::{self as native, records as nr};
use native::Envelope;

#[derive(PartialEq)]
struct Bone {
    position: nr::AnimationValue<[f32; 3]>,
    rotation: nr::AnimationValue<[f32; 4]>,
    scale: nr::AnimationValue<[f32; 3]>,
    batching: nr::AnimationValue<u32>,
    flags: u32,
    reserved: u16,
}
fn bones<E: Envelope>(model: &native::Document<'_, E>) -> Result<Vec<Bone>, Error> {
    let root = model.root_record().map_err(read_error)?;
    let reference = root.bones();
    if reference.is_empty() {
        return Ok(Vec::new());
    }
    let revision = model
        .container()
        .referenced_section(reference)
        .map_err(read_error)?
        .ok_or_else(|| Error::new("m3.companion.bones", "bone section is absent"))?
        .revision();
    macro_rules! read {
        ($revision:ty) => {
            model
                .container()
                .resolve::<nr::Bone<$revision>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| {
                    let r = r.map_err(read_error)?;
                    Ok(Bone {
                        position: r.position,
                        rotation: r.rotation,
                        scale: r.scale,
                        batching: r.batching,
                        flags: r.flags.bits(),
                        reserved: r.padding,
                    })
                })
                .collect()
        };
    }
    match revision {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        _ => Err(Error::new("m3.companion.bones", "unknown bone schema")),
    }
}
pub(super) fn check<B: Envelope, C: Envelope>(
    base: &native::Document<'_, B>,
    companion: &native::Document<'_, C>,
    context: &mut Context,
) -> Result<(), Error> {
    let imported = bones(companion)?;
    if !imported.is_empty() && imported != bones(base)? {
        context.unsupported("m3.companion.bone_controls","companion bone animation IDs, interpolation, initial/reset values or controls differ from the main scene; alternate binding semantics are not established")?;
    }
    let root = companion.root_record().map_err(read_error)?;
    let fields = native::visit_m3_root!(root, |r| vec![
        ("vertices", r.vertices.entries),
        ("divisions", r.divisions.entries),
        ("bone_lookup", r.bone_lookup.entries),
        ("inverse_bind", r.initial_reference.entries),
        ("materials", r.material_maps.entries),
        ("standard_materials", r.standard_materials.entries),
        ("displacement_materials", r.displacement_materials.entries),
        ("composite_materials", r.composite_materials.entries),
        ("terrain_materials", r.terrain_materials.entries),
        ("volume_materials", r.volume_materials.entries),
        ("hair_materials", r.hair_materials.entries),
        ("creep_materials", r.creep_materials.entries),
        (
            "volume_noise_materials",
            Slot::<native::Ref<nr::targets::VolumeNoiseMaterial>>::value(&r.volume_noise_materials)
                .entries
        ),
        (
            "splat_terrain_bake",
            Slot::<native::Ref<nr::targets::STBMaterial>>::value(&r.stb_materials).entries
        ),
        (
            "reflection",
            Slot::<native::Ref<nr::targets::ReflectionMaterial>>::value(&r.reflection_materials)
                .entries
        ),
        (
            "lens_flare",
            Slot::<native::Ref<nr::targets::LensFlare>>::value(&r.lens_flare_materials).entries
        ),
        (
            "data_driven",
            Slot::<native::Ref<nr::targets::DataDrivenMaterial>>::value(&r.data_driven_materials)
                .entries
        ),
        ("attachments", r.attachment_points.entries),
        ("attachment_addons", r.attachment_point_addons.entries),
        ("lights", r.lights.entries),
        ("cameras", r.cameras.entries),
        ("camera_addons", r.cameras_addons.entries),
        ("particles", r.particle_emitters.entries),
        ("particle_copies", r.particle_emitter_copies.entries),
        ("ribbons", r.ribbon_emitters.entries),
        ("projectors", r.projections.entries),
        ("forces", r.forces.entries),
        ("warps", r.warps.entries),
        ("bodies", r.rigid_bodies.entries),
        ("joints", r.physics_joints.entries),
        ("constraints", r.physics_constraints.entries),
        (
            "cloth",
            Slot::<native::Ref<nr::targets::ClothPhysics>>::value(&r.cloth_physics).entries
        ),
        ("billboards", r.billboard_behaviors.entries),
        ("shadow_boxes", r.shadow_boxes.entries),
        (
            "view_volumes",
            Slot::<native::Ref<nr::targets::ViewVolume>>::value(&r.view_volumes).entries
        ),
        ("ik_joints", r.ik_joints.entries),
        ("ik_two_joints", r.ik_two_joints.entries),
        (
            "ik_ccd",
            Slot::<native::Ref<nr::targets::IKCCD>>::value(&r.ik_c_c_d).entries
        ),
        ("one_bone_solvers", r.one_bone_solvers.entries),
        ("turrets", r.turret_behaviors.entries),
        ("triggers", r.turret_groups.entries),
        ("bone_animation_sets", r.bone_animation_sets.entries),
        ("collision_faces", r.collision_faces.entries),
        ("collision_vertices", r.collision_verts.entries),
        ("collision_normals", r.collision_normals.entries),
        ("hit_shapes", r.fuzzy_hit_test_objects.entries),
        ("attachment_volumes", r.attachment_volumes.entries),
    ]);
    for (field, count) in fields {
        if count != 0 {
            context.unsupported(
                format!("m3.companion.{field}"),
                "companion contains static scene data beyond its animation contribution",
            )?;
        }
    }
    Ok(())
}
