//! Body relationships use world-local handles and relocate to the emitted body table.
use super::super::{materials::read_error, write::append, Graph};
use crate::model::conversion::Error;
use crate::model::formats::m3::{records, Container, Envelope, Ref};
use crate::model::profiles::m3::{values::ConstraintFlags, BodyId, Constraint, System};
use std::collections::BTreeMap;
fn body(index: u16, bodies: &[BodyId]) -> Result<BodyId, Error> {
    bodies.get(usize::from(index)).copied().ok_or_else(|| {
        Error::new(
            "m3.physics.constraint.body",
            "constraint references an undeclared body",
        )
    })
}
pub(super) fn read<E: Envelope>(
    document: &Container<'_, E>,
    reference: Ref<records::targets::PhysicsConstraint>,
    bodies: &[BodyId],
) -> Result<Vec<Constraint>, Error> {
    let mut result = Vec::new();
    for row in document
        .resolve::<records::PhysicsConstraint>(reference.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        let mut dependents = Vec::new();
        for index in document
            .resolve::<records::U16>(row.dependents.cast())
            .map_err(read_error)?
            .iter()
        {
            dependents.push(body(index.map_err(read_error)?.value, bodies)?);
        }
        result.push(Constraint {
            body_a: body(row.rigid_body_1, bodies)?,
            body_b: body(row.rigid_body_2, bodies)?,
            dependents,
            flags: ConstraintFlags(row.flags.bits()),
            break_force: row.break_force,
        });
    }
    Ok(result)
}
fn index(id: BodyId, indices: &BTreeMap<BodyId, u16>) -> Result<u16, Error> {
    indices.get(&id).copied().ok_or_else(|| {
        Error::new(
            "m3.physics.constraint.body",
            "constraint references a foreign body",
        )
    })
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Container<'_, E>,
) -> Result<Ref<records::targets::PhysicsConstraint>, Error> {
    let mut rows = Vec::new();
    let mut offset: usize = 0;
    for (_, system) in graph.systems() {
        let System::Physics(world) = system else {
            continue;
        };
        let indices = world
            .bodies
            .iter()
            .enumerate()
            .map(|(local, body)| {
                local
                    .checked_add(offset)
                    .and_then(|value| u16::try_from(value).ok())
                    .map(|index| (body.id(), index))
                    .ok_or_else(|| {
                        Error::new(
                            "m3.physics.constraint.body",
                            "native body identity exceeds 16-bit capacity",
                        )
                    })
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        for constraint in &world.constraints {
            let dependents = constraint
                .dependents
                .iter()
                .map(|id| index(*id, &indices).map(|value| records::U16 { value }))
                .collect::<Result<Vec<_>, _>>()?;
            rows.push(records::PhysicsConstraint {
                dependents: append(document, &dependents)?.cast(),
                rigid_body_1: index(constraint.body_a, &indices)?,
                rigid_body_2: index(constraint.body_b, &indices)?,
                flags: records::ConstraintFlags(constraint.flags.bits()),
                break_force: constraint.break_force,
            });
        }
        offset = offset.checked_add(world.bodies.len()).ok_or_else(|| {
            Error::new("m3.physics.bodies", "body count exceeds address capacity")
        })?;
    }
    Ok(append(document, &rows)?.cast())
}
