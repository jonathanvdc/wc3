//! Events exported from active WEM records.
use super::*;

pub(super) fn export<V: ModelLayout + ParticleConversion + CameraConversion>(
    state: &mut SceneExport<'_, '_, V>,
    index: usize,
    _node: &wr::Node<wr::V19>,
    common: &sm::Node<'_>,
    metadata: &BTreeMap<String, i64>,
    payload: &wr::EventPayload<wr::V19>,
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

    let track = export_track::<V, u8>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1400,
        },
        starts,
        globals,
        context,
        |_, _, _| Ok(1),
    )?;
    let identifier = if payload.code.entries > 0 {
        doc_event_code(model, payload.code)?
    } else {
        (metadata.get("m2.event.identifier").copied().unwrap_or(0) as u32).to_le_bytes()
    };
    core.events.push(native::records::Event {
        identifier,
        data: payload.id,
        bone_id: actor_bone(model, index, common),
        position: common.pivot,
        enabled: track.base,
    });
    Ok(())
}

fn doc_event_code(
    model: &sm::Model<'_>,
    code: wem::Ref<wr::targets::Character>,
) -> Result<[u8; 4], Error> {
    let doc = model
        .native
        .as_ref()
        .ok_or_else(|| Error::new("events.code", "missing native graph"))?
        .document
        .raw();
    let bytes = doc
        .resolve::<wr::Character>(code.cast())?
        .iter()
        .map(|v| v.map(|v| v.value))
        .collect::<Result<Vec<_>, _>>()?;
    bytes.try_into().map_err(|_| {
        Error::new(
            "events.code",
            "M2 event code must contain exactly four bytes",
        )
    })
}
