//! Camera optics and independently named depth-of-field controls.
use super::metadata::{restore, set_text};
use super::{animated, install, native, nr, read_error, revision, wem, wr, Envelope, Error, Ref};
use std::collections::BTreeMap;
trait Slot<T> {
    fn value(self) -> T;
}
impl<T: Default> Slot<T> for () {
    fn value(self) -> T {
        T::default()
    }
}
impl Slot<u32> for u32 {
    fn value(self) -> Self {
        self
    }
}
impl Slot<nr::AnimationValue<f32>> for nr::AnimationValue<f32> {
    fn value(self) -> Self {
        self
    }
}
fn camera<E: Envelope, D: nr::CameraDialect>(
    source: &native::Document<'_, E>,
    r: nr::Camera<D>,
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut Vec<wr::Node<wr::V19>>,
) -> Result<(), Error>
where
    D::DofType: Slot<u32>,
    D::FarClip: Slot<nr::AnimationValue<f32>>,
    D::NearClip: Slot<nr::AnimationValue<f32>>,
    D::NearFalloffStart: Slot<nr::AnimationValue<f32>>,
    D::NearFalloffEnd: Slot<nr::AnimationValue<f32>>,
    D::BokehFStop: Slot<nr::AnimationValue<f32>>,
    D::BokehMaxCoCDiameter: Slot<nr::AnimationValue<f32>>,
{
    let mut values = BTreeMap::new();
    values.insert(
        "m3.camera.vertical_fov".into(),
        i64::from(r.use_vertical_f_o_v),
    );
    values.insert("m3.camera.dof_type".into(), i64::from(r.dof_type.value()));
    let far_clip = r.far_clip.value();
    let near_clip = r.near_clip.value();
    let fstop = r.bokeh_f_stop.value();
    for (key, value) in [
        ("camera.fov", r.field_of_view),
        ("camera.far_clip", far_clip),
        ("camera.near_clip", near_clip),
        ("camera.focus_distance", r.focus_distance),
        ("camera.fstop", fstop),
    ] {
        animated(&mut values, key, value);
    }
    for (key, value) in [
        ("shadow_clip_distance", r.shadow_clip_distance),
        ("far_focus_range", r.far_focus_range),
        ("near_focus_range", r.near_focus_range),
        ("near_falloff_start", r.near_falloff_start.value()),
        ("near_falloff_end", r.near_falloff_end.value()),
        ("dof_amount", r.dof_amount),
        ("bokeh_max_coc", r.bokeh_max_co_c_diameter.value()),
    ] {
        animated(&mut values, &format!("camera.{key}"), value);
        values.insert(
            format!("m3.camera.{key}.initial"),
            i64::from(value.initial_value.to_bits()),
        );
    }
    let payload = wr::CameraPayload::<wr::V19> {
        fov: r.field_of_view.initial_value,
        near_clip: near_clip.initial_value,
        far_clip: far_clip.initial_value,
        target: [0.; 3],
        focus_distance: r.focus_distance.initial_value,
        focal_length: 0.,
        f_stop: fstop.initial_value,
    };
    let name = source
        .string(r.name)
        .map_err(read_error)?
        .unwrap_or_default();
    values.insert("m3.camera.named".into(), 1);
    install(
        doc,
        nodes,
        r.bone_index,
        wr::NodePayload::Camera(Box::new(payload)),
        wr::NodeKind::Camera,
        values,
    )?;
    let node = nodes
        .last_mut()
        .ok_or_else(|| Error::new("m3.camera", "missing camera wrapper"))?;
    node.native = set_text(doc, &node.native, "m3.camera.name", name)?;
    Ok(())
}
pub(super) fn attach<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::Camera>,
    doc: &mut wem::RawDocument<'static>,
    nodes: &mut Vec<wr::Node<wr::V19>>,
) -> Result<(), Error> {
    if reference.entries == 0 {
        return Ok(());
    }
    macro_rules! read {
        ($v:ty) => {
            for r in source
                .resolve::<nr::Camera<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                camera(source, r.map_err(read_error)?, doc, nodes)?;
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        _ => return Err(Error::new("m3.cameras", "unsupported camera revision")),
    }
    Ok(())
}
pub(super) fn export<E: Envelope>(
    index: usize,
    name: &str,
    payload: wr::CameraPayload<wr::V19>,
    values: &BTreeMap<String, i64>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::Camera<nr::V5>, Error> {
    if payload.target != [0.; 3] || payload.focal_length != 0. {
        return Err(Error::new(
            format!("nodes[{index}].camera"),
            "target point and focal length need an explicit M3 transform/optics conversion",
        ));
    }
    let initial = |key| {
        f32::from_bits(
            values
                .get(&format!("m3.camera.{key}.initial"))
                .copied()
                .unwrap_or(0) as u32,
        )
    };
    Ok(nr::Camera {
        bone_index: index as u32,
        name: super::super::append_string(doc, name)?,
        field_of_view: restore(values, "camera.fov", payload.fov),
        use_vertical_f_o_v: values.get("m3.camera.vertical_fov").copied().unwrap_or(0) as u32,
        dof_type: values.get("m3.camera.dof_type").copied().unwrap_or(0) as u32,
        far_clip: restore(values, "camera.far_clip", payload.far_clip),
        near_clip: restore(values, "camera.near_clip", payload.near_clip),
        focus_distance: restore(values, "camera.focus_distance", payload.focus_distance),
        bokeh_f_stop: restore(values, "camera.fstop", payload.f_stop),
        shadow_clip_distance: restore(
            values,
            "camera.shadow_clip_distance",
            initial("shadow_clip_distance"),
        ),
        far_focus_range: restore(values, "camera.far_focus_range", initial("far_focus_range")),
        near_focus_range: restore(
            values,
            "camera.near_focus_range",
            initial("near_focus_range"),
        ),
        near_falloff_start: restore(
            values,
            "camera.near_falloff_start",
            initial("near_falloff_start"),
        ),
        near_falloff_end: restore(
            values,
            "camera.near_falloff_end",
            initial("near_falloff_end"),
        ),
        dof_amount: restore(values, "camera.dof_amount", initial("dof_amount")),
        bokeh_max_co_c_diameter: restore(values, "camera.bokeh_max_coc", initial("bokeh_max_coc")),
        ..Default::default()
    })
}
