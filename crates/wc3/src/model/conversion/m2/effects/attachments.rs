//! Attachments exported from active WEM records.
use super::*;

pub(super) fn export<V: ModelLayout + ParticleConversion + CameraConversion>(
    state: &mut SceneExport<'_, '_, V>,
    index: usize,
    _node: &wr::Node<wr::V19>,
    common: &sm::Node<'_>,
    metadata: &BTreeMap<String, i64>,
    _: &wr::AttachmentPayload,
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

    let mut value = native::records::Attachment::<V> {
        id: metadata
            .get("m2.attachment.id")
            .copied()
            .unwrap_or(index as i64) as u32,
        unknown: metadata.get("m2.attachment.reserved").copied().unwrap_or(0) as u16,
        bone_id: u16::try_from(actor_bone(model, index, common))
            .map_err(|_| Error::new("attachments.bone", "parent bone exceeds u16"))?,
        position: common.pivot,
        ..Default::default()
    };
    value.animate = export_track::<V, u8>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: u32::from(wr::Channel::VISIBILITY.0),
        },
        starts,
        globals,
        context,
        |v, _, path| byte(v, path),
    )?;
    core.attachments.push(value);
    Ok(())
}
