//! Attachments imported into active WEM hierarchy records.
use super::*;

pub(super) fn import_attachments<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
    patches: &mut Vec<NodePatch>,
) -> Result<(), Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for (index, attachment) in core.attachments.iter().enumerate() {
        let node = model.nodes.len();
        model.nodes.push(sm::Node {
            name: Cow::Owned(format!("attachment-{}", attachment.id)),
            parent: Some(u32::from(attachment.bone_id)),
            pivot: attachment.position,
            ..Default::default()
        });
        patches.push(NodePatch {
            particle: None,
            texts: Vec::new(),
            index: node,
            kind: wr::NodeKind::Attachment,
            payload: wr::NodePayload::Attachment(Box::new(wr::AttachmentPayload {
                model: u32::MAX,
                ..Default::default()
            })),
            metadata: vec![
                ("m2.attachment.id", i64::from(attachment.id)),
                ("m2.attachment.reserved", i64::from(attachment.unknown)),
            ],
        });
        for sequence in 0..model.clips.len() {
            if let Some(track) = import_track(
                source,
                &attachment.animate,
                &source.key,
                sequence,
                core.sequences
                    .get(sequence)
                    .map_or(0, |sequence| sequence.timing.window().0),
                sm::Target::Property {
                    owner: node as u32,
                    channel: u32::from(wr::Channel::Visibility.raw()),
                },
                |v| vec![f32::from(v)],
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
        let _ = index;
    }
    Ok(())
}
