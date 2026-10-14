//! Leaf arrays and scoped selectors shared by direct simulation adapters.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{physics::SectionRef, SectionId};
use crate::model::semantic::{MeshId, NodeId, Values};
use crate::model::wem::{
    records, Bool, InlineOption, RawDocument, Record, RecordLayout, Ref, Target, Write,
};
use std::collections::BTreeMap;
use zerovec::ZeroVec;

pub(super) fn read_error(value: impl std::fmt::Display) -> Error {
    Error::new("wem.physics", value.to_string())
}
pub(super) fn revision<T: Target>(
    document: &RawDocument<'_>,
    reference: Ref<T>,
) -> Result<Option<u32>, Error> {
    Ok(document
        .referenced_section(reference)
        .map_err(read_error)?
        .map(|section| section.revision()))
}
pub(super) fn append<T: Record + RecordLayout + Write>(
    document: &mut RawDocument<'_>,
    values: &[T],
) -> Result<Ref<T>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len()).map_err(read_error)?;
    let section = document.append(values).map_err(read_error)?;
    Ok(Ref::from_section(count, section.index()))
}
pub(super) fn read_words<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::U32>,
) -> Result<Values<'a, u32>, Error> {
    if reference.entries == 0 {
        return Ok(Values::default());
    }
    let values = document
        .resolve::<records::U32>(reference.cast())
        .map_err(read_error)?;
    for value in values.iter() {
        value.map_err(read_error)?;
    }
    let section = document
        .referenced_section(reference)
        .map_err(read_error)?
        .ok_or_else(|| Error::new("wem.physics", "missing leaf array"))?;
    let width = records::U32::fixed_size().expect("fixed primitive layout");
    let length = values
        .len()
        .checked_mul(width)
        .ok_or_else(|| Error::new("wem.physics", "leaf span overflows"))?;
    let bytes = section
        .bytes()
        .get(..length)
        .ok_or_else(|| Error::new("wem.physics", "leaf span exceeds its section"))?;
    Ok(Values::from_zero_vec(
        ZeroVec::<u32>::parse_bytes(bytes).map_err(read_error)?,
    ))
}
pub(super) fn read_vectors<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Vector3>,
) -> Result<Values<'a, [f32; 3]>, Error> {
    if reference.entries == 0 {
        return Ok(Values::default());
    }
    let values = document
        .resolve::<records::Vector3>(reference.cast())
        .map_err(read_error)?;
    for value in values.iter() {
        value.map_err(read_error)?;
    }
    let section = document
        .referenced_section(reference)
        .map_err(read_error)?
        .ok_or_else(|| Error::new("wem.physics", "missing leaf array"))?;
    let width = records::Vector3::fixed_size().expect("fixed primitive layout");
    let length = values
        .len()
        .checked_mul(width)
        .ok_or_else(|| Error::new("wem.physics", "leaf span overflows"))?;
    let bytes = section
        .bytes()
        .get(..length)
        .ok_or_else(|| Error::new("wem.physics", "leaf span exceeds its section"))?;
    Ok(Values::from_zero_vec(
        ZeroVec::<[f32; 3]>::parse_bytes(bytes).map_err(read_error)?,
    ))
}
pub(super) fn words(
    document: &mut RawDocument<'_>,
    values: impl IntoIterator<Item = u32>,
) -> Result<Ref<records::targets::U32>, Error> {
    Ok(append(
        document,
        &values
            .into_iter()
            .map(|value| records::U32 { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
pub(super) fn vectors(
    document: &mut RawDocument<'_>,
    values: &Values<'_, [f32; 3]>,
) -> Result<Ref<records::targets::Vector3>, Error> {
    Ok(append(
        document,
        &values
            .iter()
            .map(|value| records::Vector3 { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
pub(super) fn lookup<T: Copy>(values: &[T], index: u32, field: &str) -> Result<T, Error> {
    values.get(index as usize).copied().ok_or_else(|| {
        Error::new(
            "wem.physics",
            format!("{field} does not belong to the model"),
        )
    })
}
pub(super) fn mapped<K: Ord>(values: &BTreeMap<K, u32>, id: K, field: &str) -> Result<u32, Error> {
    values.get(&id).copied().ok_or_else(|| {
        Error::new(
            "wem.physics",
            format!("{field} has no destination identity"),
        )
    })
}
pub(super) fn section(
    value: records::SectionRef,
    meshes: &[MeshId],
    sections: &[Vec<SectionId>],
) -> Result<SectionRef, Error> {
    Ok(SectionRef {
        mesh: lookup(meshes, value.mesh, "cloth mesh")?,
        section: lookup(
            sections
                .get(value.mesh as usize)
                .ok_or_else(|| Error::new("wem.physics", "cloth mesh has no sections"))?,
            value.section,
            "cloth section",
        )?,
    })
}
pub(super) fn write_section(
    value: SectionRef,
    meshes: &BTreeMap<MeshId, u32>,
    sections: &BTreeMap<SectionId, u32>,
) -> Result<records::SectionRef, Error> {
    Ok(records::SectionRef {
        mesh: mapped(meshes, value.mesh, "cloth mesh")?,
        section: mapped(sections, value.section, "cloth section")?,
    })
}
pub(super) trait Optional<T> {
    fn optional(&self) -> Option<&T>;
}
impl<T> Optional<T> for () {
    fn optional(&self) -> Option<&T> {
        None
    }
}
impl<T> Optional<T> for InlineOption<T> {
    fn optional(&self) -> Option<&T> {
        self.value.as_ref()
    }
}
pub(in crate::model::conversion::wem) trait Scalar<T: Copy> {
    fn scalar(&self) -> Option<T>;
}
impl<T: Copy> Scalar<T> for () {
    fn scalar(&self) -> Option<T> {
        None
    }
}
macro_rules! scalar{($($ty:ty),*)=>{$(impl Scalar<$ty> for $ty{fn scalar(&self)->Option<$ty>{Some(*self)}})*};}
scalar!(
    u32,
    f32,
    records::JointMotorMode,
    Bool,
    records::WowShapeGameFlags
);
pub(super) fn node(nodes: &[NodeId], index: u32) -> Result<NodeId, Error> {
    lookup(nodes, index, "physics node")
}
pub(super) fn allocate(next: &mut u32) -> Result<u32, Error> {
    let id = *next;
    *next = next
        .checked_add(1)
        .ok_or_else(|| Error::new("wem.physics", "identity capacity exhausted"))?;
    Ok(id)
}

impl<T: Target> Scalar<Ref<T>> for Ref<T> {
    fn scalar(&self) -> Option<Ref<T>> {
        Some(*self)
    }
}
