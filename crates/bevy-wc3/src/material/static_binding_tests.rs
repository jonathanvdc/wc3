//! Runtime binding pruning preserves constants and every programme owner.
use super::*;
use crate::animation_targets::AnimationTargets;
use wc3::model::profiles::{m2, mdlx, Universal};
use wc3::model::{
    AnimationSample, Channel, Clip, Clock, Controller, Curve, Material, MaterialId,
    MaterialSurface, Model, ModelBuilder, Node, NodeId, SharedClock, Time as ModelTime, Track,
};

#[test]
fn m2_bound_without_channels_preserves_defaults_and_allows_shared_materials() {
    let mut builder = Model::<Universal>::builder("bound constant M2 material");
    let material = builder.reserve_material();
    let mut surface = m2::MaterialStep {
        color: Animated {
            default: [0.2, 0.4, 0.6],
            binding: Some(m2::properties::COLOR.bind(material).widen()),
        },
        opacity: Animated {
            default: 0.75,
            binding: Some(m2::properties::OPACITY.bind(material).widen().into()),
        },
        ..Default::default()
    };
    let unit = m2::properties::TextureUnit(0);
    let mut texture: m2::TextureUnit = m2::TextureUnit {
        contribution: Animated {
            default: 0.8,
            binding: Some(unit.weight().bind(material).widen().into()),
        },
        ..Default::default()
    };
    texture.coordinates.transform.translation.binding =
        Some(unit.translation().bind(material).widen());
    texture.coordinates.transform.rotation.binding = Some(unit.rotation().bind(material).widen());
    texture.coordinates.transform.scale.binding = Some(unit.scale().bind(material).widen());
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
    let model = builder.finish().unwrap();
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
fn pruning_retains_targets_from_timeline_shared_unselected_clip_and_controller() {
    let mut builder = Model::<Universal>::builder("all channel owners");
    let nodes: Vec<_> = (0..5)
        .map(|i| builder.add_node(Node::named(format!("node {i}"))).unwrap())
        .collect();
    let track = |node: NodeId, clock| {
        Channel::Vector3(Track {
            target: node.translation(),
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
        .add_timeline_channel(track(nodes[0], Clock::Timeline))
        .unwrap();
    let clock = builder
        .add_clock(SharedClock::Repeating {
            period: ModelTime::seconds(1.).unwrap(),
        })
        .unwrap();
    builder
        .add_shared_channel(track(nodes[1], Clock::Shared(clock)))
        .unwrap();
    let clip = builder
        .add_clip(
            Clip::named("not currently selected").with_duration(ModelTime::seconds(1.).unwrap()),
        )
        .unwrap();
    builder
        .add_channel(clip, track(nodes[2], Clock::Clip))
        .unwrap();
    let controller = builder.add_controller(Controller::default()).unwrap();
    builder
        .add_controller_channel(controller, track(nodes[3], Clock::Timeline))
        .unwrap();
    let model = builder.finish().unwrap();
    let targets = AnimationTargets::new(&model);
    for (index, node) in nodes.into_iter().enumerate() {
        let mut animation = SurfaceAnimation {
            color: Some(Animated {
                default: [0.25; 3],
                binding: Some(node.translation()),
            }),
            ..Default::default()
        };
        animation.prune_static_bindings(&targets);
        assert_eq!(animation.is_animated(), index < 4);
        assert_eq!(animation.color.unwrap().default, [0.25; 3]);
    }
}

#[test]
fn pruning_visits_every_uv_transform_weight_and_hd_control() {
    let mut builder = Model::<Universal>::builder("nested runtime bindings");
    let node = builder.add_node(Node::named("untracked")).unwrap();
    let material = builder.reserve_material();
    builder
        .set_material(
            material,
            Material {
                name: "owner".into(),
                surface: Default::default(),
            },
        )
        .unwrap();
    let model = builder.finish().unwrap();
    let scalar = || Animated {
        default: 0.375,
        binding: Some(material.opacity().into()),
    };
    let mut transform = TextureTransform::default();
    transform.translation.binding = Some(node.translation());
    transform.rotation.binding = Some(node.rotation());
    transform.scale.binding = Some(node.scale());
    let texture = Animated {
        default: None,
        binding: Some(mdlx::properties::Layer(0).texture().bind(material).widen()),
    };
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
        texture: Some(texture.clone()),
        hd: Some(HdAnimation {
            slots: std::array::from_fn(|_| Some(texture.clone())),
            gain: Some(scalar()),
            fresnel: Some(mdlx::Fresnel {
                color: Animated {
                    default: [0.2; 3],
                    binding: Some(node.translation()),
                },
                opacity: scalar(),
                team_color: scalar(),
            }),
            images: Arc::default(),
        }),
        ..Default::default()
    };
    assert!(animation.is_animated());
    animation.prune_static_bindings(&AnimationTargets::new(&model));
    assert!(!animation.is_animated());
    assert_eq!(animation.weight.unwrap().default, 0.375);
    let fresnel = animation.hd.unwrap().fresnel.unwrap();
    assert_eq!(fresnel.color.default, [0.2; 3]);
    assert_eq!(fresnel.opacity.default, 0.375);
}

fn scalar_material() -> (ModelBuilder<'static>, MaterialId) {
    let mut builder = Model::<Universal>::builder("scalar material tracks");
    let material = builder.reserve_material();
    let surface = m2::MaterialStep {
        color: Animated::constant([1.; 3]),
        opacity: Animated {
            default: 1.,
            binding: Some(material.opacity().into()),
        },
        ..Default::default()
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
    (builder, material)
}

#[test]
fn constant_scalar_tracks_matching_default_do_not_require_private_materials() {
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
            .add_timeline_channel(Channel::Scalar(Track {
                target: material.opacity(),
                clock: Clock::Timeline,
                curve,
            }))
            .unwrap();
        let model = builder.finish().unwrap();
        let pass = compile_material_with_images(&model, material, Arc::default())
            .unwrap()
            .remove(0);
        assert!(!pass.animation.is_animated());
        assert!(pass.animation.opacity.as_ref().unwrap().binding.is_none());
        assert_eq!(pass.material.base_color.alpha(), 1.);
    }
}

#[test]
fn conflicting_clip_constants_and_controller_overrides_keep_materials_live() {
    for controller_override in [false, true] {
        let (mut builder, material) = scalar_material();
        let time = ModelTime::seconds(0.).unwrap();
        let track = |value, clock| {
            Channel::Scalar(Track {
                target: material.opacity(),
                clock,
                curve: Curve::step(vec![time], vec![value]).unwrap(),
            })
        };
        let clip = builder
            .add_clip(Clip::named("first").with_duration(ModelTime::seconds(1.).unwrap()))
            .unwrap();
        builder.add_channel(clip, track(1., Clock::Clip)).unwrap();
        if controller_override {
            let controller = builder.add_controller(Controller::default()).unwrap();
            builder
                .add_controller_channel(controller, track(0.25, Clock::Timeline))
                .unwrap();
        } else {
            let other = builder
                .add_clip(Clip::named("second").with_duration(ModelTime::seconds(1.).unwrap()))
                .unwrap();
            builder.add_channel(other, track(0.5, Clock::Clip)).unwrap();
        }
        let model = builder.finish().unwrap();
        let pass = compile_material_with_images(&model, material, Arc::default())
            .unwrap()
            .remove(0);
        assert!(pass.animation.is_animated());
        assert!(pass.animation.opacity.unwrap().binding.is_some());
    }
}

#[test]
fn constant_track_different_from_fallback_cannot_be_folded() {
    let (mut builder, material) = scalar_material();
    let clip = builder
        .add_clip(
            Clip::named("only selected sometimes").with_duration(ModelTime::seconds(1.).unwrap()),
        )
        .unwrap();
    builder
        .add_channel(
            clip,
            Channel::Scalar(Track {
                target: material.opacity(),
                clock: Clock::Clip,
                curve: Curve::step(vec![ModelTime::default()], vec![0.5]).unwrap(),
            }),
        )
        .unwrap();
    let model = builder.finish().unwrap();
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
            ..Default::default()
        })
        .unwrap();
    pass.animation.apply(&sample, &mut unselected);
    assert_eq!(unselected.base_color.alpha(), 0.5);
}

#[test]
fn equal_spline_endpoints_do_not_prove_constant_scalar_output() {
    let (mut builder, material) = scalar_material();
    let times = [
        ModelTime::seconds(0.).unwrap(),
        ModelTime::seconds(1.).unwrap(),
    ];
    builder
        .add_timeline_channel(Channel::Scalar(Track {
            target: material.opacity(),
            clock: Clock::Timeline,
            curve: Curve::hermite(times, [1., 1.], [0., -2.], [2., 0.]).unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let pass = compile_material_with_images(&model, material, Arc::default())
        .unwrap()
        .remove(0);
    assert!(pass.animation.is_animated());
    assert!(pass.animation.opacity.unwrap().binding.is_some());
}

#[test]
fn folding_constant_weights_preserves_changing_uv_channels() {
    let mut builder = Model::<Universal>::builder("weight and UV");
    let node = builder.add_node(Node::named("UV source")).unwrap();
    let material = builder.reserve_material();
    let mut unit: m2::TextureUnit = m2::TextureUnit {
        contribution: Animated {
            default: 1.,
            binding: Some(material.opacity().into()),
        },
        ..Default::default()
    };
    unit.coordinates.transform.translation.binding = Some(node.translation());
    let surface = m2::MaterialStep {
        color: Animated::constant([1.; 3]),
        opacity: Animated::constant(1.),
        textures: vec![unit],
        ..Default::default()
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
        .add_timeline_channel(Channel::Scalar(Track {
            target: material.opacity(),
            clock: Clock::Timeline,
            curve: Curve::step(vec![ModelTime::default()], vec![1.]).unwrap(),
        }))
        .unwrap();
    builder
        .add_timeline_channel(Channel::Vector3(Track {
            target: node.translation(),
            clock: Clock::Timeline,
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
    let model = builder.finish().unwrap();
    let pass = compile_material_with_images(&model, material, Arc::default())
        .unwrap()
        .remove(0);
    assert!(pass.animation.weight.as_ref().unwrap().binding.is_none());
    assert!(pass
        .animation
        .transform
        .as_ref()
        .unwrap()
        .translation
        .binding
        .is_some());
    assert!(pass.animation.is_animated());
    let mut value = pass.material;
    let sample = model
        .animation_sample(AnimationSample {
            timeline_time: ModelTime::seconds(0.5).unwrap(),
            ..Default::default()
        })
        .unwrap();
    pass.animation.apply(&sample, &mut value);
    assert!((value.uv_transform.translation.y - 0.5).abs() < 1e-6);
}
