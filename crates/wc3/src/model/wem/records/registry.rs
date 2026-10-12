//! Exact dispatch of known section revisions for boundary and reference validation.
use super::*;
use crate::model::wem::{
    Decoder, NativeAbi, RawReference, Read, ReadError, ReadErrorKind, RecordLayout,
    ReferenceVisitor, Validate, VisitReferences,
};
/// Identity of an implemented section layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordDescriptor {
    /// Literal section FourCC, in file byte order.
    pub tag: [u8; 4],
    /// Independently selected payload revision.
    pub revision: u32,
}
/// All supported native section identities; ABI-sensitive sections require an explicit selection.
pub const SUPPORTED_RECORDS: &[RecordDescriptor] = &[
    RecordDescriptor {
        tag: *b"CHAR",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"U8__",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"U16_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"U32_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"U64_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"I16_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"I32_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"REAL",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"VEC2",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"VEC3",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"VEC4",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"QUAT",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"BNDS",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"BYT4",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"WDOC",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 8,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 9,
    },
    RecordDescriptor {
        tag: *b"PROF",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"SBND",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MESH",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MESH",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"SECT",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"ATTR",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"INFL",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"VSPL",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"FDRP",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"XFRM",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MTX4",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 8,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 9,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 10,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 11,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 12,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 13,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 14,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 15,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 16,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 17,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 18,
    },
    RecordDescriptor {
        tag: *b"NODE",
        revision: 19,
    },
    RecordDescriptor {
        tag: *b"TPSR",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PDLT",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"TPOS",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"AKEY",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"S2P3",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"S2RS",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"NBAG",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"NBAG",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PSCH",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PSCH",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"MATL",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MFEA",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"FFRS",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"FUVA",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"FLSH",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MKCP",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MKCP",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"MKCL",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MKCB",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MKCS",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MKLD",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MKPB",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"TEXR",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"LOOK",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"ACHN",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"STRK",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"STRK",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"STCC",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"CLEV",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"CLIP",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"CLIP",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"CLIP",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"CLIP",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"CLIP",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"CLIP",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"CLIP",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"CPWF",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"TSET",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PSTG",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PSTG",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PSRC",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PBDY",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PBDY",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PBDY",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PSHP",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PSHP",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PJNT",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PJNT",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PJNT",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PCOL",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PCOL",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"CLTH",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"CLTH",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"CBND",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"PRIG",
        revision: 8,
    },
    RecordDescriptor {
        tag: *b"SBDY",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"SLMP",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"SEFX",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"SPRT",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"CTSU",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"ATAG",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"ASET",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"NMDX",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"NM2_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"ND3_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"ND3_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"NM3_",
        revision: 1,
    },
];
fn scan<T: RecordLayout + for<'a> Read<'a>>(
    payload: &[u8],
    count: u32,
    revision: u32,
    abi: Option<NativeAbi>,
) -> Result<usize, ReadError> {
    let minimum = T::minimum_size();
    let lower = (count as usize)
        .checked_mul(minimum)
        .ok_or_else(|| ReadError::invalid(0, "record count"))?;
    if lower > payload.len() || count != 0 && minimum == 0 {
        return Err(ReadError::invalid(0, "record count"));
    }
    if let Some(stride) = T::fixed_size() {
        let length = (count as usize)
            .checked_mul(stride)
            .ok_or_else(|| ReadError::invalid(0, "record span"))?;
        if length > payload.len() {
            return Err(ReadError::invalid(0, "record span"));
        }
        return Ok(length);
    }
    let mut input = Decoder::new(payload, revision).with_native_abi(abi);
    for _ in 0..count {
        T::skip(&mut input)?;
    }
    Ok(input.position())
}
/// Determine the meaningful length of a known section without allocating decoded inline lists.
///
/// Unsupported tags/revisions and an unselected native ABI return None. Trailing
/// container alignment bytes are excluded from the returned span.
pub fn known_record_span(
    tag: [u8; 4],
    revision: u32,
    payload: &[u8],
    count: u32,
    abi: Option<NativeAbi>,
) -> Result<Option<usize>, ReadError> {
    let result: Result<Option<usize>, ReadError> = (|| {
        let span = match (&tag, revision) {
            (b"CHAR", 0) => scan::<Character>(payload, count, revision, abi)?,
            (b"U8__", 0) => scan::<Byte>(payload, count, revision, abi)?,
            (b"U16_", 0) => scan::<U16>(payload, count, revision, abi)?,
            (b"U32_", 0) => scan::<U32>(payload, count, revision, abi)?,
            (b"U64_", 0) => scan::<U64>(payload, count, revision, abi)?,
            (b"I16_", 0) => scan::<I16>(payload, count, revision, abi)?,
            (b"I32_", 0) => scan::<I32>(payload, count, revision, abi)?,
            (b"REAL", 0) => scan::<Real>(payload, count, revision, abi)?,
            (b"VEC2", 0) => scan::<Vector2>(payload, count, revision, abi)?,
            (b"VEC3", 0) => scan::<Vector3>(payload, count, revision, abi)?,
            (b"VEC4", 0) => scan::<Vector4>(payload, count, revision, abi)?,
            (b"QUAT", 0) => scan::<QuaternionRecord>(payload, count, revision, abi)?,
            (b"BNDS", 0) => scan::<Extent>(payload, count, revision, abi)?,
            (b"BYT4", 0) => scan::<Byte4>(payload, count, revision, abi)?,
            (b"WDOC", 1) => scan::<DocumentRecord<V1>>(payload, count, revision, abi)?,
            (b"MODL", 1) => scan::<ModelRecord<V1>>(payload, count, revision, abi)?,
            (b"MODL", 2) => scan::<ModelRecord<V2>>(payload, count, revision, abi)?,
            (b"MODL", 3) => scan::<ModelRecord<V3>>(payload, count, revision, abi)?,
            (b"MODL", 4) => scan::<ModelRecord<V4>>(payload, count, revision, abi)?,
            (b"MODL", 5) => scan::<ModelRecord<V5>>(payload, count, revision, abi)?,
            (b"MODL", 6) => scan::<ModelRecord<V6>>(payload, count, revision, abi)?,
            (b"MODL", 7) => scan::<ModelRecord<V7>>(payload, count, revision, abi)?,
            (b"MODL", 8) => scan::<ModelRecord<V8>>(payload, count, revision, abi)?,
            (b"MODL", 9) => scan::<ModelRecord<V9>>(payload, count, revision, abi)?,
            (b"PROF", 1) => scan::<ProfileMaterialSet>(payload, count, revision, abi)?,
            (b"SBND", 1) => scan::<SlotBinding>(payload, count, revision, abi)?,
            (b"MESH", 1) => scan::<Mesh<V1>>(payload, count, revision, abi)?,
            (b"MESH", 2) => scan::<Mesh<V2>>(payload, count, revision, abi)?,
            (b"SECT", 1) => scan::<MeshSection>(payload, count, revision, abi)?,
            (b"ATTR", 1) => scan::<AttrLayer>(payload, count, revision, abi)?,
            (b"INFL", 1) => scan::<Influence>(payload, count, revision, abi)?,
            (b"VSPL", 1) => scan::<VertexSplit>(payload, count, revision, abi)?,
            (b"FDRP", 1) => scan::<FaceRecord>(payload, count, revision, abi)?,
            (b"XFRM", 1) => scan::<Transform>(payload, count, revision, abi)?,
            (b"MTX4", 1) => scan::<Matrix44>(payload, count, revision, abi)?,
            (b"NODE", 1) => scan::<Node<V1>>(payload, count, revision, abi)?,
            (b"NODE", 2) => scan::<Node<V2>>(payload, count, revision, abi)?,
            (b"NODE", 3) => scan::<Node<V3>>(payload, count, revision, abi)?,
            (b"NODE", 4) => scan::<Node<V4>>(payload, count, revision, abi)?,
            (b"NODE", 5) => scan::<Node<V5>>(payload, count, revision, abi)?,
            (b"NODE", 6) => scan::<Node<V6>>(payload, count, revision, abi)?,
            (b"NODE", 7) => scan::<Node<V7>>(payload, count, revision, abi)?,
            (b"NODE", 8) => scan::<Node<V8>>(payload, count, revision, abi)?,
            (b"NODE", 9) => scan::<Node<V9>>(payload, count, revision, abi)?,
            (b"NODE", 10) => scan::<Node<V10>>(payload, count, revision, abi)?,
            (b"NODE", 11) => scan::<Node<V11>>(payload, count, revision, abi)?,
            (b"NODE", 12) => scan::<Node<V12>>(payload, count, revision, abi)?,
            (b"NODE", 13) => scan::<Node<V13>>(payload, count, revision, abi)?,
            (b"NODE", 14) => scan::<Node<V14>>(payload, count, revision, abi)?,
            (b"NODE", 15) => scan::<Node<V15>>(payload, count, revision, abi)?,
            (b"NODE", 16) => scan::<Node<V16>>(payload, count, revision, abi)?,
            (b"NODE", 17) => scan::<Node<V17>>(payload, count, revision, abi)?,
            (b"NODE", 18) => scan::<Node<V18>>(payload, count, revision, abi)?,
            (b"NODE", 19) => scan::<Node<V19>>(payload, count, revision, abi)?,
            (b"TPSR", 0) => scan::<TPoseSourceRecord>(payload, count, revision, abi)?,
            (b"PDLT", 1) => scan::<PoseDelta>(payload, count, revision, abi)?,
            (b"TPOS", 1) => scan::<TestPose>(payload, count, revision, abi)?,
            (b"AKEY", 1) => scan::<AssetKey>(payload, count, revision, abi)?,
            (b"S2P3", 1) => scan::<Sc2Property<[f32; 3]>>(payload, count, revision, abi)?,
            (b"S2RS", 1) => scan::<Sc2RibbonSplinePoint>(payload, count, revision, abi)?,
            (b"NBAG", 1) => scan::<NativeEntry<V1>>(payload, count, revision, abi)?,
            (b"NBAG", 2) => scan::<NativeEntry<V2>>(payload, count, revision, abi)?,
            (b"PSCH", 1) => scan::<PoseSchema<V1>>(payload, count, revision, abi)?,
            (b"PSCH", 2) => scan::<PoseSchema<V2>>(payload, count, revision, abi)?,
            (b"MATL", 1) => scan::<MaterialRecord>(payload, count, revision, abi)?,
            (b"MFEA", 1) => scan::<MaterialFeature>(payload, count, revision, abi)?,
            (b"FFRS", 1) => scan::<FresnelFeature>(payload, count, revision, abi)?,
            (b"FUVA", 1) => scan::<UvAnimationFeature>(payload, count, revision, abi)?,
            (b"FLSH", 1) => scan::<LayerShadingFeature>(payload, count, revision, abi)?,
            (b"MKCP", 1) => scan::<CompositeBody<V1>>(payload, count, revision, abi)?,
            (b"MKCP", 2) => scan::<CompositeBody<V2>>(payload, count, revision, abi)?,
            (b"MKCL", 1) => scan::<CompositeLayer>(payload, count, revision, abi)?,
            (b"MKCB", 1) => scan::<CombinersBody>(payload, count, revision, abi)?,
            (b"MKCS", 1) => scan::<CombinerStage>(payload, count, revision, abi)?,
            (b"MKLD", 1) => scan::<LegacyDeferredBody>(payload, count, revision, abi)?,
            (b"MKPB", 1) => scan::<PbrDeferredBody>(payload, count, revision, abi)?,
            (b"TEXR", 1) => scan::<TextureRecord>(payload, count, revision, abi)?,
            (b"LOOK", 1) => scan::<Look>(payload, count, revision, abi)?,
            (b"ACHN", 1) => scan::<AnimChannel>(payload, count, revision, abi)?,
            (b"STRK", 1) => scan::<SubTrack<V1>>(payload, count, revision, abi)?,
            (b"STRK", 2) => scan::<SubTrack<V2>>(payload, count, revision, abi)?,
            (b"STCC", 1) => scan::<SubTrackContainer>(payload, count, revision, abi)?,
            (b"CLEV", 1) => scan::<ClipEvent>(payload, count, revision, abi)?,
            (b"CLIP", 1) => scan::<Clip<V1>>(payload, count, revision, abi)?,
            (b"CLIP", 2) => scan::<Clip<V2>>(payload, count, revision, abi)?,
            (b"CLIP", 3) => scan::<Clip<V3>>(payload, count, revision, abi)?,
            (b"CLIP", 4) => scan::<Clip<V4>>(payload, count, revision, abi)?,
            (b"CLIP", 5) => scan::<Clip<V5>>(payload, count, revision, abi)?,
            (b"CLIP", 6) => scan::<Clip<V6>>(payload, count, revision, abi)?,
            (b"CLIP", 7) => scan::<Clip<V7>>(payload, count, revision, abi)?,
            (b"CPWF", 1) => scan::<BakeWorldForce>(payload, count, revision, abi)?,
            (b"TSET", 1) => scan::<TrackSet>(payload, count, revision, abi)?,
            (b"PSTG", 1) => scan::<PoseStage<V1>>(payload, count, revision, abi)?,
            (b"PSTG", 2) => scan::<PoseStage<V2>>(payload, count, revision, abi)?,
            (b"PSRC", 1) => scan::<StageSource>(payload, count, revision, abi)?,
            (b"PBDY", 1) => scan::<PhysicsBody<V1>>(payload, count, revision, abi)?,
            (b"PBDY", 2) => scan::<PhysicsBody<V2>>(payload, count, revision, abi)?,
            (b"PBDY", 3) => scan::<PhysicsBody<V3>>(payload, count, revision, abi)?,
            (b"PSHP", 1) => scan::<PhysicsShape<V1>>(payload, count, revision, abi)?,
            (b"PSHP", 2) => scan::<PhysicsShape<V2>>(payload, count, revision, abi)?,
            (b"PJNT", 1) => scan::<PhysicsJoint<V1>>(payload, count, revision, abi)?,
            (b"PJNT", 2) => scan::<PhysicsJoint<V2>>(payload, count, revision, abi)?,
            (b"PJNT", 3) => scan::<PhysicsJoint<V3>>(payload, count, revision, abi)?,
            (b"PCOL", 1) => scan::<ClothCollider<V1>>(payload, count, revision, abi)?,
            (b"PCOL", 2) => scan::<ClothCollider<V2>>(payload, count, revision, abi)?,
            (b"CLTH", 1) => scan::<Cloth<V1>>(payload, count, revision, abi)?,
            (b"CLTH", 2) => scan::<Cloth<V2>>(payload, count, revision, abi)?,
            (b"CBND", 1) => scan::<ClothBinding>(payload, count, revision, abi)?,
            (b"PRIG", 1) => scan::<PhysicsRig<V1>>(payload, count, revision, abi)?,
            (b"PRIG", 2) => scan::<PhysicsRig<V2>>(payload, count, revision, abi)?,
            (b"PRIG", 3) => scan::<PhysicsRig<V3>>(payload, count, revision, abi)?,
            (b"PRIG", 4) => scan::<PhysicsRig<V4>>(payload, count, revision, abi)?,
            (b"PRIG", 5) => scan::<PhysicsRig<V5>>(payload, count, revision, abi)?,
            (b"PRIG", 6) => scan::<PhysicsRig<V6>>(payload, count, revision, abi)?,
            (b"PRIG", 7) => scan::<PhysicsRig<V7>>(payload, count, revision, abi)?,
            (b"PRIG", 8) => scan::<PhysicsRig<V8>>(payload, count, revision, abi)?,
            (b"SBDY", 1) => scan::<SoftBody>(payload, count, revision, abi)?,
            (b"SLMP", 1) => scan::<SoftLump>(payload, count, revision, abi)?,
            (b"SEFX", 1) => scan::<SoftEffect>(payload, count, revision, abi)?,
            (b"SPRT", 1) => scan::<SoftPart>(payload, count, revision, abi)?,
            (b"CTSU", 1) => scan::<ClipTrackSet>(payload, count, revision, abi)?,
            (b"ATAG", 1) => scan::<AnimTag>(payload, count, revision, abi)?,
            (b"ASET", 1) => scan::<AnimSet>(payload, count, revision, abi)?,
            (b"NMDX", 1) => scan::<MdxMaterial>(payload, count, revision, abi)?,
            (b"NM2_", 1) => scan::<M2Material>(payload, count, revision, abi)?,
            (b"ND3_", 1) => scan::<D3Material<V1>>(payload, count, revision, abi)?,
            (b"ND3_", 2) => scan::<D3Material<V2>>(payload, count, revision, abi)?,
            (b"NM3_", 1) => match abi {
                Some(NativeAbi::Abi32) => {
                    scan::<M3Material<V1, Abi32>>(payload, count, revision, abi)?
                }
                Some(NativeAbi::Abi64) => {
                    scan::<M3Material<V1, Abi64>>(payload, count, revision, abi)?
                }
                None => return Ok(None),
            },
            _ => return Ok(None),
        };
        Ok(Some(span))
    })();
    match result {
        Err(error) if error.kind == ReadErrorKind::UnsupportedLayout => Ok(None),
        result => result,
    }
}
struct Callback<'v, F>(&'v mut F);
impl<F: FnMut(RawReference) -> Result<(), ReadError>> ReferenceVisitor for Callback<'_, F> {
    type Error = ReadError;
    fn visit(&mut self, reference: RawReference) -> Result<(), ReadError> {
        (self.0)(reference)
    }
}
fn validate<T: RecordLayout + Validate + VisitReferences + for<'a> Read<'a>>(
    payload: &[u8],
    count: u32,
    revision: u32,
    abi: Option<NativeAbi>,
    visitor: &mut impl ReferenceVisitor<Error = ReadError>,
) -> Result<(), ReadError> {
    let span = match scan::<T>(payload, count, revision, abi) {
        Err(error) if error.kind == ReadErrorKind::UnsupportedLayout => return Ok(()),
        result => result?,
    };
    if span != payload.len() {
        return Err(ReadError::invalid(span, "section record span"));
    }
    let mut input = Decoder::new(payload, revision).with_native_abi(abi);
    for _ in 0..count {
        let value = T::read(&mut input)?;
        value.validate()?;
        value.visit_references(visitor)?;
    }
    Ok(())
}
/// Validate exactly bounded known records and visit each active reference in serialized order.
/// Unsupported identities and ABI-unspecified NM3_ payloads remain opaque.
pub fn validate_known_section(
    tag: [u8; 4],
    revision: u32,
    payload: &[u8],
    count: u32,
    abi: Option<NativeAbi>,
    callback: &mut impl FnMut(RawReference) -> Result<(), ReadError>,
) -> Result<(), ReadError> {
    let visitor = &mut Callback(callback);
    match (&tag, revision) {
        (b"CHAR", 0) => validate::<Character>(payload, count, revision, abi, visitor)?,
        (b"U8__", 0) => validate::<Byte>(payload, count, revision, abi, visitor)?,
        (b"U16_", 0) => validate::<U16>(payload, count, revision, abi, visitor)?,
        (b"U32_", 0) => validate::<U32>(payload, count, revision, abi, visitor)?,
        (b"U64_", 0) => validate::<U64>(payload, count, revision, abi, visitor)?,
        (b"I16_", 0) => validate::<I16>(payload, count, revision, abi, visitor)?,
        (b"I32_", 0) => validate::<I32>(payload, count, revision, abi, visitor)?,
        (b"REAL", 0) => validate::<Real>(payload, count, revision, abi, visitor)?,
        (b"VEC2", 0) => validate::<Vector2>(payload, count, revision, abi, visitor)?,
        (b"VEC3", 0) => validate::<Vector3>(payload, count, revision, abi, visitor)?,
        (b"VEC4", 0) => validate::<Vector4>(payload, count, revision, abi, visitor)?,
        (b"QUAT", 0) => validate::<QuaternionRecord>(payload, count, revision, abi, visitor)?,
        (b"BNDS", 0) => validate::<Extent>(payload, count, revision, abi, visitor)?,
        (b"BYT4", 0) => validate::<Byte4>(payload, count, revision, abi, visitor)?,
        (b"WDOC", 1) => validate::<DocumentRecord<V1>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 1) => validate::<ModelRecord<V1>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 2) => validate::<ModelRecord<V2>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 3) => validate::<ModelRecord<V3>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 4) => validate::<ModelRecord<V4>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 5) => validate::<ModelRecord<V5>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 6) => validate::<ModelRecord<V6>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 7) => validate::<ModelRecord<V7>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 8) => validate::<ModelRecord<V8>>(payload, count, revision, abi, visitor)?,
        (b"MODL", 9) => validate::<ModelRecord<V9>>(payload, count, revision, abi, visitor)?,
        (b"PROF", 1) => validate::<ProfileMaterialSet>(payload, count, revision, abi, visitor)?,
        (b"SBND", 1) => validate::<SlotBinding>(payload, count, revision, abi, visitor)?,
        (b"MESH", 1) => validate::<Mesh<V1>>(payload, count, revision, abi, visitor)?,
        (b"MESH", 2) => validate::<Mesh<V2>>(payload, count, revision, abi, visitor)?,
        (b"SECT", 1) => validate::<MeshSection>(payload, count, revision, abi, visitor)?,
        (b"ATTR", 1) => validate::<AttrLayer>(payload, count, revision, abi, visitor)?,
        (b"INFL", 1) => validate::<Influence>(payload, count, revision, abi, visitor)?,
        (b"VSPL", 1) => validate::<VertexSplit>(payload, count, revision, abi, visitor)?,
        (b"FDRP", 1) => validate::<FaceRecord>(payload, count, revision, abi, visitor)?,
        (b"XFRM", 1) => validate::<Transform>(payload, count, revision, abi, visitor)?,
        (b"MTX4", 1) => validate::<Matrix44>(payload, count, revision, abi, visitor)?,
        (b"NODE", 1) => validate::<Node<V1>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 2) => validate::<Node<V2>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 3) => validate::<Node<V3>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 4) => validate::<Node<V4>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 5) => validate::<Node<V5>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 6) => validate::<Node<V6>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 7) => validate::<Node<V7>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 8) => validate::<Node<V8>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 9) => validate::<Node<V9>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 10) => validate::<Node<V10>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 11) => validate::<Node<V11>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 12) => validate::<Node<V12>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 13) => validate::<Node<V13>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 14) => validate::<Node<V14>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 15) => validate::<Node<V15>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 16) => validate::<Node<V16>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 17) => validate::<Node<V17>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 18) => validate::<Node<V18>>(payload, count, revision, abi, visitor)?,
        (b"NODE", 19) => validate::<Node<V19>>(payload, count, revision, abi, visitor)?,
        (b"TPSR", 0) => validate::<TPoseSourceRecord>(payload, count, revision, abi, visitor)?,
        (b"PDLT", 1) => validate::<PoseDelta>(payload, count, revision, abi, visitor)?,
        (b"TPOS", 1) => validate::<TestPose>(payload, count, revision, abi, visitor)?,
        (b"AKEY", 1) => validate::<AssetKey>(payload, count, revision, abi, visitor)?,
        (b"S2P3", 1) => validate::<Sc2Property<[f32; 3]>>(payload, count, revision, abi, visitor)?,
        (b"S2RS", 1) => validate::<Sc2RibbonSplinePoint>(payload, count, revision, abi, visitor)?,
        (b"NBAG", 1) => validate::<NativeEntry<V1>>(payload, count, revision, abi, visitor)?,
        (b"NBAG", 2) => validate::<NativeEntry<V2>>(payload, count, revision, abi, visitor)?,
        (b"PSCH", 1) => validate::<PoseSchema<V1>>(payload, count, revision, abi, visitor)?,
        (b"PSCH", 2) => validate::<PoseSchema<V2>>(payload, count, revision, abi, visitor)?,
        (b"MATL", 1) => validate::<MaterialRecord>(payload, count, revision, abi, visitor)?,
        (b"MFEA", 1) => validate::<MaterialFeature>(payload, count, revision, abi, visitor)?,
        (b"FFRS", 1) => validate::<FresnelFeature>(payload, count, revision, abi, visitor)?,
        (b"FUVA", 1) => validate::<UvAnimationFeature>(payload, count, revision, abi, visitor)?,
        (b"FLSH", 1) => validate::<LayerShadingFeature>(payload, count, revision, abi, visitor)?,
        (b"MKCP", 1) => validate::<CompositeBody<V1>>(payload, count, revision, abi, visitor)?,
        (b"MKCP", 2) => validate::<CompositeBody<V2>>(payload, count, revision, abi, visitor)?,
        (b"MKCL", 1) => validate::<CompositeLayer>(payload, count, revision, abi, visitor)?,
        (b"MKCB", 1) => validate::<CombinersBody>(payload, count, revision, abi, visitor)?,
        (b"MKCS", 1) => validate::<CombinerStage>(payload, count, revision, abi, visitor)?,
        (b"MKLD", 1) => validate::<LegacyDeferredBody>(payload, count, revision, abi, visitor)?,
        (b"MKPB", 1) => validate::<PbrDeferredBody>(payload, count, revision, abi, visitor)?,
        (b"TEXR", 1) => validate::<TextureRecord>(payload, count, revision, abi, visitor)?,
        (b"LOOK", 1) => validate::<Look>(payload, count, revision, abi, visitor)?,
        (b"ACHN", 1) => validate::<AnimChannel>(payload, count, revision, abi, visitor)?,
        (b"STRK", 1) => validate::<SubTrack<V1>>(payload, count, revision, abi, visitor)?,
        (b"STRK", 2) => validate::<SubTrack<V2>>(payload, count, revision, abi, visitor)?,
        (b"STCC", 1) => validate::<SubTrackContainer>(payload, count, revision, abi, visitor)?,
        (b"CLEV", 1) => validate::<ClipEvent>(payload, count, revision, abi, visitor)?,
        (b"CLIP", 1) => validate::<Clip<V1>>(payload, count, revision, abi, visitor)?,
        (b"CLIP", 2) => validate::<Clip<V2>>(payload, count, revision, abi, visitor)?,
        (b"CLIP", 3) => validate::<Clip<V3>>(payload, count, revision, abi, visitor)?,
        (b"CLIP", 4) => validate::<Clip<V4>>(payload, count, revision, abi, visitor)?,
        (b"CLIP", 5) => validate::<Clip<V5>>(payload, count, revision, abi, visitor)?,
        (b"CLIP", 6) => validate::<Clip<V6>>(payload, count, revision, abi, visitor)?,
        (b"CLIP", 7) => validate::<Clip<V7>>(payload, count, revision, abi, visitor)?,
        (b"CPWF", 1) => validate::<BakeWorldForce>(payload, count, revision, abi, visitor)?,
        (b"TSET", 1) => validate::<TrackSet>(payload, count, revision, abi, visitor)?,
        (b"PSTG", 1) => validate::<PoseStage<V1>>(payload, count, revision, abi, visitor)?,
        (b"PSTG", 2) => validate::<PoseStage<V2>>(payload, count, revision, abi, visitor)?,
        (b"PSRC", 1) => validate::<StageSource>(payload, count, revision, abi, visitor)?,
        (b"PBDY", 1) => validate::<PhysicsBody<V1>>(payload, count, revision, abi, visitor)?,
        (b"PBDY", 2) => validate::<PhysicsBody<V2>>(payload, count, revision, abi, visitor)?,
        (b"PBDY", 3) => validate::<PhysicsBody<V3>>(payload, count, revision, abi, visitor)?,
        (b"PSHP", 1) => validate::<PhysicsShape<V1>>(payload, count, revision, abi, visitor)?,
        (b"PSHP", 2) => validate::<PhysicsShape<V2>>(payload, count, revision, abi, visitor)?,
        (b"PJNT", 1) => validate::<PhysicsJoint<V1>>(payload, count, revision, abi, visitor)?,
        (b"PJNT", 2) => validate::<PhysicsJoint<V2>>(payload, count, revision, abi, visitor)?,
        (b"PJNT", 3) => validate::<PhysicsJoint<V3>>(payload, count, revision, abi, visitor)?,
        (b"PCOL", 1) => validate::<ClothCollider<V1>>(payload, count, revision, abi, visitor)?,
        (b"PCOL", 2) => validate::<ClothCollider<V2>>(payload, count, revision, abi, visitor)?,
        (b"CLTH", 1) => validate::<Cloth<V1>>(payload, count, revision, abi, visitor)?,
        (b"CLTH", 2) => validate::<Cloth<V2>>(payload, count, revision, abi, visitor)?,
        (b"CBND", 1) => validate::<ClothBinding>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 1) => validate::<PhysicsRig<V1>>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 2) => validate::<PhysicsRig<V2>>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 3) => validate::<PhysicsRig<V3>>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 4) => validate::<PhysicsRig<V4>>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 5) => validate::<PhysicsRig<V5>>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 6) => validate::<PhysicsRig<V6>>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 7) => validate::<PhysicsRig<V7>>(payload, count, revision, abi, visitor)?,
        (b"PRIG", 8) => validate::<PhysicsRig<V8>>(payload, count, revision, abi, visitor)?,
        (b"SBDY", 1) => validate::<SoftBody>(payload, count, revision, abi, visitor)?,
        (b"SLMP", 1) => validate::<SoftLump>(payload, count, revision, abi, visitor)?,
        (b"SEFX", 1) => validate::<SoftEffect>(payload, count, revision, abi, visitor)?,
        (b"SPRT", 1) => validate::<SoftPart>(payload, count, revision, abi, visitor)?,
        (b"CTSU", 1) => validate::<ClipTrackSet>(payload, count, revision, abi, visitor)?,
        (b"ATAG", 1) => validate::<AnimTag>(payload, count, revision, abi, visitor)?,
        (b"ASET", 1) => validate::<AnimSet>(payload, count, revision, abi, visitor)?,
        (b"NMDX", 1) => validate::<MdxMaterial>(payload, count, revision, abi, visitor)?,
        (b"NM2_", 1) => validate::<M2Material>(payload, count, revision, abi, visitor)?,
        (b"ND3_", 1) => validate::<D3Material<V1>>(payload, count, revision, abi, visitor)?,
        (b"ND3_", 2) => validate::<D3Material<V2>>(payload, count, revision, abi, visitor)?,
        (b"NM3_", 1) => match abi {
            Some(NativeAbi::Abi32) => {
                validate::<M3Material<V1, Abi32>>(payload, count, revision, abi, visitor)?
            }
            Some(NativeAbi::Abi64) => {
                validate::<M3Material<V1, Abi64>>(payload, count, revision, abi, visitor)?
            }
            None => return Ok(()),
        },
        _ => {}
    }
    Ok(())
}
