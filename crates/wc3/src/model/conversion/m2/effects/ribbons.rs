//! Ribbons exported from active WEM records.
use super::*;

pub(super) fn export<V: ModelLayout + ParticleConversion + CameraConversion>(
    state: &mut SceneExport<'_, '_, V>,
    index: usize,
    node: &wr::Node<wr::V19>,
    common: &sm::Node<'_>,
    metadata: &BTreeMap<String, i64>,
    payload: &wr::Wc3RibbonEmitterPayload,
) -> Result<(), Error>
where
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, f32>: ValuesFactory<f32>,
    V::TrackValues<'static, u8>: ValuesFactory<u8>,
    V::TrackValues<'static, u32>: ValuesFactory<u32>,
    V::TrackValues<'static, i16>: ValuesFactory<i16>,
    V::TrackValues<'static, u16>: ValuesFactory<u16>,
    V::SplineValues<'static, f32>: SplineFactory<f32>,
    V::SplineValues<'static, [f32; 3]>: SplineFactory<[f32; 3]>,
    V::TrackValues<'static, [f32; 3]>: ValuesFactory<[f32; 3]>,
{
    let SceneExport {
        model,
        core,
        starts,
        globals,
        context,
    } = state;

    let doc = model.native.as_ref().unwrap().document.raw();
    let mut ribbon = native::records::RibbonEmitter::<V> {
        id: metadata
            .get("m2.ribbon.id")
            .copied()
            .unwrap_or(u32::MAX as i64) as u32,
        bone_id: actor_bone(model, index, common),
        position: common.pivot,
        texture_indices: index_words(metadata_text(doc, &node.native, "m2.ribbon.textures")?)?,
        material_indices: index_words(metadata_text(doc, &node.native, "m2.ribbon.materials")?)?,
        edge_lifetime: payload.lifespan,
        gravity: payload.gravity,
        texture_rows: u16::try_from(payload.rows)
            .map_err(|_| Error::new("ribbons.rows", "atlas rows exceed u16"))?,
        texture_columns: u16::try_from(payload.columns)
            .map_err(|_| Error::new("ribbons.columns", "atlas columns exceed u16"))?,
        ..Default::default()
    };
    let rate = metadata
        .get("m2.ribbon.rate")
        .map(|v| f32::from_bits(*v as u32));
    ribbon.edges_per_second = rate
        .filter(|v| v.max(0.) as u32 == payload.emission_rate)
        .unwrap_or(payload.emission_rate as f32);
    ribbon.modern = V::ribbon_tail_from(
        [
            metadata.get("m2.ribbon.priority").copied().unwrap_or(0) as i16,
            metadata.get("m2.ribbon.color").copied().unwrap_or(0) as i16,
            metadata.get("m2.ribbon.transform").copied().unwrap_or(0) as i16,
        ],
        context,
    )?;
    ribbon.height_above = export_track::<V, f32>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1100,
        },
        starts,
        globals,
        context,
        |v, _, path| {
            v.first()
                .copied()
                .filter(|_| v.len() == 1)
                .ok_or_else(|| Error::new(path, "height needs one component"))
        },
    )?;
    ribbon.height_below = export_track::<V, f32>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1101,
        },
        starts,
        globals,
        context,
        |v, _, path| {
            v.first()
                .copied()
                .filter(|_| v.len() == 1)
                .ok_or_else(|| Error::new(path, "height needs one component"))
        },
    )?;
    ribbon.color = export_track::<V, [f32; 3]>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1102,
        },
        starts,
        globals,
        context,
        |v, _, path| {
            v.try_into()
                .map_err(|_| Error::new(path, "color needs three components"))
        },
    )?;
    ribbon.alpha = export_track::<V, i16>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1103,
        },
        starts,
        globals,
        context,
        |v, context, path| {
            let value = v
                .first()
                .copied()
                .filter(|_| v.len() == 1)
                .ok_or_else(|| Error::new(path, "alpha needs one component"))?;
            if !value.is_finite()
                || value * 32767. < f32::from(i16::MIN)
                || value * 32767. > f32::from(i16::MAX)
            {
                return Err(Error::new(path, "alpha exceeds signed fixed16"));
            }
            let result = (value * 32767.).round() as i16;
            context.approximate(
                path,
                (value - f32::from(result) / 32767.).abs(),
                "ribbon alpha quantization",
            )?;
            Ok(result)
        },
    )?;
    ribbon.texture_slot = export_track::<V, u16>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1104,
        },
        starts,
        globals,
        context,
        |v, _, path| {
            let value = v
                .first()
                .copied()
                .filter(|_| v.len() == 1)
                .ok_or_else(|| Error::new(path, "atlas slot needs one component"))?;
            if !value.is_finite()
                || value < 0.
                || value > f32::from(u16::MAX)
                || value.fract() != 0.
            {
                return Err(Error::new(path, "atlas slot cannot be represented as u16"));
            }
            Ok(value as u16)
        },
    )?;
    ribbon.visibility = export_track::<V, u8>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1105,
        },
        starts,
        globals,
        context,
        |v, _, path| byte(v, path),
    )?;
    ensure_constant(&mut ribbon.height_above, payload.height_above, globals);
    ensure_constant(&mut ribbon.height_below, payload.height_below, globals);
    ensure_constant(&mut ribbon.color, payload.color, globals);
    ensure_constant(
        &mut ribbon.alpha,
        (payload.alpha * 32767.).round() as i16,
        globals,
    );
    ensure_constant(
        &mut ribbon.texture_slot,
        payload.texture_slot as u16,
        globals,
    );
    ensure_constant(&mut ribbon.visibility, 1u8, globals);
    core.ribbons.push(ribbon);
    Ok(())
}
