//! Editable ik two joints controls.
use super::super::*;
pub(in crate::model::conversion::m3::controls) fn import_ik_two_joints<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::IKTwoJoint>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.ik_two_joints.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::IKTwoJoint>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.ik_two_joints[{index}]");
        fields.ints(
            &format!("{prefix}.dependents"),
            doc.resolve::<nr::U16>(r.dependents.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| u32::from(r.value)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        );
        fields.uint(&format!("{prefix}.bone_base"), u32::from(r.bone_base));
        fields.uint(&format!("{prefix}.bone_target"), u32::from(r.bone_target));
        fields.uint(&format!("{prefix}.bone_end"), u32::from(r.bone_end));
        fields.uint(&format!("{prefix}.padding"), u32::from(r.padding));
        fields.floats(&format!("{prefix}.hinge_axis"), r.hinge_axis);
        fields.float(&format!("{prefix}.max_angle_inner"), r.max_angle_inner);
        fields.float(&format!("{prefix}.max_angle_outer"), r.max_angle_outer);
        fields.float(&format!("{prefix}.search_up"), r.search_up);
        fields.float(&format!("{prefix}.search_down"), r.search_down);
    }
    Ok(())
}

pub(in crate::model::conversion::m3::controls) fn export_ik_two_joints<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::IKTwoJoint>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.ik_two_joints.count") {
        let prefix = format!("m3.controls.ik_two_joints[{index}]");
        rows.push(nr::IKTwoJoint {
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
            bone_base: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone_base")),
                bone_map,
                &format!("{prefix}.bone_base"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone_base"),
                    "value exceeds native integer width",
                )
            })?,
            bone_target: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone_target")),
                bone_map,
                &format!("{prefix}.bone_target"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone_target"),
                    "value exceeds native integer width",
                )
            })?,
            bone_end: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone_end")),
                bone_map,
                &format!("{prefix}.bone_end"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone_end"),
                    "value exceeds native integer width",
                )
            })?,
            padding: u16::try_from(fields.uint(&format!("{prefix}.padding"))).map_err(|_| {
                Error::new(
                    format!("{prefix}.padding"),
                    "value exceeds native integer width",
                )
            })?,
            hinge_axis: fields.array(&format!("{prefix}.hinge_axis"))?,
            max_angle_inner: fields.float(&format!("{prefix}.max_angle_inner")),
            max_angle_outer: fields.float(&format!("{prefix}.max_angle_outer")),
            search_up: fields.float(&format!("{prefix}.search_up")),
            search_down: fields.float(&format!("{prefix}.search_down")),
        });
    }
    Ok(append(doc, &rows)?.cast())
}
