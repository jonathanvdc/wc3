use super::*;
/// Checks geometry dimensions, hierarchy, skin references, and key payloads.
pub fn validate_model(model: &Model<'_>) -> Result<(), ReadError> {
    if !model.unit_scale.is_finite() || model.unit_scale <= 0. {
        return Err(ReadError::invalid(
            0,
            "unit scale must be finite and positive",
        ));
    }
    for (i, node) in model.nodes.iter().enumerate() {
        if node
            .parent
            .is_some_and(|p| p as usize >= model.nodes.len() || p as usize == i)
        {
            return Err(ReadError::invalid(0, "node parent"));
        }
        if node
            .pivot
            .iter()
            .chain(&node.translation)
            .chain(&node.rotation)
            .chain(&node.scale)
            .any(|v| !v.is_finite())
            || node
                .inverse_bind
                .as_ref()
                .is_some_and(|m| m.iter().flatten().any(|v| !v.is_finite()))
        {
            return Err(ReadError::invalid(0, "node transform"));
        }
        let mut current = node.parent;
        let mut depth = 0;
        while let Some(parent) = current {
            depth += 1;
            if depth > model.nodes.len() {
                return Err(ReadError::invalid(0, "node hierarchy cycle"));
            }
            current = model
                .nodes
                .get(parent as usize)
                .ok_or_else(|| ReadError::invalid(0, "node ancestor"))?
                .parent;
        }
    }
    for mesh in &model.meshes {
        let count = mesh.positions.len();
        if mesh
            .positions
            .iter()
            .flatten()
            .chain(mesh.normals.iter().flatten())
            .chain(mesh.tangents.iter().flatten())
            .chain(mesh.colors.iter().flatten())
            .chain(mesh.uv_sets.iter().flat_map(|u| u.iter().flatten()))
            .any(|v| !v.is_finite())
        {
            return Err(ReadError::invalid(0, "vertex attribute value"));
        }
        if [mesh.normals.len(), mesh.tangents.len(), mesh.colors.len()]
            .into_iter()
            .chain(mesh.uv_sets.iter().map(|u| u.len()))
            .any(|n| n != 0 && n != count)
        {
            return Err(ReadError::invalid(0, "vertex attribute count"));
        }
        if mesh
            .triangles
            .iter()
            .flatten()
            .any(|&v| v as usize >= count)
        {
            return Err(ReadError::invalid(0, "triangle index"));
        }
        if !mesh.influences.is_empty() && mesh.influences.len() != count {
            return Err(ReadError::invalid(0, "skin vertex count"));
        }
        for influences in &mesh.influences {
            if influences.iter().any(|i| {
                i.node as usize >= model.nodes.len() || !i.weight.is_finite() || i.weight < 0.
            }) {
                return Err(ReadError::invalid(0, "skin influence"));
            }
            if !influences.is_empty()
                && (influences.iter().map(|i| i.weight).sum::<f32>() - 1.).abs() > 0.0001
            {
                return Err(ReadError::invalid(0, "skin weight sum"));
            }
        }
        if mesh.material_slot != u32::MAX
            && !model.materials.is_empty()
            && mesh.material_slot as usize >= model.materials.len()
        {
            return Err(ReadError::invalid(0, "material slot"));
        }
    }
    for material in &model.materials {
        if material
            .texture
            .is_some_and(|t| t as usize >= model.textures.len())
        {
            return Err(ReadError::invalid(0, "material texture"));
        }
    }
    for clip in &model.clips {
        if !clip.duration.is_finite() || clip.duration < 0. {
            return Err(ReadError::invalid(0, "clip duration"));
        }
        for track in &clip.integer_tracks {
            integer::validate(track)?;
        }
        for track in &clip.tracks {
            if track
                .global_duration
                .is_some_and(|d| !d.is_finite() || d <= 0.)
            {
                return Err(ReadError::invalid(0, "global animation duration"));
            }
            let (node, width) = match track.target {
                Target::Translation(n) | Target::Scale(n) => (n, Some(3)),
                Target::Rotation(n) => (n, Some(4)),
                Target::Property { owner, .. } | Target::NativeProperty { owner, .. } => {
                    (owner, None)
                }
            };
            if width.is_some() && node as usize >= model.nodes.len() {
                return Err(ReadError::invalid(0, "track node"));
            }
            let key_width = width.or_else(|| track.keys.first().map(|k| k.value.len()));
            if key_width.is_some_and(|w| !(1..=4).contains(&w)) {
                return Err(ReadError::invalid(0, "animation component count"));
            }
            let mut last = f32::NEG_INFINITY;
            for key in &track.keys {
                if !key.time.is_finite()
                    || key.time < last
                    || key_width.is_some_and(|w| w != key.value.len())
                    || key.value.iter().any(|v| !v.is_finite())
                {
                    return Err(ReadError::invalid(0, "animation key"));
                }
                last = key.time;
                if matches!(
                    track.interpolation,
                    Interpolation::Hermite | Interpolation::Bezier
                ) && [&key.incoming, &key.outgoing].iter().any(|t| {
                    t.as_ref().is_none_or(|v| {
                        v.len() != key.value.len() || v.iter().any(|v| !v.is_finite())
                    })
                }) {
                    return Err(ReadError::invalid(0, "spline tangent"));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_ancestor_is_an_error_without_indexing_out_of_bounds() {
        let model = Model {
            nodes: vec![
                Node {
                    parent: Some(1),
                    ..Default::default()
                },
                Node {
                    parent: Some(9),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert!(validate_model(&model).is_err());
    }
    #[test]
    fn property_keys_require_a_consistent_payload_width() {
        let model = Model {
            clips: vec![Clip {
                duration: 1.,
                tracks: vec![Track {
                    layer: TrackLayer::default(),
                    target: Target::NativeProperty { owner: 0, id: 17 },
                    interpolation: Interpolation::Linear,
                    global_duration: None,
                    keys: vec![
                        Key {
                            time: 0.,
                            value: vec![1.],
                            incoming: None,
                            outgoing: None,
                        },
                        Key {
                            time: 1.,
                            value: vec![1., 2.],
                            incoming: None,
                            outgoing: None,
                        },
                    ],
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(validate_model(&model).is_err());
    }
}
