//! Native output profiles select record revisions at the codec boundary.
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use native::Ref;
/// Native envelope and root revision selected for a rebuilt M3 model.
pub trait ExportProfile: native::CreationProfile {
    #[doc(hidden)]
    fn root(fields: RootFields) -> Result<Self::Root, Error>;
}
/// Relocated references passed only between native writer stages.
#[doc(hidden)]
#[derive(Default)]
pub struct RootFields {
    pub partitions: Ref<nr::targets::BoneAnimationSet>,
    pub partition_control: u32,
    pub constraints: Ref<nr::targets::PhysicsConstraint>,
    pub turrets: Ref<nr::targets::TurretBehavior>,
    pub turret_groups: Ref<nr::targets::TurretGroup>,
    pub tight_hit: nr::HitTestShape<nr::V1>,
    pub fuzzy_hit: Ref<nr::targets::HitTestShape>,
    pub attachment_volumes: Ref<nr::targets::AttachmentVolume>,
    pub cloths: Ref<nr::targets::ClothPhysics>,
    pub rigid_bodies: Ref<nr::targets::RigidBody>,
    pub physics_joints: Ref<nr::targets::PhysicsJoint>,
    pub collision_positions: Ref<nr::targets::Vector3>,
    pub collision_faces: Ref<nr::targets::U16>,
    pub collision_normals: Ref<nr::targets::Vector3>,
    pub collision_bounds: nr::Extent,
    pub particles: Ref<nr::targets::ParticleEmitter>,
    pub copies: Ref<nr::targets::ParticleEmitterCopy>,
    pub ribbons: Ref<nr::targets::RibbonEmitter>,
    pub projectors: Ref<nr::targets::Projector>,
    pub attachments: Ref<nr::targets::AttachmentPoint>,
    pub billboards: Ref<nr::targets::BillboardBehavior>,
    pub shadow: Ref<nr::targets::ShadowBox>,
    pub view_volumes: Ref<nr::targets::ViewVolume>,
    pub name: Ref<nr::targets::Character>,
    pub flags: nr::ModelFlags,
    pub bones: Ref<nr::targets::Bone>,
    pub vertices: Ref<nr::targets::Byte>,
    pub vertex_flags: nr::VertexFlags,
    pub divisions: Ref<nr::targets::MeshDivision>,
    pub bone_lookup: Ref<nr::targets::U16>,
    pub initial_reference: Ref<nr::targets::InitialReference>,
    pub skin_bone_count: u32,
    pub bounds: nr::Extent,
    pub material_maps: Ref<nr::targets::MaterialMap>,
    pub standard: Ref<nr::targets::StandardMaterial>,
    pub displacement: Ref<nr::targets::DisplacementMaterial>,
    pub composite: Ref<nr::targets::CompositeMaterial>,
    pub terrain: Ref<nr::targets::TerrainMaterial>,
    pub volume: Ref<nr::targets::VolumeMaterial>,
    pub volume_noise: Ref<nr::targets::VolumeNoiseMaterial>,
    pub creep: Ref<nr::targets::CreepMaterial>,
    pub hair: Ref<nr::targets::HairMaterial>,
    pub splat_terrain_bake: Ref<nr::targets::STBMaterial>,
    pub reflection: Ref<nr::targets::ReflectionMaterial>,
    pub lens_flare: Ref<nr::targets::LensFlare>,
    pub data_driven: Ref<nr::targets::DataDrivenMaterial>,
    pub sequences: Ref<nr::targets::Sequence>,
    pub controllers: Ref<nr::targets::SubTrackContainer>,
    pub groups: Ref<nr::targets::AnimationGroup>,
    pub states: Ref<nr::targets::AnimationState>,
    pub lights: Ref<nr::targets::Light>,
    pub cameras: Ref<nr::targets::Camera>,
    pub forces: Ref<nr::targets::Force>,
    pub warps: Ref<nr::targets::Warp>,
    pub ik_joints: Ref<nr::targets::IKJoint>,
    pub ik_two_joints: Ref<nr::targets::IKTwoJoint>,
    pub ik_ccd: Ref<nr::targets::IKCCD>,
    pub one_bone: Ref<nr::targets::OneBoneSolver>,
}
trait Optional<T: native::Target>: Sized {
    fn assign(value: Ref<T>) -> Result<Self, Error>;
}
impl<T: native::Target> Optional<T> for Ref<T> {
    fn assign(value: Ref<T>) -> Result<Self, Error> {
        Ok(value)
    }
}
impl<T: native::Target> Optional<T> for () {
    fn assign(value: Ref<T>) -> Result<Self, Error> {
        if value.is_empty() {
            Ok(())
        } else {
            Err(Error::new(
                "m3.profile",
                "selected native root cannot contain this populated record family",
            ))
        }
    }
}
fn optional<T: native::Target, R: Optional<T>>(value: Ref<T>) -> Result<R, Error> {
    R::assign(value)
}
macro_rules! profile {
    ($profile:ty,$revision:ty) => {
        impl ExportProfile for $profile {
            fn root(f: RootFields) -> Result<Self::Root, Error> {
                Ok(nr::ModelRecord::<$revision> {
                    cloth_physics: optional(f.cloths)?,
                    rigid_bodies: f.rigid_bodies,
                    physics_joints: f.physics_joints,
                    physics_constraints: f.constraints,
                    collision_verts: f.collision_positions,
                    collision_faces: f.collision_faces,
                    collision_normals: f.collision_normals,
                    collision_bounds: f.collision_bounds,
                    particle_emitters: f.particles,
                    particle_emitter_copies: f.copies,
                    ribbon_emitters: f.ribbons,
                    projections: f.projectors,
                    attachment_points: f.attachments,
                    turret_behaviors: f.turrets,
                    turret_groups: f.turret_groups,
                    tight_hit_test_object: f.tight_hit,
                    fuzzy_hit_test_objects: f.fuzzy_hit,
                    attachment_volumes: f.attachment_volumes,
                    billboard_behaviors: f.billboards,
                    shadow_boxes: f.shadow,
                    view_volumes: optional(f.view_volumes)?,
                    name: f.name,
                    flags: f.flags,
                    animation_split_count: f.partition_control,
                    bone_animation_sets: f.partitions,
                    bones: f.bones,
                    vertices: f.vertices,
                    vertex_flags: f.vertex_flags,
                    divisions: f.divisions,
                    bone_lookup: f.bone_lookup,
                    initial_reference: f.initial_reference,
                    skin_bone_count: f.skin_bone_count,
                    bounds: f.bounds,
                    material_maps: f.material_maps,
                    standard_materials: f.standard,
                    displacement_materials: f.displacement,
                    composite_materials: f.composite,
                    terrain_materials: f.terrain,
                    volume_materials: f.volume,
                    volume_noise_materials: optional(f.volume_noise)?,
                    creep_materials: f.creep,
                    hair_materials: f.hair,
                    stb_materials: optional(f.splat_terrain_bake)?,
                    reflection_materials: optional(f.reflection)?,
                    lens_flare_materials: optional(f.lens_flare)?,
                    data_driven_materials: optional(f.data_driven)?,
                    sequences: f.sequences,
                    sub_track_collections: f.controllers,
                    animation_groups: f.groups,
                    animation_states: f.states,
                    lights: f.lights,
                    cameras: f.cameras,
                    forces: f.forces,
                    warps: f.warps,
                    ik_joints: f.ik_joints,
                    ik_two_joints: f.ik_two_joints,
                    ik_c_c_d: optional(f.ik_ccd)?,
                    one_bone_solvers: f.one_bone,
                    ..Default::default()
                })
            }
        }
    };
}
profile!(native::Beta, nr::V23);
profile!(native::StarCraft2, nr::V29);
profile!(native::HeroesOfTheStorm, nr::V30);
