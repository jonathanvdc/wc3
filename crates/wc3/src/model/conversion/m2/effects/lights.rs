//! Lights exported from active WEM records.
use super::*;

pub(super) fn export<V: ModelLayout + ParticleConversion + CameraConversion>(
    state: &mut SceneExport<'_, '_, V>,
    index: usize,
    _node: &wr::Node<wr::V19>,
    common: &sm::Node<'_>,
    metadata: &BTreeMap<String, i64>,
    payload: &wr::LightPayload<wr::V19>,
) -> Result<(), Error>
where
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, f32>: ValuesFactory<f32>,
    V::TrackValues<'static, u8>: ValuesFactory<u8>,
    V::TrackValues<'static, u32>: ValuesFactory<u32>,
    V::TrackValues<'static, i16>: ValuesFactory<i16>,
    V::TrackValues<'static, u16>: ValuesFactory<u16>,
    V::SplineValues<'static, f32>: SplineFactory<f32>,
    V::SplineValues<'static, [f32; 3]>: SplineFactory<[f32; 3]>,
    V::TrackValues<'static, [f32; 3]>: ValuesFactory<[f32; 3]>,
{
    let SceneExport {
        model,
        core,
        starts,
        globals,
        context,
    } = state;

    let requested_kind = match payload.kind {
        wr::LightKind::Directional => LightKind::Directional,
        wr::LightKind::Omni => LightKind::Point,
        _ => {
            context.unsupported("lights.kind", "light operation has no native M2 equivalent")?;
            LightKind::Point
        }
    };
    let original_kind = metadata
        .get("m2.light.kind")
        .map(|&value| LightKind::from_raw(value as u16))
        .unwrap_or(requested_kind);
    let projection = if original_kind == LightKind::Directional {
        wr::LightKind::Directional
    } else {
        wr::LightKind::Omni
    };
    let kind = if projection == payload.kind {
        original_kind
    } else {
        requested_kind
    };
    let mut light = native::records::Light::<V> {
        kind,
        bone_id: if actor_has_transform(model, index, common) {
            Some(index as u32)
        } else {
            common.parent
        }
        .map(|value| {
            i16::try_from(value).map_err(|_| Error::new("lights.bone", "light parent exceeds i16"))
        })
        .transpose()?
        .unwrap_or(-1),
        position: common.pivot,
        ..Default::default()
    };
    light.diffuse_color = export_track::<V, [f32; 3]>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: u32::from(wr::Channel::Color.raw()),
        },
        starts,
        globals,
        context,
        |v, _, path| {
            v.try_into()
                .map_err(|_| Error::new(path, "light color must have three components"))
        },
    )?;
    light.ambient_color = export_track::<V, [f32; 3]>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 200,
        },
        starts,
        globals,
        context,
        |v, _, path| {
            v.try_into()
                .map_err(|_| Error::new(path, "ambient color must have three components"))
        },
    )?;
    macro_rules! scalar {
        ($field:ident,$channel:expr) => {
            light.$field = export_track::<V, f32>(
                model,
                sm::Target::Property {
                    owner: index as u32,
                    channel: u32::from($channel),
                },
                starts,
                globals,
                context,
                |v, _, path| {
                    v.first().copied().filter(|_| v.len() == 1).ok_or_else(|| {
                        Error::new(path, "scalar light channel must have one component")
                    })
                },
            )?;
        };
    }
    scalar!(diffuse_intensity, wr::Channel::Intensity.raw());
    scalar!(attenuation_start, wr::Channel::AttenuationStart.raw());
    scalar!(attenuation_end, wr::Channel::AttenuationEnd.raw());
    scalar!(ambient_intensity, 201u8);
    light.visibility = export_track::<V, u8>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: u32::from(wr::Channel::Visibility.raw()),
        },
        starts,
        globals,
        context,
        |v, _, path| byte(v, path),
    )?;
    ensure_constant(&mut light.diffuse_color, payload.color, globals);
    ensure_constant(&mut light.diffuse_intensity, payload.intensity, globals);
    ensure_constant(
        &mut light.attenuation_start,
        payload.attenuation_start,
        globals,
    );
    ensure_constant(&mut light.attenuation_end, payload.attenuation_end, globals);
    ensure_constant(&mut light.visibility, 1u8, globals);
    core.lights.push(light);
    Ok(())
}
