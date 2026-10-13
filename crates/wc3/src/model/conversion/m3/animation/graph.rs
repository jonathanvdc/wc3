//! Controller boundaries, concurrent layers, and source channel identities.
use super::super::effects::metadata as native_metadata;
use super::*;
pub(super) fn graph<E: Envelope>(
    model: &sem::Model<'_>,
    clip_index: usize,
    doc: &mut native::Document<'static, E>,
    bones: &mut [nr::Bone<nr::V1>],
    context: &mut Context,
) -> Result<Option<Vec<nr::SubTrackContainer<nr::V4>>>, Error> {
    let Some(native) = &model.native else {
        return Ok(None);
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let clips: Vec<_> = raw
        .resolve::<wr::Clip<wr::V7>>(root.clips.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let Some(clip) = clips
        .iter()
        .filter(|clip| clip.model == native.model)
        .nth(clip_index)
    else {
        return Ok(None);
    };
    let containers: Vec<_> = raw
        .resolve::<wr::SubTrackContainer>(clip.containers.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut native_graph = false;
    for container in &containers {
        let values = native_metadata::read(raw, &container.native)?;
        native_graph |= values.contains_key("m3.controller.unknown");
    }
    if !native_graph {
        return Ok(None);
    }
    let mut output = Vec::new();
    for (index, container) in containers.iter().enumerate() {
        let values = native_metadata::read(raw, &container.native)?;
        let mut ids = Vec::new();
        let mut selectors = Vec::new();
        let blocks = export_native(
            model,
            clip_index,
            doc,
            NativeOutput {
                ids: &mut ids,
                selectors: &mut selectors,
            },
            context,
            NativeExportOptions {
                selected_container: Some(index),
                bones: Some(bones),
                ..Default::default()
            },
        )?;
        let mut pairs: Vec<_> = ids.into_iter().zip(selectors).collect();
        pairs.sort_by_key(|(id, _)| id.value);
        let (ids, selectors): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        let mut row = nr::SubTrackContainer::<nr::V4> {
            name: append_string(doc, raw.string(container.name)?.unwrap_or_default())?,
            runs_concurrent: values
                .get("m3.controller.runs_concurrent")
                .copied()
                .filter(|&value| (value != 0) == container.concurrent.get())
                .unwrap_or(i64::from(container.concurrent.get()))
                as u16,
            anim_priority: u16::try_from(container.priority)
                .map_err(|_| Error::new("animation.priority", "priority exceeds native u16"))?,
            anim_ids: append(doc, &ids)?.cast(),
            anim_refs: append(doc, &selectors)?.cast(),
            unknown: values.get("m3.controller.unknown").copied().unwrap_or(0) as u32,
            animation_state_index: values
                .get("m3.controller.state_index")
                .copied()
                .unwrap_or(0) as u16,
            animation_state_index_copy: values
                .get("m3.controller.state_index_copy")
                .copied()
                .unwrap_or(0) as u16,
            ..Default::default()
        };
        blocks.install(doc, &mut row)?;
        output.push(row);
    }
    Ok(Some(output))
}
