//! Revision-aware access to WEM model and node records.
use super::*;

pub(super) fn graph(
    raw: &RawDocument<'_>,
    reference: Ref<wr::targets::Model>,
    index: usize,
) -> Result<wr::ModelRecord<wr::V9>, Error> {
    let section = raw
        .referenced_section(reference)?
        .ok_or_else(|| Error::new("model", "missing model section"))?;
    macro_rules! row {
        ($v:ty) => {{
            let value = raw
                .resolve::<wr::ModelRecord<$v>>(reference.cast())?
                .get(index)
                .transpose()?
                .ok_or_else(|| Error::new("model", "model index out of bounds"))?;
            wr::ModelRecord::<wr::V9> {
                name: value.name,
                meshes: value.meshes,
                nodes: wr::NodeTree {
                    nodes: value.nodes.nodes,
                    pose_schema: value.nodes.pose_schema,
                    authoritative_pose: value.nodes.authoritative_pose,
                    ..Default::default()
                },
                material_slots: value.material_slots,
                anim_channels: value.anim_channels,
                anim_set: value.anim_set,
                profile_sets: value.profile_sets,
                bounds: value.bounds,
                ..Default::default()
            }
        }};
    }
    Ok(match section.revision() {
        1 => row!(wr::V1),
        2 => row!(wr::V2),
        3 => row!(wr::V3),
        4 => row!(wr::V4),
        5 => row!(wr::V5),
        6 => row!(wr::V6),
        7 => row!(wr::V7),
        8 => row!(wr::V8),
        9 => row!(wr::V9),
        _ => return Err(Error::new("model", "unsupported model section revision")),
    })
}
fn optional<T: wem::Write, U: for<'data> wem::Read<'data>>(value: &T) -> Result<Option<U>, Error> {
    let mut output = wem::Encoder::new(19);
    output.write(value)?;
    let bytes = output.finish();
    if bytes.is_empty() {
        return Ok(None);
    }
    let mut decoder = wem::Decoder::new(&bytes, 19);
    Ok(Some(decoder.read()?))
}
fn normalize<D>(record: wr::Node<D>) -> Result<wr::Node<wr::V19>, Error>
where
    D: wr::NodeDialect<Payload = wr::NodePayload<D>> + wr::NodePayloadDialect,
    <D as wr::BonePayloadDialect>::GateMesh: wem::Write,
    <D as wr::LightPayloadDialect>::ShadowCasting: wem::Write,
    <D as wr::LightPayloadDialect>::ShadowCastingStart: wem::Write,
    <D as wr::LightPayloadDialect>::ShadowCastingEnd: wem::Write,
    <D as wr::LightPayloadDialect>::QuadraticFalloff: wem::Write,
    <D as wr::LightPayloadDialect>::LinearFalloff: wem::Write,
    <D as wr::LightPayloadDialect>::Damping: wem::Write,
    <D as wr::CameraPayloadDialect>::Target: wem::Write,
    <D as wr::CameraPayloadDialect>::FocusDistance: wem::Write,
    <D as wr::CameraPayloadDialect>::FocalLength: wem::Write,
    <D as wr::CameraPayloadDialect>::FStop: wem::Write,
    <D as wr::EventPayloadDialect>::Category: wem::Write,
    <D as wr::EventPayloadDialect>::Code: wem::Write,
    <D as wr::NodeDialect>::PoseMatrices: wem::Write,
{
    let payload = match record.payload {
        wr::NodePayload::Helper(value) => wr::NodePayload::Helper(value),
        wr::NodePayload::Bone(value) => {
            wr::NodePayload::Bone(Box::new(wr::BonePayload::<wr::V19> {
                bounds: value.bounds,
                sphere: value.sphere,
                gate_mesh: optional(&value.gate_mesh)?.unwrap_or(u32::MAX),
            }))
        }
        wr::NodePayload::Attachment(value) => wr::NodePayload::Attachment(value),
        wr::NodePayload::Light(value) => {
            wr::NodePayload::Light(Box::new(wr::LightPayload::<wr::V19> {
                kind: value.kind,
                color: value.color,
                intensity: value.intensity,
                attenuation_start: value.attenuation_start,
                attenuation_end: value.attenuation_end,
                hot_spot: value.hot_spot,
                falloff: value.falloff,
                shadow_casting: optional(&value.shadow_casting)?.unwrap_or_default(),
                shadow_casting_start: optional(&value.shadow_casting_start)?.unwrap_or_default(),
                shadow_casting_end: optional(&value.shadow_casting_end)?.unwrap_or_default(),
                quadratic_falloff: optional(&value.quadratic_falloff)?.unwrap_or(0.0005),
                linear_falloff: optional(&value.linear_falloff)?.unwrap_or_default(),
                damping: optional(&value.damping)?.unwrap_or(0.00001),
            }))
        }
        wr::NodePayload::Camera(value) => {
            wr::NodePayload::Camera(Box::new(wr::CameraPayload::<wr::V19> {
                fov: value.fov,
                near_clip: value.near_clip,
                far_clip: value.far_clip,
                target: optional(&value.target)?.unwrap_or_default(),
                focus_distance: optional(&value.focus_distance)?.unwrap_or_default(),
                focal_length: optional(&value.focal_length)?.unwrap_or_default(),
                f_stop: optional(&value.f_stop)?.unwrap_or_default(),
            }))
        }
        wr::NodePayload::Event(value) => {
            wr::NodePayload::Event(Box::new(wr::EventPayload::<wr::V19> {
                id: value.id,
                category: optional(&value.category)?.unwrap_or_default(),
                code: optional(&value.code)?.unwrap_or_default(),
            }))
        }
        wr::NodePayload::Collision(value) => wr::NodePayload::Collision(value),
        wr::NodePayload::Wc3ParticleEmitter1(value) => wr::NodePayload::Wc3ParticleEmitter1(value),
        wr::NodePayload::Wc3ParticleEmitter2(value) => wr::NodePayload::Wc3ParticleEmitter2(value),
        wr::NodePayload::Wc3RibbonEmitter(value) => wr::NodePayload::Wc3RibbonEmitter(value),
        wr::NodePayload::Wc3CornEmitter(value) => wr::NodePayload::Wc3CornEmitter(value),
        wr::NodePayload::Wc3FaceFx(value) => wr::NodePayload::Wc3FaceFx(value),
        _ => {
            return Err(Error::new(
                "node.payload",
                "this source profile node payload has no MDLX mapping",
            ));
        }
    };
    Ok(wr::Node::<wr::V19> {
        name: record.name,
        parent: record.parent,
        kind: record.kind,
        flags: record.flags,
        pivot: record.pivot,
        local: record.local,
        uniform_scale_only: record.uniform_scale_only,
        poses: record.poses,
        pose_matrices: optional(&record.pose_matrices)?.unwrap_or_default(),
        payload,
        native: record.native,
        ..Default::default()
    })
}
pub(super) fn nodes(
    raw: &RawDocument<'_>,
    reference: Ref<wr::targets::Node>,
) -> Result<Vec<wr::Node<wr::V19>>, Error> {
    if reference.is_empty() {
        return Ok(Vec::new());
    }
    let section = raw
        .referenced_section(reference)?
        .ok_or_else(|| Error::new("nodes", "missing section"))?;
    macro_rules! rows {
        ($v:ty) => {
            raw.resolve::<wr::Node<$v>>(reference.cast())?
                .iter()
                .map(|value| normalize(value?))
                .collect::<Result<Vec<_>, Error>>()?
        };
    }
    Ok(match section.revision() {
        1 => rows!(wr::V1),
        2 => rows!(wr::V2),
        3 => rows!(wr::V3),
        4 => rows!(wr::V4),
        5 => rows!(wr::V5),
        6 => rows!(wr::V6),
        7 => rows!(wr::V7),
        8 => rows!(wr::V8),
        9 => rows!(wr::V9),
        10 => rows!(wr::V10),
        11 => rows!(wr::V11),
        12 => rows!(wr::V12),
        13 => rows!(wr::V13),
        14 => rows!(wr::V14),
        15 => rows!(wr::V15),
        16 => rows!(wr::V16),
        17 => rows!(wr::V17),
        18 => rows!(wr::V18),
        19 => raw
            .resolve::<wr::Node<wr::V19>>(reference.cast())?
            .iter()
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(Error::new("nodes", "unsupported section revision")),
    })
}
