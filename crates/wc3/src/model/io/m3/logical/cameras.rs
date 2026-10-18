//! Authored camera optics, with absent optional controls remaining absent.
use super::{
    bindings::{self, Bindings},
    hierarchy::Skeleton,
    materials::{anim, read_error, revision, string},
};
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::io::m3::logical::GraphBuilder;
use crate::model::profiles::m3::properties::{Field, FieldPath};
use crate::model::{profiles::m3 as surface, OwnerId};
use native::{Envelope, Ref};
trait Optional<T> {
    fn optional(self) -> Option<T>;
}
impl<T> Optional<T> for () {
    fn optional(self) -> Option<T> {
        None
    }
}
impl Optional<f32> for f32 {
    fn optional(self) -> Option<f32> {
        Some(self)
    }
}
impl Optional<u32> for u32 {
    fn optional(self) -> Option<u32> {
        Some(self)
    }
}
impl<T> Optional<nr::AnimationValue<T>> for nr::AnimationValue<T> {
    fn optional(self) -> Option<Self> {
        Some(self)
    }
}
pub(super) fn read<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reference: Ref<nr::targets::Camera>,
    builder: &mut GraphBuilder<'a>,
    skeleton: &Skeleton<'a>,
    registry: &mut Bindings,
) -> Result<(), Error> {
    if reference.is_empty() {
        return Ok(());
    }
    macro_rules! read {
        ($v:ty) => {
            for row in source
                .resolve::<nr::Camera<$v>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                let node = *skeleton.nodes.get(r.bone_index as usize).ok_or_else(|| {
                    Error::new("m3.camera.node", "camera anchor exceeds skeleton")
                })?;
                let feature = builder.reserve_feature(node);
                let mut camera = surface::Camera {
                    name: string(source, r.name)?,
                    use_vertical_f_o_v: r.use_vertical_f_o_v,
                    optics: optics(OpticalFields {
                        legacy: r
                            .reserved_0
                            .optional()
                            .zip(r.reserved_1.optional())
                            .map(|(a, b)| [a, b]),
                        far: r
                            .far_clip
                            .optional()
                            .map(|value| anim(value, builder.values_mut())),
                        near: r
                            .near_clip
                            .optional()
                            .map(|value| anim(value, builder.values_mut())),
                        start: r
                            .near_falloff_start
                            .optional()
                            .map(|value| anim(value, builder.values_mut())),
                        end: r
                            .near_falloff_end
                            .optional()
                            .map(|value| anim(value, builder.values_mut())),
                        mode: r.dof_type.optional(),
                        f_stop: r
                            .bokeh_f_stop
                            .optional()
                            .map(|value| anim(value, builder.values_mut())),
                        maximum_circle: r
                            .bokeh_max_co_c_diameter
                            .optional()
                            .map(|value| anim(value, builder.values_mut())),
                    })?,
                    field_of_view: anim(r.field_of_view, builder.values_mut()),
                    shadow_clip_distance: anim(r.shadow_clip_distance, builder.values_mut()),
                    focus_distance: anim(r.focus_distance, builder.values_mut()),
                    far_focus_range: anim(r.far_focus_range, builder.values_mut()),
                    near_focus_range: anim(r.near_focus_range, builder.values_mut()),
                    dof_amount: anim(r.dof_amount, builder.values_mut()),
                };
                bindings::field(
                    &mut camera.field_of_view,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Camera).child(Field::FieldOfView),
                    registry,
                );
                bindings::field(
                    &mut camera.shadow_clip_distance,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Camera).child(Field::ShadowClipDistance),
                    registry,
                );
                bindings::field(
                    &mut camera.focus_distance,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Camera).child(Field::FocusDistance),
                    registry,
                );
                bindings::field(
                    &mut camera.far_focus_range,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Camera).child(Field::FarFocusRange),
                    registry,
                );
                bindings::field(
                    &mut camera.near_focus_range,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Camera).child(Field::NearFocusRange),
                    registry,
                );
                bindings::field(
                    &mut camera.dof_amount,
                    OwnerId::Feature(feature),
                    &FieldPath::root(Field::Camera).child(Field::DofAmount),
                    registry,
                );
                bind_optics(&mut camera.optics, OwnerId::Feature(feature), registry);
                builder.set_feature(feature, surface::Feature::Camera((camera).into()))?;
            }
        };
    }
    match revision(source, reference).map_err(read_error)? {
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        _ => return Err(Error::new("m3.camera", "unsupported camera revision")),
    }
    Ok(())
}

struct OpticalFields {
    legacy: Option<[f32; 2]>,
    far: Option<surface::PropertyBinding<f32>>,
    near: Option<surface::PropertyBinding<f32>>,
    start: Option<surface::PropertyBinding<f32>>,
    end: Option<surface::PropertyBinding<f32>>,
    mode: Option<u32>,
    f_stop: Option<surface::PropertyBinding<f32>>,
    maximum_circle: Option<surface::PropertyBinding<f32>>,
}
fn optics(fields: OpticalFields) -> Result<surface::CameraOptics, Error> {
    let OpticalFields {
        legacy,
        far,
        near,
        start,
        end,
        mode,
        f_stop,
        maximum_circle,
    } = fields;
    match (legacy, far, near, start, end, mode, f_stop, maximum_circle) {
        (Some(controls), None, None, None, None, None, None, None) => {
            Ok(surface::CameraOptics::Legacy { controls })
        }
        (None, Some(far), Some(near), None, None, None, None, None) => {
            Ok(surface::CameraOptics::Clipping(surface::CameraClipping {
                far,
                near,
            }))
        }
        (None, Some(far), Some(near), Some(start), Some(end), None, None, None) => {
            Ok(surface::CameraOptics::Falloff {
                clipping: surface::CameraClipping { far, near },
                falloff: surface::CameraFalloff { start, end },
            })
        }
        (
            None,
            Some(far),
            Some(near),
            Some(start),
            Some(end),
            Some(mode),
            Some(f_stop),
            Some(maximum_circle),
        ) => Ok(surface::CameraOptics::Bokeh {
            clipping: surface::CameraClipping { far, near },
            falloff: surface::CameraFalloff { start, end },
            bokeh: surface::CameraBokeh {
                mode,
                f_stop,
                maximum_circle,
            },
        }),
        _ => Err(Error::new(
            "m3.camera.optics",
            "incomplete native optical shape",
        )),
    }
}
fn bind_optics(optics: &mut surface::CameraOptics, owner: OwnerId, registry: &mut Bindings) {
    match optics {
        surface::CameraOptics::Legacy { .. } => {}
        surface::CameraOptics::Clipping(clipping) => bind_clipping(clipping, owner, registry),
        surface::CameraOptics::Falloff { clipping, falloff } => {
            bind_clipping(clipping, owner, registry);
            bind_falloff(falloff, owner, registry);
        }
        surface::CameraOptics::Bokeh {
            clipping,
            falloff,
            bokeh,
        } => {
            bind_clipping(clipping, owner, registry);
            bind_falloff(falloff, owner, registry);
            bindings::field(
                &mut bokeh.f_stop,
                owner,
                &FieldPath::root(Field::Camera).child(Field::BokehFStop),
                registry,
            );
            bindings::field(
                &mut bokeh.maximum_circle,
                owner,
                &FieldPath::root(Field::Camera).child(Field::BokehMaxCoCDiameter),
                registry,
            );
        }
    }
}
fn bind_clipping(clipping: &mut surface::CameraClipping, owner: OwnerId, registry: &mut Bindings) {
    bindings::field(
        &mut clipping.far,
        owner,
        &FieldPath::root(Field::Camera).child(Field::FarClip),
        registry,
    );
    bindings::field(
        &mut clipping.near,
        owner,
        &FieldPath::root(Field::Camera).child(Field::NearClip),
        registry,
    );
}
fn bind_falloff(falloff: &mut surface::CameraFalloff, owner: OwnerId, registry: &mut Bindings) {
    bindings::field(
        &mut falloff.start,
        owner,
        &FieldPath::root(Field::Camera).child(Field::NearFalloffStart),
        registry,
    );
    bindings::field(
        &mut falloff.end,
        owner,
        &FieldPath::root(Field::Camera).child(Field::NearFalloffEnd),
        registry,
    );
}

pub(super) fn write<E: Envelope>(
    model: &super::Graph<'_>,
    document: &mut native::Container<'_, E>,
    bones: &std::collections::BTreeMap<crate::model::NodeId, u32>,
    ids: &super::animation::AnimationIds,
) -> Result<Ref<nr::targets::Camera>, Error> {
    use super::write::{animated, append, string};
    let mut legacy = Vec::new();
    let mut clipping = Vec::new();
    let mut falloff = Vec::new();
    let mut bokeh = Vec::new();
    for (_, feature) in model.features() {
        let surface::Feature::Camera(camera) = &feature.payload else {
            continue;
        };
        let bone_index = *bones
            .get(&feature.node)
            .ok_or_else(|| Error::new("m3.camera.node", "camera anchor has no native bone"))?;
        let name = string(document, &camera.name)?;
        macro_rules! camera {
            ($revision:ty,$reserved0:expr,$reserved1:expr,$dof:expr,$far:expr,$near:expr,$start:expr,$end:expr,$stop:expr,$circle:expr) => {
                nr::Camera::<$revision> {
                    bone_index,
                    name,
                    use_vertical_f_o_v: camera.use_vertical_f_o_v,
                    reserved_0: $reserved0,
                    reserved_1: $reserved1,
                    dof_type: $dof,
                    field_of_view: animated(&camera.field_of_view, ids),
                    shadow_clip_distance: animated(&camera.shadow_clip_distance, ids),
                    focus_distance: animated(&camera.focus_distance, ids),
                    far_focus_range: animated(&camera.far_focus_range, ids),
                    near_focus_range: animated(&camera.near_focus_range, ids),
                    dof_amount: animated(&camera.dof_amount, ids),
                    far_clip: $far,
                    near_clip: $near,
                    near_falloff_start: $start,
                    near_falloff_end: $end,
                    bokeh_f_stop: $stop,
                    bokeh_max_co_c_diameter: $circle,
                }
            };
        }
        match &camera.optics {
            surface::CameraOptics::Legacy { controls } => legacy.push(camera!(
                nr::V2,
                controls[0],
                controls[1],
                (),
                (),
                (),
                (),
                (),
                (),
                ()
            )),
            surface::CameraOptics::Clipping(c) => clipping.push(camera!(
                nr::V3,
                (),
                (),
                (),
                animated(&c.far, ids),
                animated(&c.near, ids),
                (),
                (),
                (),
                ()
            )),
            surface::CameraOptics::Falloff {
                clipping: c,
                falloff: f,
            } => falloff.push(camera!(
                nr::V4,
                (),
                (),
                (),
                animated(&c.far, ids),
                animated(&c.near, ids),
                animated(&f.start, ids),
                animated(&f.end, ids),
                (),
                ()
            )),
            surface::CameraOptics::Bokeh {
                clipping: c,
                falloff: f,
                bokeh: b,
            } => bokeh.push(camera!(
                nr::V5,
                (),
                (),
                b.mode,
                animated(&c.far, ids),
                animated(&c.near, ids),
                animated(&f.start, ids),
                animated(&f.end, ids),
                animated(&b.f_stop, ids),
                animated(&b.maximum_circle, ids)
            )),
        }
    }
    let populated = [
        !legacy.is_empty(),
        !clipping.is_empty(),
        !falloff.is_empty(),
        !bokeh.is_empty(),
    ]
    .into_iter()
    .filter(|x| *x)
    .count();
    if populated > 1 {
        return Err(Error::new(
            "m3.camera.optics",
            "one native camera table cannot mix incompatible complete optical shapes",
        ));
    }
    if !legacy.is_empty() {
        Ok(append(document, &legacy)?.cast())
    } else if !clipping.is_empty() {
        Ok(append(document, &clipping)?.cast())
    } else if !falloff.is_empty() {
        Ok(append(document, &falloff)?.cast())
    } else {
        Ok(append(document, &bokeh)?.cast())
    }
}
