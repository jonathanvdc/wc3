//! Native cage bindings and particle/solver settings mapped to active WEM records.
use super::*;
pub(in crate::model::conversion::m3) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
    context: &mut Context,
) -> Result<wem::Document<'static>, Error> {
    let reference: Ref<nr::targets::ClothPhysics> =
        native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| {
            ClothReference::cloth_reference(&r.cloth_physics)
        });
    let rows = decode::rows(source.document(), reference, context)?;
    if rows.is_empty() {
        return Ok(document);
    }
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let mut raw = document.into_raw();
    let mut graph = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let meshes = raw
        .resolve::<wr::Mesh<wr::V2>>(graph.meshes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let region = |raw: &wem::RawDocument<'_>, index: u32| -> Result<u32, Error> {
        let name = format!("LOD 0 region {index}");
        meshes
            .iter()
            .enumerate()
            .find_map(|(index, mesh)| {
                (raw.string(mesh.name).ok().flatten() == Some(name.as_str()))
                    .then_some(index as u32)
            })
            .ok_or_else(|| Error::new("cloth.region", format!("missing cage/proxy region {index}")))
    };
    let mut nodes = raw
        .resolve::<wr::Node<wr::V19>>(graph.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut cloths = raw
        .resolve::<wr::Cloth<wr::V2>>(graph.physics.cloths.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut colliders = raw
        .resolve::<wr::ClothCollider<wr::V2>>(graph.physics.colliders.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for SourceCloth {
        record: r,
        mut values,
    } in rows
    {
        let id = graph.physics.next_id.max(1);
        graph.physics.next_id = id + 1;
        values.insert("m3.auxiliary".into(), 1);
        values.insert("m3.cloth.id".into(), i64::from(id));
        values.insert(
            "m3.cloth.skin_bone_count".into(),
            i64::from(r.skin_bone_count),
        );
        values.insert(
            "m3.cloth.local_channels".into(),
            i64::from(r.local_channels),
        );
        values.insert("m3.cloth.flatten".into(), i64::from(r.flatten));
        values.insert(
            "m3.cloth.use_skin_collision".into(),
            i64::from(r.use_skin_collision),
        );
        let skin = source
            .document()
            .resolve::<nr::U16>(r.skin_bones.cast())
            .map_err(read_error)?
            .iter()
            .map(|row| {
                row.map(|row| wr::U32 {
                    value: u32::from(row.value),
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_error)?;
        let mut texts = BTreeMap::new();
        let sim = source
            .document()
            .resolve::<nr::Byte>(r.sim_enabled.cast())
            .map_err(read_error)?
            .iter()
            .map(|row| row.map(|row| row.value))
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_error)?;
        texts.insert("m3.cloth.sim_enabled".into(), numbers(sim));
        for (name, reference) in [
            ("m3.cloth.vertex_bones", r.vertex_bones),
            ("m3.cloth.vertex_weights", r.vertex_weights),
        ] {
            let rows = source
                .document()
                .resolve::<nr::U32>(reference.cast())
                .map_err(read_error)?
                .iter()
                .map(|row| row.map(|row| row.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?;
            texts.insert(name.into(), numbers(rows));
        }
        let mut collider_ids = Vec::new();
        for row in source
            .document()
            .resolve::<nr::ClothCollider>(r.colliders.cast())
            .map_err(read_error)?
            .iter()
        {
            let row = row.map_err(read_error)?;
            let collider_id = graph.physics.next_id;
            graph.physics.next_id += 1;
            collider_ids.push(wr::U32 { value: collider_id });
            colliders.push(wr::ClothCollider::<wr::V2> {
                id: collider_id,
                node: if row.bone == 0xffff {
                    u32::MAX
                } else {
                    row.bone
                },
                kind: wr::ClothColliderKind::Capsule,
                transform: row.transform,
                radius: row.radius,
                length: row.height,
                ..Default::default()
            });
        }
        let mut bindings = Vec::new();
        for (index, proxy) in source
            .document()
            .resolve::<nr::ClothProxy>(r.proxies.cast())
            .map_err(read_error)?
            .iter()
            .enumerate()
        {
            let proxy = proxy.map_err(read_error)?;
            bindings.push(wr::ClothBinding {
                section: wr::SectionRef {
                    mesh: region(&raw, proxy.proxy_index)?,
                    section: 0,
                },
            });
            values.insert(
                format!("m3.cloth.proxy.{index}.cage"),
                i64::from(region(&raw, proxy.cloth_index)?),
            );
            let vertices = source
                .document()
                .resolve::<nr::U64>(proxy.proxy_vertices.cast())
                .map_err(read_error)?
                .iter()
                .map(|row| row.map(|row| row.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?;
            texts.insert(
                format!("m3.cloth.proxy.{index}.vertices"),
                numbers(vertices),
            );
            let weights = source
                .document()
                .resolve::<nr::U32>(proxy.proxy_weights.cast())
                .map_err(read_error)?
                .iter()
                .map(|row| row.map(|row| row.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?;
            texts.insert(format!("m3.cloth.proxy.{index}.weights"), numbers(weights));
        }
        let bag = metadata::extend(&mut raw, &wr::NativeBag::default(), &values)?;
        let mut bag = bag;
        for (key, value) in texts {
            bag = metadata::set_text(&mut raw, &bag, &key, &value)?;
        }
        nodes.push(wr::Node::<wr::V19> {
            name: metadata::text(&mut raw, &format!("Cloth {id} controls"))?,
            parent: u32::MAX,
            kind: wr::NodeKind::Helper,
            local: wr::Transform {
                rotation: [0., 0., 0., 1.],
                scale: [1.; 3],
                ..Default::default()
            },
            native: bag,
            ..Default::default()
        });
        cloths.push(wr::Cloth::<wr::V2> {
            id,
            cage: wr::SectionRef {
                mesh: region(&raw, r.cage_region)?,
                section: 0,
            },
            bindings: metadata::put(&mut raw, &bindings)?.cast(),
            colliders: metadata::put(&mut raw, &collider_ids)?.cast(),
            active: Bool::from(r.active.initial_value != 0),
            density: r.density,
            damping: r.damping,
            friction: r.friction,
            stretch_stiffness: r.stretch_stiffness,
            bend_stiffness: r.bending_stiffness,
            gravity_scale: r.gravity,
            wind: r.local_wind,
            sc2: InlineOption::from(Some(wr::Sc2ClothParams {
                tracking: r.tracking,
                horizontal_stiffness: r.horizontal_stiffness,
                shear_stiffness: r.shear_stiffness,
                explosion_scale: r.explosion_scale,
                wind_scale: r.wind_scale,
                drag_factor: r.drag_factor,
                lift_factor: r.lift_factor,
                sphere_stiffness: r.sphere_stiffness,
                flatten: Bool::from(r.flatten != 0),
                use_skin_collision: Bool::from(r.use_skin_collision != 0),
                skin_offset: r.skin_offset,
                skin_exponent: r.skin_exponent,
                skin_stiffness: r.skin_stiffness,
            })),
            recipe: InlineOption::from(Some(wr::ClothRecipe {
                bones: metadata::put(&mut raw, &skin)?.cast(),
                cage_from: r.cage_region,
                ..Default::default()
            })),
        });
    }
    graph.nodes.nodes = metadata::put(&mut raw, &nodes)?.cast();
    graph.physics.cloths = metadata::put(&mut raw, &cloths)?.cast();
    graph.physics.colliders = metadata::put(&mut raw, &colliders)?.cast();
    raw.replace(
        raw.section_id(root.models.index)
            .ok_or_else(|| Error::new("wem.model", "missing section"))?,
        &[graph],
    )?;
    Ok(wem::Document::from_raw(raw)?)
}
