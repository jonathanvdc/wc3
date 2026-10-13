//! Editable one bone solvers controls.
use super::super::*;
pub(in crate::model::conversion::m3::controls) fn import_one_bone_solvers<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::OneBoneSolver>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.one_bone_solvers.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::OneBoneSolver>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.one_bone_solvers[{index}]");
        fields.ints(
            &format!("{prefix}.dependents"),
            doc.resolve::<nr::U16>(r.dependents.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| u32::from(r.value)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        );
        fields.uint(&format!("{prefix}.bone"), u32::from(r.bone));
        fields.uint(
            &format!("{prefix}.bone_fallback"),
            u32::from(r.bone_fallback),
        );
        fields.uint(&format!("{prefix}.flags"), r.flags);
        fields.float(&format!("{prefix}.max_angle"), r.max_angle);
    }
    Ok(())
}

pub(in crate::model::conversion::m3::controls) fn export_one_bone_solvers<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::OneBoneSolver>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.one_bone_solvers.count") {
        let prefix = format!("m3.controls.one_bone_solvers[{index}]");
        rows.push(nr::OneBoneSolver {
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
            bone: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone")),
                bone_map,
                &format!("{prefix}.bone"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone"),
                    "value exceeds native integer width",
                )
            })?,
            bone_fallback: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone_fallback")),
                bone_map,
                &format!("{prefix}.bone_fallback"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone_fallback"),
                    "value exceeds native integer width",
                )
            })?,
            flags: fields.uint(&format!("{prefix}.flags")),
            max_angle: fields.float(&format!("{prefix}.max_angle")),
        });
    }
    Ok(append(doc, &rows)?.cast())
}
