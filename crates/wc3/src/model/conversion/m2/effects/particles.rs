//! Particles exported from active WEM records.
use super::*;

pub(super) fn export<V: ModelLayout + ParticleConversion + CameraConversion>(
    state: &mut SceneExport<'_, '_, V>,
    index: usize,
    _node: &wr::Node<wr::V19>,
    common: &sm::Node<'_>,
    _metadata: &BTreeMap<String, i64>,
    payload: &wr::M2ParticleEmitterPayload<wr::V19>,
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

    let mut particle = V::export_particle(
        payload,
        model.native.as_ref().unwrap().document.raw(),
        context,
    )?;
    particle.bone_id = u16::try_from(actor_bone(model, index, common))
        .map_err(|_| Error::new("particles.bone", "bone index exceeds u16"))?;
    particle.position = common.pivot;
    macro_rules! scalar {
        ($field:ident,$property:expr) => {
            particle.$field = export_track::<V, f32>(
                model,
                sm::Target::Property {
                    owner: index as u32,
                    channel: $property,
                },
                starts,
                globals,
                context,
                |v, _, path| {
                    v.first().copied().filter(|_| v.len() == 1).ok_or_else(|| {
                        Error::new(path, "particle scalar key must have one component")
                    })
                },
            )?;
        };
    }
    scalar!(emission_speed, 1000);
    scalar!(speed_variation, 1001);
    scalar!(vertical_range, 1002);
    scalar!(horizontal_range, 1003);
    scalar!(lifespan, 1005);
    scalar!(emission_rate, 1006);
    scalar!(emission_area_width, 1007);
    scalar!(emission_area_length, 1008);
    scalar!(z_source, 1009);
    particle.gravity = export_track::<V, u32>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1004,
        },
        starts,
        globals,
        context,
        |v, context, path| {
            gravity_word(
                v.try_into()
                    .map_err(|_| Error::new(path, "gravity needs three components"))?,
                payload.flags & 0x800000 != 0,
                context,
                path,
            )
        },
    )?;
    particle.enabled = export_track::<V, u8>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1010,
        },
        starts,
        globals,
        context,
        |v, _, path| byte(v, path),
    )?;
    ensure_constant(&mut particle.emission_speed, payload.speed, globals);
    ensure_constant(
        &mut particle.speed_variation,
        payload.speed_variation,
        globals,
    );
    ensure_constant(
        &mut particle.vertical_range,
        payload.vertical_range,
        globals,
    );
    ensure_constant(
        &mut particle.horizontal_range,
        payload.horizontal_range,
        globals,
    );
    ensure_constant(
        &mut particle.gravity,
        gravity_word(
            payload.gravity,
            payload.flags & 0x800000 != 0,
            context,
            "particles.gravity",
        )?,
        globals,
    );
    ensure_constant(&mut particle.lifespan, payload.lifespan, globals);
    ensure_constant(&mut particle.emission_rate, payload.emission_rate, globals);
    ensure_constant(&mut particle.emission_area_width, payload.width, globals);
    ensure_constant(&mut particle.emission_area_length, payload.length, globals);
    ensure_constant(&mut particle.z_source, payload.z_source, globals);
    ensure_constant(&mut particle.enabled, 1u8, globals);
    core.flags |= V::required_flags(&particle);
    core.particles.push(particle);
    Ok(())
}
