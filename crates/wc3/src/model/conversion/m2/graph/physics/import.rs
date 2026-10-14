//! Imports native rigid-body documents without an intermediate model encoding.
use super::super::overrides::resource;
use super::*;
use profile::{
    values::{PhysicsBodyKind, PhysicsKind},
    Body, Physics, Vegetation,
};
pub(in super::super) fn import<'a, 'data: 'a, D: Dialect>(
    source: &'a Bundle<'data, D>,
    nodes: &[NodeId],
    builder: &mut GraphBuilder<'a>,
    _context: &mut Context,
) -> Result<Vec<Physics<'a>>, Error> {
    let mut documents: Vec<_> = source
        .files()
        .filter_map(|(key, file)| {
            if let File::Physics(file) = file {
                Some((Some(key), file))
            } else {
                None
            }
        })
        .collect();
    for chunk in source.model.extensions() {
        if let Chunk::Physics(file) = chunk {
            documents.push((None, file));
        }
    }
    let mut output = Vec::with_capacity(documents.len());
    for (key, file) in documents {
        file.validate_with_bone_count(nodes.len())
            .map_err(failure)?;
        let shapes = shapes::shapes(file)?;
        let native_bodies = bodies::bodies(file);
        let mut placements = Vec::new();
        for (index, body) in native_bodies.iter().enumerate() {
            let bone = *nodes
                .get(body.bone as usize)
                .ok_or_else(|| Error::new("physics.body.bone", "bone is absent"))?;
            let node = Node::named(format!("physics-body-{index}"))
                .with_parent(bone)
                .with_transform(Transform {
                    translation: body.position,
                    ..Transform::default()
                })?;
            placements.push(builder.add_node(node)?);
        }
        let mut bodies = Vec::new();
        for (index, body) in native_bodies.iter().enumerate() {
            let first = usize::try_from(body.first)
                .map_err(|_| Error::new("physics.body.shapes", "negative first shape"))?;
            let count = usize::try_from(body.count)
                .map_err(|_| Error::new("physics.body.shapes", "negative shape count"))?;
            let end = first
                .checked_add(count)
                .ok_or_else(|| Error::new("physics.body.shapes", "shape range overflow"))?;
            let selected = shapes
                .get(first..end)
                .ok_or_else(|| Error::new("physics.body.shapes", "shape range is absent"))?;
            let parent_body = if body.attachment.parent_body() == 0 {
                None
            } else {
                Some(
                    *placements
                        .get(body.attachment.parent_body() as usize)
                        .ok_or_else(|| {
                            Error::new("physics.body.parent", "parent body is absent")
                        })?,
                )
            };
            bodies.push(Body {
                node: placements[index],
                kind: PhysicsBodyKind::from_raw(body.body_type.raw()),
                gravity_scale: body.gravity,
                inertia_scale: body.inertia,
                linear_damping: body.linear,
                angular_damping: body.angular,
                follow_factor: body.follow,
                parent_body,
                attachment_host: body.attachment.attachment_host(),
                ragdoll_root: body.attachment.ragdoll_root(),
                shapes: selected.to_vec(),
            });
        }
        let kind = file
            .chunks
            .iter()
            .find_map(|chunk| {
                if let Pc::Kind(kind) = chunk {
                    Some(PhysicsKind::from_raw(kind.raw()))
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let follow_allow_list = file
            .chunks
            .iter()
            .find_map(|chunk| {
                if let Pc::AllowList { keys, .. } = chunk {
                    Some(Values::view(keys.len(), move |i| keys.get(i).unwrap()))
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let vegetation = file
            .chunks
            .iter()
            .filter_map(|chunk| {
                if let Pc::Vegetation(rows) = chunk {
                    Some(rows)
                } else {
                    None
                }
            })
            .flatten()
            .map(|v| Vegetation {
                position_max_push: v.position_max_push,
                position_push_amount: v.position_push_amount,
                position_relax_speed: v.position_relax_speed,
                velocity_max_push: v.velocity_max_push,
                velocity_speed: v.velocity_speed,
                minimum_push_distance_squared: v.minimum_push_distance_squared,
            })
            .collect();
        output.push(Physics {
            kind,
            resource: key.map(resource),
            bodies,
            joints: joints::joints(file, &placements)?,
            collision_pairs: Vec::new(),
            wind_coefficients: None,
            follow_allow_list,
            vegetation,
        });
    }
    Ok(output)
}
