//! Cameras exported from active WEM records.
use super::*;

pub(super) fn export<V: ModelLayout + ParticleConversion + CameraConversion>(
    state: &mut SceneExport<'_, '_, V>,
    index: usize,
    _node: &wr::Node<wr::V19>,
    common: &sm::Node<'_>,
    metadata: &BTreeMap<String, i64>,
    payload: &wr::CameraPayload<wr::V19>,
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

    let mut camera = native::records::Camera::<V> {
        kind: metadata.get("m2.camera.kind").copied().unwrap_or(0) as u32,
        near_clip: payload.near_clip,
        far_clip: payload.far_clip,
        position_base: common.pivot,
        target_position_base: payload.target,
        ..Default::default()
    };
    for (property, target) in [
        (1200, &mut camera.positions),
        (1201, &mut camera.target_positions),
    ] {
        *target = export_spline::<V, [f32; 3]>(
            model,
            sm::Target::Property {
                owner: index as u32,
                channel: property,
            },
            starts,
            globals,
            context,
            |v, _, path| {
                v.try_into()
                    .map_err(|_| Error::new(path, "camera vector needs three components"))
            },
        )?;
    }
    camera.roll = export_spline::<V, f32>(
        model,
        sm::Target::Property {
            owner: index as u32,
            channel: 1202,
        },
        starts,
        globals,
        context,
        |v, _, path| {
            v.first()
                .copied()
                .filter(|_| v.len() == 1)
                .ok_or_else(|| Error::new(path, "roll needs one component"))
        },
    )?;
    V::export_fov(
        &mut camera,
        (
            payload.fov,
            sm::Target::Property {
                owner: index as u32,
                channel: 1203,
            },
        ),
        model,
        starts,
        globals,
        context,
    )?;
    if common.translation!=[0.;3]||common.rotation!=[0.,0.,0.,1.]||common.scale!=[1.;3]||model.clips.iter().flat_map(|clip|&clip.tracks).any(|track|matches!(track.target,sm::Target::Translation(owner)|sm::Target::Rotation(owner)|sm::Target::Scale(owner)if owner as usize==index))||common.parent.is_some(){
        context.unsupported(format!("cameras[{index}].hierarchy"),"M2 cameras need world position/target curves rather than a node-local transform or bone attachment")?;
    }
    core.cameras.push(camera);
    Ok(())
}
