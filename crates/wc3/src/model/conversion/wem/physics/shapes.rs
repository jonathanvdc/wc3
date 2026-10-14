//! Collision geometry is borrowed independently of native record wrappers.
use super::codec::{
    append, read_error, read_vectors, read_words, revision, vectors, words, Scalar,
};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::m2::values::PhysicsGameFlags;
use crate::model::semantic::profiles::wem::physics::{Physics, Shape, ShapeId, ShapeKind};
use crate::model::semantic::scene::ContactMaterial;
use crate::model::semantic::Values;
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::BTreeMap;
use zerovec::ZeroVec;
pub(super) type Cache = BTreeMap<(u32, usize), ShapeId>;
pub(super) fn read<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::PhysicsShape>,
    target: &mut Physics<'a>,
    cache: &mut Cache,
) -> Result<Vec<ShapeId>, Error> {
    if reference.entries == 0 {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    macro_rules! rows {
        ($revision:ty) => {{
            for (ordinal, row) in document
                .resolve::<records::PhysicsShape<$revision>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let row = row.map_err(read_error)?;
                let id = if let Some(id) = cache.get(&(reference.index, ordinal)) {
                    *id
                } else {
                    let id = target.add_shape(shape(document, &row)?);
                    cache.insert((reference.index, ordinal), id);
                    id
                };
                ids.push(id);
            }
        }};
    }
    match revision(document, reference)? {
        Some(1) => rows!(records::V1),
        Some(2) => rows!(records::V2),
        _ => {
            return Err(Error::new(
                "wem.physics.shape",
                "unsupported collider record revision",
            ));
        }
    }
    Ok(ids)
}
fn shape<'a, D: records::PhysicsShapeDialect>(
    document: &'a RawDocument<'_>,
    value: &records::PhysicsShape<D>,
) -> Result<Shape<'a>, Error>
where
    D::GameFlags: Scalar<records::WowShapeGameFlags>,
{
    let words = read_words(document, value.triangles)?;
    if words.len() % 3 != 0 {
        return Err(Error::new(
            "wem.physics.shape",
            "triangle index count is not a multiple of three",
        ));
    }
    let triangles = if let Some(bytes) = words.borrowed_canonical_bytes() {
        Values::from_zero_vec(ZeroVec::<[u32; 3]>::parse_bytes(bytes).map_err(read_error)?)
    } else {
        Values::view(words.len() / 3, move |index| {
            [
                words.get(index * 3).unwrap(),
                words.get(index * 3 + 1).unwrap(),
                words.get(index * 3 + 2).unwrap(),
            ]
        })
    };
    Ok(Shape {
        kind: match value.kind {
            records::PhysicsShapeKind::Box => ShapeKind::Box,
            records::PhysicsShapeKind::Sphere => ShapeKind::Sphere,
            records::PhysicsShapeKind::Capsule => ShapeKind::Capsule,
            records::PhysicsShapeKind::Cylinder => ShapeKind::Cylinder,
            records::PhysicsShapeKind::ConvexHull => ShapeKind::ConvexHull,
            records::PhysicsShapeKind::TriangleMesh => ShapeKind::TriangleMesh,
            records::PhysicsShapeKind::Count => ShapeKind::Count,
            records::PhysicsShapeKind::Unknown(code) => ShapeKind::Unknown(code),
        },
        transform: value.transform,
        half_extents: value.half_extents,
        radius: value.radius,
        length: value.length,
        points: read_vectors(document, value.points)?,
        vertices: read_vectors(document, value.vertices)?,
        triangles,
        material: ContactMaterial {
            density: value.material.density,
            friction: value.material.friction,
            restitution: value.material.restitution,
        },
        contact_flags: PhysicsGameFlags::from_bits_retain(
            Scalar::scalar(&value.game_flags).map_or(0, |flags| flags.bits()),
        ),
    })
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    value: &Physics<'_>,
    ids: &[ShapeId],
) -> Result<Ref<records::targets::PhysicsShape>, Error> {
    let mut rows = Vec::new();
    for id in ids {
        let value = value.shape(*id).ok_or_else(|| {
            Error::new(
                "wem.physics.shape",
                "collider belongs to another simulation domain",
            )
        })?;
        rows.push(records::PhysicsShape::<records::V2> {
            kind: match value.kind {
                ShapeKind::Box => records::PhysicsShapeKind::Box,
                ShapeKind::Sphere => records::PhysicsShapeKind::Sphere,
                ShapeKind::Capsule => records::PhysicsShapeKind::Capsule,
                ShapeKind::Cylinder => records::PhysicsShapeKind::Cylinder,
                ShapeKind::ConvexHull => records::PhysicsShapeKind::ConvexHull,
                ShapeKind::TriangleMesh => records::PhysicsShapeKind::TriangleMesh,
                ShapeKind::Count => records::PhysicsShapeKind::Count,
                ShapeKind::Unknown(code) => records::PhysicsShapeKind::Unknown(code),
            },
            transform: value.transform,
            half_extents: value.half_extents,
            radius: value.radius,
            length: value.length,
            points: vectors(document, &value.points)?,
            vertices: vectors(document, &value.vertices)?,
            triangles: words(document, value.triangles.iter().flatten())?,
            material: records::PhysicsMaterial {
                density: value.material.density,
                friction: value.material.friction,
                restitution: value.material.restitution,
            },
            game_flags: records::WowShapeGameFlags::from_bits_retain(value.contact_flags.bits()),
        });
    }
    Ok(append(document, &rows)?.cast())
}

#[cfg(test)]
mod tests {
    use super::super::codec::append;
    use super::*;
    #[test]
    fn shared_prefix_references_keep_one_editable_shape_identity() {
        let mut document = RawDocument::new();
        let reference = append(
            &mut document,
            &[
                records::PhysicsShape::<records::V2>::default(),
                records::PhysicsShape::<records::V2>::default(),
            ],
        )
        .unwrap();
        let mut physics = Physics::default();
        let mut cache = Cache::new();
        let both = read(&document, reference.cast(), &mut physics, &mut cache).unwrap();
        let mut prefix = reference.cast();
        prefix.entries = 1;
        let first = read(&document, prefix, &mut physics, &mut cache).unwrap();
        assert_eq!(first[0], both[0]);
        assert_eq!(physics.shapes().len(), 2);
        let mut shape = physics.shape(first[0]).unwrap().clone();
        shape.radius = 7.;
        physics.set_shape(first[0], shape).unwrap();
        let mut output = RawDocument::new();
        let written = write(&mut output, &physics, &both).unwrap();
        let values = output
            .resolve::<records::PhysicsShape<records::V2>>(written.cast())
            .unwrap();
        assert_eq!(values.get(0).unwrap().unwrap().radius, 7.);
    }
}
