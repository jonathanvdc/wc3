//! Per-node view-volume extents and animation-property controls.
use super::*;
pub(super) fn import<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::ViewVolume>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.view_volumes.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::ViewVolume>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.view_volumes[{index}]");
        fields.uint(&format!("{prefix}.node_index"), r.node_index);
        metadata::header(
            &mut fields.values,
            &format!("controls.view_volumes[{index}].size"),
            &r.size,
        );
        fields.floats(&format!("{prefix}.size.initial"), r.size.initial_value);
        fields.floats(&format!("{prefix}.size.null"), r.size.null_value);
    }
    Ok(())
}
pub(super) fn export<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::ViewVolume>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.view_volumes.count") {
        let prefix = format!("m3.controls.view_volumes[{index}]");
        let node_index = u32::from(bone(
            fields.uint(&format!("{prefix}.node_index")),
            bone_map,
            &prefix,
        )?);
        rows.push(nr::ViewVolume {
            node_index,
            size: metadata::native_property(
                &fields.values,
                &format!("controls.view_volumes[{index}].size"),
                wr::Sc2Property {
                    init_value: fields.array::<3>(&format!("{prefix}.size.initial"))?,
                    null_value: fields.array::<3>(&format!("{prefix}.size.null"))?,
                },
            ),
        });
    }
    Ok(append(doc, &rows)?.cast())
}
