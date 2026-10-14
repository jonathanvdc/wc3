//! Saved conventions and their node-owned transforms resolve actual pose identities.
use super::super::{actors::common::Field, text};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::poses::{
    Components, NodePose, Pose, PoseId, PoseRepresentation, PoseSpace, Poses, TestPose,
};
use crate::model::semantic::{ClipId, Quaternion, Time};
use crate::model::wem::{self, records, Bool, RawDocument, Record, RecordLayout, Ref, Target};
use std::collections::BTreeMap;

pub(in crate::model::conversion::wem) fn rows<T, U: Target>(
    document: &RawDocument<'_>,
    reference: Ref<U>,
) -> Result<Vec<T>, Error>
where
    T: Record + RecordLayout + for<'record> wem::Read<'record>,
{
    document
        .resolve(reference.cast::<T>())
        .map_err(|error| Error::new("wem.poses", error.to_string()))?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| Error::new("wem.poses", error.to_string()))
}
pub(in crate::model::conversion::wem) fn put<T: Record + wem::Write, U: Target>(
    document: &mut RawDocument<'_>,
    values: &[T],
) -> Result<Ref<U>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len())
        .map_err(|_| Error::new("wem.poses", "saved pose count exceeds u32"))?;
    let section = document
        .append(values)
        .map_err(|error| Error::new("wem.poses", error.to_string()))?;
    Ok(Ref::from_section(count, section.index()))
}
impl Field<records::PoseStorage> for records::PoseStorage {
    fn read(&self) -> Option<records::PoseStorage> {
        Some(*self)
    }
    fn write(value: Option<records::PoseStorage>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.poses.storage",
                "selected layout requires a representation declaration",
            )
        })
    }
}
fn schemas<'view, D: records::PoseSchemaDialect>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::PoseSchema>,
) -> Result<Poses<'view>, Error>
where
    D::Storage: Field<records::PoseStorage>,
    records::PoseSchema<D>: RecordLayout + for<'record> wem::Read<'record>,
{
    let mut poses = Poses::default();
    for row in rows::<records::PoseSchema<D>, _>(document, reference)? {
        poses.add(Pose {
            name: text::read(document, row.name)?,
            space: match row.space {
                records::PoseSpace::Model => PoseSpace::Model,
                records::PoseSpace::ParentRelative => PoseSpace::Parent,
                records::PoseSpace::Unknown(value) => PoseSpace::Unknown(value),
            },
            inverse: row.inverse.get(),
            representation: match row.storage.read().unwrap_or(records::PoseStorage::Trs) {
                records::PoseStorage::Trs => PoseRepresentation::Components,
                records::PoseStorage::Matrix => PoseRepresentation::Matrix,
                records::PoseStorage::Unknown(value) => PoseRepresentation::Unknown(value),
            },
        });
    }
    Ok(poses)
}
pub(in crate::model::conversion::wem) fn read<'view>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::PoseSchema>,
    authoritative: u32,
) -> Result<Poses<'view>, Error> {
    let mut value = if let Some(section) = document
        .referenced_section(reference)
        .map_err(|error| Error::new("wem.poses", error.to_string()))?
    {
        match section.revision() {
            1 => schemas::<records::V1>(document, reference)?,
            2 => schemas::<records::V2>(document, reference)?,
            _ => {
                return Err(Error::new(
                    "wem.poses",
                    "unsupported saved convention revision",
                ));
            }
        }
    } else {
        Poses::default()
    };
    let ids: Vec<_> = value.iter().map(|(id, _)| id).collect();
    let selected = if authoritative == u32::MAX {
        None
    } else {
        Some(*ids.get(authoritative as usize).ok_or_else(|| {
            Error::new(
                "wem.poses.reference",
                "reference convention is outside the saved pose table",
            )
        })?)
    };
    value.set_reference(selected)?;
    Ok(value)
}
type EncodedPoses = (
    Ref<records::targets::PoseSchema>,
    u32,
    BTreeMap<PoseId, u32>,
);
pub(in crate::model::conversion::wem) fn write(
    document: &mut RawDocument<'_>,
    poses: &Poses<'_>,
) -> Result<EncodedPoses, Error> {
    let mut ids = BTreeMap::new();
    let mut values = Vec::new();
    for (id, value) in poses.iter() {
        let index = u32::try_from(values.len())
            .map_err(|_| Error::new("wem.poses", "saved convention count exceeds u32"))?;
        ids.insert(id, index);
        values.push(records::PoseSchema::<records::V2> {
            name: text::write_bytes(document, value.name.as_bytes())?,
            space: match value.space {
                PoseSpace::Model => records::PoseSpace::Model,
                PoseSpace::Parent => records::PoseSpace::ParentRelative,
                PoseSpace::Unknown(value) => records::PoseSpace::Unknown(value),
            },
            inverse: Bool::from(value.inverse),
            storage: match value.representation {
                PoseRepresentation::Components => records::PoseStorage::Trs,
                PoseRepresentation::Matrix => records::PoseStorage::Matrix,
                PoseRepresentation::Unknown(value) => records::PoseStorage::Unknown(value),
            },
        });
    }
    let selected = poses
        .reference()
        .map(|id| {
            ids.get(&id).copied().ok_or_else(|| {
                Error::new(
                    "wem.poses.reference",
                    "reference convention belongs to another hierarchy",
                )
            })
        })
        .transpose()?
        .unwrap_or(u32::MAX);
    Ok((put(document, &values)?, selected, ids))
}
pub(in crate::model::conversion::wem) fn read_values<D: records::NodeDialect>(
    document: &RawDocument<'_>,
    node: &records::Node<D>,
    poses: &Poses<'_>,
) -> Result<Vec<NodePose>, Error>
where
    D::PoseMatrices: Field<Ref<records::targets::Matrix44>>,
{
    let components = rows::<records::Transform, _>(document, node.poses)?;
    let matrices = node
        .pose_matrices
        .read()
        .map(|reference| rows::<records::Matrix44, _>(document, reference))
        .transpose()?
        .unwrap_or_default();
    let ids: Vec<_> = poses.iter().map(|(id, _)| id).collect();
    if components.len() > ids.len() || matrices.len() > ids.len() {
        return Err(Error::new(
            "wem.node.poses",
            "saved transforms exceed the hierarchy convention table",
        ));
    }
    let mut values = Vec::new();
    for (index, id) in ids.into_iter().enumerate() {
        let components = components.get(index).map(|value| Components {
            translation: value.translation,
            rotation: Quaternion(value.rotation),
            scale: value.scale,
        });
        let matrix = matrices.get(index).map(|value| value.value);
        if components.is_some() || matrix.is_some() {
            values.push(NodePose {
                pose: id,
                components,
                matrix,
            });
        }
    }
    poses.validate_node(&values, &[])?;
    Ok(values)
}
pub(in crate::model::conversion::wem) fn write_values(
    document: &mut RawDocument<'_>,
    values: &[NodePose],
    poses: &Poses<'_>,
) -> Result<
    (
        Ref<records::targets::Transform>,
        Ref<records::targets::Matrix44>,
    ),
    Error,
> {
    poses.validate_node(values, &[])?;
    let mut components = Vec::new();
    let mut matrices = Vec::new();
    let mut component_gap = false;
    let mut matrix_gap = false;
    for (id, _) in poses.iter() {
        let value = values.iter().find(|value| value.pose == id);
        match value.and_then(|value| value.components.as_ref()) {
            Some(value) if !component_gap => components.push(records::Transform {
                translation: value.translation,
                rotation: value.rotation.components(),
                scale: value.scale,
            }),
            Some(_) => {
                return Err(Error::new(
                    "wem.node.poses",
                    "component poses cannot contain an absent interior slot",
                ));
            }
            None => component_gap = true,
        };
        match value.and_then(|value| value.matrix) {
            Some(value) if !matrix_gap => matrices.push(records::Matrix44 { value }),
            Some(_) => {
                return Err(Error::new(
                    "wem.node.poses",
                    "matrix poses cannot contain an absent interior slot",
                ));
            }
            None => matrix_gap = true,
        };
    }
    Ok((put(document, &components)?, put(document, &matrices)?))
}
pub(in crate::model::conversion::wem) fn read_test<'view>(
    document: &'view RawDocument<'_>,
    source: &records::TestPose,
    clips: &[ClipId],
) -> Result<TestPose<'view>, Error> {
    Ok(TestPose {
        name: text::read(document, source.name)?,
        clip: if source.clip == u32::MAX {
            None
        } else {
            Some(*clips.get(source.clip as usize).ok_or_else(|| {
                Error::new(
                    "wem.test_pose.clip",
                    "base clip is outside the document clip table",
                )
            })?)
        },
        time: Time::seconds(f64::from(source.time_ms) / 1000.)?,
    })
}
pub(in crate::model::conversion::wem) fn write_test(
    document: &mut RawDocument<'_>,
    source: &TestPose<'_>,
    clips: &BTreeMap<ClipId, u32>,
) -> Result<records::TestPose, Error> {
    let time = source.time.as_seconds() * 1000.;
    let encoded = time as f32;
    if !encoded.is_finite() || f64::from(encoded) / 1000. != source.time.as_seconds() {
        return Err(Error::new(
            "wem.test_pose.time",
            "time cannot be represented exactly as milliseconds",
        ));
    }
    Ok(records::TestPose {
        name: text::write_bytes(document, source.name.as_bytes())?,
        clip: source
            .clip
            .map(|id| {
                clips.get(&id).copied().ok_or_else(|| {
                    Error::new(
                        "wem.test_pose.clip",
                        "base clip belongs to another document",
                    )
                })
            })
            .transpose()?
            .unwrap_or(u32::MAX),
        time_ms: encoded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conventions_and_values_remain_editable_and_reject_foreign_slots() {
        let mut document = RawDocument::new();
        let mut conventions = Poses::default();
        let id = conventions.add(Pose {
            name: "bind".into(),
            space: PoseSpace::Parent,
            inverse: true,
            representation: PoseRepresentation::Matrix,
        });
        conventions.set_reference(Some(id)).unwrap();
        let (reference, selected, _) = write(&mut document, &conventions).unwrap();
        let decoded = read(&document, reference, selected).unwrap();
        let (decoded_id, pose) = decoded.iter().next().unwrap();
        assert_eq!(pose.name, "bind");
        assert_eq!(pose.space, PoseSpace::Parent);
        assert!(pose.inverse);
        assert_eq!(decoded.reference(), Some(decoded_id));
        let mut value = NodePose {
            pose: id,
            components: None,
            matrix: Some([[0.; 4]; 4]),
        };
        value.matrix.as_mut().unwrap()[3][0] = 42.;
        let mut destination = RawDocument::new();
        let (_, matrices) = write_values(&mut destination, &[value.clone()], &conventions).unwrap();
        let emitted = rows::<records::Matrix44, _>(&destination, matrices).unwrap();
        assert_eq!(emitted[0].value[3][0], 42.);
        assert!(write_values(&mut destination, &[value], &decoded).is_err());
    }
    #[test]
    fn old_schema_has_actual_component_meaning_without_fabricated_storage() {
        let mut document = RawDocument::new();
        let reference = put(
            &mut document,
            &[records::PoseSchema::<records::V1> {
                name: Ref::default(),
                space: records::PoseSpace::Model,
                inverse: false.into(),
                storage: (),
            }],
        )
        .unwrap();
        let decoded = read(&document, reference, u32::MAX).unwrap();
        assert_eq!(
            decoded.iter().next().unwrap().1.representation,
            PoseRepresentation::Components
        );
        assert!(decoded.reference().is_none());
        assert!(read(&document, reference, 1).is_err());
    }
}
