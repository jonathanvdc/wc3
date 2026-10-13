//! Editable ik joints controls.
use super::super::*;
pub(in crate::model::conversion::m3::controls) fn import_ik_joints<E: Envelope>(
    doc: &native::Document<'_, E>,
    reference: Ref<nr::targets::IKJoint>,
    fields: &mut Fields,
) -> Result<(), Error> {
    fields.uint("m3.controls.ik_joints.count", reference.entries);
    for (index, row) in doc
        .resolve::<nr::IKJoint>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let r = row.map_err(read_error)?;
        let prefix = format!("m3.controls.ik_joints[{index}]");
        fields.ints(
            &format!("{prefix}.dependents"),
            doc.resolve::<nr::U16>(r.dependents.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| r.map(|r| u32::from(r.value)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        );
        fields.uint(&format!("{prefix}.bone_index1"), u32::from(r.bone_index1));
        fields.uint(&format!("{prefix}.bone_index2"), u32::from(r.bone_index2));
        fields.float(&format!("{prefix}.raycast_up"), r.raycast_up);
        fields.float(&format!("{prefix}.raycast_down"), r.raycast_down);
        fields.float(&format!("{prefix}.max_speed"), r.max_speed);
        fields.float(&format!("{prefix}.goal_threshold"), r.goal_threshold);
    }
    Ok(())
}

pub(in crate::model::conversion::m3::controls) fn export_ik_joints<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
) -> Result<Ref<nr::targets::IKJoint>, Error> {
    let mut rows = Vec::new();
    for index in 0..fields.uint("m3.controls.ik_joints.count") {
        let prefix = format!("m3.controls.ik_joints[{index}]");
        rows.push(nr::IKJoint {
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
            bone_index1: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone_index1")),
                bone_map,
                &format!("{prefix}.bone_index1"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone_index1"),
                    "value exceeds native integer width",
                )
            })?,
            bone_index2: u16::try_from(u32::from(bone(
                fields.uint(&format!("{prefix}.bone_index2")),
                bone_map,
                &format!("{prefix}.bone_index2"),
            )?))
            .map_err(|_| {
                Error::new(
                    format!("{prefix}.bone_index2"),
                    "value exceeds native integer width",
                )
            })?,
            raycast_up: fields.float(&format!("{prefix}.raycast_up")),
            raycast_down: fields.float(&format!("{prefix}.raycast_down")),
            max_speed: fields.float(&format!("{prefix}.max_speed")),
            goal_threshold: fields.float(&format!("{prefix}.goal_threshold")),
        });
    }
    Ok(append(doc, &rows)?.cast())
}
