//! Typed native creation profiles and root assembly.
use super::{native, nr};
use native::Ref;
/// Native root creation revision selected independently of the envelope.
/// This trait exposes only shared conversion fields; existing roots retain their own revision on import.
pub trait ExportProfile: native::CreationProfile {
    /// Builds the profile root from references to newly encoded sections.
    fn root(fields: RootFields) -> Self::Root;
}
/// References and bounds shared by the supported native creation profiles.
#[derive(Clone, Copy, Debug, Default)]
pub struct RootFields {
    /// Native cloth simulation records.
    pub cloth: Ref<nr::targets::ClothPhysics>,
    /// Native projector records.
    pub projections: Ref<nr::targets::Projector>,
    /// Editable native behavior records.
    pub controls: super::Controls,
    /// Native animation split count.
    pub animation_split_count: u32,
    /// Collision surfaces and editable selection volumes.
    pub collision: super::effects::collision::Collision,
    /// Source model state flags.
    pub flags: u32,
    /// Model resource name.
    pub name: Ref<nr::targets::Character>,
    /// Hierarchy table.
    pub bones: Ref<nr::targets::Bone>,
    /// Packed geometry.
    pub vertices: Ref<nr::targets::Byte>,
    /// Native packed vertex layout selector.
    pub vertex_flags: u32,
    /// Mesh draw divisions.
    pub divisions: Ref<nr::targets::MeshDivision>,
    /// Skinning palette.
    pub bone_lookup: Ref<nr::targets::U16>,
    /// Inverse bind matrices.
    pub initial_reference: Ref<nr::targets::InitialReference>,
    /// Material family lookup.
    pub material_maps: Ref<nr::targets::MaterialMap>,
    /// Standard materials.
    pub standard_materials: Ref<nr::targets::StandardMaterial>,
    /// Native displacement materials.
    pub displacement_materials: Ref<nr::targets::DisplacementMaterial>,
    /// Native composite materials.
    pub composite_materials: Ref<nr::targets::CompositeMaterial>,
    /// Native terrain materials.
    pub terrain_materials: Ref<nr::targets::TerrainMaterial>,
    /// Native volume materials.
    pub volume_materials: Ref<nr::targets::VolumeMaterial>,
    /// Native volume noise materials.
    pub volume_noise_materials: Ref<nr::targets::VolumeNoiseMaterial>,
    /// Native creep materials.
    pub creep_materials: Ref<nr::targets::CreepMaterial>,
    /// Native stb materials.
    pub stb_materials: Ref<nr::targets::STBMaterial>,
    /// Native reflection materials.
    pub reflection_materials: Ref<nr::targets::ReflectionMaterial>,
    /// Native lens flare shader entries.
    pub lens_flare_materials: Ref<nr::targets::LensFlare>,
    /// Native data-driven shader entries.
    pub data_driven_materials: Ref<nr::targets::DataDrivenMaterial>,
    /// Animation clips.
    pub sequences: Ref<nr::targets::Sequence>,
    /// Clip controller groups.
    pub animation_groups: Ref<nr::targets::AnimationGroup>,
    /// Animation controllers.
    pub sub_track_collections: Ref<nr::targets::SubTrackContainer>,
    /// Animation state identities.
    pub animation_states: Ref<nr::targets::AnimationState>,
    /// Number of skinning bones.
    pub skin_bone_count: u32,
    /// Native particle copy table.
    pub particle_copies: Ref<nr::targets::ParticleEmitterCopy>,
    /// Native particle emitter table.
    pub particles: Ref<nr::targets::ParticleEmitter>,
    /// Native ribbon emitter table.
    pub ribbons: Ref<nr::targets::RibbonEmitter>,
    /// Native physics-joint table.
    pub physics_joints: Ref<nr::targets::PhysicsJoint>,
    /// Native rigid-body table.
    pub rigid_bodies: Ref<nr::targets::RigidBody>,
    /// Native attachment-point table.
    pub attachments: Ref<nr::targets::AttachmentPoint>,
    /// Native camera table.
    pub cameras: Ref<nr::targets::Camera>,
    /// Native light table.
    pub lights: Ref<nr::targets::Light>,
    /// Native force-field table.
    pub forces: Ref<nr::targets::Force>,
    /// Native vertex-warp table.
    pub warps: Ref<nr::targets::Warp>,
    /// Geometry bounds.
    pub bounds: nr::Extent,
}
trait Assign<T: native::Target> {
    fn assign(value: Ref<T>) -> Self;
}
impl<T: native::Target> Assign<T> for Ref<T> {
    fn assign(value: Ref<T>) -> Self {
        value
    }
}
impl<T: native::Target> Assign<T> for () {
    fn assign(_: Ref<T>) -> Self {}
}
fn assign<T: native::Target, R: Assign<T>>(value: Ref<T>) -> R {
    R::assign(value)
}
macro_rules! profile {
    ($profile:ty, $revision:ty) => {
        impl ExportProfile for $profile {
            fn root(f: RootFields) -> Self::Root {
                nr::ModelRecord::<$revision> {
                    rigid_bodies: f.rigid_bodies,
                    physics_joints: f.physics_joints,
                    particle_emitters: f.particles,
                    particle_emitter_copies: f.particle_copies,
                    ribbon_emitters: f.ribbons,
                    attachment_points: f.attachments,
                    cameras: f.cameras,
                    lights: f.lights,
                    forces: f.forces,
                    warps: f.warps,
                    flags: f.flags,
                    name: f.name,
                    bones: f.bones,
                    vertices: f.vertices,
                    vertex_flags: f.vertex_flags,
                    divisions: f.divisions,
                    bone_lookup: f.bone_lookup,
                    initial_reference: f.initial_reference,
                    material_maps: f.material_maps,
                    standard_materials: f.standard_materials,
                    displacement_materials: assign(f.displacement_materials),
                    composite_materials: assign(f.composite_materials),
                    terrain_materials: assign(f.terrain_materials),
                    volume_materials: assign(f.volume_materials),
                    volume_noise_materials: assign(f.volume_noise_materials),
                    creep_materials: assign(f.creep_materials),
                    stb_materials: assign(f.stb_materials),
                    reflection_materials: assign(f.reflection_materials),
                    lens_flare_materials: assign(f.lens_flare_materials),
                    data_driven_materials: assign(f.data_driven_materials),
                    sequences: f.sequences,
                    animation_groups: f.animation_groups,
                    sub_track_collections: f.sub_track_collections,
                    animation_states: f.animation_states,
                    skin_bone_count: f.skin_bone_count,
                    bounds: f.bounds,
                    cloth_physics: assign(f.cloth),
                    projections: f.projections,
                    animation_split_count: f.animation_split_count,
                    bone_animation_sets: f.controls.bone_animation_sets,
                    shadow_boxes: f.controls.shadow_boxes,
                    ik_joints: f.controls.ik_joints,
                    ik_two_joints: f.controls.ik_two_joints,
                    ik_c_c_d: assign(f.controls.ik_c_c_d),
                    one_bone_solvers: f.controls.one_bone_solvers,
                    billboard_behaviors: f.controls.billboard_behaviors,
                    trigger_data: f.controls.trigger_data,
                    turret_behaviors: f.controls.turret_behaviors,
                    view_volumes: assign(f.controls.view_volumes),
                    trailing_models: assign(f.controls.trailing_models),
                    collision_bounds: f.collision.bounds,
                    collision_verts: f.collision.vertices,
                    collision_faces: f.collision.faces,
                    collision_normals: f.collision.normals,
                    tight_hit_test_object: f.collision.tight,
                    fuzzy_hit_test_objects: f.collision.fuzzy,
                    attachment_volumes: f.collision.volumes,
                    attachment_point_addons: f.collision.attachment_addons,
                    cameras_addons: f.collision.camera_addons,
                    attachment_volumes_addon0: assign(f.collision.volume_addon0),
                    attachment_volumes_addon1: assign(f.collision.volume_addon1),
                    ..Default::default()
                }
            }
        }
    };
}
profile!(native::Beta, nr::V23);
profile!(native::StarCraft2, nr::V29);
profile!(native::HeroesOfTheStorm, nr::V30);
