use super::super::super::codec::{lookup, mapped, read_vectors, read_words, vectors, words};
use super::super::leaf::{real_records, reals};
use super::{ReadContext, WriteContext};
use crate::model::conversion::Error;
use crate::model::semantic::{
    profiles::wem::physics::soft::{Impulse, Jiggle, JigglePreset, SurfacePoint},
    Values,
};
use crate::model::wem::{records, Bool};
pub(super) fn read<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::SoftJiggle,
) -> Result<Jiggle<'a>, Error> {
    let ids = read_words(context.document, value.kick_ids)?;
    let meshes = read_words(context.document, value.kick_mesh)?;
    let faces = read_words(context.document, value.kick_face)?;
    let weights = read_vectors(context.document, value.kick_weights)?;
    let directions = read_vectors(context.document, value.kick_direction)?;
    let strengths = reals(context.document, value.kick_strength)?;
    let len = ids.len();
    if [meshes.len(), faces.len(), weights.len(), strengths.len()]
        .into_iter()
        .any(|n| n != len)
        || directions.len() > len
    {
        return Err(Error::new(
            "wem.soft.jiggle.impulses",
            "parallel impulse streams have incompatible lengths",
        ));
    }
    for id in meshes.iter() {
        lookup(context.meshes, id, "soft impulse mesh")?;
    }
    let targets = context.meshes.to_vec();
    let impulses = Values::view(len, move |index| Impulse {
        id: ids.get(index).expect("checked impulse"),
        place: SurfacePoint {
            mesh: targets[meshes.get(index).expect("checked impulse") as usize],
            face: faces.get(index).expect("checked impulse"),
            weights: weights.get(index).expect("checked impulse"),
            radius: 0.,
        },
        direction: directions.get(index),
        strength: strengths.get(index).expect("checked impulse"),
    });
    Ok(Jiggle {
        preset: JigglePreset::from_raw(value.preset.raw()),
        looseness: value.looseness,
        bounce: value.bounce,
        weight: value.weight,
        hold_shape: value.hold_shape,
        keep_volume: value.keep_volume,
        limit: value.limit,
        keep_in_place: value.keep_in_place.get(),
        meets_floor: value.meets_floor.get(),
        world_forces: value.world_forces.get(),
        impulses,
    })
}
pub(super) fn write(
    context: &mut WriteContext<'_, '_>,
    value: &Jiggle<'_>,
) -> Result<records::SoftJiggle, Error> {
    let impulses: Vec<_> = value.impulses.iter().collect();
    let mesh_ids = impulses
        .iter()
        .map(|v| mapped(context.meshes, v.place.mesh, "soft impulse mesh"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut absent = false;
    let mut direction = Vec::new();
    for impulse in &impulses {
        match impulse.direction {
            Some(value) if !absent => direction.push(value),
            Some(_) => {
                return Err(Error::new(
                    "wem.soft.jiggle.direction",
                    "omitted directions must form a trailing suffix",
                ));
            }
            None => absent = true,
        }
        if impulse.place.radius != 0. {
            return Err(Error::new(
                "wem.soft.jiggle.radius",
                "impulse attachments have no radius field",
            ));
        }
    }
    Ok(records::SoftJiggle {
        preset: records::SoftJigglePreset::from_raw(value.preset.raw()),
        looseness: value.looseness,
        bounce: value.bounce,
        weight: value.weight,
        hold_shape: value.hold_shape,
        keep_volume: value.keep_volume,
        limit: value.limit,
        keep_in_place: Bool::from(value.keep_in_place),
        meets_floor: Bool::from(value.meets_floor),
        world_forces: Bool::from(value.world_forces),
        kick_ids: words(context.document, impulses.iter().map(|v| v.id))?,
        kick_mesh: words(context.document, mesh_ids)?,
        kick_face: words(context.document, impulses.iter().map(|v| v.place.face))?,
        kick_weights: vectors(
            context.document,
            &Values::owned(impulses.iter().map(|v| v.place.weights).collect::<Vec<_>>()),
        )?,
        kick_direction: vectors(context.document, &Values::owned(direction))?,
        kick_strength: real_records(context.document, impulses.iter().map(|v| v.strength))?,
    })
}
