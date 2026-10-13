//! Shadow-volume dimensions and their per-bone animation identities.
use super::super::*;
pub(in crate::model::conversion::m3::controls) fn import_shadow_boxes<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::ShadowBox>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.shadow_boxes.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::ShadowBox>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.shadow_boxes[{index}]");
        fields.uint(&format!("{prefix}.bone_index"), u32::from(r.bone_index));
        fields.uint(&format!("{prefix}.padding"), u32::from(r.padding));
        for (name, value) in [
            ("length", r.length),
            ("width", r.width),
            ("height", r.height),
        ] {
            metadata::header(
                &mut fields.values,
                &format!("controls.shadow_boxes[{index}].{name}"),
                &value,
            );
            fields.float(&format!("{prefix}.{name}.initial"), value.initial_value);
            fields.float(&format!("{prefix}.{name}.null"), value.null_value);
        }
    }
    Ok(())
}
pub(in crate::model::conversion::m3::controls) fn export_shadow_boxes<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    map: &[u32],
) -> Result<Ref<nr::targets::ShadowBox>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.shadow_boxes.count") {
        let prefix = format!("m3.controls.shadow_boxes[{index}]");
        let property = |name: &str| {
            metadata::native_property(
                &fields.values,
                &format!("controls.shadow_boxes[{index}].{name}"),
                wr::Sc2Property {
                    init_value: fields.float(&format!("{prefix}.{name}.initial")),
                    null_value: fields.float(&format!("{prefix}.{name}.null")),
                },
            )
        };
        rows.push(nr::ShadowBox {
            bone_index: bone(fields.uint(&format!("{prefix}.bone_index")), map, &prefix)?,
            padding: u16::try_from(fields.uint(&format!("{prefix}.padding")))
                .map_err(|_| Error::new(prefix.clone(), "padding exceeds u16"))?,
            length: property("length"),
            width: property("width"),
            height: property("height"),
        });
    }
    Ok(append(doc, &rows)?.cast())
}
