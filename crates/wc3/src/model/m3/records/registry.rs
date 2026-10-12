//! Exact native section-revision dispatch for bounded decoding and graph traversal.
use super::*;
use crate::model::m3::codec::Decoder;
use crate::model::m3::{
    Envelope, RawReference, Read, ReadError, RecordLayout, ReferenceVisitor, Validate,
    VisitReferences,
};
/// Identity of one supported native section revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordDescriptor {
    /// Logical section FourCC.
    pub tag: [u8; 4],
    /// Independent section revision.
    pub revision: u32,
}
/// Supported native section identities, excluding file headers and unknown tags.
pub const SUPPORTED_RECORDS: &[RecordDescriptor] = &[
    RecordDescriptor {
        tag: *b"EVNT",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"EVNT",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"EVNT",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"SEQS",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SEQS",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"SEQS",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"STC_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"STC_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"STC_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"STC_",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"STC_",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"STG_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"STS_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"BSET",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"BONE",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"BONE",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"REGN",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"REGN",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"REGN",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"REGN",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"REGN",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"REGN",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"BAT_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"BAT_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MSEC",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"MSEC",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"DIV_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"DIV_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"DIV_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"IREF",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"ATT_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"ATT_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MATM",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 8,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 9,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 10,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 11,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 12,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 13,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 14,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 15,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 16,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 17,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 18,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 19,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 20,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 21,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 22,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 23,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 24,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 25,
    },
    RecordDescriptor {
        tag: *b"LAYR",
        revision: 26,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 8,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 9,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 10,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 11,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 12,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 13,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 14,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 15,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 16,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 17,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 18,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 19,
    },
    RecordDescriptor {
        tag: *b"MAT_",
        revision: 20,
    },
    RecordDescriptor {
        tag: *b"DIS_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"DIS_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"DIS_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"DIS_",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"DIS_",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"CMS_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"CMP_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"CMP_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"CMP_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"TER_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"TER_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"VOL_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"HAI_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"VON_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"CREP",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"CREP",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"STBM",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"REF_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"REF_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"REF_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"REF_",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"LFSB",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"LFSB",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"LFSB",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"LFLR",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"LFLR",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"LFLR",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"LFLR",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"MADD",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"MADD",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"MADD",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"MADD",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 10,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 11,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 12,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 13,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 14,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 15,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 16,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 17,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 18,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 19,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 20,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 21,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 22,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 23,
    },
    RecordDescriptor {
        tag: *b"PAR_",
        revision: 24,
    },
    RecordDescriptor {
        tag: *b"PARC",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SRIB",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"RIB_",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"RIB_",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"RIB_",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"RIB_",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"RIB_",
        revision: 8,
    },
    RecordDescriptor {
        tag: *b"RIB_",
        revision: 9,
    },
    RecordDescriptor {
        tag: *b"PROJ",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"PROJ",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"SSGS",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SSGS",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"ATVL",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"TRGD",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PATU",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PATU",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PATU",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PATU",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PATU",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"BBSC",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"IKJT",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"IK2J",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"IKCC",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PAOB",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SHBX",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"VVOL",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"TMD_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"TMD_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"FOR_",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"FOR_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"FOR_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"WRP_",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"DMSE",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"DMMN",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"DMMN",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"DMMT",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"DMME",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHSH",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHSH",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PHSH",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PHSH",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PHRB",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHRB",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PHRB",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PHRB",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PHRB",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"PHYJ",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHCC",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHAC",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHCL",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHCL",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"PHCL",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"PHCL",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"PHCL",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 1,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 6,
    },
    RecordDescriptor {
        tag: *b"LITE",
        revision: 7,
    },
    RecordDescriptor {
        tag: *b"CAM_",
        revision: 2,
    },
    RecordDescriptor {
        tag: *b"CAM_",
        revision: 3,
    },
    RecordDescriptor {
        tag: *b"CAM_",
        revision: 4,
    },
    RecordDescriptor {
        tag: *b"CAM_",
        revision: 5,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 20,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 21,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 22,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 23,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 24,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 25,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 26,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 27,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 28,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 29,
    },
    RecordDescriptor {
        tag: *b"MODL",
        revision: 30,
    },
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
        tag: *b"FLAG",
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
        tag: *b"\0COL",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"BNDS",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDEV",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SD2V",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SD3V",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDR3",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SD4Q",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDCC",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDU8",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDS6",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDU6",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDS3",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDU3",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDFG",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SDMB",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SS32",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SU32",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SR32",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SVC3",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SVC2",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"MT16",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"MT32",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"SCHR",
        revision: 0,
    },
    RecordDescriptor {
        tag: *b"PHCT",
        revision: 0,
    },
];
/// Inline byte width for an exact known section revision and envelope.
pub fn known_record_size<E: Envelope>(tag: [u8; 4], revision: u32) -> Option<usize> {
    match (&tag, revision) {
        (b"EVNT", 0) => Some(<Event<V0> as RecordLayout<E>>::size()),
        (b"EVNT", 1) => Some(<Event<V1> as RecordLayout<E>>::size()),
        (b"EVNT", 2) => Some(<Event<V2> as RecordLayout<E>>::size()),
        (b"SEQS", 0) => Some(<Sequence<V0> as RecordLayout<E>>::size()),
        (b"SEQS", 1) => Some(<Sequence<V1> as RecordLayout<E>>::size()),
        (b"SEQS", 2) => Some(<Sequence<V2> as RecordLayout<E>>::size()),
        (b"STC_", 0) => Some(<SubTrackContainer<V0> as RecordLayout<E>>::size()),
        (b"STC_", 1) => Some(<SubTrackContainer<V1> as RecordLayout<E>>::size()),
        (b"STC_", 2) => Some(<SubTrackContainer<V2> as RecordLayout<E>>::size()),
        (b"STC_", 3) => Some(<SubTrackContainer<V3> as RecordLayout<E>>::size()),
        (b"STC_", 4) => Some(<SubTrackContainer<V4> as RecordLayout<E>>::size()),
        (b"STG_", 0) => Some(<AnimationGroup as RecordLayout<E>>::size()),
        (b"STS_", 0) => Some(<AnimationState as RecordLayout<E>>::size()),
        (b"BSET", 0) => Some(<BoneAnimationSet as RecordLayout<E>>::size()),
        (b"BONE", 0) => Some(<Bone<V0> as RecordLayout<E>>::size()),
        (b"BONE", 1) => Some(<Bone<V1> as RecordLayout<E>>::size()),
        (b"REGN", 0) => Some(<Region<V0> as RecordLayout<E>>::size()),
        (b"REGN", 1) => Some(<Region<V1> as RecordLayout<E>>::size()),
        (b"REGN", 2) => Some(<Region<V2> as RecordLayout<E>>::size()),
        (b"REGN", 3) => Some(<Region<V3> as RecordLayout<E>>::size()),
        (b"REGN", 4) => Some(<Region<V4> as RecordLayout<E>>::size()),
        (b"REGN", 5) => Some(<Region<V5> as RecordLayout<E>>::size()),
        (b"BAT_", 0) => Some(<Batch<V0> as RecordLayout<E>>::size()),
        (b"BAT_", 1) => Some(<Batch<V1> as RecordLayout<E>>::size()),
        (b"MSEC", 0) => Some(<MeshSection<V0> as RecordLayout<E>>::size()),
        (b"MSEC", 1) => Some(<MeshSection<V1> as RecordLayout<E>>::size()),
        (b"DIV_", 0) => Some(<MeshDivision<V0> as RecordLayout<E>>::size()),
        (b"DIV_", 1) => Some(<MeshDivision<V1> as RecordLayout<E>>::size()),
        (b"DIV_", 2) => Some(<MeshDivision<V2> as RecordLayout<E>>::size()),
        (b"IREF", 0) => Some(<InitialReference as RecordLayout<E>>::size()),
        (b"ATT_", 0) => Some(<AttachmentPoint<V0> as RecordLayout<E>>::size()),
        (b"ATT_", 1) => Some(<AttachmentPoint<V1> as RecordLayout<E>>::size()),
        (b"MATM", 0) => Some(<MaterialMap as RecordLayout<E>>::size()),
        (b"LAYR", 0) => Some(<TextureLayer<V0> as RecordLayout<E>>::size()),
        (b"LAYR", 1) => Some(<TextureLayer<V1> as RecordLayout<E>>::size()),
        (b"LAYR", 2) => Some(<TextureLayer<V2> as RecordLayout<E>>::size()),
        (b"LAYR", 3) => Some(<TextureLayer<V3> as RecordLayout<E>>::size()),
        (b"LAYR", 4) => Some(<TextureLayer<V4> as RecordLayout<E>>::size()),
        (b"LAYR", 5) => Some(<TextureLayer<V5> as RecordLayout<E>>::size()),
        (b"LAYR", 6) => Some(<TextureLayer<V6> as RecordLayout<E>>::size()),
        (b"LAYR", 7) => Some(<TextureLayer<V7> as RecordLayout<E>>::size()),
        (b"LAYR", 8) => Some(<TextureLayer<V8> as RecordLayout<E>>::size()),
        (b"LAYR", 9) => Some(<TextureLayer<V9> as RecordLayout<E>>::size()),
        (b"LAYR", 10) => Some(<TextureLayer<V10> as RecordLayout<E>>::size()),
        (b"LAYR", 11) => Some(<TextureLayer<V11> as RecordLayout<E>>::size()),
        (b"LAYR", 12) => Some(<TextureLayer<V12> as RecordLayout<E>>::size()),
        (b"LAYR", 13) => Some(<TextureLayer<V13> as RecordLayout<E>>::size()),
        (b"LAYR", 14) => Some(<TextureLayer<V14> as RecordLayout<E>>::size()),
        (b"LAYR", 15) => Some(<TextureLayer<V15> as RecordLayout<E>>::size()),
        (b"LAYR", 16) => Some(<TextureLayer<V16> as RecordLayout<E>>::size()),
        (b"LAYR", 17) => Some(<TextureLayer<V17> as RecordLayout<E>>::size()),
        (b"LAYR", 18) => Some(<TextureLayer<V18> as RecordLayout<E>>::size()),
        (b"LAYR", 19) => Some(<TextureLayer<V19> as RecordLayout<E>>::size()),
        (b"LAYR", 20) => Some(<TextureLayer<V20> as RecordLayout<E>>::size()),
        (b"LAYR", 21) => Some(<TextureLayer<V21> as RecordLayout<E>>::size()),
        (b"LAYR", 22) => Some(<TextureLayer<V22> as RecordLayout<E>>::size()),
        (b"LAYR", 23) => Some(<TextureLayer<V23> as RecordLayout<E>>::size()),
        (b"LAYR", 24) => Some(<TextureLayer<V24> as RecordLayout<E>>::size()),
        (b"LAYR", 25) => Some(<TextureLayer<V25> as RecordLayout<E>>::size()),
        (b"LAYR", 26) => Some(<TextureLayer<V26> as RecordLayout<E>>::size()),
        (b"MAT_", 0) => Some(<StandardMaterial<V0> as RecordLayout<E>>::size()),
        (b"MAT_", 1) => Some(<StandardMaterial<V1> as RecordLayout<E>>::size()),
        (b"MAT_", 2) => Some(<StandardMaterial<V2> as RecordLayout<E>>::size()),
        (b"MAT_", 3) => Some(<StandardMaterial<V3> as RecordLayout<E>>::size()),
        (b"MAT_", 4) => Some(<StandardMaterial<V4> as RecordLayout<E>>::size()),
        (b"MAT_", 5) => Some(<StandardMaterial<V5> as RecordLayout<E>>::size()),
        (b"MAT_", 6) => Some(<StandardMaterial<V6> as RecordLayout<E>>::size()),
        (b"MAT_", 7) => Some(<StandardMaterial<V7> as RecordLayout<E>>::size()),
        (b"MAT_", 8) => Some(<StandardMaterial<V8> as RecordLayout<E>>::size()),
        (b"MAT_", 9) => Some(<StandardMaterial<V9> as RecordLayout<E>>::size()),
        (b"MAT_", 10) => Some(<StandardMaterial<V10> as RecordLayout<E>>::size()),
        (b"MAT_", 11) => Some(<StandardMaterial<V11> as RecordLayout<E>>::size()),
        (b"MAT_", 12) => Some(<StandardMaterial<V12> as RecordLayout<E>>::size()),
        (b"MAT_", 13) => Some(<StandardMaterial<V13> as RecordLayout<E>>::size()),
        (b"MAT_", 14) => Some(<StandardMaterial<V14> as RecordLayout<E>>::size()),
        (b"MAT_", 15) => Some(<StandardMaterial<V15> as RecordLayout<E>>::size()),
        (b"MAT_", 16) => Some(<StandardMaterial<V16> as RecordLayout<E>>::size()),
        (b"MAT_", 17) => Some(<StandardMaterial<V17> as RecordLayout<E>>::size()),
        (b"MAT_", 18) => Some(<StandardMaterial<V18> as RecordLayout<E>>::size()),
        (b"MAT_", 19) => Some(<StandardMaterial<V19> as RecordLayout<E>>::size()),
        (b"MAT_", 20) => Some(<StandardMaterial<V20> as RecordLayout<E>>::size()),
        (b"DIS_", 0) => Some(<DisplacementMaterial<V0> as RecordLayout<E>>::size()),
        (b"DIS_", 1) => Some(<DisplacementMaterial<V1> as RecordLayout<E>>::size()),
        (b"DIS_", 2) => Some(<DisplacementMaterial<V2> as RecordLayout<E>>::size()),
        (b"DIS_", 3) => Some(<DisplacementMaterial<V3> as RecordLayout<E>>::size()),
        (b"DIS_", 4) => Some(<DisplacementMaterial<V4> as RecordLayout<E>>::size()),
        (b"CMS_", 0) => Some(<CompositeSection as RecordLayout<E>>::size()),
        (b"CMP_", 0) => Some(<CompositeMaterial<V0> as RecordLayout<E>>::size()),
        (b"CMP_", 1) => Some(<CompositeMaterial<V1> as RecordLayout<E>>::size()),
        (b"CMP_", 2) => Some(<CompositeMaterial<V2> as RecordLayout<E>>::size()),
        (b"TER_", 0) => Some(<TerrainMaterial<V0> as RecordLayout<E>>::size()),
        (b"TER_", 1) => Some(<TerrainMaterial<V1> as RecordLayout<E>>::size()),
        (b"VOL_", 0) => Some(<VolumeMaterial as RecordLayout<E>>::size()),
        (b"HAI_", 0) => Some(<HairMaterial as RecordLayout<E>>::size()),
        (b"VON_", 0) => Some(<VolumeNoiseMaterial as RecordLayout<E>>::size()),
        (b"CREP", 0) => Some(<CreepMaterial<V0> as RecordLayout<E>>::size()),
        (b"CREP", 1) => Some(<CreepMaterial<V1> as RecordLayout<E>>::size()),
        (b"STBM", 0) => Some(<STBMaterial as RecordLayout<E>>::size()),
        (b"REF_", 0) => Some(<ReflectionMaterial<V0> as RecordLayout<E>>::size()),
        (b"REF_", 1) => Some(<ReflectionMaterial<V1> as RecordLayout<E>>::size()),
        (b"REF_", 2) => Some(<ReflectionMaterial<V2> as RecordLayout<E>>::size()),
        (b"REF_", 3) => Some(<ReflectionMaterial<V3> as RecordLayout<E>>::size()),
        (b"LFSB", 0) => Some(<SubFlare<V0> as RecordLayout<E>>::size()),
        (b"LFSB", 1) => Some(<SubFlare<V1> as RecordLayout<E>>::size()),
        (b"LFSB", 2) => Some(<SubFlare<V2> as RecordLayout<E>>::size()),
        (b"LFLR", 0) => Some(<LensFlare<V0> as RecordLayout<E>>::size()),
        (b"LFLR", 1) => Some(<LensFlare<V1> as RecordLayout<E>>::size()),
        (b"LFLR", 2) => Some(<LensFlare<V2> as RecordLayout<E>>::size()),
        (b"LFLR", 3) => Some(<LensFlare<V3> as RecordLayout<E>>::size()),
        (b"MADD", 0) => Some(<DataDrivenMaterial<V0> as RecordLayout<E>>::size()),
        (b"MADD", 1) => Some(<DataDrivenMaterial<V1> as RecordLayout<E>>::size()),
        (b"MADD", 2) => Some(<DataDrivenMaterial<V2> as RecordLayout<E>>::size()),
        (b"MADD", 3) => Some(<DataDrivenMaterial<V3> as RecordLayout<E>>::size()),
        (b"PAR_", 10) => Some(<ParticleEmitter<V10> as RecordLayout<E>>::size()),
        (b"PAR_", 11) => Some(<ParticleEmitter<V11> as RecordLayout<E>>::size()),
        (b"PAR_", 12) => Some(<ParticleEmitter<V12> as RecordLayout<E>>::size()),
        (b"PAR_", 13) => Some(<ParticleEmitter<V13> as RecordLayout<E>>::size()),
        (b"PAR_", 14) => Some(<ParticleEmitter<V14> as RecordLayout<E>>::size()),
        (b"PAR_", 15) => Some(<ParticleEmitter<V15> as RecordLayout<E>>::size()),
        (b"PAR_", 16) => Some(<ParticleEmitter<V16> as RecordLayout<E>>::size()),
        (b"PAR_", 17) => Some(<ParticleEmitter<V17> as RecordLayout<E>>::size()),
        (b"PAR_", 18) => Some(<ParticleEmitter<V18> as RecordLayout<E>>::size()),
        (b"PAR_", 19) => Some(<ParticleEmitter<V19> as RecordLayout<E>>::size()),
        (b"PAR_", 20) => Some(<ParticleEmitter<V20> as RecordLayout<E>>::size()),
        (b"PAR_", 21) => Some(<ParticleEmitter<V21> as RecordLayout<E>>::size()),
        (b"PAR_", 22) => Some(<ParticleEmitter<V22> as RecordLayout<E>>::size()),
        (b"PAR_", 23) => Some(<ParticleEmitter<V23> as RecordLayout<E>>::size()),
        (b"PAR_", 24) => Some(<ParticleEmitter<V24> as RecordLayout<E>>::size()),
        (b"PARC", 0) => Some(<ParticleEmitterCopy as RecordLayout<E>>::size()),
        (b"SRIB", 0) => Some(<SplineRibbon as RecordLayout<E>>::size()),
        (b"RIB_", 4) => Some(<RibbonEmitter<V4> as RecordLayout<E>>::size()),
        (b"RIB_", 5) => Some(<RibbonEmitter<V5> as RecordLayout<E>>::size()),
        (b"RIB_", 6) => Some(<RibbonEmitter<V6> as RecordLayout<E>>::size()),
        (b"RIB_", 7) => Some(<RibbonEmitter<V7> as RecordLayout<E>>::size()),
        (b"RIB_", 8) => Some(<RibbonEmitter<V8> as RecordLayout<E>>::size()),
        (b"RIB_", 9) => Some(<RibbonEmitter<V9> as RecordLayout<E>>::size()),
        (b"PROJ", 4) => Some(<Projector<V4> as RecordLayout<E>>::size()),
        (b"PROJ", 5) => Some(<Projector<V5> as RecordLayout<E>>::size()),
        (b"SSGS", 0) => Some(<HitTestShape<V0> as RecordLayout<E>>::size()),
        (b"SSGS", 1) => Some(<HitTestShape<V1> as RecordLayout<E>>::size()),
        (b"ATVL", 0) => Some(<AttachmentVolume as RecordLayout<E>>::size()),
        (b"TRGD", 0) => Some(<TriggerData as RecordLayout<E>>::size()),
        (b"PATU", 0) => Some(<TurretBehavior<V0> as RecordLayout<E>>::size()),
        (b"PATU", 1) => Some(<TurretBehavior<V1> as RecordLayout<E>>::size()),
        (b"PATU", 2) => Some(<TurretBehavior<V2> as RecordLayout<E>>::size()),
        (b"PATU", 3) => Some(<TurretBehavior<V3> as RecordLayout<E>>::size()),
        (b"PATU", 4) => Some(<TurretBehavior<V4> as RecordLayout<E>>::size()),
        (b"BBSC", 0) => Some(<BillboardBehavior as RecordLayout<E>>::size()),
        (b"IKJT", 0) => Some(<IKJoint as RecordLayout<E>>::size()),
        (b"IK2J", 0) => Some(<IKTwoJoint as RecordLayout<E>>::size()),
        (b"IKCC", 0) => Some(<IKCCD as RecordLayout<E>>::size()),
        (b"PAOB", 0) => Some(<OneBoneSolver as RecordLayout<E>>::size()),
        (b"SHBX", 0) => Some(<ShadowBox as RecordLayout<E>>::size()),
        (b"VVOL", 0) => Some(<ViewVolume as RecordLayout<E>>::size()),
        (b"TMD_", 0) => Some(<TrailingModel<V0> as RecordLayout<E>>::size()),
        (b"TMD_", 1) => Some(<TrailingModel<V1> as RecordLayout<E>>::size()),
        (b"FOR_", 0) => Some(<Force<V0> as RecordLayout<E>>::size()),
        (b"FOR_", 1) => Some(<Force<V1> as RecordLayout<E>>::size()),
        (b"FOR_", 2) => Some(<Force<V2> as RecordLayout<E>>::size()),
        (b"WRP_", 1) => Some(<Warp as RecordLayout<E>>::size()),
        (b"DMSE", 0) => Some(<ConvexHullHalfEdge as RecordLayout<E>>::size()),
        (b"DMMN", 0) => Some(<PhysicsMeshBvhNode<V0> as RecordLayout<E>>::size()),
        (b"DMMN", 1) => Some(<PhysicsMeshBvhNode<V1> as RecordLayout<E>>::size()),
        (b"DMMT", 0) => Some(<PhysicsMeshTriangle as RecordLayout<E>>::size()),
        (b"DMME", 0) => Some(<PhysicsMeshEdge as RecordLayout<E>>::size()),
        (b"PHSH", 0) => Some(<PhysicsShape<V0> as RecordLayout<E>>::size()),
        (b"PHSH", 1) => Some(<PhysicsShape<V1> as RecordLayout<E>>::size()),
        (b"PHSH", 2) => Some(<PhysicsShape<V2> as RecordLayout<E>>::size()),
        (b"PHSH", 3) => Some(<PhysicsShape<V3> as RecordLayout<E>>::size()),
        (b"PHRB", 0) => Some(<RigidBody<V0> as RecordLayout<E>>::size()),
        (b"PHRB", 1) => Some(<RigidBody<V1> as RecordLayout<E>>::size()),
        (b"PHRB", 2) => Some(<RigidBody<V2> as RecordLayout<E>>::size()),
        (b"PHRB", 3) => Some(<RigidBody<V3> as RecordLayout<E>>::size()),
        (b"PHRB", 4) => Some(<RigidBody<V4> as RecordLayout<E>>::size()),
        (b"PHYJ", 0) => Some(<PhysicsJoint as RecordLayout<E>>::size()),
        (b"PHCC", 0) => Some(<ClothCollider as RecordLayout<E>>::size()),
        (b"PHAC", 0) => Some(<ClothProxy as RecordLayout<E>>::size()),
        (b"PHCL", 0) => Some(<ClothPhysics<V0> as RecordLayout<E>>::size()),
        (b"PHCL", 1) => Some(<ClothPhysics<V1> as RecordLayout<E>>::size()),
        (b"PHCL", 2) => Some(<ClothPhysics<V2> as RecordLayout<E>>::size()),
        (b"PHCL", 3) => Some(<ClothPhysics<V3> as RecordLayout<E>>::size()),
        (b"PHCL", 4) => Some(<ClothPhysics<V4> as RecordLayout<E>>::size()),
        (b"LITE", 0) => Some(<Light<V0> as RecordLayout<E>>::size()),
        (b"LITE", 1) => Some(<Light<V1> as RecordLayout<E>>::size()),
        (b"LITE", 2) => Some(<Light<V2> as RecordLayout<E>>::size()),
        (b"LITE", 3) => Some(<Light<V3> as RecordLayout<E>>::size()),
        (b"LITE", 4) => Some(<Light<V4> as RecordLayout<E>>::size()),
        (b"LITE", 5) => Some(<Light<V5> as RecordLayout<E>>::size()),
        (b"LITE", 6) => Some(<Light<V6> as RecordLayout<E>>::size()),
        (b"LITE", 7) => Some(<Light<V7> as RecordLayout<E>>::size()),
        (b"CAM_", 2) => Some(<Camera<V2> as RecordLayout<E>>::size()),
        (b"CAM_", 3) => Some(<Camera<V3> as RecordLayout<E>>::size()),
        (b"CAM_", 4) => Some(<Camera<V4> as RecordLayout<E>>::size()),
        (b"CAM_", 5) => Some(<Camera<V5> as RecordLayout<E>>::size()),
        (b"MODL", 20) => Some(<ModelRecord<V20> as RecordLayout<E>>::size()),
        (b"MODL", 21) => Some(<ModelRecord<V21> as RecordLayout<E>>::size()),
        (b"MODL", 22) => Some(<ModelRecord<V22> as RecordLayout<E>>::size()),
        (b"MODL", 23) => Some(<ModelRecord<V23> as RecordLayout<E>>::size()),
        (b"MODL", 24) => Some(<ModelRecord<V24> as RecordLayout<E>>::size()),
        (b"MODL", 25) => Some(<ModelRecord<V25> as RecordLayout<E>>::size()),
        (b"MODL", 26) => Some(<ModelRecord<V26> as RecordLayout<E>>::size()),
        (b"MODL", 27) => Some(<ModelRecord<V27> as RecordLayout<E>>::size()),
        (b"MODL", 28) => Some(<ModelRecord<V28> as RecordLayout<E>>::size()),
        (b"MODL", 29) => Some(<ModelRecord<V29> as RecordLayout<E>>::size()),
        (b"MODL", 30) => Some(<ModelRecord<V30> as RecordLayout<E>>::size()),
        (b"CHAR", 0) => Some(<Character as RecordLayout<E>>::size()),
        (b"U8__", 0) => Some(<Byte as RecordLayout<E>>::size()),
        (b"U16_", 0) => Some(<U16 as RecordLayout<E>>::size()),
        (b"U32_", 0) => Some(<U32 as RecordLayout<E>>::size()),
        (b"U64_", 0) => Some(<U64 as RecordLayout<E>>::size()),
        (b"I16_", 0) => Some(<I16 as RecordLayout<E>>::size()),
        (b"I32_", 0) => Some(<I32 as RecordLayout<E>>::size()),
        (b"REAL", 0) => Some(<Real as RecordLayout<E>>::size()),
        (b"FLAG", 0) => Some(<Flag as RecordLayout<E>>::size()),
        (b"VEC2", 0) => Some(<Vector2 as RecordLayout<E>>::size()),
        (b"VEC3", 0) => Some(<Vector3 as RecordLayout<E>>::size()),
        (b"VEC4", 0) => Some(<Vector4 as RecordLayout<E>>::size()),
        (b"QUAT", 0) => Some(<Quaternion as RecordLayout<E>>::size()),
        (b"\0COL", 0) => Some(<Color as RecordLayout<E>>::size()),
        (b"BNDS", 0) => Some(<ExtentRecord as RecordLayout<E>>::size()),
        (b"SDEV", 0) => Some(<AnimationBlockEvent as RecordLayout<E>>::size()),
        (b"SD2V", 0) => Some(<AnimationBlockVector2 as RecordLayout<E>>::size()),
        (b"SD3V", 0) => Some(<AnimationBlockVector3 as RecordLayout<E>>::size()),
        (b"SDR3", 0) => Some(<AnimationBlockReal as RecordLayout<E>>::size()),
        (b"SD4Q", 0) => Some(<AnimationBlockQuaternion as RecordLayout<E>>::size()),
        (b"SDCC", 0) => Some(<AnimationBlockColor as RecordLayout<E>>::size()),
        (b"SDU8", 0) => Some(<AnimationBlockByte as RecordLayout<E>>::size()),
        (b"SDS6", 0) => Some(<AnimationBlockI16 as RecordLayout<E>>::size()),
        (b"SDU6", 0) => Some(<AnimationBlockU16 as RecordLayout<E>>::size()),
        (b"SDS3", 0) => Some(<AnimationBlockI32 as RecordLayout<E>>::size()),
        (b"SDU3", 0) => Some(<AnimationBlockU32 as RecordLayout<E>>::size()),
        (b"SDFG", 0) => Some(<AnimationBlockFlag as RecordLayout<E>>::size()),
        (b"SDMB", 0) => Some(<AnimationBlockExtent as RecordLayout<E>>::size()),
        (b"SS32", 0) => Some(<AnimationValueI32 as RecordLayout<E>>::size()),
        (b"SU32", 0) => Some(<AnimationValueU32 as RecordLayout<E>>::size()),
        (b"SR32", 0) => Some(<AnimationValueReal as RecordLayout<E>>::size()),
        (b"SVC3", 0) => Some(<AnimationValueVector3 as RecordLayout<E>>::size()),
        (b"SVC2", 0) => Some(<AnimationValueVector2 as RecordLayout<E>>::size()),
        (b"MT16", 0) => Some(<MeshTriangle16 as RecordLayout<E>>::size()),
        (b"MT32", 0) => Some(<MeshTriangle32 as RecordLayout<E>>::size()),
        (b"SCHR", 0) => Some(<StringReference as RecordLayout<E>>::size()),
        (b"PHCT", 0) => Some(<PhysicsConstraint as RecordLayout<E>>::size()),
        _ => None,
    }
}
/// Enumerates known native layouts with their envelope-specific inline widths.
pub fn known_layouts<E: Envelope>() -> impl Iterator<Item = (RecordDescriptor, usize)> {
    SUPPORTED_RECORDS.iter().copied().filter_map(|descriptor| {
        known_record_size::<E>(descriptor.tag, descriptor.revision).map(|size| (descriptor, size))
    })
}
fn visit_records<E: Envelope, T, V>(
    payload: &[u8],
    count: u32,
    visitor: &mut V,
) -> Result<(), ReadError>
where
    T: for<'data> Read<'data, E> + RecordLayout<E> + VisitReferences + Validate,
    V: ReferenceVisitor<Error = ReadError>,
{
    let width = T::size();
    let length = (count as usize)
        .checked_mul(width)
        .ok_or_else(|| ReadError::invalid(0, "section record count overflow"))?;
    if length != payload.len() {
        return Err(ReadError::invalid(0, "known section record span"));
    }
    let mut input = Decoder::<E>::new(&payload[..length]);
    for _ in 0..count {
        let start = input.position();
        let value = T::read(&mut input)?;
        if input.position() != start + width {
            return Err(ReadError::invalid(start, "native record width"));
        }
        value.validate()?;
        value.visit_references(visitor)?;
    }
    Ok(())
}
/// Visits every native reference in an exact known section revision.
/// Returns false when no established schema exists for the tag/revision pair.
pub fn visit_known_references<E: Envelope, V: ReferenceVisitor<Error = ReadError>>(
    tag: [u8; 4],
    revision: u32,
    payload: &[u8],
    count: u32,
    visitor: &mut V,
) -> Result<bool, ReadError> {
    match (&tag, revision) {
        (b"EVNT", 0) => visit_records::<E, Event<V0>, V>(payload, count, visitor)?,
        (b"EVNT", 1) => visit_records::<E, Event<V1>, V>(payload, count, visitor)?,
        (b"EVNT", 2) => visit_records::<E, Event<V2>, V>(payload, count, visitor)?,
        (b"SEQS", 0) => visit_records::<E, Sequence<V0>, V>(payload, count, visitor)?,
        (b"SEQS", 1) => visit_records::<E, Sequence<V1>, V>(payload, count, visitor)?,
        (b"SEQS", 2) => visit_records::<E, Sequence<V2>, V>(payload, count, visitor)?,
        (b"STC_", 0) => visit_records::<E, SubTrackContainer<V0>, V>(payload, count, visitor)?,
        (b"STC_", 1) => visit_records::<E, SubTrackContainer<V1>, V>(payload, count, visitor)?,
        (b"STC_", 2) => visit_records::<E, SubTrackContainer<V2>, V>(payload, count, visitor)?,
        (b"STC_", 3) => visit_records::<E, SubTrackContainer<V3>, V>(payload, count, visitor)?,
        (b"STC_", 4) => visit_records::<E, SubTrackContainer<V4>, V>(payload, count, visitor)?,
        (b"STG_", 0) => visit_records::<E, AnimationGroup, V>(payload, count, visitor)?,
        (b"STS_", 0) => visit_records::<E, AnimationState, V>(payload, count, visitor)?,
        (b"BSET", 0) => visit_records::<E, BoneAnimationSet, V>(payload, count, visitor)?,
        (b"BONE", 0) => visit_records::<E, Bone<V0>, V>(payload, count, visitor)?,
        (b"BONE", 1) => visit_records::<E, Bone<V1>, V>(payload, count, visitor)?,
        (b"REGN", 0) => visit_records::<E, Region<V0>, V>(payload, count, visitor)?,
        (b"REGN", 1) => visit_records::<E, Region<V1>, V>(payload, count, visitor)?,
        (b"REGN", 2) => visit_records::<E, Region<V2>, V>(payload, count, visitor)?,
        (b"REGN", 3) => visit_records::<E, Region<V3>, V>(payload, count, visitor)?,
        (b"REGN", 4) => visit_records::<E, Region<V4>, V>(payload, count, visitor)?,
        (b"REGN", 5) => visit_records::<E, Region<V5>, V>(payload, count, visitor)?,
        (b"BAT_", 0) => visit_records::<E, Batch<V0>, V>(payload, count, visitor)?,
        (b"BAT_", 1) => visit_records::<E, Batch<V1>, V>(payload, count, visitor)?,
        (b"MSEC", 0) => visit_records::<E, MeshSection<V0>, V>(payload, count, visitor)?,
        (b"MSEC", 1) => visit_records::<E, MeshSection<V1>, V>(payload, count, visitor)?,
        (b"DIV_", 0) => visit_records::<E, MeshDivision<V0>, V>(payload, count, visitor)?,
        (b"DIV_", 1) => visit_records::<E, MeshDivision<V1>, V>(payload, count, visitor)?,
        (b"DIV_", 2) => visit_records::<E, MeshDivision<V2>, V>(payload, count, visitor)?,
        (b"IREF", 0) => visit_records::<E, InitialReference, V>(payload, count, visitor)?,
        (b"ATT_", 0) => visit_records::<E, AttachmentPoint<V0>, V>(payload, count, visitor)?,
        (b"ATT_", 1) => visit_records::<E, AttachmentPoint<V1>, V>(payload, count, visitor)?,
        (b"MATM", 0) => visit_records::<E, MaterialMap, V>(payload, count, visitor)?,
        (b"LAYR", 0) => visit_records::<E, TextureLayer<V0>, V>(payload, count, visitor)?,
        (b"LAYR", 1) => visit_records::<E, TextureLayer<V1>, V>(payload, count, visitor)?,
        (b"LAYR", 2) => visit_records::<E, TextureLayer<V2>, V>(payload, count, visitor)?,
        (b"LAYR", 3) => visit_records::<E, TextureLayer<V3>, V>(payload, count, visitor)?,
        (b"LAYR", 4) => visit_records::<E, TextureLayer<V4>, V>(payload, count, visitor)?,
        (b"LAYR", 5) => visit_records::<E, TextureLayer<V5>, V>(payload, count, visitor)?,
        (b"LAYR", 6) => visit_records::<E, TextureLayer<V6>, V>(payload, count, visitor)?,
        (b"LAYR", 7) => visit_records::<E, TextureLayer<V7>, V>(payload, count, visitor)?,
        (b"LAYR", 8) => visit_records::<E, TextureLayer<V8>, V>(payload, count, visitor)?,
        (b"LAYR", 9) => visit_records::<E, TextureLayer<V9>, V>(payload, count, visitor)?,
        (b"LAYR", 10) => visit_records::<E, TextureLayer<V10>, V>(payload, count, visitor)?,
        (b"LAYR", 11) => visit_records::<E, TextureLayer<V11>, V>(payload, count, visitor)?,
        (b"LAYR", 12) => visit_records::<E, TextureLayer<V12>, V>(payload, count, visitor)?,
        (b"LAYR", 13) => visit_records::<E, TextureLayer<V13>, V>(payload, count, visitor)?,
        (b"LAYR", 14) => visit_records::<E, TextureLayer<V14>, V>(payload, count, visitor)?,
        (b"LAYR", 15) => visit_records::<E, TextureLayer<V15>, V>(payload, count, visitor)?,
        (b"LAYR", 16) => visit_records::<E, TextureLayer<V16>, V>(payload, count, visitor)?,
        (b"LAYR", 17) => visit_records::<E, TextureLayer<V17>, V>(payload, count, visitor)?,
        (b"LAYR", 18) => visit_records::<E, TextureLayer<V18>, V>(payload, count, visitor)?,
        (b"LAYR", 19) => visit_records::<E, TextureLayer<V19>, V>(payload, count, visitor)?,
        (b"LAYR", 20) => visit_records::<E, TextureLayer<V20>, V>(payload, count, visitor)?,
        (b"LAYR", 21) => visit_records::<E, TextureLayer<V21>, V>(payload, count, visitor)?,
        (b"LAYR", 22) => visit_records::<E, TextureLayer<V22>, V>(payload, count, visitor)?,
        (b"LAYR", 23) => visit_records::<E, TextureLayer<V23>, V>(payload, count, visitor)?,
        (b"LAYR", 24) => visit_records::<E, TextureLayer<V24>, V>(payload, count, visitor)?,
        (b"LAYR", 25) => visit_records::<E, TextureLayer<V25>, V>(payload, count, visitor)?,
        (b"LAYR", 26) => visit_records::<E, TextureLayer<V26>, V>(payload, count, visitor)?,
        (b"MAT_", 0) => visit_records::<E, StandardMaterial<V0>, V>(payload, count, visitor)?,
        (b"MAT_", 1) => visit_records::<E, StandardMaterial<V1>, V>(payload, count, visitor)?,
        (b"MAT_", 2) => visit_records::<E, StandardMaterial<V2>, V>(payload, count, visitor)?,
        (b"MAT_", 3) => visit_records::<E, StandardMaterial<V3>, V>(payload, count, visitor)?,
        (b"MAT_", 4) => visit_records::<E, StandardMaterial<V4>, V>(payload, count, visitor)?,
        (b"MAT_", 5) => visit_records::<E, StandardMaterial<V5>, V>(payload, count, visitor)?,
        (b"MAT_", 6) => visit_records::<E, StandardMaterial<V6>, V>(payload, count, visitor)?,
        (b"MAT_", 7) => visit_records::<E, StandardMaterial<V7>, V>(payload, count, visitor)?,
        (b"MAT_", 8) => visit_records::<E, StandardMaterial<V8>, V>(payload, count, visitor)?,
        (b"MAT_", 9) => visit_records::<E, StandardMaterial<V9>, V>(payload, count, visitor)?,
        (b"MAT_", 10) => visit_records::<E, StandardMaterial<V10>, V>(payload, count, visitor)?,
        (b"MAT_", 11) => visit_records::<E, StandardMaterial<V11>, V>(payload, count, visitor)?,
        (b"MAT_", 12) => visit_records::<E, StandardMaterial<V12>, V>(payload, count, visitor)?,
        (b"MAT_", 13) => visit_records::<E, StandardMaterial<V13>, V>(payload, count, visitor)?,
        (b"MAT_", 14) => visit_records::<E, StandardMaterial<V14>, V>(payload, count, visitor)?,
        (b"MAT_", 15) => visit_records::<E, StandardMaterial<V15>, V>(payload, count, visitor)?,
        (b"MAT_", 16) => visit_records::<E, StandardMaterial<V16>, V>(payload, count, visitor)?,
        (b"MAT_", 17) => visit_records::<E, StandardMaterial<V17>, V>(payload, count, visitor)?,
        (b"MAT_", 18) => visit_records::<E, StandardMaterial<V18>, V>(payload, count, visitor)?,
        (b"MAT_", 19) => visit_records::<E, StandardMaterial<V19>, V>(payload, count, visitor)?,
        (b"MAT_", 20) => visit_records::<E, StandardMaterial<V20>, V>(payload, count, visitor)?,
        (b"DIS_", 0) => visit_records::<E, DisplacementMaterial<V0>, V>(payload, count, visitor)?,
        (b"DIS_", 1) => visit_records::<E, DisplacementMaterial<V1>, V>(payload, count, visitor)?,
        (b"DIS_", 2) => visit_records::<E, DisplacementMaterial<V2>, V>(payload, count, visitor)?,
        (b"DIS_", 3) => visit_records::<E, DisplacementMaterial<V3>, V>(payload, count, visitor)?,
        (b"DIS_", 4) => visit_records::<E, DisplacementMaterial<V4>, V>(payload, count, visitor)?,
        (b"CMS_", 0) => visit_records::<E, CompositeSection, V>(payload, count, visitor)?,
        (b"CMP_", 0) => visit_records::<E, CompositeMaterial<V0>, V>(payload, count, visitor)?,
        (b"CMP_", 1) => visit_records::<E, CompositeMaterial<V1>, V>(payload, count, visitor)?,
        (b"CMP_", 2) => visit_records::<E, CompositeMaterial<V2>, V>(payload, count, visitor)?,
        (b"TER_", 0) => visit_records::<E, TerrainMaterial<V0>, V>(payload, count, visitor)?,
        (b"TER_", 1) => visit_records::<E, TerrainMaterial<V1>, V>(payload, count, visitor)?,
        (b"VOL_", 0) => visit_records::<E, VolumeMaterial, V>(payload, count, visitor)?,
        (b"HAI_", 0) => visit_records::<E, HairMaterial, V>(payload, count, visitor)?,
        (b"VON_", 0) => visit_records::<E, VolumeNoiseMaterial, V>(payload, count, visitor)?,
        (b"CREP", 0) => visit_records::<E, CreepMaterial<V0>, V>(payload, count, visitor)?,
        (b"CREP", 1) => visit_records::<E, CreepMaterial<V1>, V>(payload, count, visitor)?,
        (b"STBM", 0) => visit_records::<E, STBMaterial, V>(payload, count, visitor)?,
        (b"REF_", 0) => visit_records::<E, ReflectionMaterial<V0>, V>(payload, count, visitor)?,
        (b"REF_", 1) => visit_records::<E, ReflectionMaterial<V1>, V>(payload, count, visitor)?,
        (b"REF_", 2) => visit_records::<E, ReflectionMaterial<V2>, V>(payload, count, visitor)?,
        (b"REF_", 3) => visit_records::<E, ReflectionMaterial<V3>, V>(payload, count, visitor)?,
        (b"LFSB", 0) => visit_records::<E, SubFlare<V0>, V>(payload, count, visitor)?,
        (b"LFSB", 1) => visit_records::<E, SubFlare<V1>, V>(payload, count, visitor)?,
        (b"LFSB", 2) => visit_records::<E, SubFlare<V2>, V>(payload, count, visitor)?,
        (b"LFLR", 0) => visit_records::<E, LensFlare<V0>, V>(payload, count, visitor)?,
        (b"LFLR", 1) => visit_records::<E, LensFlare<V1>, V>(payload, count, visitor)?,
        (b"LFLR", 2) => visit_records::<E, LensFlare<V2>, V>(payload, count, visitor)?,
        (b"LFLR", 3) => visit_records::<E, LensFlare<V3>, V>(payload, count, visitor)?,
        (b"MADD", 0) => visit_records::<E, DataDrivenMaterial<V0>, V>(payload, count, visitor)?,
        (b"MADD", 1) => visit_records::<E, DataDrivenMaterial<V1>, V>(payload, count, visitor)?,
        (b"MADD", 2) => visit_records::<E, DataDrivenMaterial<V2>, V>(payload, count, visitor)?,
        (b"MADD", 3) => visit_records::<E, DataDrivenMaterial<V3>, V>(payload, count, visitor)?,
        (b"PAR_", 10) => visit_records::<E, ParticleEmitter<V10>, V>(payload, count, visitor)?,
        (b"PAR_", 11) => visit_records::<E, ParticleEmitter<V11>, V>(payload, count, visitor)?,
        (b"PAR_", 12) => visit_records::<E, ParticleEmitter<V12>, V>(payload, count, visitor)?,
        (b"PAR_", 13) => visit_records::<E, ParticleEmitter<V13>, V>(payload, count, visitor)?,
        (b"PAR_", 14) => visit_records::<E, ParticleEmitter<V14>, V>(payload, count, visitor)?,
        (b"PAR_", 15) => visit_records::<E, ParticleEmitter<V15>, V>(payload, count, visitor)?,
        (b"PAR_", 16) => visit_records::<E, ParticleEmitter<V16>, V>(payload, count, visitor)?,
        (b"PAR_", 17) => visit_records::<E, ParticleEmitter<V17>, V>(payload, count, visitor)?,
        (b"PAR_", 18) => visit_records::<E, ParticleEmitter<V18>, V>(payload, count, visitor)?,
        (b"PAR_", 19) => visit_records::<E, ParticleEmitter<V19>, V>(payload, count, visitor)?,
        (b"PAR_", 20) => visit_records::<E, ParticleEmitter<V20>, V>(payload, count, visitor)?,
        (b"PAR_", 21) => visit_records::<E, ParticleEmitter<V21>, V>(payload, count, visitor)?,
        (b"PAR_", 22) => visit_records::<E, ParticleEmitter<V22>, V>(payload, count, visitor)?,
        (b"PAR_", 23) => visit_records::<E, ParticleEmitter<V23>, V>(payload, count, visitor)?,
        (b"PAR_", 24) => visit_records::<E, ParticleEmitter<V24>, V>(payload, count, visitor)?,
        (b"PARC", 0) => visit_records::<E, ParticleEmitterCopy, V>(payload, count, visitor)?,
        (b"SRIB", 0) => visit_records::<E, SplineRibbon, V>(payload, count, visitor)?,
        (b"RIB_", 4) => visit_records::<E, RibbonEmitter<V4>, V>(payload, count, visitor)?,
        (b"RIB_", 5) => visit_records::<E, RibbonEmitter<V5>, V>(payload, count, visitor)?,
        (b"RIB_", 6) => visit_records::<E, RibbonEmitter<V6>, V>(payload, count, visitor)?,
        (b"RIB_", 7) => visit_records::<E, RibbonEmitter<V7>, V>(payload, count, visitor)?,
        (b"RIB_", 8) => visit_records::<E, RibbonEmitter<V8>, V>(payload, count, visitor)?,
        (b"RIB_", 9) => visit_records::<E, RibbonEmitter<V9>, V>(payload, count, visitor)?,
        (b"PROJ", 4) => visit_records::<E, Projector<V4>, V>(payload, count, visitor)?,
        (b"PROJ", 5) => visit_records::<E, Projector<V5>, V>(payload, count, visitor)?,
        (b"SSGS", 0) => visit_records::<E, HitTestShape<V0>, V>(payload, count, visitor)?,
        (b"SSGS", 1) => visit_records::<E, HitTestShape<V1>, V>(payload, count, visitor)?,
        (b"ATVL", 0) => visit_records::<E, AttachmentVolume, V>(payload, count, visitor)?,
        (b"TRGD", 0) => visit_records::<E, TriggerData, V>(payload, count, visitor)?,
        (b"PATU", 0) => visit_records::<E, TurretBehavior<V0>, V>(payload, count, visitor)?,
        (b"PATU", 1) => visit_records::<E, TurretBehavior<V1>, V>(payload, count, visitor)?,
        (b"PATU", 2) => visit_records::<E, TurretBehavior<V2>, V>(payload, count, visitor)?,
        (b"PATU", 3) => visit_records::<E, TurretBehavior<V3>, V>(payload, count, visitor)?,
        (b"PATU", 4) => visit_records::<E, TurretBehavior<V4>, V>(payload, count, visitor)?,
        (b"BBSC", 0) => visit_records::<E, BillboardBehavior, V>(payload, count, visitor)?,
        (b"IKJT", 0) => visit_records::<E, IKJoint, V>(payload, count, visitor)?,
        (b"IK2J", 0) => visit_records::<E, IKTwoJoint, V>(payload, count, visitor)?,
        (b"IKCC", 0) => visit_records::<E, IKCCD, V>(payload, count, visitor)?,
        (b"PAOB", 0) => visit_records::<E, OneBoneSolver, V>(payload, count, visitor)?,
        (b"SHBX", 0) => visit_records::<E, ShadowBox, V>(payload, count, visitor)?,
        (b"VVOL", 0) => visit_records::<E, ViewVolume, V>(payload, count, visitor)?,
        (b"TMD_", 0) => visit_records::<E, TrailingModel<V0>, V>(payload, count, visitor)?,
        (b"TMD_", 1) => visit_records::<E, TrailingModel<V1>, V>(payload, count, visitor)?,
        (b"FOR_", 0) => visit_records::<E, Force<V0>, V>(payload, count, visitor)?,
        (b"FOR_", 1) => visit_records::<E, Force<V1>, V>(payload, count, visitor)?,
        (b"FOR_", 2) => visit_records::<E, Force<V2>, V>(payload, count, visitor)?,
        (b"WRP_", 1) => visit_records::<E, Warp, V>(payload, count, visitor)?,
        (b"DMSE", 0) => visit_records::<E, ConvexHullHalfEdge, V>(payload, count, visitor)?,
        (b"DMMN", 0) => visit_records::<E, PhysicsMeshBvhNode<V0>, V>(payload, count, visitor)?,
        (b"DMMN", 1) => visit_records::<E, PhysicsMeshBvhNode<V1>, V>(payload, count, visitor)?,
        (b"DMMT", 0) => visit_records::<E, PhysicsMeshTriangle, V>(payload, count, visitor)?,
        (b"DMME", 0) => visit_records::<E, PhysicsMeshEdge, V>(payload, count, visitor)?,
        (b"PHSH", 0) => visit_records::<E, PhysicsShape<V0>, V>(payload, count, visitor)?,
        (b"PHSH", 1) => visit_records::<E, PhysicsShape<V1>, V>(payload, count, visitor)?,
        (b"PHSH", 2) => visit_records::<E, PhysicsShape<V2>, V>(payload, count, visitor)?,
        (b"PHSH", 3) => visit_records::<E, PhysicsShape<V3>, V>(payload, count, visitor)?,
        (b"PHRB", 0) => visit_records::<E, RigidBody<V0>, V>(payload, count, visitor)?,
        (b"PHRB", 1) => visit_records::<E, RigidBody<V1>, V>(payload, count, visitor)?,
        (b"PHRB", 2) => visit_records::<E, RigidBody<V2>, V>(payload, count, visitor)?,
        (b"PHRB", 3) => visit_records::<E, RigidBody<V3>, V>(payload, count, visitor)?,
        (b"PHRB", 4) => visit_records::<E, RigidBody<V4>, V>(payload, count, visitor)?,
        (b"PHYJ", 0) => visit_records::<E, PhysicsJoint, V>(payload, count, visitor)?,
        (b"PHCC", 0) => visit_records::<E, ClothCollider, V>(payload, count, visitor)?,
        (b"PHAC", 0) => visit_records::<E, ClothProxy, V>(payload, count, visitor)?,
        (b"PHCL", 0) => visit_records::<E, ClothPhysics<V0>, V>(payload, count, visitor)?,
        (b"PHCL", 1) => visit_records::<E, ClothPhysics<V1>, V>(payload, count, visitor)?,
        (b"PHCL", 2) => visit_records::<E, ClothPhysics<V2>, V>(payload, count, visitor)?,
        (b"PHCL", 3) => visit_records::<E, ClothPhysics<V3>, V>(payload, count, visitor)?,
        (b"PHCL", 4) => visit_records::<E, ClothPhysics<V4>, V>(payload, count, visitor)?,
        (b"LITE", 0) => visit_records::<E, Light<V0>, V>(payload, count, visitor)?,
        (b"LITE", 1) => visit_records::<E, Light<V1>, V>(payload, count, visitor)?,
        (b"LITE", 2) => visit_records::<E, Light<V2>, V>(payload, count, visitor)?,
        (b"LITE", 3) => visit_records::<E, Light<V3>, V>(payload, count, visitor)?,
        (b"LITE", 4) => visit_records::<E, Light<V4>, V>(payload, count, visitor)?,
        (b"LITE", 5) => visit_records::<E, Light<V5>, V>(payload, count, visitor)?,
        (b"LITE", 6) => visit_records::<E, Light<V6>, V>(payload, count, visitor)?,
        (b"LITE", 7) => visit_records::<E, Light<V7>, V>(payload, count, visitor)?,
        (b"CAM_", 2) => visit_records::<E, Camera<V2>, V>(payload, count, visitor)?,
        (b"CAM_", 3) => visit_records::<E, Camera<V3>, V>(payload, count, visitor)?,
        (b"CAM_", 4) => visit_records::<E, Camera<V4>, V>(payload, count, visitor)?,
        (b"CAM_", 5) => visit_records::<E, Camera<V5>, V>(payload, count, visitor)?,
        (b"MODL", 20) => visit_records::<E, ModelRecord<V20>, V>(payload, count, visitor)?,
        (b"MODL", 21) => visit_records::<E, ModelRecord<V21>, V>(payload, count, visitor)?,
        (b"MODL", 22) => visit_records::<E, ModelRecord<V22>, V>(payload, count, visitor)?,
        (b"MODL", 23) => visit_records::<E, ModelRecord<V23>, V>(payload, count, visitor)?,
        (b"MODL", 24) => visit_records::<E, ModelRecord<V24>, V>(payload, count, visitor)?,
        (b"MODL", 25) => visit_records::<E, ModelRecord<V25>, V>(payload, count, visitor)?,
        (b"MODL", 26) => visit_records::<E, ModelRecord<V26>, V>(payload, count, visitor)?,
        (b"MODL", 27) => visit_records::<E, ModelRecord<V27>, V>(payload, count, visitor)?,
        (b"MODL", 28) => visit_records::<E, ModelRecord<V28>, V>(payload, count, visitor)?,
        (b"MODL", 29) => visit_records::<E, ModelRecord<V29>, V>(payload, count, visitor)?,
        (b"MODL", 30) => visit_records::<E, ModelRecord<V30>, V>(payload, count, visitor)?,
        (b"CHAR", 0) => visit_records::<E, Character, V>(payload, count, visitor)?,
        (b"U8__", 0) => visit_records::<E, Byte, V>(payload, count, visitor)?,
        (b"U16_", 0) => visit_records::<E, U16, V>(payload, count, visitor)?,
        (b"U32_", 0) => visit_records::<E, U32, V>(payload, count, visitor)?,
        (b"U64_", 0) => visit_records::<E, U64, V>(payload, count, visitor)?,
        (b"I16_", 0) => visit_records::<E, I16, V>(payload, count, visitor)?,
        (b"I32_", 0) => visit_records::<E, I32, V>(payload, count, visitor)?,
        (b"REAL", 0) => visit_records::<E, Real, V>(payload, count, visitor)?,
        (b"FLAG", 0) => visit_records::<E, Flag, V>(payload, count, visitor)?,
        (b"VEC2", 0) => visit_records::<E, Vector2, V>(payload, count, visitor)?,
        (b"VEC3", 0) => visit_records::<E, Vector3, V>(payload, count, visitor)?,
        (b"VEC4", 0) => visit_records::<E, Vector4, V>(payload, count, visitor)?,
        (b"QUAT", 0) => visit_records::<E, Quaternion, V>(payload, count, visitor)?,
        (b"\0COL", 0) => visit_records::<E, Color, V>(payload, count, visitor)?,
        (b"BNDS", 0) => visit_records::<E, ExtentRecord, V>(payload, count, visitor)?,
        (b"SDEV", 0) => visit_records::<E, AnimationBlockEvent, V>(payload, count, visitor)?,
        (b"SD2V", 0) => visit_records::<E, AnimationBlockVector2, V>(payload, count, visitor)?,
        (b"SD3V", 0) => visit_records::<E, AnimationBlockVector3, V>(payload, count, visitor)?,
        (b"SDR3", 0) => visit_records::<E, AnimationBlockReal, V>(payload, count, visitor)?,
        (b"SD4Q", 0) => visit_records::<E, AnimationBlockQuaternion, V>(payload, count, visitor)?,
        (b"SDCC", 0) => visit_records::<E, AnimationBlockColor, V>(payload, count, visitor)?,
        (b"SDU8", 0) => visit_records::<E, AnimationBlockByte, V>(payload, count, visitor)?,
        (b"SDS6", 0) => visit_records::<E, AnimationBlockI16, V>(payload, count, visitor)?,
        (b"SDU6", 0) => visit_records::<E, AnimationBlockU16, V>(payload, count, visitor)?,
        (b"SDS3", 0) => visit_records::<E, AnimationBlockI32, V>(payload, count, visitor)?,
        (b"SDU3", 0) => visit_records::<E, AnimationBlockU32, V>(payload, count, visitor)?,
        (b"SDFG", 0) => visit_records::<E, AnimationBlockFlag, V>(payload, count, visitor)?,
        (b"SDMB", 0) => visit_records::<E, AnimationBlockExtent, V>(payload, count, visitor)?,
        (b"SS32", 0) => visit_records::<E, AnimationValueI32, V>(payload, count, visitor)?,
        (b"SU32", 0) => visit_records::<E, AnimationValueU32, V>(payload, count, visitor)?,
        (b"SR32", 0) => visit_records::<E, AnimationValueReal, V>(payload, count, visitor)?,
        (b"SVC3", 0) => visit_records::<E, AnimationValueVector3, V>(payload, count, visitor)?,
        (b"SVC2", 0) => visit_records::<E, AnimationValueVector2, V>(payload, count, visitor)?,
        (b"MT16", 0) => visit_records::<E, MeshTriangle16, V>(payload, count, visitor)?,
        (b"MT32", 0) => visit_records::<E, MeshTriangle32, V>(payload, count, visitor)?,
        (b"SCHR", 0) => visit_records::<E, StringReference, V>(payload, count, visitor)?,
        (b"PHCT", 0) => visit_records::<E, PhysicsConstraint, V>(payload, count, visitor)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// Validates and traverses a known section with a caller-provided reference callback.
pub fn validate_known_section<E: Envelope>(
    tag: [u8; 4],
    revision: u32,
    payload: &[u8],
    count: u32,
    visit: &mut impl FnMut(RawReference) -> Result<(), ReadError>,
) -> Result<(), ReadError> {
    struct Callback<'a, F>(&'a mut F);
    impl<F: FnMut(RawReference) -> Result<(), ReadError>> ReferenceVisitor for Callback<'_, F> {
        type Error = ReadError;
        fn visit(&mut self, reference: RawReference) -> Result<(), ReadError> {
            (self.0)(reference)
        }
    }
    visit_known_references::<E, _>(tag, revision, payload, count, &mut Callback(visit))?;
    Ok(())
}
