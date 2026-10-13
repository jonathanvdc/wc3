//! Editable billboard behaviors controls.
use super::super::*;
pub(in crate::model::conversion::m3::controls) fn import_billboard_behaviors<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::BillboardBehavior>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.billboard_behaviors.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::BillboardBehavior>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.billboard_behaviors[{index}]");
        fields.ints(
            &format!("{prefix}.dependents"),
            doc.resolve::<nr::U16>(r.dependents.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| u32::from(r.value)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        );
        fields.uint(&format!("{prefix}.bone_index"), u32::from(r.bone_index));
        fields.uint(
            &format!("{prefix}.billboard_type"),
            u32::from(r.billboard_type),
        );
        fields.uint(
            &format!("{prefix}.camera_look_at"),
            u32::from(r.camera_look_at),
        );
        fields.floats(&format!("{prefix}.up"), r.up);
        fields.floats(&format!("{prefix}.forward"), r.forward);
    }
    Ok(())
}

pub(in crate::model::conversion::m3::controls) fn export_billboard_behaviors<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::BillboardBehavior>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.billboard_behaviors.count") {
        let prefix = format!("m3.controls.billboard_behaviors[{index}]");
        rows.push(nr::BillboardBehavior {
            dependents: {
                let rows = fields
                    .ints(&format!("{prefix}.dependents"))?
                    .into_iter()
                    .map(|value| {
                        Ok(nr::U16 {
                            value: u16::try_from(u32::from(bone(
                                value,
                                bone_map,
                                &format!("{prefix}.dependents"),
                            )?))
                            .map_err(|_| {
                                Error::new(
                                    format!("{prefix}.dependents"),
                                    "array value exceeds native width",
                                )
                            })?,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                append(doc, &rows)?.cast()
            },
            bone_index: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone_index")),
                bone_map,
                &format!("{prefix}.bone_index"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone_index"),
                    "value exceeds native integer width",
                )
            })?,
            billboard_type: u8::try_from(fields.uint(&format!("{prefix}.billboard_type")))
                .map_err(|_| {
                    Error::new(
                        format!("{prefix}.billboard_type"),
                        "value exceeds native integer width",
                    )
                })?,
            camera_look_at: u8::try_from(fields.uint(&format!("{prefix}.camera_look_at")))
                .map_err(|_| {
                    Error::new(
                        format!("{prefix}.camera_look_at"),
                        "value exceeds native integer width",
                    )
                })?,
            up: fields.array(&format!("{prefix}.up"))?,
            forward: fields.array(&format!("{prefix}.forward"))?,
        });
    }
    Ok(append(doc, &rows)?.cast())
}
