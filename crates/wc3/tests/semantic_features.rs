use std::borrow::Cow;
use wc3::model::profiles::{m2, mdlx, wmo, Extends, Mdlx, Narrows, Universal, Wmo, M2};
use wc3::model::{
    Animated, AttachmentPointDefinition, LightEmission, Model, Node, PhotographicControls,
    VisibleLight,
};
use wc3::model::{
    AtlasParticle, AtlasParticleRendering, ModelParticle, ParticleEmission, ParticleLifetime,
    ParticleMotion, StagedParticleAppearance,
};
use wc3::model::{GravityTrailControls, TrailAppearance};

#[test]
fn complete_mdlx_feature_vocabulary_round_trips_through_universal() {
    let features = [
        mdlx::Node::GeometryAnimation(mdlx::GeometryAnimation::default()),
        mdlx::Node::Particle(mdlx::Particle {
            details: ModelParticle {
                resource: Cow::Borrowed("particle.mdx"),
                model_resource: true,
                ..Default::default()
            },
            emission: ParticleEmission {
                rate: Animated::constant(23.),
                variation: (),
                trigger: (),
                ..Default::default()
            },
            motion: ParticleMotion {
                variation: (),
                ..Default::default()
            },
            lifetime: ParticleLifetime {
                variation: (),
                ..Default::default()
            },
            appearance: (),
        }),
        mdlx::Node::AtlasParticles(mdlx::Particle2 {
            details: AtlasParticle {
                rendering: AtlasParticleRendering {
                    atlas: [3, 5],
                    ..Default::default()
                },
                ..Default::default()
            },
            appearance: StagedParticleAppearance {
                uv_intervals: [[1, 3, 2], [4, 6, 3], [7, 9, 4], [10, 12, 5]],
                colors: [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6], [0.7, 0.8, 0.9]],
                alpha: [255, 127, 1],
                scale: [1., 2., 0.],
                ..Default::default()
            },
            emission: ParticleEmission {
                trigger: true,
                variation: (),
                ..Default::default()
            },
            lifetime: ParticleLifetime {
                variation: (),
                ..Default::default()
            },
            ..Default::default()
        }),
        mdlx::Node::Ribbon(mdlx::Ribbon {
            controls: GravityTrailControls {
                gravity: 9.8,
                ..Default::default()
            },
            appearance: TrailAppearance {
                atlas: [2, 4],
                ..Default::default()
            },
            ..Default::default()
        }),
        mdlx::Node::ParticleProgram(mdlx::Popcorn {
            resource: Cow::Borrowed("fire.pkfx"),
            visibility_guide: Cow::Borrowed("Stand=on\nDeath=off"),
            scaling: true,
            ..Default::default()
        }),
        mdlx::Node::Light(mdlx::Light {
            kind: mdlx::LightKind::Unknown(0x8000_0001),
            activation: VisibleLight {
                shadow_casting: Some(false),
                shadow_intensity: Some(0.25),
                falloff: Some([
                    Animated::constant(0.1),
                    Animated::constant(0.2),
                    Animated::constant(0.3),
                ]),
                ..Default::default()
            },
            ..Default::default()
        }),
        mdlx::Node::Camera(mdlx::Camera {
            target: [1., 2., 3.],
            clipping: [0.1, 1000.],
            field_of_view: 0.75,
            convention: PhotographicControls {
                f_stop: Some(Animated::constant(2.8)),
                ..Default::default()
            },
            ..Default::default()
        }),
        mdlx::Node::Attachment(mdlx::Attachment {
            resource: Cow::Borrowed("attachment.mdx"),
            point: AttachmentPointDefinition {
                identity: 17,
                ..Default::default()
            },
            ..Default::default()
        }),
        mdlx::Node::Event(mdlx::Events {
            name: Cow::Borrowed("SNDxFootstep"),
        }),
        mdlx::Node::CollisionShape(mdlx::Collision::Unknown {
            kind: 0x8000_0001,
            vertices: vec![[1., 2., 3.], [4., 5., 6.]],
            radius: Some(7.),
        }),
    ];
    for feature in features {
        let expected = feature.clone();
        let widened = <Universal as Extends<Mdlx>>::feature(feature);
        let restored = <Mdlx as Narrows<Universal>>::feature(widened).unwrap();
        assert_eq!(restored, expected);
    }
}

#[test]
fn feature_conversion_moves_owned_resource_paths() {
    let resource = String::from("owned-particle.mdx");
    let pointer = resource.as_ptr();
    let source = mdlx::Node::Particle(mdlx::Particle {
        details: ModelParticle {
            resource: Cow::Owned(resource),
            ..Default::default()
        },
        emission: ParticleEmission {
            variation: (),
            trigger: (),
            ..Default::default()
        },
        motion: ParticleMotion {
            variation: (),
            ..Default::default()
        },
        lifetime: ParticleLifetime {
            variation: (),
            ..Default::default()
        },
        appearance: (),
    });
    let widened = <Universal as Extends<Mdlx>>::feature(source);
    let restored = <Mdlx as Narrows<Universal>>::feature(widened).unwrap();
    let mdlx::Node::Particle(particle) = restored else {
        panic!("particle changed behavior")
    };
    assert!(matches!(particle.details.resource, Cow::Owned(_)));
    assert_eq!(particle.details.resource.as_ptr(), pointer);
}

#[test]
fn full_model_feature_identity_survives_widening_and_narrowing() {
    let mut draft = Model::<Mdlx>::builder("features");
    let node = draft.add_node(Node::named("emitter")).unwrap();
    let feature = draft
        .add_feature(node, mdlx::Node::AtlasParticles(mdlx::Particle2::default()))
        .unwrap();
    let model = draft.finish().unwrap();
    let widened = model.widen::<Universal>();
    assert_eq!(widened.feature(feature).unwrap().node, node);
    let restored = widened.try_narrow::<Mdlx>().unwrap();
    assert_eq!(restored.feature(feature).unwrap().node, node);
    assert!(matches!(
        restored.feature(feature).unwrap().payload,
        mdlx::Node::AtlasParticles(_)
    ));
}

#[test]
fn world_features_retain_light_relationship_categories() {
    let features = [
        wmo::Node::Attachment(wmo::Doodad::default()),
        wmo::Node::Light(Box::new(wmo::Light {
            directional_terms: Some([([1., 2., 3.], 4.); 6]),
            modern: Some(wmo::ModernLight {
                controls: [0x8000_0001; 7],
                trailing_controls: [17; 11],
                ..Default::default()
            }),
            ..Default::default()
        })),
        wmo::Node::PointLight(wmo::PointLight {
            color: [1, 2, 3, 4],
            control: 0x8000_0001,
            ..Default::default()
        }),
    ];
    for source in features {
        let expected = source.clone();
        let widened = <Universal as Extends<Wmo>>::feature(source);
        let restored = <Wmo as Narrows<Universal>>::feature(widened).unwrap();
        assert_eq!(restored, expected);
    }
}

#[test]
fn m2_enable_and_camera_conventions_survive_projection() {
    let features = [
        m2::ActorKind::Attachment(m2::Attachment {
            identity: 31,
            activation: Animated::constant(false),
            control: (),
        }),
        m2::ActorKind::Light(m2::Light {
            kind: m2::values::LightKind::Unknown(65535),
            ambient: LightEmission {
                color: Animated::constant([0.1, 0.2, 0.3]),
                intensity: Animated::constant(0.4),
            },
            diffuse: LightEmission {
                color: Animated::constant([0.5, 0.6, 0.7]),
                intensity: Animated::constant(0.8),
            },
            attenuation: [Animated::constant(1.), Animated::constant(20.)],
            activation: Animated::constant(false),
        }),
        m2::ActorKind::Camera(m2::Camera {
            convention: m2::values::CameraKind::Unknown(0x8000_0001),
            field_of_view: Animated::constant(0.8),
            clipping: [0.1, 1000.],
            target: [1., 2., 3.],
            eye_offset: Animated::constant([4., 5., 6.]),
            target_offset: Animated::constant([7., 8., 9.]),
            roll: Animated::constant(0.5),
        }),
        m2::ActorKind::Event(m2::Event {
            identifier: *b"SNDX",
            data: 0x8000_0001,
        }),
    ];
    for source in features {
        let expected = format!("{source:?}");
        let widened = <Universal as Extends<M2>>::feature(source);
        let restored = <M2 as Narrows<Universal>>::feature(widened).unwrap();
        assert_eq!(format!("{restored:?}"), expected);
    }
}

#[test]
fn editing_shared_light_fields_survives_exact_native_projection() {
    use wc3::model::{Light, NodeFeature};
    let source = mdlx::Node::Light(mdlx::Light::default());
    let mut widened = <Universal as Extends<Mdlx>>::feature(source);
    let NodeFeature::Light(Light::AmbientDiffuse(light)) = &mut widened else {
        panic!("expected shared ambient and diffuse illumination")
    };
    light.diffuse.intensity.default = 7.;
    light.ambient.color.default = [0.1, 0.2, 0.3];
    let mdlx::Node::Light(restored) = <Mdlx as Narrows<Universal>>::feature(widened).unwrap()
    else {
        panic!("light changed behavior")
    };
    assert_eq!(restored.diffuse.intensity.default, 7.);
    assert_eq!(restored.ambient.color.default, [0.1, 0.2, 0.3]);
}

#[test]
fn common_animation_destinations_validate_and_narrow_with_borrowed_keys() {
    use wc3::model::common::ZeroVec;
    use wc3::model::feature_properties::light;
    use wc3::model::{Channel, Clock, Curve, Time, Track};
    let mut draft = Model::<Mdlx>::builder("animated light");
    let node = draft.add_node(Node::named("light")).unwrap();
    let feature = draft.reserve_feature(node);
    draft
        .set_feature(
            feature,
            mdlx::Node::Light(mdlx::Light {
                diffuse: LightEmission {
                    intensity: Animated::new(1., mdlx::properties::light::INTENSITY.bind(feature)),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .unwrap();
    let mut model = draft.finish().unwrap().widen::<Universal>();
    let values = ZeroVec::alloc_from_slice(&[1f32, 3.]);
    let curve = Curve::linear(
        [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
        values.as_slice().as_zerovec(),
    )
    .unwrap();
    let pointer = curve.values().as_bytes().as_ptr();
    model
        .edit(|draft| {
            draft.add_timeline_channel(Channel::Scalar(Track {
                target: light::DIFFUSE_INTENSITY.bind(feature),
                curve,
                clock: Clock::Timeline,
            }))
        })
        .unwrap();
    let native = model.try_narrow::<Mdlx>().unwrap();
    let channel = native.timeline_channels().first().unwrap();
    let Channel::Scalar(track) = channel else {
        panic!("channel value domain changed")
    };
    assert_eq!(
        track.target.target().identity(),
        mdlx::properties::light::INTENSITY
            .bind(feature)
            .target()
            .identity()
    );
    assert_eq!(track.curve.values().as_bytes().as_ptr(), pointer);
}

#[test]
fn particle_copy_relationship_rejects_a_different_particle_simulation() {
    use wc3::model::profiles::m3;
    use wc3::model::NodeFeature;
    let mut draft = Model::<Universal>::builder("invalid particle inheritance");
    let node = draft.add_node(Node::named("effects")).unwrap();
    let source = draft
        .add_feature(
            node,
            mdlx::Node::AtlasParticles(mdlx::Particle2::default()).into(),
        )
        .unwrap();
    draft
        .add_feature(
            node,
            NodeFeature::ParticleCopy(Box::new(m3::ParticleCopy {
                source,
                emission_rate: Default::default(),
                squirt_amount: Default::default(),
            })),
        )
        .unwrap();
    assert!(draft.finish().is_err());
}

#[test]
fn camera_narrowing_rejects_newly_authored_animation_on_static_field_of_view() {
    use wc3::model::{Camera, NodeFeature};
    let mut ids = Model::<Universal>::builder("camera IDs");
    let node = ids.add_node(Node::named("camera")).unwrap();
    let id = ids.reserve_feature(node);
    let mut feature =
        <Universal as Extends<Mdlx>>::feature(mdlx::Node::Camera(mdlx::Camera::default()));
    let NodeFeature::Camera(Camera::Aimed(camera)) = &mut feature else {
        panic!("expected aimed camera")
    };
    camera.field_of_view.property = Some(m2::properties::camera::FIELD_OF_VIEW.bind(id).widen());
    assert!(<Mdlx as Narrows<Universal>>::feature(feature).is_err());
}

#[test]
fn external_attachments_narrow_by_resource_meaning_without_source_tags() {
    use wc3::model::{scene, NodeFeature, Resource};
    let source = scene::Feature::Attachment(scene::Attachment {
        resource: Resource::Path(Cow::Borrowed(b"effects/attachment.mdx")),
        point: AttachmentPointDefinition {
            activation: Animated::constant(0.5),
            identity: 21,
            control: (),
        },
        appearance: (),
    });
    let feature = NodeFeature::from(source);
    let mdlx::Node::Attachment(restored) = <Mdlx as Narrows<Universal>>::feature(feature).unwrap()
    else {
        panic!("attachment changed behavior")
    };
    assert_eq!(restored.resource, "effects/attachment.mdx");
    assert!(matches!(restored.resource, Cow::Borrowed(_)));
    assert_eq!(restored.point.identity, 21);
    assert_eq!(restored.point.activation.default, 0.5);
    for resource in [
        Resource::Path(Cow::Borrowed(&[255])),
        Resource::FileDataId(123),
    ] {
        let feature = NodeFeature::from(scene::Feature::Attachment(scene::Attachment {
            resource,
            point: AttachmentPointDefinition {
                activation: Animated::constant(1.),
                identity: 0,
                control: (),
            },
            appearance: (),
        }));
        assert!(<Mdlx as Narrows<Universal>>::feature(feature).is_err());
    }
}

#[test]
fn unresolved_light_kind_cannot_be_reinterpreted_by_changing_activation() {
    use wc3::model::{Light, LightActivation, NodeFeature};
    let mut feature = <Universal as Extends<Mdlx>>::feature(mdlx::Node::Light(mdlx::Light {
        kind: mdlx::LightKind::Unknown(17),
        ..Default::default()
    }));
    let NodeFeature::Light(Light::AmbientDiffuse(light)) = &mut feature else {
        panic!("expected light")
    };
    light.activation = LightActivation::Enabled(Animated::constant(true));
    assert!(<M2 as Narrows<Universal>>::feature(feature).is_err());
}
