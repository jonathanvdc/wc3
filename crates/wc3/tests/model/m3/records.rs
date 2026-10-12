//! Native revision matrix and bit-preserving record codecs under both envelopes.
use m3::codec::{Decoder, Encoder};
use m3::records::*;
use wc3::model::m3::{self, Envelope, Md33, Md34, Read, RecordLayout, Write};
fn roundtrip<E: Envelope, T: for<'data> Read<'data, E> + Write<E> + RecordLayout<E>>() {
    let width = T::size();
    let bytes: Vec<u8> = (0..width)
        .map(|i| (i as u8).wrapping_mul(73).wrapping_add(19))
        .collect();
    let mut input = Decoder::<E>::new(&bytes);
    let value = T::read(&mut input).unwrap();
    assert_eq!(input.position(), width);
    let mut output = Encoder::<E>::new();
    value.write(&mut output).unwrap();
    assert_eq!(output.finish(), bytes);
    if width != 0 {
        assert!(T::read(&mut Decoder::<E>::new(&bytes[..width - 1])).is_err());
    }
}
#[test]
fn every_defined_record_revision_preserves_native_bits() {
    roundtrip::<Md33, Event<V0>>();
    roundtrip::<Md34, Event<V0>>();
    roundtrip::<Md33, Event<V1>>();
    roundtrip::<Md34, Event<V1>>();
    roundtrip::<Md33, Event<V2>>();
    roundtrip::<Md34, Event<V2>>();
    roundtrip::<Md33, Sequence<V0>>();
    roundtrip::<Md34, Sequence<V0>>();
    roundtrip::<Md33, Sequence<V1>>();
    roundtrip::<Md34, Sequence<V1>>();
    roundtrip::<Md33, Sequence<V2>>();
    roundtrip::<Md34, Sequence<V2>>();
    roundtrip::<Md33, SubTrackContainer<V0>>();
    roundtrip::<Md34, SubTrackContainer<V0>>();
    roundtrip::<Md33, SubTrackContainer<V1>>();
    roundtrip::<Md34, SubTrackContainer<V1>>();
    roundtrip::<Md33, SubTrackContainer<V2>>();
    roundtrip::<Md34, SubTrackContainer<V2>>();
    roundtrip::<Md33, SubTrackContainer<V3>>();
    roundtrip::<Md34, SubTrackContainer<V3>>();
    roundtrip::<Md33, SubTrackContainer<V4>>();
    roundtrip::<Md34, SubTrackContainer<V4>>();
    roundtrip::<Md33, AnimationGroup>();
    roundtrip::<Md34, AnimationGroup>();
    roundtrip::<Md33, AnimationState>();
    roundtrip::<Md34, AnimationState>();
    roundtrip::<Md33, BoneAnimationSet>();
    roundtrip::<Md34, BoneAnimationSet>();
    roundtrip::<Md33, Bone<V0>>();
    roundtrip::<Md34, Bone<V0>>();
    roundtrip::<Md33, Bone<V1>>();
    roundtrip::<Md34, Bone<V1>>();
    roundtrip::<Md33, Region<V0>>();
    roundtrip::<Md34, Region<V0>>();
    roundtrip::<Md33, Region<V1>>();
    roundtrip::<Md34, Region<V1>>();
    roundtrip::<Md33, Region<V2>>();
    roundtrip::<Md34, Region<V2>>();
    roundtrip::<Md33, Region<V3>>();
    roundtrip::<Md34, Region<V3>>();
    roundtrip::<Md33, Region<V4>>();
    roundtrip::<Md34, Region<V4>>();
    roundtrip::<Md33, Region<V5>>();
    roundtrip::<Md34, Region<V5>>();
    roundtrip::<Md33, Batch<V0>>();
    roundtrip::<Md34, Batch<V0>>();
    roundtrip::<Md33, Batch<V1>>();
    roundtrip::<Md34, Batch<V1>>();
    roundtrip::<Md33, MeshSection<V0>>();
    roundtrip::<Md34, MeshSection<V0>>();
    roundtrip::<Md33, MeshSection<V1>>();
    roundtrip::<Md34, MeshSection<V1>>();
    roundtrip::<Md33, MeshDivision<V0>>();
    roundtrip::<Md34, MeshDivision<V0>>();
    roundtrip::<Md33, MeshDivision<V1>>();
    roundtrip::<Md34, MeshDivision<V1>>();
    roundtrip::<Md33, MeshDivision<V2>>();
    roundtrip::<Md34, MeshDivision<V2>>();
    roundtrip::<Md33, InitialReference>();
    roundtrip::<Md34, InitialReference>();
    roundtrip::<Md33, AttachmentPoint<V0>>();
    roundtrip::<Md34, AttachmentPoint<V0>>();
    roundtrip::<Md33, AttachmentPoint<V1>>();
    roundtrip::<Md34, AttachmentPoint<V1>>();
    roundtrip::<Md33, MaterialMap>();
    roundtrip::<Md34, MaterialMap>();
    roundtrip::<Md33, TextureLayer<V0>>();
    roundtrip::<Md34, TextureLayer<V0>>();
    roundtrip::<Md33, TextureLayer<V1>>();
    roundtrip::<Md34, TextureLayer<V1>>();
    roundtrip::<Md33, TextureLayer<V2>>();
    roundtrip::<Md34, TextureLayer<V2>>();
    roundtrip::<Md33, TextureLayer<V3>>();
    roundtrip::<Md34, TextureLayer<V3>>();
    roundtrip::<Md33, TextureLayer<V4>>();
    roundtrip::<Md34, TextureLayer<V4>>();
    roundtrip::<Md33, TextureLayer<V5>>();
    roundtrip::<Md34, TextureLayer<V5>>();
    roundtrip::<Md33, TextureLayer<V6>>();
    roundtrip::<Md34, TextureLayer<V6>>();
    roundtrip::<Md33, TextureLayer<V7>>();
    roundtrip::<Md34, TextureLayer<V7>>();
    roundtrip::<Md33, TextureLayer<V8>>();
    roundtrip::<Md34, TextureLayer<V8>>();
    roundtrip::<Md33, TextureLayer<V9>>();
    roundtrip::<Md34, TextureLayer<V9>>();
    roundtrip::<Md33, TextureLayer<V10>>();
    roundtrip::<Md34, TextureLayer<V10>>();
    roundtrip::<Md33, TextureLayer<V11>>();
    roundtrip::<Md34, TextureLayer<V11>>();
    roundtrip::<Md33, TextureLayer<V12>>();
    roundtrip::<Md34, TextureLayer<V12>>();
    roundtrip::<Md33, TextureLayer<V13>>();
    roundtrip::<Md34, TextureLayer<V13>>();
    roundtrip::<Md33, TextureLayer<V14>>();
    roundtrip::<Md34, TextureLayer<V14>>();
    roundtrip::<Md33, TextureLayer<V15>>();
    roundtrip::<Md34, TextureLayer<V15>>();
    roundtrip::<Md33, TextureLayer<V16>>();
    roundtrip::<Md34, TextureLayer<V16>>();
    roundtrip::<Md33, TextureLayer<V17>>();
    roundtrip::<Md34, TextureLayer<V17>>();
    roundtrip::<Md33, TextureLayer<V18>>();
    roundtrip::<Md34, TextureLayer<V18>>();
    roundtrip::<Md33, TextureLayer<V19>>();
    roundtrip::<Md34, TextureLayer<V19>>();
    roundtrip::<Md33, TextureLayer<V20>>();
    roundtrip::<Md34, TextureLayer<V20>>();
    roundtrip::<Md33, TextureLayer<V21>>();
    roundtrip::<Md34, TextureLayer<V21>>();
    roundtrip::<Md33, TextureLayer<V22>>();
    roundtrip::<Md34, TextureLayer<V22>>();
    roundtrip::<Md33, TextureLayer<V23>>();
    roundtrip::<Md34, TextureLayer<V23>>();
    roundtrip::<Md33, TextureLayer<V24>>();
    roundtrip::<Md34, TextureLayer<V24>>();
    roundtrip::<Md33, TextureLayer<V25>>();
    roundtrip::<Md34, TextureLayer<V25>>();
    roundtrip::<Md33, TextureLayer<V26>>();
    roundtrip::<Md34, TextureLayer<V26>>();
    roundtrip::<Md33, StandardMaterial<V0>>();
    roundtrip::<Md34, StandardMaterial<V0>>();
    roundtrip::<Md33, StandardMaterial<V1>>();
    roundtrip::<Md34, StandardMaterial<V1>>();
    roundtrip::<Md33, StandardMaterial<V2>>();
    roundtrip::<Md34, StandardMaterial<V2>>();
    roundtrip::<Md33, StandardMaterial<V3>>();
    roundtrip::<Md34, StandardMaterial<V3>>();
    roundtrip::<Md33, StandardMaterial<V4>>();
    roundtrip::<Md34, StandardMaterial<V4>>();
    roundtrip::<Md33, StandardMaterial<V5>>();
    roundtrip::<Md34, StandardMaterial<V5>>();
    roundtrip::<Md33, StandardMaterial<V6>>();
    roundtrip::<Md34, StandardMaterial<V6>>();
    roundtrip::<Md33, StandardMaterial<V7>>();
    roundtrip::<Md34, StandardMaterial<V7>>();
    roundtrip::<Md33, StandardMaterial<V8>>();
    roundtrip::<Md34, StandardMaterial<V8>>();
    roundtrip::<Md33, StandardMaterial<V9>>();
    roundtrip::<Md34, StandardMaterial<V9>>();
    roundtrip::<Md33, StandardMaterial<V10>>();
    roundtrip::<Md34, StandardMaterial<V10>>();
    roundtrip::<Md33, StandardMaterial<V11>>();
    roundtrip::<Md34, StandardMaterial<V11>>();
    roundtrip::<Md33, StandardMaterial<V12>>();
    roundtrip::<Md34, StandardMaterial<V12>>();
    roundtrip::<Md33, StandardMaterial<V13>>();
    roundtrip::<Md34, StandardMaterial<V13>>();
    roundtrip::<Md33, StandardMaterial<V14>>();
    roundtrip::<Md34, StandardMaterial<V14>>();
    roundtrip::<Md33, StandardMaterial<V15>>();
    roundtrip::<Md34, StandardMaterial<V15>>();
    roundtrip::<Md33, StandardMaterial<V16>>();
    roundtrip::<Md34, StandardMaterial<V16>>();
    roundtrip::<Md33, StandardMaterial<V17>>();
    roundtrip::<Md34, StandardMaterial<V17>>();
    roundtrip::<Md33, StandardMaterial<V18>>();
    roundtrip::<Md34, StandardMaterial<V18>>();
    roundtrip::<Md33, StandardMaterial<V19>>();
    roundtrip::<Md34, StandardMaterial<V19>>();
    roundtrip::<Md33, StandardMaterial<V20>>();
    roundtrip::<Md34, StandardMaterial<V20>>();
    roundtrip::<Md33, DisplacementMaterial<V0>>();
    roundtrip::<Md34, DisplacementMaterial<V0>>();
    roundtrip::<Md33, DisplacementMaterial<V1>>();
    roundtrip::<Md34, DisplacementMaterial<V1>>();
    roundtrip::<Md33, DisplacementMaterial<V2>>();
    roundtrip::<Md34, DisplacementMaterial<V2>>();
    roundtrip::<Md33, DisplacementMaterial<V3>>();
    roundtrip::<Md34, DisplacementMaterial<V3>>();
    roundtrip::<Md33, DisplacementMaterial<V4>>();
    roundtrip::<Md34, DisplacementMaterial<V4>>();
    roundtrip::<Md33, CompositeSection>();
    roundtrip::<Md34, CompositeSection>();
    roundtrip::<Md33, CompositeMaterial<V0>>();
    roundtrip::<Md34, CompositeMaterial<V0>>();
    roundtrip::<Md33, CompositeMaterial<V1>>();
    roundtrip::<Md34, CompositeMaterial<V1>>();
    roundtrip::<Md33, CompositeMaterial<V2>>();
    roundtrip::<Md34, CompositeMaterial<V2>>();
    roundtrip::<Md33, TerrainMaterial<V0>>();
    roundtrip::<Md34, TerrainMaterial<V0>>();
    roundtrip::<Md33, TerrainMaterial<V1>>();
    roundtrip::<Md34, TerrainMaterial<V1>>();
    roundtrip::<Md33, VolumeMaterial>();
    roundtrip::<Md34, VolumeMaterial>();
    roundtrip::<Md33, HairMaterial>();
    roundtrip::<Md34, HairMaterial>();
    roundtrip::<Md33, VolumeNoiseMaterial>();
    roundtrip::<Md34, VolumeNoiseMaterial>();
    roundtrip::<Md33, CreepMaterial<V0>>();
    roundtrip::<Md34, CreepMaterial<V0>>();
    roundtrip::<Md33, CreepMaterial<V1>>();
    roundtrip::<Md34, CreepMaterial<V1>>();
    roundtrip::<Md33, STBMaterial>();
    roundtrip::<Md34, STBMaterial>();
    roundtrip::<Md33, ReflectionMaterial<V0>>();
    roundtrip::<Md34, ReflectionMaterial<V0>>();
    roundtrip::<Md33, ReflectionMaterial<V1>>();
    roundtrip::<Md34, ReflectionMaterial<V1>>();
    roundtrip::<Md33, ReflectionMaterial<V2>>();
    roundtrip::<Md34, ReflectionMaterial<V2>>();
    roundtrip::<Md33, ReflectionMaterial<V3>>();
    roundtrip::<Md34, ReflectionMaterial<V3>>();
    roundtrip::<Md33, SubFlare<V0>>();
    roundtrip::<Md34, SubFlare<V0>>();
    roundtrip::<Md33, SubFlare<V1>>();
    roundtrip::<Md34, SubFlare<V1>>();
    roundtrip::<Md33, SubFlare<V2>>();
    roundtrip::<Md34, SubFlare<V2>>();
    roundtrip::<Md33, LensFlare<V0>>();
    roundtrip::<Md34, LensFlare<V0>>();
    roundtrip::<Md33, LensFlare<V1>>();
    roundtrip::<Md34, LensFlare<V1>>();
    roundtrip::<Md33, LensFlare<V2>>();
    roundtrip::<Md34, LensFlare<V2>>();
    roundtrip::<Md33, LensFlare<V3>>();
    roundtrip::<Md34, LensFlare<V3>>();
    roundtrip::<Md33, DataDrivenMaterial<V0>>();
    roundtrip::<Md34, DataDrivenMaterial<V0>>();
    roundtrip::<Md33, DataDrivenMaterial<V1>>();
    roundtrip::<Md34, DataDrivenMaterial<V1>>();
    roundtrip::<Md33, DataDrivenMaterial<V2>>();
    roundtrip::<Md34, DataDrivenMaterial<V2>>();
    roundtrip::<Md33, DataDrivenMaterial<V3>>();
    roundtrip::<Md34, DataDrivenMaterial<V3>>();
    roundtrip::<Md33, ParticleEmitter<V10>>();
    roundtrip::<Md34, ParticleEmitter<V10>>();
    roundtrip::<Md33, ParticleEmitter<V11>>();
    roundtrip::<Md34, ParticleEmitter<V11>>();
    roundtrip::<Md33, ParticleEmitter<V12>>();
    roundtrip::<Md34, ParticleEmitter<V12>>();
    roundtrip::<Md33, ParticleEmitter<V13>>();
    roundtrip::<Md34, ParticleEmitter<V13>>();
    roundtrip::<Md33, ParticleEmitter<V14>>();
    roundtrip::<Md34, ParticleEmitter<V14>>();
    roundtrip::<Md33, ParticleEmitter<V15>>();
    roundtrip::<Md34, ParticleEmitter<V15>>();
    roundtrip::<Md33, ParticleEmitter<V16>>();
    roundtrip::<Md34, ParticleEmitter<V16>>();
    roundtrip::<Md33, ParticleEmitter<V17>>();
    roundtrip::<Md34, ParticleEmitter<V17>>();
    roundtrip::<Md33, ParticleEmitter<V18>>();
    roundtrip::<Md34, ParticleEmitter<V18>>();
    roundtrip::<Md33, ParticleEmitter<V19>>();
    roundtrip::<Md34, ParticleEmitter<V19>>();
    roundtrip::<Md33, ParticleEmitter<V20>>();
    roundtrip::<Md34, ParticleEmitter<V20>>();
    roundtrip::<Md33, ParticleEmitter<V21>>();
    roundtrip::<Md34, ParticleEmitter<V21>>();
    roundtrip::<Md33, ParticleEmitter<V22>>();
    roundtrip::<Md34, ParticleEmitter<V22>>();
    roundtrip::<Md33, ParticleEmitter<V23>>();
    roundtrip::<Md34, ParticleEmitter<V23>>();
    roundtrip::<Md33, ParticleEmitter<V24>>();
    roundtrip::<Md34, ParticleEmitter<V24>>();
    roundtrip::<Md33, ParticleEmitterCopy>();
    roundtrip::<Md34, ParticleEmitterCopy>();
    roundtrip::<Md33, SplineRibbon>();
    roundtrip::<Md34, SplineRibbon>();
    roundtrip::<Md33, RibbonEmitter<V4>>();
    roundtrip::<Md34, RibbonEmitter<V4>>();
    roundtrip::<Md33, RibbonEmitter<V5>>();
    roundtrip::<Md34, RibbonEmitter<V5>>();
    roundtrip::<Md33, RibbonEmitter<V6>>();
    roundtrip::<Md34, RibbonEmitter<V6>>();
    roundtrip::<Md33, RibbonEmitter<V7>>();
    roundtrip::<Md34, RibbonEmitter<V7>>();
    roundtrip::<Md33, RibbonEmitter<V8>>();
    roundtrip::<Md34, RibbonEmitter<V8>>();
    roundtrip::<Md33, RibbonEmitter<V9>>();
    roundtrip::<Md34, RibbonEmitter<V9>>();
    roundtrip::<Md33, Projector<V4>>();
    roundtrip::<Md34, Projector<V4>>();
    roundtrip::<Md33, Projector<V5>>();
    roundtrip::<Md34, Projector<V5>>();
    roundtrip::<Md33, HitTestShape<V0>>();
    roundtrip::<Md34, HitTestShape<V0>>();
    roundtrip::<Md33, HitTestShape<V1>>();
    roundtrip::<Md34, HitTestShape<V1>>();
    roundtrip::<Md33, AttachmentVolume>();
    roundtrip::<Md34, AttachmentVolume>();
    roundtrip::<Md33, TriggerData>();
    roundtrip::<Md34, TriggerData>();
    roundtrip::<Md33, TurretBehavior<V0>>();
    roundtrip::<Md34, TurretBehavior<V0>>();
    roundtrip::<Md33, TurretBehavior<V1>>();
    roundtrip::<Md34, TurretBehavior<V1>>();
    roundtrip::<Md33, TurretBehavior<V2>>();
    roundtrip::<Md34, TurretBehavior<V2>>();
    roundtrip::<Md33, TurretBehavior<V3>>();
    roundtrip::<Md34, TurretBehavior<V3>>();
    roundtrip::<Md33, TurretBehavior<V4>>();
    roundtrip::<Md34, TurretBehavior<V4>>();
    roundtrip::<Md33, BillboardBehavior>();
    roundtrip::<Md34, BillboardBehavior>();
    roundtrip::<Md33, IKJoint>();
    roundtrip::<Md34, IKJoint>();
    roundtrip::<Md33, IKTwoJoint>();
    roundtrip::<Md34, IKTwoJoint>();
    roundtrip::<Md33, IKCCD>();
    roundtrip::<Md34, IKCCD>();
    roundtrip::<Md33, OneBoneSolver>();
    roundtrip::<Md34, OneBoneSolver>();
    roundtrip::<Md33, ShadowBox>();
    roundtrip::<Md34, ShadowBox>();
    roundtrip::<Md33, ViewVolume>();
    roundtrip::<Md34, ViewVolume>();
    roundtrip::<Md33, TrailingModel<V0>>();
    roundtrip::<Md34, TrailingModel<V0>>();
    roundtrip::<Md33, TrailingModel<V1>>();
    roundtrip::<Md34, TrailingModel<V1>>();
    roundtrip::<Md33, Force<V0>>();
    roundtrip::<Md34, Force<V0>>();
    roundtrip::<Md33, Force<V1>>();
    roundtrip::<Md34, Force<V1>>();
    roundtrip::<Md33, Force<V2>>();
    roundtrip::<Md34, Force<V2>>();
    roundtrip::<Md33, Warp>();
    roundtrip::<Md34, Warp>();
    roundtrip::<Md33, ConvexHullHalfEdge>();
    roundtrip::<Md34, ConvexHullHalfEdge>();
    roundtrip::<Md33, PhysicsMeshBvhNode<V0>>();
    roundtrip::<Md34, PhysicsMeshBvhNode<V0>>();
    roundtrip::<Md33, PhysicsMeshBvhNode<V1>>();
    roundtrip::<Md34, PhysicsMeshBvhNode<V1>>();
    roundtrip::<Md33, PhysicsMeshTriangle>();
    roundtrip::<Md34, PhysicsMeshTriangle>();
    roundtrip::<Md33, PhysicsMeshEdge>();
    roundtrip::<Md34, PhysicsMeshEdge>();
    roundtrip::<Md33, PhysicsShape<V0>>();
    roundtrip::<Md34, PhysicsShape<V0>>();
    roundtrip::<Md33, PhysicsShape<V1>>();
    roundtrip::<Md34, PhysicsShape<V1>>();
    roundtrip::<Md33, PhysicsShape<V2>>();
    roundtrip::<Md34, PhysicsShape<V2>>();
    roundtrip::<Md33, PhysicsShape<V3>>();
    roundtrip::<Md34, PhysicsShape<V3>>();
    roundtrip::<Md33, RigidBody<V0>>();
    roundtrip::<Md34, RigidBody<V0>>();
    roundtrip::<Md33, RigidBody<V1>>();
    roundtrip::<Md34, RigidBody<V1>>();
    roundtrip::<Md33, RigidBody<V2>>();
    roundtrip::<Md34, RigidBody<V2>>();
    roundtrip::<Md33, RigidBody<V3>>();
    roundtrip::<Md34, RigidBody<V3>>();
    roundtrip::<Md33, RigidBody<V4>>();
    roundtrip::<Md34, RigidBody<V4>>();
    roundtrip::<Md33, PhysicsJoint>();
    roundtrip::<Md34, PhysicsJoint>();
    roundtrip::<Md33, ClothCollider>();
    roundtrip::<Md34, ClothCollider>();
    roundtrip::<Md33, ClothProxy>();
    roundtrip::<Md34, ClothProxy>();
    roundtrip::<Md33, ClothPhysics<V0>>();
    roundtrip::<Md34, ClothPhysics<V0>>();
    roundtrip::<Md33, ClothPhysics<V1>>();
    roundtrip::<Md34, ClothPhysics<V1>>();
    roundtrip::<Md33, ClothPhysics<V2>>();
    roundtrip::<Md34, ClothPhysics<V2>>();
    roundtrip::<Md33, ClothPhysics<V3>>();
    roundtrip::<Md34, ClothPhysics<V3>>();
    roundtrip::<Md33, ClothPhysics<V4>>();
    roundtrip::<Md34, ClothPhysics<V4>>();
    roundtrip::<Md33, Light<V0>>();
    roundtrip::<Md34, Light<V0>>();
    roundtrip::<Md33, Light<V1>>();
    roundtrip::<Md34, Light<V1>>();
    roundtrip::<Md33, Light<V2>>();
    roundtrip::<Md34, Light<V2>>();
    roundtrip::<Md33, Light<V3>>();
    roundtrip::<Md34, Light<V3>>();
    roundtrip::<Md33, Light<V4>>();
    roundtrip::<Md34, Light<V4>>();
    roundtrip::<Md33, Light<V5>>();
    roundtrip::<Md34, Light<V5>>();
    roundtrip::<Md33, Light<V6>>();
    roundtrip::<Md34, Light<V6>>();
    roundtrip::<Md33, Light<V7>>();
    roundtrip::<Md34, Light<V7>>();
    roundtrip::<Md33, Camera<V2>>();
    roundtrip::<Md34, Camera<V2>>();
    roundtrip::<Md33, Camera<V3>>();
    roundtrip::<Md34, Camera<V3>>();
    roundtrip::<Md33, Camera<V4>>();
    roundtrip::<Md34, Camera<V4>>();
    roundtrip::<Md33, Camera<V5>>();
    roundtrip::<Md34, Camera<V5>>();
    roundtrip::<Md33, ModelRecord<V20>>();
    roundtrip::<Md34, ModelRecord<V20>>();
    roundtrip::<Md33, ModelRecord<V21>>();
    roundtrip::<Md34, ModelRecord<V21>>();
    roundtrip::<Md33, ModelRecord<V22>>();
    roundtrip::<Md34, ModelRecord<V22>>();
    roundtrip::<Md33, ModelRecord<V23>>();
    roundtrip::<Md34, ModelRecord<V23>>();
    roundtrip::<Md33, ModelRecord<V24>>();
    roundtrip::<Md34, ModelRecord<V24>>();
    roundtrip::<Md33, ModelRecord<V25>>();
    roundtrip::<Md34, ModelRecord<V25>>();
    roundtrip::<Md33, ModelRecord<V26>>();
    roundtrip::<Md34, ModelRecord<V26>>();
    roundtrip::<Md33, ModelRecord<V27>>();
    roundtrip::<Md34, ModelRecord<V27>>();
    roundtrip::<Md33, ModelRecord<V28>>();
    roundtrip::<Md34, ModelRecord<V28>>();
    roundtrip::<Md33, ModelRecord<V29>>();
    roundtrip::<Md34, ModelRecord<V29>>();
    roundtrip::<Md33, ModelRecord<V30>>();
    roundtrip::<Md34, ModelRecord<V30>>();
    roundtrip::<Md33, Character>();
    roundtrip::<Md34, Character>();
    roundtrip::<Md33, Byte>();
    roundtrip::<Md34, Byte>();
    roundtrip::<Md33, U16>();
    roundtrip::<Md34, U16>();
    roundtrip::<Md33, U32>();
    roundtrip::<Md34, U32>();
    roundtrip::<Md33, U64>();
    roundtrip::<Md34, U64>();
    roundtrip::<Md33, I16>();
    roundtrip::<Md34, I16>();
    roundtrip::<Md33, I32>();
    roundtrip::<Md34, I32>();
    roundtrip::<Md33, Real>();
    roundtrip::<Md34, Real>();
    roundtrip::<Md33, Flag>();
    roundtrip::<Md34, Flag>();
    roundtrip::<Md33, Vector2>();
    roundtrip::<Md34, Vector2>();
    roundtrip::<Md33, Vector3>();
    roundtrip::<Md34, Vector3>();
    roundtrip::<Md33, Vector4>();
    roundtrip::<Md34, Vector4>();
    roundtrip::<Md33, Quaternion>();
    roundtrip::<Md34, Quaternion>();
    roundtrip::<Md33, Color>();
    roundtrip::<Md34, Color>();
    roundtrip::<Md33, ExtentRecord>();
    roundtrip::<Md34, ExtentRecord>();
    roundtrip::<Md33, AnimationBlockEvent>();
    roundtrip::<Md34, AnimationBlockEvent>();
    roundtrip::<Md33, AnimationBlockVector2>();
    roundtrip::<Md34, AnimationBlockVector2>();
    roundtrip::<Md33, AnimationBlockVector3>();
    roundtrip::<Md34, AnimationBlockVector3>();
    roundtrip::<Md33, AnimationBlockReal>();
    roundtrip::<Md34, AnimationBlockReal>();
    roundtrip::<Md33, AnimationBlockQuaternion>();
    roundtrip::<Md34, AnimationBlockQuaternion>();
    roundtrip::<Md33, AnimationBlockColor>();
    roundtrip::<Md34, AnimationBlockColor>();
    roundtrip::<Md33, AnimationBlockByte>();
    roundtrip::<Md34, AnimationBlockByte>();
    roundtrip::<Md33, AnimationBlockI16>();
    roundtrip::<Md34, AnimationBlockI16>();
    roundtrip::<Md33, AnimationBlockU16>();
    roundtrip::<Md34, AnimationBlockU16>();
    roundtrip::<Md33, AnimationBlockI32>();
    roundtrip::<Md34, AnimationBlockI32>();
    roundtrip::<Md33, AnimationBlockU32>();
    roundtrip::<Md34, AnimationBlockU32>();
    roundtrip::<Md33, AnimationBlockFlag>();
    roundtrip::<Md34, AnimationBlockFlag>();
    roundtrip::<Md33, AnimationBlockExtent>();
    roundtrip::<Md34, AnimationBlockExtent>();
    roundtrip::<Md33, AnimationValueI32>();
    roundtrip::<Md34, AnimationValueI32>();
    roundtrip::<Md33, AnimationValueU32>();
    roundtrip::<Md34, AnimationValueU32>();
    roundtrip::<Md33, AnimationValueReal>();
    roundtrip::<Md34, AnimationValueReal>();
    roundtrip::<Md33, AnimationValueVector3>();
    roundtrip::<Md34, AnimationValueVector3>();
    roundtrip::<Md33, AnimationValueVector2>();
    roundtrip::<Md34, AnimationValueVector2>();
    roundtrip::<Md33, MeshTriangle16>();
    roundtrip::<Md34, MeshTriangle16>();
    roundtrip::<Md33, MeshTriangle32>();
    roundtrip::<Md34, MeshTriangle32>();
    roundtrip::<Md33, StringReference>();
    roundtrip::<Md34, StringReference>();
    roundtrip::<Md33, PhysicsConstraint>();
    roundtrip::<Md34, PhysicsConstraint>();
}
#[test]
fn observed_sizes_follow_independent_native_layout_facts() {
    fn size(tag: [u8; 4], revision: u32, md34: usize, md33: usize) {
        assert_eq!(known_record_size::<Md34>(tag, revision), Some(md34));
        assert_eq!(known_record_size::<Md33>(tag, revision), Some(md33));
    }
    for (version, bytes) in [
        (23, 784),
        (24, 796),
        (25, 808),
        (26, 820),
        (28, 844),
        (29, 856),
        (30, 868),
    ] {
        assert_eq!(known_record_size::<Md34>(*b"MODL", version), Some(bytes));
    }
    size(*b"MODL", 20, 744, 548);
    size(*b"MODL", 21, 760, 560);
    size(*b"SEQS", 1, 96, 88);
    size(*b"SEQS", 2, 92, 84);
    size(*b"STC_", 4, 204, 140);
    size(*b"BONE", 1, 160, 156);
    size(*b"REGN", 2, 28, 28);
    size(*b"REGN", 3, 36, 36);
    size(*b"REGN", 4, 40, 40);
    size(*b"REGN", 5, 48, 48);
    size(*b"CAM_", 2, 148, 144);
    size(*b"CAM_", 3, 180, 176);
    size(*b"CAM_", 4, 220, 216);
    size(*b"CAM_", 5, 264, 260);
    for (version, bytes) in [
        (20, 356),
        (21, 356),
        (22, 356),
        (23, 428),
        (24, 436),
        (25, 468),
        (26, 464),
    ] {
        size(*b"LAYR", version, bytes, bytes - 4);
    }
    for (version, bytes, refs) in [
        (15, 268, 14),
        (16, 280, 15),
        (17, 280, 15),
        (18, 280, 15),
        (19, 340, 20),
        (20, 352, 20),
    ] {
        size(*b"MAT_", version, bytes, bytes - 4 * refs);
    }
    for (version, bytes) in [(0, 96), (1, 132), (2, 292), (3, 300)] {
        assert_eq!(known_record_size::<Md34>(*b"PHSH", version), Some(bytes));
    }
    for (version, bytes) in [(0, 72), (1, 96), (2, 104), (3, 56), (4, 80)] {
        size(*b"PHRB", version, bytes, bytes - 4);
    }
    for (version, bytes) in [(0, 140), (1, 116), (2, 128), (3, 192), (4, 192)] {
        assert_eq!(known_record_size::<Md34>(*b"PHCL", version), Some(bytes));
    }
    size(*b"EVNT", 0, 100, 92);
    size(*b"EVNT", 1, 104, 96);
    size(*b"EVNT", 2, 108, 100);
    assert_eq!(known_record_size::<Md34>(*b"M3??", 0), None);
    assert_eq!(known_record_size::<Md34>(*b"BONE", 999), None);
}
#[test]
fn region_beta_field_offsets_preserve_both_index_range_words() {
    let mut bytes = [0u8; 28];
    bytes[4..6].copy_from_slice(&7u16.to_le_bytes());
    bytes[6..8].copy_from_slice(&8u16.to_le_bytes());
    bytes[8..12].copy_from_slice(&90_000u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&12u32.to_le_bytes());
    let region = Region::<V2>::read(&mut Decoder::<Md33>::new(&bytes)).unwrap();
    assert_eq!(region.first_vertex, 7);
    assert_eq!(region.vertex_count, 8);
    assert_eq!(region.first_index, 90_000);
    assert_eq!(region.index_count, 12);
}
#[test]
fn nested_and_discarded_references_remain_visible_in_field_order() {
    let value = ModelRecord::<V29> {
        physics_constraints: m3::Ref::new(1, 23, 7),
        tight_hit_test_object: HitTestShape::<V1> {
            vertex_positions: m3::Ref::new(3, 24, 9),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut output = Encoder::<Md34>::new();
    value.write(&mut output).unwrap();
    let bytes = output.finish();
    let mut seen = Vec::new();
    validate_known_section::<Md34>(*b"MODL", 29, &bytes, 1, &mut |reference| {
        if reference.entries != 0 {
            seen.push(reference);
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].target, Some(*b"PHCT"));
    assert_eq!((seen[0].index, seen[0].flags), (23, 7));
    assert_eq!(seen[1].target, Some(*b"VEC3"));
    assert_eq!((seen[1].index, seen[1].flags), (24, 9));
    let mut extended = bytes;
    extended.push(0);
    assert!(validate_known_section::<Md34>(*b"MODL", 29, &extended, 1, &mut |_| Ok(())).is_err());
}
#[test]
fn floating_point_nan_payloads_are_not_canonicalized() {
    let bytes = 0x7fc12345u32.to_le_bytes();
    let value = Real::read(&mut Decoder::<Md34>::new(&bytes)).unwrap();
    assert!(value.value.is_nan());
    let mut output = Encoder::<Md34>::new();
    value.write(&mut output).unwrap();
    assert_eq!(output.finish(), bytes);
}

#[test]
fn old_bvh_normals_and_reserved_material_words_keep_their_native_roles() {
    assert_eq!(known_record_size::<Md34>(*b"DMMN", 0), Some(12));
    assert_eq!(known_record_size::<Md34>(*b"DMMN", 1), Some(8));
    let mut material = DataDrivenMaterial::<V3>::default();
    material.reserved_0.entries = 17;
    material.reserved_0.index = u32::MAX;
    material.reserved_0.flags = 93;
    let mut output = Encoder::<Md34>::new();
    material.write(&mut output).unwrap();
    let bytes = output.finish();
    let mut active = Vec::new();
    validate_known_section::<Md34>(*b"MADD", 3, &bytes, 1, &mut |reference| {
        if reference.entries != 0 {
            active.push(reference);
        }
        Ok(())
    })
    .unwrap();
    assert!(active.is_empty());
    let decoded = DataDrivenMaterial::<V3>::read(&mut Decoder::<Md34>::new(&bytes)).unwrap();
    assert_eq!(decoded.reserved_0, material.reserved_0);
}

#[test]
fn descriptor_shaped_material_metadata_is_not_a_document_graph_edge() {
    let mut material = DataDrivenMaterial::<V3>::default();
    material.reserved_0.entries = u32::MAX;
    material.reserved_0.index = u32::MAX;
    material.reserved_0.flags = 0xdead_beef;
    let mut document = m3::Document::<Md34>::new();
    let id = document.append(&[material]).unwrap();
    document.validate().unwrap();
    let bytes = document.write().unwrap();
    let decoded = m3::Document::<Md34>::read(&bytes).unwrap();
    decoded.validate().unwrap();
    let decoded_id = decoded.section_id(id.index()).unwrap();
    let array = decoded.array::<DataDrivenMaterial<V3>>(decoded_id).unwrap();
    assert_eq!(
        array.get(0).unwrap().unwrap().reserved_0,
        material.reserved_0
    );
}
