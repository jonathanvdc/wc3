//! Editable trigger data controls.
use super::super::*;
pub(in crate::model::conversion::m3::controls) fn import_trigger_data<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::TriggerData>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.trigger_data.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::TriggerData>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.trigger_data[{index}]");
        fields.ints(
            &format!("{prefix}.data_indices"),
            doc.resolve::<nr::U32>(r.data_indices.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| r.value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        );
        fields.texts.insert(
            format!("{prefix}.name"),
            string(doc, r.name).map_err(read_error)?,
        );
    }
    Ok(())
}

pub(in crate::model::conversion::m3::controls) fn export_trigger_data<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    _bone_map: &[u32],
) -> Result<Ref<nr::targets::TriggerData>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.trigger_data.count") {
        let prefix = format!("m3.controls.trigger_data[{index}]");
        rows.push(nr::TriggerData {
            data_indices: {
                let rows = fields
                    .ints(&format!("{prefix}.data_indices"))?
                    .into_iter()
                    .map(|value| Ok(nr::U32 { value }))
                    .collect::<Result<Vec<_>, Error>>()?;
                append(doc, &rows)?.cast()
            },
            name: append_string(
                doc,
                fields
                    .texts
                    .get(&format!("{prefix}.name"))
                    .map(String::as_str)
                    .unwrap_or_default(),
            )?,
        });
    }
    Ok(append(doc, &rows)?.cast())
}
