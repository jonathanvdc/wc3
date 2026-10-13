//! Authored per-division animated bounds, independent of vertex geometry.
use super::*;
pub(super) fn import<E: Envelope>(
    source: &native::Model<'_, E>,
    fields: &mut Fields,
) -> Result<(), Error> {
    let reference =
        native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| r.divisions);
    let doc = source.document();
    fields.uint("m3.controls.divisions.count", reference.entries);
    macro_rules! divisions {
        ($v:ty) => {
            for (di, row) in doc
                .resolve::<nr::MeshDivision<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
                .enumerate()
            {
                let r = row.map_err(read_error)?;
                fields.uint(
                    &format!("m3.controls.divisions[{di}].instances"),
                    r.instances,
                );
                let prefix = format!("m3.controls.divisions[{di}].msec");
                fields.uint(&format!("{prefix}.count"), r.msec.entries);
                if r.msec.entries == 0 {
                    continue;
                }
                fields.uint(
                    &format!("{prefix}.version"),
                    revision(doc, r.msec).map_err(read_error)?,
                );
                macro_rules! msec {
                    ($s:ty) => {
                        for (index, row) in doc
                            .resolve::<nr::MeshSection<$s>>(r.msec.cast())
                            .map_err(read_error)?
                            .iter()
                            .enumerate()
                        {
                            let r = row.map_err(read_error)?;
                            let prefix = format!("{prefix}[{index}]");
                            fields.uint(&format!("{prefix}.node_index"), r.node_index);
                            metadata::header(
                                &mut fields.values,
                                &format!("controls.divisions[{di}].msec[{index}].bounds"),
                                &r.bounds,
                            );
                            fields.floats(
                                &format!("{prefix}.bounds.initial"),
                                r.bounds
                                    .initial_value
                                    .minimum
                                    .into_iter()
                                    .chain(r.bounds.initial_value.maximum)
                                    .chain([r.bounds.initial_value.radius]),
                            );
                            fields.floats(
                                &format!("{prefix}.bounds.null"),
                                r.bounds
                                    .null_value
                                    .minimum
                                    .into_iter()
                                    .chain(r.bounds.null_value.maximum)
                                    .chain([r.bounds.null_value.radius]),
                            );
                        }
                    };
                }
                match revision(doc, r.msec).map_err(read_error)? {
                    0 => msec!(nr::V0),
                    1 => msec!(nr::V1),
                    _ => return Err(Error::new("mesh.bounds.revision", "unknown MSEC revision")),
                }
            }
        };
    }
    if reference.entries != 0 {
        match revision(doc, reference).map_err(read_error)? {
            0 => divisions!(nr::V0),
            1 => divisions!(nr::V1),
            2 => divisions!(nr::V2),
            _ => {
                return Err(Error::new(
                    "mesh.divisions.revision",
                    "unknown division revision",
                ));
            }
        }
    }
    Ok(())
}
fn extent(fields: &View, key: &str) -> Result<nr::Extent, Error> {
    let v = fields.array::<7>(key)?;
    Ok(nr::Extent {
        minimum: [v[0], v[1], v[2]],
        maximum: [v[3], v[4], v[5]],
        radius: v[6],
    })
}
pub(super) fn export<E: Envelope>(
    fields: &View,
    doc: &mut native::Document<'static, E>,
    reference: Ref<nr::targets::MeshDivision>,
    map: &[u32],
) -> Result<Ref<nr::targets::MeshDivision>, Error> {
    let mut divisions = doc
        .resolve::<nr::MeshDivision<nr::V2>>(reference.cast())
        .map_err(read_error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(read_error)?;
    let count = fields.uint("m3.controls.divisions.count") as usize;
    divisions.resize_with(
        divisions.len().max(count),
        nr::MeshDivision::<nr::V2>::default,
    );
    for (di, division) in divisions.iter_mut().enumerate() {
        let instances_key = format!("m3.controls.divisions[{di}].instances");
        if division.regions.entries == 0 && fields.values.contains_key(&instances_key) {
            division.instances = fields.uint(&instances_key);
        }
        let base = format!("m3.controls.divisions[{di}].msec");
        let count = fields.uint(&format!("{base}.count"));
        if count == 0 {
            continue;
        }
        macro_rules! write {
            ($v:ty) => {{
                let mut rows = Vec::new();
                for index in 0..count {
                    let prefix = format!("{base}[{index}]");
                    let initial = extent(fields, &format!("{prefix}.bounds.initial"))?;
                    let null = extent(fields, &format!("{prefix}.bounds.null"))?;
                    let size = wr::Sc2Property {
                        init_value: 0f32,
                        null_value: 0.,
                    };
                    let header = metadata::native_property(
                        &fields.values,
                        &format!("controls.divisions[{di}].msec[{index}].bounds"),
                        size,
                    );
                    rows.push(nr::MeshSection::<$v> {
                        node_index: u32::from(bone(
                            fields.uint(&format!("{prefix}.node_index")),
                            map,
                            &prefix,
                        )?),
                        bounds: nr::AnimationValue {
                            interpolation: header.interpolation,
                            flags: header.flags,
                            animation_id: header.animation_id,
                            initial_value: initial,
                            null_value: null,
                            reserved: header.reserved,
                        },
                    });
                }
                division.msec = append(doc, &rows)?.cast();
            }};
        }
        match fields.uint(&format!("{base}.version")) {
            0 => write!(nr::V0),
            1 => write!(nr::V1),
            _ => return Err(Error::new("mesh.bounds.revision", "unknown MSEC revision")),
        }
    }
    Ok(append(doc, &divisions)?.cast())
}
