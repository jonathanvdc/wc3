//! Typed Warcraft III export planning.
use super::*;
/// Plans a typed native model while enforcing loss and approximation policies.
pub fn plan<T: ModelDialect + 'static>(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<Plan<native::Model<'static, T>>, Error>
where
    T::Extension: for<'input> mdx::Extension<'input>,
{
    let mut context = Context::export(options);
    let semantic = asset.model(options)?;
    if semantic.unit_scale != 1. {
        context.unsupported("unit_scale","native model has no global unit scale; numeric geometry, physics, and effect rescaling needs an explicit policy")?;
    }
    if semantic.space != wr::CoordSpace::BLIZZARD {
        context.issue(
            "space",
            IssueKind::Normalized,
            "Authored numeric coordinates preserved; no basis transform applied",
        );
    }
    for (clip_index, clip) in semantic.clips.iter().enumerate() {
        for (index, layer) in clip
            .tracks
            .iter()
            .map(|track| &track.layer)
            .chain(clip.integer_tracks.iter().map(|track| match track {
                s::IntegerTrack::Unsigned(track) | s::IntegerTrack::Unsigned4(track) => {
                    &track.layer
                }
                s::IntegerTrack::Signed(track) => &track.layer,
                s::IntegerTrack::Unsigned16(track) => &track.layer,
                s::IntegerTrack::Bytes4(track) | s::IntegerTrack::Boolean(track) => &track.layer,
            }))
            .enumerate()
        {
            if layer.priority != 0 || layer.concurrent {
                context.unsupported(format!("clips[{clip_index}].tracks[{index}].layer"),"MDLX has one timeline per channel and cannot express prioritized or concurrent controllers")?;
            }
        }
        for (track_index, track) in clip.tracks.iter().enumerate() {
            if matches!(track.target, s::Target::NativeProperty { .. }) {
                context.unsupported(
                    format!("clips[{clip_index}].tracks[{track_index}]"),
                    "Native profile channel has no MDLX property mapping",
                )?;
            }
        }
    }

    for (index, node) in semantic.nodes.iter().enumerate() {
        if node.flags & wr::NodeFlags::HIDDEN.0 != 0 {
            context.unsupported(
                format!("nodes[{index}].hidden"),
                "MDLX does not store visibility on generic transform nodes",
            )?;
        }

        if node.inverse_bind.is_some() {
            context.unsupported(
                format!("nodes[{index}].inverse_bind"),
                "An arbitrary inverse bind matrix needs an explicit MDLX bind-pose convention",
            )?;
        }
    }

    let root = asset.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let raw = asset.document.raw();
    let section = raw
        .referenced_section(root.models)?
        .ok_or_else(|| Error::new("model", "missing model section"))?;
    macro_rules! check_physics {
        ($v:ty) => {{
            let graph = raw
                .resolve::<wr::ModelRecord<$v>>(root.models.cast())?
                .get(options.model as usize)
                .transpose()?
                .ok_or_else(|| Error::new("model", "missing selected model"))?;
            if !graph.physics.bodies.is_empty()
                || !graph.physics.joints.is_empty()
                || !graph.physics.colliders.is_empty()
                || !graph.physics.cloths.is_empty()
                || !graph.physics.rigs.is_empty()
            {
                context.unsupported(
                    "physics",
                    "MDLX cannot express rigid simulations, constraints, or cloth authoring",
                )?;
            }
        }};
    }
    match section.revision() {
        8 => check_physics!(wr::V8),
        9 => {
            check_physics!(wr::V9);
            let graph = raw
                .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
                .get(options.model as usize)
                .transpose()?
                .ok_or_else(|| Error::new("model", "missing selected model"))?;
            if !graph.physics.soft.is_empty() {
                context.unsupported("physics.soft", "MDLX cannot express soft-body simulation")?;
            }
        }
        _ => {}
    }
    let (active_nodes, active_materials) = active_graph(asset, options)?;
    let starts = clip_starts(&semantic, &mut context)?;
    if !is_wc3(semantic.profile) && semantic.profile != ProfileId::GENERIC {
        for (index, material) in semantic.materials.iter().enumerate() {
            if material.flags != 0 {
                context.unsupported(
                    format!("materials[{index}].flags"),
                    "Native profile render flags have no equivalent MDLX meaning",
                )?;
            }
        }
    }
    let mut baseline = native::Model::<T>::new();
    let ids = native_ids(&active_nodes, asset.document.raw())?;
    let mut updates = BTreeMap::new();
    let mut globals = native_globals(asset, options)?;
    for (index, node) in semantic.nodes.iter().enumerate() {
        let mut native_node = native::Node::new(&node.name, ids[index])
            .map_err(|error| failure(format!("nodes[{index}].name"), error))?;
        native_node.parent_id = node
            .parent
            .map(|parent| {
                ids.get(parent as usize).copied().ok_or_else(|| {
                    Error::new(
                        format!("nodes[{index}].parent"),
                        "parent index out of bounds",
                    )
                })
            })
            .transpose()?
            .unwrap_or(u32::MAX);
        let values = metadata(asset.document.raw(), active_nodes[index].native.clone())?;
        let original = values
            .get("mdlx.node_flags")
            .map_or(0, |(value, _)| *value as u32);
        native_node.flags = native::scene::NodeFlags(native_flags(node.flags, original));
        if node.translation != [0.; 3] {
            native_node.translation = Some(Track::constant(node.translation));
        }
        if node.rotation != [0., 0., 0., 1.] {
            native_node.rotation = Some(Track::constant(node.rotation));
        }
        if node.scale != [1.; 3] {
            native_node.scaling = Some(Track::constant(node.scale));
        }
        let mut trans = None;
        let mut rot = None;
        let mut scale = None;
        for (clip_index, clip) in semantic.clips.iter().enumerate() {
            for (track_index, track) in clip.tracks.iter().enumerate() {
                let path = format!("clips[{clip_index}].tracks[{track_index}]");
                let start = starts[clip_index] as f32 / 1000.;
                match track.target {
                    s::Target::Translation(owner) if owner == index as u32 => {
                        let value =
                            native_track::<3>(track, start, &mut globals, &mut context, &path)?;
                        merge_track(&mut trans, value, &path)?;
                    }
                    s::Target::Rotation(owner) if owner == index as u32 => {
                        let value =
                            native_track::<4>(track, start, &mut globals, &mut context, &path)?;
                        merge_track(&mut rot, value, &path)?;
                    }
                    s::Target::Scale(owner) if owner == index as u32 => {
                        let value =
                            native_track::<3>(track, start, &mut globals, &mut context, &path)?;
                        merge_track(&mut scale, value, &path)?;
                    }
                    _ => {}
                }
            }
        }
        native_node.translation = trans.or(native_node.translation);
        native_node.rotation = rot.or(native_node.rotation);
        native_node.scaling = scale.or(native_node.scaling);
        updates.insert(ids[index], native_node);
    }
    patch_hierarchy(&mut baseline, &updates);
    let existing: Vec<_> = baseline.nodes().iter().map(|node| node.object_id).collect();
    let mut bones: Vec<_> = baseline.bones().cloned().collect();
    let mut helpers: Vec<_> = baseline.helpers().cloned().collect();
    for (index, node) in semantic.nodes.iter().enumerate() {
        if !existing.contains(&ids[index]) {
            let native_node = updates[&ids[index]].clone();
            if node.bone {
                bones.push(native::Bone::new(native_node, u32::MAX, u32::MAX));
            } else {
                helpers.push(native_node);
            }
        }
    }
    baseline.set_bones(bones);
    baseline.set_helpers(helpers);
    let max = ids
        .iter()
        .copied()
        .max()
        .map(|id| id as usize + 1)
        .unwrap_or(0);
    if max > 1_000_000 {
        return Err(Error::new(
            "nodes",
            "sparse node identities exceed pivot allocation limit",
        ));
    }
    let mut pivots = vec![[0.; 3]; max];
    for (index, node) in semantic.nodes.iter().enumerate() {
        pivots[ids[index] as usize] = node.pivot;
    }
    baseline.set_pivot_points(&pivots);
    baseline.set_global_sequences(&globals);
    let mut info = baseline.model_info().unwrap_or(
        native::ModelInfo::new(&semantic.name).map_err(|error| failure("model.name", error))?,
    );
    info.name
        .set_text(&semantic.name)
        .map_err(|error| failure("model.name", error))?;
    info.minimum_extent = semantic.bounds.minimum;
    info.maximum_extent = semantic.bounds.maximum;
    info.bounds_radius = semantic.bounds.sphere_radius;
    baseline.set_model_info(&info);
    let mut meshes = Vec::new();
    for (index, mesh) in semantic.meshes.iter().enumerate() {
        let path = format!("meshes[{index}]");
        if mesh.positions.len() > 65536 {
            return Err(Error::new(
                &path,
                "MDLX geoset vertex count exceeds 16-bit index range",
            ));
        }
        let normals = if mesh.normals.is_empty() {
            context.issue(
                format!("{path}.normals"),
                IssueKind::Initialized,
                "Missing normals initialized to zero",
            );
            vec![[0.; 3]; mesh.positions.len()]
        } else {
            mesh.normals.to_vec()
        };
        let faces = mesh
            .triangles
            .iter()
            .flat_map(|face| face.iter())
            .map(|index| {
                u16::try_from(*index)
                    .map_err(|_| Error::new(&path, "triangle index exceeds 16-bit range"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut geoset = native::Geoset::<T::Version>::new(mesh.positions.to_vec(), normals, faces)
            .map_err(|error| failure(&path, error))?;
        geoset.material_id = mesh.material_slot;
        geoset.selection_group = mesh.selection_group;
        geoset.extent = native_extent(mesh.bounds.clone());
        if let Err(error) = geoset.try_set_level_of_detail(mesh.lod) {
            if mesh.lod != 0 {
                context.unsupported(format!("{path}.lod"), error.to_string())?;
            }
        }
        if let Err(error) = geoset.try_set_name(&mesh.name) {
            if !mesh.name.is_empty() {
                context.unsupported(format!("{path}.name"), error.to_string())?;
            }
        }
        if !mesh.uv_sets.is_empty() {
            geoset.set_uv_sets(
                mesh.uv_sets
                    .iter()
                    .map(|values| values.iter().copied().collect())
                    .collect(),
            );
        }
        if !mesh.tangents.is_empty() {
            if let Err(error) =
                geoset.try_set_tangents(Some(mesh.tangents.iter().copied().collect()))
            {
                context.unsupported(format!("{path}.tangents"), error.to_string())?;
            }
        }
        if !mesh.colors.is_empty() {
            context.unsupported(
                format!("{path}.colors"),
                "MDLX geometry has no vertex-color attribute",
            )?;
        }
        if !mesh.influences.is_empty() {
            if mesh.influences.len() != mesh.positions.len() {
                return Err(Error::new(
                    format!("{path}.skin"),
                    "influence count differs from vertex count",
                ));
            }
            let mut groups = Vec::<Vec<u32>>::new();
            let mut group_ids = Vec::new();
            let mut weights = Vec::new();
            let mut all_equal = true;
            for (vertex, row) in mesh.influences.iter().enumerate() {
                if row.len() > 4 {
                    context.unsupported(
                        format!("{path}.skin[{vertex}]"),
                        "MDLX weighted skin supports four influences",
                    )?;
                }
                let row = &row[..row.len().min(4)];
                let mut group = Vec::new();
                let mut bone_indices = [0u16; 4];
                let mut quantized = [0u16; 4];
                for (slot, influence) in row.iter().enumerate() {
                    let node = *ids.get(influence.node as usize).ok_or_else(|| {
                        Error::new(
                            format!("{path}.skin[{vertex}]"),
                            "bone index is out of bounds",
                        )
                    })?;
                    group.push(node);
                    bone_indices[slot] = u16::try_from(node).map_err(|_| {
                        Error::new(&path, "skin node identity exceeds 16-bit range")
                    })?;
                    let value = (influence.weight * 255.).round();
                    if !value.is_finite() || !(0.0..=255.).contains(&value) {
                        return Err(Error::new(&path, "invalid skin weight"));
                    }
                    quantized[slot] = value as u16;
                    if (influence.weight - 1. / row.len() as f32).abs() > 1e-6 {
                        all_equal = false;
                    }
                }
                let group_id = groups
                    .iter()
                    .position(|existing| existing == &group)
                    .unwrap_or_else(|| {
                        groups.push(group);
                        groups.len() - 1
                    });
                group_ids
                    .push(u8::try_from(group_id).map_err(|_| {
                        Error::new(&path, "classic matrix group count exceeds 256")
                    })?);
                weights.push(native::geometry::SkinWeights {
                    bone_indices,
                    weights: quantized,
                });
            }
            geoset
                .set_matrix_groups(&groups)
                .map_err(|error| failure(&path, error))?;
            geoset
                .set_vertex_groups(&group_ids)
                .map_err(|error| failure(&path, error))?;
            if T::Version::NUMBER >= 900 && !all_equal {
                for (vertex, (row, weights)) in mesh.influences.iter().zip(&weights).enumerate() {
                    for (slot, influence) in row.iter().take(4).enumerate() {
                        context.approximate(
                            format!("{path}.skin[{vertex}][{slot}]"),
                            (influence.weight - f32::from(weights.weights[slot]) / 255.).abs(),
                            "Skin weight quantized to 1/255",
                        )?;
                    }
                }
                let rows=weights.into_iter().map(|row|<T::Version as native::geometry::GeosetLayout>::SkinWeights::from_influences(row.bone_indices,row.weights).ok_or_else(||Error::new(format!("{path}.skin"),"bone identity exceeds target skin index range"))).collect::<Result<Vec<_>,_>>()?;
                geoset
                    .try_set_skin_weights(Some(rows.into_iter().collect()))
                    .map_err(|error| failure(&path, error))?;
            } else if T::Version::NUMBER < 900 && !all_equal {
                context.unsupported(
                    format!("{path}.skin"),
                    "Classic MDLX supports only equal-weight matrix groups",
                )?;
            }
        }
        meshes.push(geoset);
    }
    baseline.set_geosets(meshes);
    overlay_effects(
        &mut baseline,
        &active_nodes,
        &ids,
        &updates,
        asset.document.raw(),
        &semantic,
        &mut context,
    )?;
    let mut textures = Vec::new();
    for (index, texture) in semantic.textures.iter().enumerate() {
        if texture.id.is_some() {
            context.unsupported(
                format!("textures[{index}].id"),
                "MDLX textures require filenames or replaceable categories",
            )?;
        }
        let mut value = native::Texture::new(&texture.path)
            .map_err(|error| failure(format!("textures[{index}]"), error))?;
        value.replaceable_id = texture.replaceable;
        value.flags = native::TextureFlags(texture.flags);
        textures.push(value);
    }
    baseline.set_textures(textures);
    let mut materials = Vec::new();
    for (index, material) in semantic.materials.iter().enumerate() {
        let mut value = native::Material::<T::Version>::new();
        value.layers.push(native::Layer::new());
        if let Some((record, _)) = active_materials.get(index) {
            overlay_material(&mut value, record, asset.document.raw(), &mut context)?;
        }
        if active_materials
            .get(index)
            .is_none_or(|(_, sync)| *sync == wr::NativeSync::COMMON_EDITED)
        {
            value.render_mode = native::MaterialRenderFlags(material.flags);
            if let Some(layer) = value.layers.first_mut() {
                layer.filter_mode = match material.blend {
                    0 => native::LayerFilterMode::None,
                    1 => native::LayerFilterMode::Transparent,
                    2 => native::LayerFilterMode::Blend,
                    3 => native::LayerFilterMode::Additive,
                    4 => native::LayerFilterMode::AddAlpha,
                    5 => native::LayerFilterMode::Modulate,
                    6 => native::LayerFilterMode::Modulate2x,
                    value => native::LayerFilterMode::Unknown(value),
                };
                layer.texture_id.set_value(material.texture.unwrap_or(0));
            }
        }
        materials.push(value);
    }
    baseline.set_materials(materials);
    let mut sequences = Vec::new();
    for (index, clip) in semantic.clips.iter().enumerate() {
        let duration = clip.duration * 1000.;
        if !duration.is_finite() || duration < 0. || duration > u32::MAX as f32 {
            return Err(Error::new(
                format!("clips[{index}].duration"),
                "invalid clip duration",
            ));
        }
        let end = starts[index]
            .checked_add(duration.round() as u32)
            .ok_or_else(|| Error::new(format!("clips[{index}]"), "clip interval overflow"))?;
        context.approximate(
            format!("clips[{index}].duration"),
            (duration - duration.round()).abs() / 1000.,
            "Clip duration rounded to milliseconds",
        )?;
        let mut value = native::Sequence::new(&clip.name, [starts[index], end])
            .map_err(|error| failure(format!("clips[{index}]"), error))?;
        value.flags = native::SequenceFlags(clip.flags);
        value.extent = native_extent(semantic.bounds.clone());
        sequences.push(value);
    }
    baseline.set_sequences(sequences);
    overlay_metadata(
        &mut baseline,
        asset,
        options,
        &active_nodes,
        semantic
            .native
            .as_ref()
            .map_or(&[][..], |native| native.mesh_projections.as_slice()),
        &ids,
        &mut context,
    )?;
    overlay_properties(
        &mut baseline,
        &semantic,
        &active_nodes,
        &ids,
        &starts,
        &mut globals,
        &mut context,
    )?;
    overlay_integer(
        &mut baseline,
        &semantic,
        &ids,
        &starts,
        &mut globals,
        &mut context,
    )?;
    baseline.set_global_sequences(&globals);
    Ok(Plan::new(baseline, context.report))
}
