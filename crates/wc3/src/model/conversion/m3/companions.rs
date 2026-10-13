//! Companion animation documents share the model's static property owners.
use super::{native, nr, read_error, revision, Context, Error};
use native::{Envelope, Ref, Target};

trait Count {
    fn count(&self) -> u32;
}
impl Count for () {
    fn count(&self) -> u32 {
        0
    }
}
impl<T: Target> Count for Ref<T> {
    fn count(&self) -> u32 {
        self.entries
    }
}

trait IdBits {
    fn bits(&self) -> u32;
}
impl IdBits for u32 {
    fn bits(&self) -> u32 {
        *self
    }
}
impl IdBits for i32 {
    fn bits(&self) -> u32 {
        self.cast_unsigned()
    }
}

pub(super) fn check<E: Envelope, M: Envelope>(
    source: &native::Model<'_, E>,
    main: &native::Model<'_, M>,
    hash: u32,
    context: &mut Context,
) -> Result<(), Error> {
    let root = source.root_record().map_err(read_error)?;
    macro_rules! fields {
        ($r:ident, $($field:ident),* $(,)?) => {
            $(if Count::count(&$r.$field) != 0 {
                context.unsupported(format!("m3.animations[{hash:08x}].{}", stringify!($field)),
                    "A companion static payload cannot be materialized into the owning model's animation clips")?;
            })*
        };
    }
    native::visit_m3_root!(root, |r| {
        fields!(
            r,
            vertices,
            divisions,
            bone_lookup,
            initial_reference,
            collision_faces,
            collision_verts,
            collision_normals,
            attachment_points,
            attachment_point_addons,
            lights,
            shadow_boxes,
            cameras,
            cameras_addons,
            material_maps,
            standard_materials,
            displacement_materials,
            composite_materials,
            terrain_materials,
            volume_materials,
            hair_materials,
            creep_materials,
            volume_noise_materials,
            stb_materials,
            reflection_materials,
            lens_flare_materials,
            data_driven_materials,
            particle_emitters,
            particle_emitter_copies,
            ribbon_emitters,
            projections,
            forces,
            warps,
            view_volumes,
            rigid_bodies,
            physics_constraints,
            physics_joints,
            cloth_physics,
            ik_two_joints,
            ik_c_c_d,
            ik_joints,
            one_bone_solvers,
            turret_behaviors,
            trigger_data,
            fuzzy_hit_test_objects,
            attachment_volumes,
            attachment_volumes_addon0,
            attachment_volumes_addon1,
            billboard_behaviors,
            trailing_models,
            bone_animation_sets,
        );
        let tight = &r.tight_hit_test_object;
        if tight.vertex_positions.entries != 0
            || tight.face_indices.entries != 0
            || tight.size_x != 0.
            || tight.size_y != 0.
            || tight.size_z != 0.
        {
            context.unsupported(
                format!("m3.animations[{hash:08x}].tight_hit_test_object"),
                "A companion selection volume has no static owner in the inline animation layout",
            )?;
        }
        if r.bounds != nr::Extent::default() || r.collision_bounds != nr::Extent::default() {
            context.unsupported(
                format!("m3.animations[{hash:08x}].bounds"),
                "Companion static model bounds are independent of the owning model's bounds",
            )?;
        }
        Ok::<(), Error>(())
    })?;
    let main_bones = bone_properties(main)?;
    for (index, bone) in bone_properties(source)?.iter().enumerate() {
        let owner = main_bones.iter().find(|owner| {
            owner.name == bone.name
                && owner.id == bone.id
                && owner.position.animation_id == bone.position.animation_id
                && owner.rotation.animation_id == bone.rotation.animation_id
                && owner.scale.animation_id == bone.scale.animation_id
                && owner.batching.animation_id == bone.batching.animation_id
        });
        if owner != Some(bone) {
            context.unsupported(format!("m3.animations[{hash:08x}].bones[{index}].fallbacks"),
                "Companion bone metadata, transform headers, or initial/reset values differ from the owning model; inline clips share one static property owner")?;
        }
    }
    Ok(())
}

#[derive(PartialEq)]
struct Properties {
    id: u32,
    name: String,
    flags: u32,
    parent_index: u16,
    padding: u16,
    position: nr::AnimationValue<[f32; 3]>,
    rotation: nr::AnimationValue<[f32; 4]>,
    scale: nr::AnimationValue<[f32; 3]>,
    batching: nr::AnimationValue<u32>,
}
fn bone_properties<E: Envelope>(source: &native::Model<'_, E>) -> Result<Vec<Properties>, Error> {
    let reference = native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| r.bones);
    if reference.entries == 0 {
        return Ok(Vec::new());
    }
    macro_rules! read {
        ($v:ty) => {
            source
                .document()
                .resolve::<nr::Bone<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .map(|row| {
                    let r = row.map_err(read_error)?;
                    Ok(Properties {
                        id: IdBits::bits(&r.id),
                        name: source
                            .document()
                            .string(r.name)
                            .map_err(read_error)?
                            .unwrap_or_default()
                            .to_owned(),
                        flags: r.flags,
                        parent_index: r.parent_index,
                        padding: r.padding,
                        position: r.position,
                        rotation: r.rotation,
                        scale: r.scale,
                        batching: r.batching,
                    })
                })
                .collect()
        };
    }
    match revision(source.document(), reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        _ => Err(Error::new(
            "m3.bones.revision",
            "unknown companion bone revision",
        )),
    }
}
