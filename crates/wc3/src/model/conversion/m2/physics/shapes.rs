//! Primitive and hull geometry referenced by native rigid bodies.
use super::*;
pub(super) fn shapes(
    source: &PhysicsFile<'_>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<Vec<wr::PhysicsShape<wr::V2>>, Error> {
    let mut definitions = Vec::new();
    for chunk in &source.chunks {
        match chunk {
            Pc::Shapes(values) => definitions.extend(values.iter()),
            Pc::Shapes2(values) => definitions.extend(values.iter().map(|v| &v.shape)),
            _ => {}
        }
    }
    let mut rows = Vec::new();
    for definition in definitions {
        let index = usize::try_from(definition.shape_index)
            .map_err(|_| Error::new("physics.shape", "negative geometry index"))?;
        let mut row = wr::PhysicsShape::<wr::V2> {
            transform: identity(),
            material: wr::PhysicsMaterial {
                density: definition.density,
                friction: definition.friction,
                restitution: definition.restitution,
            },
            game_flags: wr::WowShapeGameFlags::from_bits_retain(definition.game_flags.bits()),
            ..Default::default()
        };
        match definition.shape_type {
            PhysicsShapeKind::Box => {
                let value = source
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Boxes(values) = chunk {
                            Some(values)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| Error::new("physics.box", "missing box geometry"))?;
                row.kind = wr::PhysicsShapeKind::Box;
                row.transform = frame(&value.frame);
                row.half_extents = value.half_extents;
            }
            PhysicsShapeKind::Capsule => {
                let value = source
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Capsules(values) = chunk {
                            Some(values)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| Error::new("physics.capsule", "missing capsule geometry"))?;
                row.kind = wr::PhysicsShapeKind::Capsule;
                row.radius = value.radius;
                row.length = value
                    .position2
                    .into_iter()
                    .zip(value.position1)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f32>()
                    .sqrt();
                row.points = put(
                    doc,
                    &[
                        wr::Vector3 {
                            value: value.position1,
                        },
                        wr::Vector3 {
                            value: value.position2,
                        },
                    ],
                )?
                .cast();
            }
            PhysicsShapeKind::Sphere => {
                let value = source
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Spheres(values) = chunk {
                            Some(values)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| Error::new("physics.sphere", "missing sphere geometry"))?;
                row.kind = wr::PhysicsShapeKind::Sphere;
                row.transform = translate(value.position);
                row.radius = value.radius;
            }
            PhysicsShapeKind::ConvexHull => {
                let value = source
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Polytopes(values) = chunk {
                            Some(values)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| Error::new("physics.hull", "missing convex polytope"))?;
                row.kind = wr::PhysicsShapeKind::ConvexHull;
                row.points = put(
                    doc,
                    &value
                        .vertices
                        .iter()
                        .map(|value| wr::Vector3 { value })
                        .collect::<Vec<_>>(),
                )?
                .cast();
            }
            _ => {
                return Err(Error::new(
                    "physics.shape_type",
                    "unknown native geometry category",
                ));
            }
        };
        rows.push(row);
    }
    Ok(rows)
}
