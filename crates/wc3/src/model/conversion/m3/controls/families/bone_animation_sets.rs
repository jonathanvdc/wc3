//! Editable bone animation sets controls.
use super::super::*;
pub(in crate::model::conversion::m3::controls) fn import_bone_animation_sets<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::BoneAnimationSet>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.bone_animation_sets.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::BoneAnimationSet>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.bone_animation_sets[{index}]");
        fields.uint(&format!("{prefix}.flags"), (r.flags).bits());
        fields.uint(
            &format!("{prefix}.animation_sequence_index"),
            u32::from(r.animation_sequence_index),
        );
        fields.uint(
            &format!("{prefix}.fallback_sequence_index"),
            u32::from(r.fallback_sequence_index),
        );
        fields.texts.insert(
            format!("{prefix}.name"),
            string(doc, r.name).map_err(read_error)?,
        );
        fields.ints(
            &format!("{prefix}.split_items"),
            doc.resolve::<nr::U16>(r.split_items.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| u32::from(r.value)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        );
    }
    Ok(())
}

pub(in crate::model::conversion::m3::controls) fn export_bone_animation_sets<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    _bone_map: &[u32],
) -> Result<Ref<nr::targets::BoneAnimationSet>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.bone_animation_sets.count") {
        let prefix = format!("m3.controls.bone_animation_sets[{index}]");
        rows.push(nr::BoneAnimationSet {
            flags: nr::BoneSetFlags(fields.uint(&format!("{prefix}.flags"))),
            animation_sequence_index: u16::try_from(
                fields.uint(&format!("{prefix}.animation_sequence_index")),
            )
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.animation_sequence_index"),
                    "value exceeds native integer width",
                )
            })?,
            fallback_sequence_index: u16::try_from(
                fields.uint(&format!("{prefix}.fallback_sequence_index")),
            )
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.fallback_sequence_index"),
                    "value exceeds native integer width",
                )
            })?,
            name: append_string(
                doc,
                fields
                    .texts
                    .get(&format!("{prefix}.name"))
                    .map(String::as_str)
                    .unwrap_or_default(),
            )?,
            split_items: {
                let rows = fields
                    .ints(&format!("{prefix}.split_items"))?
                    .into_iter()
                    .map(|value| {
                        Ok(nr::U16 {
                            value: u16::try_from(value).map_err(|_| {
                                Error::new(
                                    format!("{prefix}.split_items"),
                                    "array value exceeds native width",
                                )
                            })?,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                append(doc, &rows)?.cast()
            },
        });
    }
    Ok(append(doc, &rows)?.cast())
}
