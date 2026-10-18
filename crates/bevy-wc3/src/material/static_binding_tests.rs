//! Runtime binding pruning preserves constants and every programme owner.
use super::*;
use crate::animation_targets::AnimationTargets;
use crate::material::native_fixtures::m2;
use wc3::model::profiles::{mdlx, Universal};
use wc3::model::ModelMetadata;
use wc3::model::{
    Animation, AnimationSample, Clip, Curve, DynamicTrack, Material, MaterialId, MaterialSurface,
    Model, ModelBuilder, SharedClock, Time as ModelTime, TimeSource, Track,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

#[test]
fn m2_bound_without_channels_preserves_defaults_and_allows_shared_materials() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let material = builder.reserve_material();
    let mut surface = m2::Material {
        color: slot_values.add([0.2, 0.4, 0.6]),
        opacity: slot_values.add(0.75),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let texture: m2::TextureUnit = m2::TextureUnit {
        contribution: slot_values.add(0.8),
        ..SlotConstruct::construct(&mut slot_values)
    };

    surface.textures.push(texture);
    builder
        .set_material(
            material,
            Material {
                name: "bound".into(),
                surface: MaterialSurface::from(surface),
            },
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let mut pass =
        compile_material_with_images(&model, material, Arc::new(TextureImages::default()))
            .unwrap()
            .remove(0);
    assert!(
        !pass.animation.is_animated(),
        "constant passes should use the shared material cache"
    );
    assert_eq!(
        pass.material.base_color.to_linear(),
        LinearRgba::new(0.2, 0.4, 0.6, 0.75 * 0.8)
    );
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    let before = pass.material.base_color;
    pass.animation.apply(&sample, &mut pass.material);
    assert_eq!(pass.material.base_color, before);
}

#[test]
fn pruning_retains_targets_from_timeline_shared_unselected_clip_and_animation() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let nodes: Vec<_> = (0..5)
        .map(|i| builder.add_named_node(format!("node {i}")).unwrap())
        .collect();
    let translations: Vec<_> = nodes
        .iter()
        .map(|id| builder.model().node(*id).unwrap().transform.translation)
        .collect();
    let track = |target, clock| {
        DynamicTrack::Vector3(Track {
            target,
            clock,
            curve: Curve::linear(
                [
                    ModelTime::seconds(0.).unwrap(),
                    ModelTime::seconds(1.).unwrap(),
                ],
                [[0.; 3], [1.; 3]],
            )
            .unwrap(),
        })
    };
    builder
        .add_global_track(track(translations[0], TimeSource::Animation))
        .unwrap();
    let clock = builder
        .add_clock(SharedClock::Repeating {
            period: ModelTime::seconds(1.).unwrap(),
        })
        .unwrap();
    builder
        .add_global_track(track(translations[1], TimeSource::Shared(clock)))
        .unwrap();
    let clip = builder
        .add_clip(
            Clip::named("not currently selected").with_duration(ModelTime::seconds(1.).unwrap()),
        )
        .unwrap();
    builder
        .add_track(clip, track(translations[2], TimeSource::Animation))
        .unwrap();
    let animation = builder.add_animation(Animation::default());
    builder
        .add_animation_track(animation, track(translations[3], TimeSource::Animation))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let targets = AnimationTargets::new(&model);
    for (index, node) in nodes.into_iter().enumerate() {
        let mut animation = SurfaceAnimation {
            color: Some(model.node(node).unwrap().transform.translation),
            ..SlotConstruct::construct(&mut slot_values)
        };
        animation.prune_static_bindings(&targets);
        assert_eq!(animation.is_animated(), index < 4);
        assert_eq!(
            animation.color.unwrap(),
            model.node(node).unwrap().transform.translation
        );
    }
}

#[test]
fn pruning_visits_every_uv_transform_weight_and_hd_control() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("untracked").unwrap();
    let material = builder.reserve_material();
    builder
        .set_material(
            material,
            Material {
                name: "owner".into(),
                surface: SlotConstruct::construct(&mut slot_values),
            },
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let scalar_slot = slot_values.add(0.375);
    let scalar = || scalar_slot;
    let mut transform = TextureTransform::construct(&mut slot_values);
    transform.translation = model.node(node).unwrap().transform.translation;
    transform.rotation = model.node(node).unwrap().transform.rotation;
    transform.scale = model.node(node).unwrap().transform.scale;
    let texture = slot_values.add(None);
    let mut animation = SurfaceAnimation {
        opacity: Some(scalar()),
        weight: Some(scalar()),
        pass_weights: vec![scalar()],
        input_weights: std::array::from_fn(|_| Some(scalar())),
        transform: Some(transform.clone()),
        secondary_transform: Some(transform.clone()),
        third_transform: Some(transform.clone()),
        fourth_transform: Some(transform),
        emissive_gain: Some(scalar()),
        texture: Some(texture),
        hd: Some(HdAnimation {
            slots: std::array::from_fn(|_| Some(texture)),
            gain: Some(scalar()),
            fresnel: Some(mdlx::Fresnel {
                color: slot_values.add([0.2; 3]),
                opacity: scalar(),
                team_color: scalar(),
            }),
            images: Arc::default(),
        }),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut model = model;
    model
        .edit(|builder| builder.import_values(&slot_values))
        .unwrap();
    animation.prune_static_bindings(&AnimationTargets::new(&model));
    assert!(!animation.is_animated());
    assert_eq!(model.values()[animation.weight.unwrap()], 0.375);
    let fresnel = animation.hd.unwrap().fresnel.unwrap();
    assert_eq!(model.values()[fresnel.color], [0.2; 3]);
    assert_eq!(model.values()[fresnel.opacity], 0.375);
}

fn scalar_material() -> (ModelBuilder<'static>, MaterialId) {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let material = builder.reserve_material();
    let surface = m2::Material {
        color: slot_values.add([1.; 3]),
        opacity: slot_values.add(1.),
        ..SlotConstruct::construct(&mut slot_values)
    };
    builder
        .set_material(
            material,
            Material {
                name: "scalar".into(),
                surface: MaterialSurface::from(surface),
            },
        )
        .unwrap();
    builder.import_values(&slot_values).unwrap();
    (builder, material)
}

#[test]
fn constant_scalar_tracks_matching_default_do_not_require_private_materials() {
    let slot_values = SlotValues::default();
    for linear in [false, true] {
        let (mut builder, material) = scalar_material();
        let times = [
            ModelTime::seconds(0.).unwrap(),
            ModelTime::seconds(1.).unwrap(),
        ];
        let curve = if linear {
            Curve::linear(times, [1., 1.]).unwrap()
        } else {
            Curve::step(vec![times[0]], vec![1.]).unwrap()
        };
        builder
            .add_global_track(DynamicTrack::Scalar(Track {
                target: opacity(&builder, material),
                clock: TimeSource::Animation,
                curve,
            }))
            .unwrap();
        let model = {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap();
        let pass = compile_material_with_images(&model, material, Arc::default())
            .unwrap()
            .remove(0);
        assert!(!pass.animation.is_animated());
        assert!(pass.animation.opacity.is_some());
        assert_eq!(pass.material.base_color.alpha(), 1.);
    }
}

#[test]
fn conflicting_clip_constants_and_animation_overrides_keep_materials_live() {
    let slot_values = SlotValues::default();
    for animation_override in [false, true] {
        let (mut builder, material) = scalar_material();
        let time = ModelTime::seconds(0.).unwrap();
        let target = opacity(&builder, material);
        let track = |value, clock| {
            DynamicTrack::Scalar(Track {
                target,
                clock,
                curve: Curve::step(vec![time], vec![value]).unwrap(),
            })
        };
        let clip = builder
            .add_clip(Clip::named("first").with_duration(ModelTime::seconds(1.).unwrap()))
            .unwrap();
        builder
            .add_track(clip, track(1., TimeSource::Animation))
            .unwrap();
        if animation_override {
            let animation = builder.add_animation(Animation::default());
            builder
                .add_animation_track(animation, track(0.25, TimeSource::Animation))
                .unwrap();
        } else {
            let other = builder
                .add_clip(Clip::named("second").with_duration(ModelTime::seconds(1.).unwrap()))
                .unwrap();
            builder
                .add_track(other, track(0.5, TimeSource::Animation))
                .unwrap();
        }
        let model = {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap();
        let pass = compile_material_with_images(&model, material, Arc::default())
            .unwrap()
            .remove(0);
        assert!(pass.animation.is_animated());
        assert!(pass.animation.opacity.is_some());
    }
}

#[test]
fn constant_track_different_from_fallback_cannot_be_folded() {
    let mut slot_values = SlotValues::default();
    let (mut builder, material) = scalar_material();
    let clip = builder
        .add_clip(
            Clip::named("only selected sometimes").with_duration(ModelTime::seconds(1.).unwrap()),
        )
        .unwrap();
    builder
        .add_track(
            clip,
            DynamicTrack::Scalar(Track {
                target: opacity(&builder, material),
                clock: TimeSource::Animation,
                curve: Curve::step(vec![ModelTime::default()], vec![0.5]).unwrap(),
            }),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let pass = compile_material_with_images(&model, material, Arc::default())
        .unwrap()
        .remove(0);
    assert!(pass.animation.is_animated());
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    let mut unselected = pass.material.clone();
    pass.animation.apply(&sample, &mut unselected);
    assert_eq!(unselected.base_color.alpha(), 1.);
    let sample = model
        .animation_sample(AnimationSample {
            clip: Some(clip),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    pass.animation.apply(&sample, &mut unselected);
    assert_eq!(unselected.base_color.alpha(), 0.5);
}

#[test]
fn equal_spline_endpoints_do_not_prove_constant_scalar_output() {
    let slot_values = SlotValues::default();
    let (mut builder, material) = scalar_material();
    let times = [
        ModelTime::seconds(0.).unwrap(),
        ModelTime::seconds(1.).unwrap(),
    ];
    builder
        .add_global_track(DynamicTrack::Scalar(Track {
            target: opacity(&builder, material),
            clock: TimeSource::Animation,
            curve: Curve::hermite(times, [1., 1.], [0., -2.], [2., 0.]).unwrap(),
        }))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let pass = compile_material_with_images(&model, material, Arc::default())
        .unwrap()
        .remove(0);
    assert!(pass.animation.is_animated());
    assert!(pass.animation.opacity.is_some());
}

#[test]
fn folding_constant_weights_preserves_changing_uv_channels() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let node = builder.add_named_node("UV source").unwrap();
    let material = builder.reserve_material();
    let mut unit: m2::TextureUnit = m2::TextureUnit {
        contribution: slot_values.add(1.),
        ..SlotConstruct::construct(&mut slot_values)
    };
    unit.coordinates.transform.translation =
        builder.model().node(node).unwrap().transform.translation;
    let weight = unit.contribution;
    let surface = m2::Material {
        color: slot_values.add([1.; 3]),
        opacity: slot_values.add(1.),
        textures: vec![unit],
        ..SlotConstruct::construct(&mut slot_values)
    };
    builder
        .set_material(
            material,
            Material {
                name: "UV".into(),
                surface: MaterialSurface::from(surface),
            },
        )
        .unwrap();
    builder
        .add_global_track(DynamicTrack::Scalar(Track {
            target: weight,
            clock: TimeSource::Animation,
            curve: Curve::step(vec![ModelTime::default()], vec![1.]).unwrap(),
        }))
        .unwrap();
    builder
        .add_global_track(DynamicTrack::Vector3(Track {
            target: builder.model().node(node).unwrap().transform.translation,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [
                    ModelTime::seconds(0.).unwrap(),
                    ModelTime::seconds(1.).unwrap(),
                ],
                [[0.; 3], [0., 1., 0.]],
            )
            .unwrap(),
        }))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let pass = compile_material_with_images(&model, material, Arc::default())
        .unwrap()
        .remove(0);
    assert!(pass.animation.weight.is_some());
    assert!(pass.animation.is_animated());
    let mut value = pass.material;
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: ModelTime::seconds(0.5).unwrap(),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    pass.animation.apply(&sample, &mut value);
    assert!((value.uv_transform.translation.y - 0.5).abs() < 1e-6);
}

fn opacity(builder: &ModelBuilder<'static>, id: MaterialId) -> Property<f32> {
    let wc3::model::MaterialStep::M2(step) =
        &builder.model().material(id).unwrap().surface.steps[0]
    else {
        panic!("M2")
    };
    step.declaration().opacity
}
