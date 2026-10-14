//! Native model record revisions.
use super::dialect::*;
use super::values::{ModelFlags, VertexFlags};
use super::{targets, Extent, HitTestShape};
use crate::model::m3::{self, Record, Ref};
use core::fmt::Debug;
/// Native field selections for independent `Model` section revisions.
pub trait ModelDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `volume_noise_materials`; `()` omits the field.
    type VolumeNoiseMaterials: NativeField;
    /// Field representation for `stb_materials`; `()` omits the field.
    type StbMaterials: NativeField;
    /// Field representation for `reflection_materials`; `()` omits the field.
    type ReflectionMaterials: NativeField;
    /// Field representation for `lens_flare_materials`; `()` omits the field.
    type LensFlareMaterials: NativeField;
    /// Field representation for `data_driven_materials`; `()` omits the field.
    type DataDrivenMaterials: NativeField;
    /// Field representation for `view_volumes`; `()` omits the field.
    type ViewVolumes: NativeField;
    /// Field representation for `cloth_physics`; `()` omits the field.
    type ClothPhysics: NativeField;
    /// Field representation for `ik_c_c_d`; `()` omits the field.
    type IkCCD: NativeField;
    /// Field representation for `attachment_volumes_addon0`; `()` omits the field.
    type AttachmentVolumesAddon0: NativeField;
    /// Field representation for `attachment_volumes_addon1`; `()` omits the field.
    type AttachmentVolumesAddon1: NativeField;
    /// Field representation for `trailing_models`; `()` omits the field.
    type TrailingModels: NativeField;
    /// Field representation for `m3a_anim_hash`; `()` omits the field.
    type M3aAnimHash: NativeField + OptionalHash;
    /// Field representation for `m3a_anim_hashes`; `()` omits the field.
    type M3aAnimHashes: NativeField + OptionalHashes;
}

impl ModelDialect for V20 {
    const VERSION: u32 = 20;
    type VolumeNoiseMaterials = ();
    type StbMaterials = ();
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = ();
    type ClothPhysics = ();
    type IkCCD = ();
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = ();
    type M3aAnimHash = ();
    type M3aAnimHashes = ();
}

impl ModelDialect for V21 {
    const VERSION: u32 = 21;
    type VolumeNoiseMaterials = ();
    type StbMaterials = ();
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = ();
    type ClothPhysics = ();
    type IkCCD = ();
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = ();
}

impl ModelDialect for V22 {
    const VERSION: u32 = 22;
    type VolumeNoiseMaterials = ();
    type StbMaterials = ();
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = ();
    type IkCCD = ();
    type AttachmentVolumesAddon0 = ();
    type AttachmentVolumesAddon1 = ();
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = ();
}

impl ModelDialect for V23 {
    const VERSION: u32 = 23;
    type VolumeNoiseMaterials = ();
    type StbMaterials = ();
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = ();
    type IkCCD = ();
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

impl ModelDialect for V24 {
    const VERSION: u32 = 24;
    type VolumeNoiseMaterials = ();
    type StbMaterials = ();
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = ();
    type IkCCD = Ref<targets::IKCCD>;
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

impl ModelDialect for V25 {
    const VERSION: u32 = 25;
    type VolumeNoiseMaterials = Ref<targets::VolumeNoiseMaterial>;
    type StbMaterials = ();
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = ();
    type IkCCD = Ref<targets::IKCCD>;
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

impl ModelDialect for V26 {
    const VERSION: u32 = 26;
    type VolumeNoiseMaterials = Ref<targets::VolumeNoiseMaterial>;
    type StbMaterials = Ref<targets::STBMaterial>;
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = ();
    type IkCCD = Ref<targets::IKCCD>;
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

impl ModelDialect for V27 {
    const VERSION: u32 = 27;
    type VolumeNoiseMaterials = Ref<targets::VolumeNoiseMaterial>;
    type StbMaterials = Ref<targets::STBMaterial>;
    type ReflectionMaterials = ();
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = ();
    type IkCCD = Ref<targets::IKCCD>;
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

impl ModelDialect for V28 {
    const VERSION: u32 = 28;
    type VolumeNoiseMaterials = Ref<targets::VolumeNoiseMaterial>;
    type StbMaterials = Ref<targets::STBMaterial>;
    type ReflectionMaterials = Ref<targets::ReflectionMaterial>;
    type LensFlareMaterials = ();
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = Ref<targets::ClothPhysics>;
    type IkCCD = Ref<targets::IKCCD>;
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

impl ModelDialect for V29 {
    const VERSION: u32 = 29;
    type VolumeNoiseMaterials = Ref<targets::VolumeNoiseMaterial>;
    type StbMaterials = Ref<targets::STBMaterial>;
    type ReflectionMaterials = Ref<targets::ReflectionMaterial>;
    type LensFlareMaterials = Ref<targets::LensFlare>;
    type DataDrivenMaterials = ();
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = Ref<targets::ClothPhysics>;
    type IkCCD = Ref<targets::IKCCD>;
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

impl ModelDialect for V30 {
    const VERSION: u32 = 30;
    type VolumeNoiseMaterials = Ref<targets::VolumeNoiseMaterial>;
    type StbMaterials = Ref<targets::STBMaterial>;
    type ReflectionMaterials = Ref<targets::ReflectionMaterial>;
    type LensFlareMaterials = Ref<targets::LensFlare>;
    type DataDrivenMaterials = Ref<targets::DataDrivenMaterial>;
    type ViewVolumes = Ref<targets::ViewVolume>;
    type ClothPhysics = Ref<targets::ClothPhysics>;
    type IkCCD = Ref<targets::IKCCD>;
    type AttachmentVolumesAddon0 = Ref<targets::U16>;
    type AttachmentVolumesAddon1 = Ref<targets::U16>;
    type TrailingModels = Ref<targets::TrailingModel>;
    type M3aAnimHash = u32;
    type M3aAnimHashes = Ref<targets::U32>;
}

/// Fields of the native `MODL` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ModelRecord<D: ModelDialect> {
    /// CHAR section containing the model resource name.
    pub name: Ref<targets::Character>,
    /// Model state bits, including sorted tracks (0x40), valid animated-base flags (0x800), and solved bone flags (0x80000).
    pub flags: ModelFlags,
    /// SEQS entries defining the available animation intervals.
    pub sequences: Ref<targets::Sequence>,
    /// STC_ containers associating animation identities with typed key arrays.
    pub sub_track_collections: Ref<targets::SubTrackContainer>,
    /// STG_ entries grouping sub-track containers for sequences.
    pub animation_groups: Ref<targets::AnimationGroup>,
    /// BSET section reference; commonly empty in model files.
    pub bone_animation_sets: Ref<targets::BoneAnimationSet>,
    /// Native animation split count; zero is the usual value.
    pub animation_split_count: u32,
    /// STS_ entries listing animation identities used by track containers.
    pub animation_states: Ref<targets::AnimationState>,
    /// BONE section containing the skeleton hierarchy and animated transforms.
    pub bones: Ref<targets::Bone>,
    /// Number of skeleton bones participating in vertex skinning.
    pub skin_bone_count: u32,
    /// Bit mask selecting the packed vertex record layout.
    pub vertex_flags: VertexFlags,
    /// U8__ section containing packed vertices in the layout selected by vertex_flags.
    pub vertices: Ref<targets::Byte>,
    /// DIV_ section describing mesh regions, face arrays, and material batches.
    pub divisions: Ref<targets::MeshDivision>,
    /// U16_ palette mapping region-local skin indices to skeleton bones.
    pub bone_lookup: Ref<targets::U16>,
    /// Model-space bounding box and enclosing sphere radius.
    pub bounds: Extent,
    /// Bounding volume for the separate collision geometry.
    pub collision_bounds: Extent,
    /// U16_ index array for collision triangles.
    pub collision_faces: Ref<targets::U16>,
    /// VEC3 positions for collision vertices.
    pub collision_verts: Ref<targets::Vector3>,
    /// VEC3 normal for each collision triangle.
    pub collision_normals: Ref<targets::Vector3>,
    /// ATT_ entries naming attachment locations on skeleton bones.
    pub attachment_points: Ref<targets::AttachmentPoint>,
    /// U16_ attachment addon data; entries are retained without interpretation.
    pub attachment_point_addons: Ref<targets::U16>,
    /// LITE section containing model-attached light sources.
    pub lights: Ref<targets::Light>,
    /// SHBX entries defining shadow-volume bounds.
    pub shadow_boxes: Ref<targets::ShadowBox>,
    /// CAM_ section containing animated camera definitions.
    pub cameras: Ref<targets::Camera>,
    /// U16_ camera addon data; entries are retained without interpretation.
    pub cameras_addons: Ref<targets::U16>,
    /// MATM entries selecting a material family and its index within that family.
    pub material_maps: Ref<targets::MaterialMap>,
    /// MAT_ section containing standard materials.
    pub standard_materials: Ref<targets::StandardMaterial>,
    /// DIS_ section containing displacement materials.
    pub displacement_materials: Ref<targets::DisplacementMaterial>,
    /// CMP_ section containing composite materials.
    pub composite_materials: Ref<targets::CompositeMaterial>,
    /// TER_ section containing terrain materials.
    pub terrain_materials: Ref<targets::TerrainMaterial>,
    /// VOL_ section containing volumetric materials.
    pub volume_materials: Ref<targets::VolumeMaterial>,
    /// HAI_ section containing legacy hair materials.
    pub hair_materials: Ref<targets::HairMaterial>,
    /// CREP section containing creep materials.
    pub creep_materials: Ref<targets::CreepMaterial>,
    /// VON_ section containing volume-noise materials when included by the selected revision.
    pub volume_noise_materials: D::VolumeNoiseMaterials,
    /// STBM section containing splat terrain bake materials when included by the selected revision.
    pub stb_materials: D::StbMaterials,
    /// REF_ section containing reflection materials when included by the selected revision.
    pub reflection_materials: D::ReflectionMaterials,
    /// LFLR section containing lens-flare materials when included by the selected revision.
    pub lens_flare_materials: D::LensFlareMaterials,
    /// MADD section containing data-driven materials when included by the selected revision.
    pub data_driven_materials: D::DataDrivenMaterials,
    /// PAR_ section containing particle emitter parameters and animation bindings.
    pub particle_emitters: Ref<targets::ParticleEmitter>,
    /// PARC entries instancing existing particle emitters.
    pub particle_emitter_copies: Ref<targets::ParticleEmitterCopy>,
    /// RIB_ section containing ribbon emitter parameters and animation bindings.
    pub ribbon_emitters: Ref<targets::RibbonEmitter>,
    /// PROJ entries defining projected materials or decals.
    pub projections: Ref<targets::Projector>,
    /// FOR_ entries describing force fields affecting effects.
    pub forces: Ref<targets::Force>,
    /// WRP_ entries describing animated warp effects.
    pub warps: Ref<targets::Warp>,
    /// VVOL view-volume reference in revisions that include this slot; otherwise ().
    pub view_volumes: D::ViewVolumes,
    /// PHRB section describing rigid-body associations.
    pub rigid_bodies: Ref<targets::RigidBody>,
    /// PHCT section reference retained even when a client does not use these constraints.
    pub physics_constraints: Ref<targets::PhysicsConstraint>,
    /// PHYJ section describing connections between physics bodies.
    pub physics_joints: Ref<targets::PhysicsJoint>,
    /// PHCL cloth simulation reference when present in the selected revision.
    pub cloth_physics: D::ClothPhysics,
    /// IK2J entries defining two-joint inverse-kinematics solvers.
    pub ik_two_joints: Ref<targets::IKTwoJoint>,
    /// IKCC solver reference when present in the selected revision.
    pub ik_c_c_d: D::IkCCD,
    /// IKJT entries defining inverse-kinematics joint parameters.
    pub ik_joints: Ref<targets::IKJoint>,
    /// PAOB entries defining one-bone solver parameters.
    pub one_bone_solvers: Ref<targets::OneBoneSolver>,
    /// PATU entries defining constrained turret bone motion.
    pub turret_behaviors: Ref<targets::TurretBehavior>,
    /// TRGD named groups selecting PATU turret parts.
    pub turret_groups: Ref<targets::TurretGroup>,
    /// IREF section containing the skeleton reference-pose matrices.
    pub initial_reference: Ref<targets::InitialReference>,
    /// Inline SSGS revision-1 shape used for precise hit testing.
    pub tight_hit_test_object: HitTestShape<V1>,
    /// SSGS section containing additional approximate hit-test shapes.
    pub fuzzy_hit_test_objects: Ref<targets::HitTestShape>,
    /// ATVL section defining attachment volumes.
    pub attachment_volumes: Ref<targets::AttachmentVolume>,
    /// First U16_ attachment-volume addon reference in revisions that serialize it.
    pub attachment_volumes_addon0: D::AttachmentVolumesAddon0,
    /// Second U16_ attachment-volume addon reference in revisions that serialize it.
    pub attachment_volumes_addon1: D::AttachmentVolumesAddon1,
    /// BBSC entries defining camera-facing bone behavior.
    pub billboard_behaviors: Ref<targets::BillboardBehavior>,
    /// TMD_ secondary-model reference when present in the selected revision.
    pub trailing_models: D::TrailingModels,
    /// Single M3A animation identity hash in revisions 21 and later; () in revision 20.
    pub m3a_anim_hash: D::M3aAnimHash,
    /// U32_ animation identity hash array reference in revisions 23 and later; otherwise ().
    pub m3a_anim_hashes: D::M3aAnimHashes,
}

impl<D: ModelDialect> Record for ModelRecord<D> {
    const TAG: [u8; 4] = *b"MODL";
    const VERSION: u32 = D::VERSION;
}

/// Optional single animation hash selected by a native MODL revision.
pub trait OptionalHash {
    /// Return the stored hash when this revision has one.
    fn optional_hash(&self) -> Option<u32>;
}
impl OptionalHash for () {
    fn optional_hash(&self) -> Option<u32> {
        None
    }
}
impl OptionalHash for u32 {
    fn optional_hash(&self) -> Option<u32> {
        Some(*self)
    }
}
/// Optional animation hash array selected by a native MODL revision.
pub trait OptionalHashes {
    /// Return the stored section reference when present.
    fn optional_hashes(&self) -> Option<Ref<targets::U32>>;
}
impl OptionalHashes for () {
    fn optional_hashes(&self) -> Option<Ref<targets::U32>> {
        None
    }
}
impl OptionalHashes for Ref<targets::U32> {
    fn optional_hashes(&self) -> Option<Ref<targets::U32>> {
        Some(*self)
    }
}
/// References needed to inspect packed geometry and separate M3A animation data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootMetadata {
    /// Native packed vertex-format selector.
    pub vertex_flags: VertexFlags,
    /// Packed vertex byte section.
    pub vertices: Ref<targets::Byte>,
    /// Skeleton bone section.
    pub bones: Ref<targets::Bone>,
    /// Optional single animation identity hash.
    pub m3a_anim_hash: Option<u32>,
    /// Optional per-animation hash section.
    pub m3a_anim_hashes: Option<Ref<targets::U32>>,
}
impl<D: ModelDialect> ModelRecord<D> {
    /// Extract geometry and animation-resource metadata without resolving sections.
    pub fn metadata(&self) -> RootMetadata {
        RootMetadata {
            vertex_flags: self.vertex_flags,
            vertices: self.vertices,
            bones: self.bones,
            m3a_anim_hash: self.m3a_anim_hash.optional_hash(),
            m3a_anim_hashes: self.m3a_anim_hashes.optional_hashes(),
        }
    }
}
