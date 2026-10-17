use std::borrow::Cow;
use wc3::model::profiles::{m2, mdlx, wmo, Extends, Mdlx, Narrows, Universal, Wmo, M2};
use wc3::model::FromFeature as SlotFromFeature;
use wc3::model::{
    AtlasParticle, AtlasParticleRendering, ModelParticle, ParticleEmission, ParticleLifetime,
    ParticleMotion, StagedParticleAppearance,
};
use wc3::model::{Construct as SlotConstruct, Values as SlotValues};
use wc3::model::{GravityTrailControls, TrailAppearance};
use wc3::model::{LightEmission, Model, PhotographicControls, SocketDefinition, VisibleLight};

#[test]
fn complete_mdlx_feature_vocabulary_round_trips_through_universal() {
    let mut slot_values = SlotValues::default();
    let mut draft = Model::<Mdlx>::builder("resources");
    let particle = draft.add_model_resource(wc3::model::PathResource::Path(
        b"particle.mdx".as_slice().into(),
    ));
    let effect = draft.add_effect_resource(wc3::model::PathResource::Path(
        b"fire.pkfx".as_slice().into(),
    ));
    let attachment = draft.add_model_resource(wc3::model::PathResource::Path(
        b"attachment.mdx".as_slice().into(),
    ));
    let features = [
        mdlx::Node::GeometryAnimation(mdlx::GeometryAnimation::construct(&mut slot_values)),
        mdlx::Node::Particle(mdlx::Particle {
            details: ModelParticle {
                resource: wc3::model::ParticleResource::Model(Some(particle)),
                ..SlotConstruct::construct(&mut slot_values)
            },
            emission: ParticleEmission {
                rate: slot_values.add(23.),
                variation: (),
                trigger: (),
                ..SlotConstruct::construct(&mut slot_values)
            },
            motion: ParticleMotion {
                variation: (),
                ..SlotConstruct::construct(&mut slot_values)
            },
            lifetime: ParticleLifetime {
                variation: (),
                ..SlotConstruct::construct(&mut slot_values)
            },
            appearance: (),
        }),
        mdlx::Node::AtlasParticles(mdlx::Particle2 {
            details: AtlasParticle {
                rendering: AtlasParticleRendering {
                    atlas: [3, 5],
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            },
            appearance: StagedParticleAppearance {
                uv_intervals: [[1, 3, 2], [4, 6, 3], [7, 9, 4], [10, 12, 5]],
                colors: [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6], [0.7, 0.8, 0.9]],
                alpha: [255, 127, 1],
                scale: [1., 2., 0.],
                ..SlotConstruct::construct(&mut slot_values)
            },
            emission: ParticleEmission {
                trigger: true,
                variation: (),
                ..SlotConstruct::construct(&mut slot_values)
            },
            lifetime: ParticleLifetime {
                variation: (),
                ..SlotConstruct::construct(&mut slot_values)
            },
            ..SlotConstruct::construct(&mut slot_values)
        }),
        mdlx::Node::Ribbon(mdlx::Ribbon {
            controls: GravityTrailControls {
                gravity: 9.8,
                ..SlotConstruct::construct(&mut slot_values)
            },
            appearance: TrailAppearance {
                atlas: [2, 4],
                ..SlotConstruct::construct(&mut slot_values)
            },
            ..SlotConstruct::construct(&mut slot_values)
        }),
        mdlx::Node::ParticleProgram(mdlx::Popcorn {
            resource: Some(effect),
            visibility_guide: Cow::Borrowed("Stand=on\nDeath=off"),
            scaling: true,
            ..SlotConstruct::construct(&mut slot_values)
        }),
        mdlx::Node::Light(mdlx::Light {
            kind: mdlx::LightKind::Unknown(0x8000_0001),
            activation: VisibleLight {
                shadow_casting: Some(false),
                shadow_intensity: Some(0.25),
                falloff: Some([
                    slot_values.add(0.1),
                    slot_values.add(0.2),
                    slot_values.add(0.3),
                ]),
                ..SlotConstruct::construct(&mut slot_values)
            },
            ..SlotConstruct::construct(&mut slot_values)
        }),
        mdlx::Node::Camera(mdlx::Camera {
            target: [1., 2., 3.],
            clipping: [0.1, 1000.],
            field_of_view: 0.75,
            convention: PhotographicControls {
                f_stop: Some(slot_values.add(2.8)),
                ..SlotConstruct::construct(&mut slot_values)
            },
            ..SlotConstruct::construct(&mut slot_values)
        }),
        mdlx::Node::Attachment(mdlx::Attachment {
            resource: attachment,
            appearance: (),
            socket: SocketDefinition {
                identity: 17,
                ..SlotConstruct::construct(&mut slot_values)
            },
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
        let widened = <Universal as Extends<Mdlx>>::feature(feature, &mut slot_values);
        let restored = <Mdlx as Narrows<Universal>>::feature(widened, &slot_values).unwrap();
        assert_eq!(restored, expected);
    }
}

#[test]
fn resource_conversion_moves_owned_paths_and_preserves_handles() {
    let slot_values = SlotValues::default();
    let path = b"owned-particle.mdx".to_vec();
    let pointer = path.as_ptr();
    let mut draft = Model::<Mdlx>::builder("resources");
    let id = draft.add_model_resource(wc3::model::PathResource::Path(Cow::Owned(path)));
    let widened = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
    let restored = widened.try_narrow::<Mdlx>().unwrap();
    let wc3::model::PathResource::Path(path) = restored.model_resource(id).unwrap();
    assert!(matches!(path, Cow::Owned(_)));
    assert_eq!(path.as_ptr(), pointer);
}

#[test]
fn full_model_feature_identity_survives_widening_and_narrowing() {
    let mut slot_values = SlotValues::default();
    let mut draft = Model::<Mdlx>::builder("features");
    let node = draft.add_named_node("emitter").unwrap();
    let feature = draft
        .add_feature(
            node,
            mdlx::Node::AtlasParticles(mdlx::Particle2::construct(&mut slot_values)),
        )
        .unwrap();
    let model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap();
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
    let mut slot_values = SlotValues::default();
    let resource = Model::<Wmo>::builder("resource").add_model_resource(
        wc3::model::WorldModelResource::Path(b"tree.m2".as_slice().into()),
    );
    let features = [
        wmo::Node::Attachment(wmo::Doodad {
            resource,
            socket: (),
            appearance: SlotConstruct::construct(&mut slot_values),
        }),
        wmo::Node::Light(Box::new(wmo::Light {
            directional_terms: Some([([1., 2., 3.], 4.); 6]),
            modern: Some(wmo::ModernLight {
                controls: [0x8000_0001; 7],
                trailing_controls: [17; 11],
                ..SlotConstruct::construct(&mut slot_values)
            }),
            ..SlotConstruct::construct(&mut slot_values)
        })),
        wmo::Node::PointLight(wmo::PointLight {
            color: [1, 2, 3, 4],
            control: 0x8000_0001,
            ..SlotConstruct::construct(&mut slot_values)
        }),
    ];
    for source in features {
        let expected = source.clone();
        let widened = <Universal as Extends<Wmo>>::feature(source, &mut slot_values);
        let restored = <Wmo as Narrows<Universal>>::feature(widened, &slot_values).unwrap();
        assert_eq!(restored, expected);
    }
}

#[test]
fn m2_enable_and_camera_conventions_survive_projection() {
    let mut slot_values = SlotValues::default();
    let features = [
        m2::ActorKind::Attachment(m2::Attachment {
            identity: 31,
            visibility: slot_values.add(false),
            control: (),
        }),
        m2::ActorKind::Light(m2::Light {
            kind: m2::values::LightKind::Unknown(65535),
            ambient: LightEmission {
                color: slot_values.add([0.1, 0.2, 0.3]),
                intensity: slot_values.add(0.4),
            },
            diffuse: LightEmission {
                color: slot_values.add([0.5, 0.6, 0.7]),
                intensity: slot_values.add(0.8),
            },
            attenuation: [slot_values.add(1.), slot_values.add(20.)],
            activation: slot_values.add(false),
        }),
        m2::ActorKind::Camera(m2::Camera {
            convention: m2::values::CameraKind::Unknown(0x8000_0001),
            field_of_view: slot_values.add(0.8),
            clipping: [0.1, 1000.],
            target: [1., 2., 3.],
            eye_offset: slot_values.add([4., 5., 6.]),
            target_offset: slot_values.add([7., 8., 9.]),
            roll: slot_values.add(0.5),
        }),
        m2::ActorKind::Event(m2::Event {
            identifier: *b"SNDX",
            data: 0x8000_0001,
        }),
    ];
    for source in features {
        let expected = format!("{source:?}");
        let widened = <Universal as Extends<M2>>::feature(source, &mut slot_values);
        let restored = <M2 as Narrows<Universal>>::feature(widened, &slot_values).unwrap();
        assert_eq!(format!("{restored:?}"), expected);
    }
}

#[test]
fn editing_shared_light_fields_survives_exact_native_projection() {
    let mut slot_values = SlotValues::default();
    use wc3::model::{Light, NodeFeature};
    let source = mdlx::Node::Light(mdlx::Light::construct(&mut slot_values));
    let mut widened = <Universal as Extends<Mdlx>>::feature(source, &mut slot_values);
    let NodeFeature::Light(Light::AmbientDiffuse(light)) = &mut widened else {
        panic!("expected shared ambient and diffuse illumination")
    };
    slot_values.set(light.diffuse.intensity, 7.).unwrap();
    slot_values
        .set(light.ambient.color, [0.1, 0.2, 0.3])
        .unwrap();
    let mdlx::Node::Light(restored) =
        <Mdlx as Narrows<Universal>>::feature(widened, &slot_values).unwrap()
    else {
        panic!("light changed behavior")
    };
    assert_eq!(slot_values[restored.diffuse.intensity], 7.);
    assert_eq!(slot_values[restored.ambient.color], [0.1, 0.2, 0.3]);
}

#[test]
fn common_animation_destinations_validate_and_narrow_with_borrowed_keys() {
    let mut slot_values = SlotValues::default();
    use wc3::model::common::ZeroVec;

    use wc3::model::{Channel, Clock, Curve, Time, Track};
    let mut draft = Model::<Mdlx>::builder("animated light");
    let node = draft.add_named_node("light").unwrap();
    let feature = draft.reserve_feature(node);
    let intensity = slot_values.add(1f32);
    draft
        .set_feature(
            feature,
            mdlx::Node::Light(mdlx::Light {
                diffuse: LightEmission {
                    intensity,
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }),
        )
        .unwrap();
    let mut model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
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
                target: intensity,
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
    assert_eq!(track.target, intensity);
    assert_eq!(track.curve.values().as_bytes().as_ptr(), pointer);
}

#[test]
fn particle_copy_relationship_rejects_a_different_particle_simulation() {
    let mut slot_values = SlotValues::default();
    use wc3::model::profiles::m3;
    use wc3::model::NodeFeature;
    let mut draft = Model::<Universal>::builder("invalid particle inheritance");
    let node = draft.add_named_node("effects").unwrap();
    let source = draft
        .add_feature(
            node,
            NodeFeature::from_feature(
                mdlx::Node::AtlasParticles(mdlx::Particle2::construct(&mut slot_values)),
                &mut slot_values,
            ),
        )
        .unwrap();
    draft
        .add_feature(
            node,
            NodeFeature::ParticleCopy(Box::new(m3::ParticleCopy {
                source,
                emission_rate: SlotConstruct::construct(&mut slot_values),
                squirt_amount: SlotConstruct::construct(&mut slot_values),
            })),
        )
        .unwrap();
    assert!({
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .is_err());
}

#[test]
fn camera_slot_alone_does_not_imply_authored_animation() {
    let mut slot_values = SlotValues::default();
    let source = mdlx::Camera {
        field_of_view: 0.75,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let feature =
        <Universal as Extends<Mdlx>>::feature(mdlx::Node::Camera(source), &mut slot_values);
    let mdlx::Node::Camera(restored) =
        <Mdlx as Narrows<Universal>>::feature(feature, &slot_values).unwrap()
    else {
        panic!()
    };
    assert_eq!(restored.field_of_view, 0.75);
}

#[test]
fn external_attachment_conversion_preserves_its_resource_handle() {
    let mut slot_values = SlotValues::default();
    use wc3::model::{scene, NodeFeature, ResourceLocator};
    let mut draft = Model::<Universal>::builder("resources");
    let resource = draft.add_model_resource(ResourceLocator::Path(Cow::Borrowed(
        b"effects/attachment.mdx",
    )));
    let source = scene::Feature::Attachment(scene::Attachment {
        resource,
        socket: SocketDefinition {
            visibility: slot_values.add(0.5),
            identity: 21,
            control: (),
        },
        appearance: (),
    });
    let feature = NodeFeature::from_feature(source, &mut slot_values);
    let mdlx::Node::Attachment(restored) =
        <Mdlx as Narrows<Universal>>::feature(feature, &slot_values).unwrap()
    else {
        panic!("attachment changed behavior")
    };
    assert_eq!(restored.resource, resource);
    assert_eq!(restored.socket.identity, 21);
    assert_eq!(slot_values[restored.socket.visibility], 0.5);
}

#[test]
fn unresolved_light_kind_cannot_be_reinterpreted_by_changing_activation() {
    let mut slot_values = SlotValues::default();
    use wc3::model::{Light, LightActivation, NodeFeature};
    let mut feature = <Universal as Extends<Mdlx>>::feature(
        mdlx::Node::Light(mdlx::Light {
            kind: mdlx::LightKind::Unknown(17),
            ..SlotConstruct::construct(&mut slot_values)
        }),
        &mut slot_values,
    );
    let NodeFeature::Light(Light::AmbientDiffuse(light)) = &mut feature else {
        panic!("expected light")
    };
    light.activation = LightActivation::Enabled(slot_values.add(true));
    assert!(<M2 as Narrows<Universal>>::feature(feature, &slot_values).is_err());
}
