//! World illumination and environmental volume definitions.
use super::rgba;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wmo as profile;
use crate::model::wmo::records::FogParams;
use crate::model::wmo::root::RootChunk;
use crate::model::wmo::Dialect;
fn params(value: &FogParams) -> profile::FogParameters {
    profile::FogParameters {
        end: value.end,
        start_fraction: value.start_scalar,
        color: rgba(value.color),
    }
}
fn set(owner: crate::model::semantic::SystemId, value: u16) -> Option<profile::DoodadSetId> {
    (value != 0).then_some(profile::DoodadSetId {
        system: owner,
        index: u32::from(value),
    })
}
pub(super) fn read<'a, 'data: 'a>(
    root: &'a impl super::root_view::CommonRoot<'data>,
    world: &mut profile::WorldSystem<'a>,
    owner: crate::model::semantic::SystemId,
) -> Result<(), Error> {
    let extras = root.common_chunks().find_map(|chunk| {
        if let RootChunk::FogExtras(values) = chunk {
            Some(values)
        } else {
            None
        }
    });
    for chunk in root
        .common_chunks()
        .filter(|chunk| !matches!(chunk, RootChunk::FogBoxes(_)))
        .chain(
            root.common_chunks()
                .filter(|chunk| matches!(chunk, RootChunk::FogBoxes(_))),
        )
    {
        match chunk {
            RootChunk::Fogs(values) => {
                for (index, row) in values.iter().enumerate() {
                    let row = row.map_err(|error| Error::new("world.fog", error.to_string()))?;
                    let doodad_set = extras
                        .and_then(|values| values.get(index))
                        .transpose()
                        .map_err(|error| Error::new("world.fog.set", error.to_string()))?
                        .and_then(|value| set(owner, value.doodad_set_id));
                    let mut flags = row.flags;
                    flags.set_infinite_radius(false);
                    world.fogs.push(profile::Fog {
                        volume: profile::Volume::Sphere {
                            center: row.position,
                            radii: [row.radius_start, row.radius_end],
                        },
                        above_water: params(&row.fog),
                        underwater: params(&row.underwater),
                        doodad_set,
                        infinite: row.flags.infinite_radius(),
                        controls: flags.bits().into(),
                    });
                }
            }
            RootChunk::FogBoxes(values) => {
                for row in values.iter() {
                    let row = row.map_err(|error| Error::new("world.fog", error.to_string()))?;
                    world.fogs.push(profile::Fog {
                        volume: profile::Volume::Planes {
                            planes: row.planes,
                            fade: row.fade_distance,
                        },
                        above_water: params(&row.fog),
                        underwater: params(&row.underwater),
                        doodad_set: set(owner, row.doodad_set_id),
                        infinite: false,
                        controls: row.flags.bits().into(),
                    });
                }
            }
            RootChunk::AmbientVolumes(values) | RootChunk::GlobalAmbients(values) => {
                for row in values.iter() {
                    let row =
                        row.map_err(|error| Error::new("world.ambient", error.to_string()))?;
                    let mut flags = row.flags;
                    flags.set_distinct_colors(false);
                    world.ambient_volumes.push(profile::Ambient {
                        global: matches!(chunk, RootChunk::GlobalAmbients(_)),
                        volume: profile::Volume::Sphere {
                            center: row.position,
                            radii: [row.start, row.end],
                        },
                        colors: row.colors.map(rgba),
                        distinct_colors: row.flags.distinct_colors(),
                        doodad_set: set(owner, row.doodad_set_id),
                        controls: flags.bits().into(),
                    });
                }
            }
            RootChunk::AmbientBoxes(values) => {
                for row in values.iter() {
                    let row =
                        row.map_err(|error| Error::new("world.ambient", error.to_string()))?;
                    world.ambient_volumes.push(profile::Ambient {
                        global: false,
                        volume: profile::Volume::Planes {
                            planes: row.planes,
                            fade: row.end,
                        },
                        colors: row.colors.map(rgba),
                        distinct_colors: false,
                        doodad_set: set(owner, row.doodad_set_id),
                        controls: row.flags.bits().into(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn write<'a, D: Dialect>(
    world: &profile::WorldSystem<'_>,
    root: &mut impl super::root_output::RootOutput<'a, D>,
) -> Result<(), Error> {
    use crate::model::wmo::records::{
        AmbientBox, AmbientBoxFlags, AmbientFlags, AmbientVolume, Fog, FogBox, FogBoxFlags,
        FogExtra, FogFlags,
    };
    let params = |value: &profile::FogParameters| FogParams {
        end: value.end,
        start_scalar: value.start_fraction,
        color: rgba(value.color),
    };
    let set = |value: Option<profile::DoodadSetId>| -> Result<u16, Error> {
        value.map_or(Ok(0), |value| {
            if value.index == 0 {
                return Err(Error::new(
                    "world.environment.set",
                    "native zero denotes unrestricted activation and cannot restrict to set zero",
                ));
            }
            u16::try_from(value.index).map_err(|_| {
                Error::new(
                    "world.environment.set",
                    "set identity exceeds native capacity",
                )
            })
        })
    };
    let mut fogs = Vec::new();
    let mut extras = Vec::new();
    let mut boxes = Vec::new();
    for source in &world.fogs {
        match source.volume {
            profile::Volume::Sphere { center, radii } => {
                let mut flags = FogFlags(source.controls.bits());
                flags.set_infinite_radius(source.infinite);
                fogs.push(Fog {
                    flags,
                    position: center,
                    radius_start: radii[0],
                    radius_end: radii[1],
                    fog: params(&source.above_water),
                    underwater: params(&source.underwater),
                });
                extras.push(FogExtra {
                    doodad_set_id: set(source.doodad_set)?,
                    ..Default::default()
                });
            }
            profile::Volume::Planes { planes, fade } => {
                if source.infinite {
                    return Err(Error::new(
                        "world.fog.infinite",
                        "plane-bounded native fog has no infinite-radius control",
                    ));
                }
                boxes.push(FogBox {
                    planes,
                    fade_distance: fade,
                    fog: params(&source.above_water),
                    underwater: params(&source.underwater),
                    flags: FogBoxFlags(source.controls.bits()),
                    doodad_set_id: set(source.doodad_set)?,
                    ..Default::default()
                });
            }
        }
    }
    if !fogs.is_empty() {
        super::encode::append(root, RootChunk::Fogs(super::encode::array(&fogs)?))?;
        super::encode::append(root, RootChunk::FogExtras(super::encode::array(&extras)?))?;
    }
    if !boxes.is_empty() {
        super::encode::append(root, RootChunk::FogBoxes(super::encode::array(&boxes)?))?;
    }
    let mut radial = Vec::new();
    let mut global = Vec::new();
    let mut boxes = Vec::new();
    for source in &world.ambient_volumes {
        match source.volume {
            profile::Volume::Sphere { center, radii } => {
                let mut flags = AmbientFlags(source.controls.bits());
                flags.set_distinct_colors(source.distinct_colors);
                let row = AmbientVolume {
                    position: center,
                    start: radii[0],
                    end: radii[1],
                    colors: source.colors.map(rgba),
                    flags,
                    doodad_set_id: set(source.doodad_set)?,
                    ..Default::default()
                };
                if source.global {
                    global.push(row)
                } else {
                    radial.push(row)
                }
            }
            profile::Volume::Planes { planes, fade } => {
                if source.global || source.distinct_colors {
                    return Err(Error::new(
                        "world.ambient.controls",
                        "plane-bounded native ambient volume has no global or distinct-color switch",
                    ));
                }
                boxes.push(AmbientBox {
                    planes,
                    end: fade,
                    colors: source.colors.map(rgba),
                    flags: AmbientBoxFlags(source.controls.bits()),
                    doodad_set_id: set(source.doodad_set)?,
                    ..Default::default()
                });
            }
        }
    }
    if !radial.is_empty() {
        super::encode::append(
            root,
            RootChunk::AmbientVolumes(super::encode::array(&radial)?),
        )?;
    }
    if !global.is_empty() {
        super::encode::append(
            root,
            RootChunk::GlobalAmbients(super::encode::array(&global)?),
        )?;
    }
    if !boxes.is_empty() {
        super::encode::append(root, RootChunk::AmbientBoxes(super::encode::array(&boxes)?))?;
    }
    Ok(())
}
