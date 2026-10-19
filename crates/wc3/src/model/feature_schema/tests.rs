use super::{CameraView, FeatureSchema, FeatureView, LightView, ParticlesView};
use crate::model::profiles::{m2, mdlx, Either};
use crate::model::LightEmission;
use crate::model::{scene, Axis, NodeFeature};
use crate::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use crate::model::{M2RibbonControls, M2RibbonRendering, TrailAppearance, TrailGeometry};
use crate::model::{ParticleEmission, ParticleLifetime, ParticleMotion};
use glam::Vec3;
use std::ptr;
#[test]
fn emission_distinguishes_continuous_rates_from_keyed_burst_counts() {
    let mut slot_values = SlotValues::default();
    let mut authored = mdlx::ParticleEmitter2 {
        emission: ParticleEmission {
            rate: slot_values.add(37.5),
            variation: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        lifetime: ParticleLifetime {
            duration: 2.25,
            variation: (),
        },
        ..SlotConstruct::construct(&mut slot_values)
    };
    let view = &authored;
    let super::EmissionRef::Continuous(rate) =
        ParticlesView::MdlxEmitter2(view).emission().unwrap()
    else {
        panic!("continuous emitter must expose a rate");
    };
    assert!(ptr::eq(rate, &authored.emission.rate));
    assert_eq!(slot_values[rate], 37.5);
    authored.emission.trigger = true;
    let super::EmissionRef::BurstAtKeys(count) =
        ParticlesView::MdlxEmitter2(&authored).emission().unwrap()
    else {
        panic!("squirt emitter must expose a burst count");
    };
    assert!(ptr::eq(count, &authored.emission.rate));
    let super::LifetimeRef::Seconds(lifetime) =
        ParticlesView::MdlxEmitter2(&authored).lifetime().unwrap()
    else {
        unreachable!();
    };
    assert!(ptr::eq(lifetime, &authored.lifetime.duration));
    assert_eq!(*lifetime, 2.25);
}

#[test]
fn model_particle_emission_retains_animated_values_without_copying() {
    let mut slot_values = SlotValues::default();
    let authored = mdlx::ParticleEmitter1 {
        emission: ParticleEmission {
            rate: slot_values.add(18.),
            variation: (),
            trigger: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        lifetime: ParticleLifetime {
            duration: slot_values.add(4.5),
            variation: (),
        },
        motion: ParticleMotion {
            variation: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        appearance: (),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let view = ParticlesView::MdlxEmitter1(&authored);
    let super::EmissionRef::Continuous(rate) = view.emission().unwrap() else {
        unreachable!()
    };
    let super::LifetimeRef::AnimatedSeconds(lifetime) = view.lifetime().unwrap() else {
        unreachable!()
    };
    assert!(ptr::eq(rate, &authored.emission.rate));
    assert!(ptr::eq(lifetime, &authored.lifetime.duration));
    assert_eq!(slot_values[rate], 18.);
    assert_eq!(slot_values[lifetime], 4.5);
}

#[test]
fn ribbon_components_preserve_distinct_rate_and_atlas_integer_domains() {
    let mut slot_values = SlotValues::default();
    let segmented = mdlx::RibbonEmitter {
        appearance: TrailAppearance {
            color: slot_values.add(Vec3::from_array([0.25, 0.5, 0.75])),
            opacity: slot_values.add(0.6),
            texture_slot: slot_values.add(70_000),
            ..SlotConstruct::construct(&mut slot_values)
        },
        geometry: TrailGeometry {
            height_above: slot_values.add(3.5),
            ..SlotConstruct::construct(&mut slot_values)
        },
        lifetime: 2.5,
        emission_rate: 17,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let view = super::RibbonView::Mdlx(&segmented).edge_strip().unwrap();
    assert!(ptr::eq(view.color, &segmented.appearance.color));
    assert_eq!(*view.axis, Axis::PositiveY);
    assert!(ptr::eq(view.opacity, &segmented.appearance.opacity));
    assert_eq!(slot_values[view.height[0]], 3.5);
    assert_eq!(*view.lifetime_seconds, 2.5);
    let super::TrailRateRef::Integral(rate) = view.emission_rate else {
        unreachable!()
    };
    let super::AtlasCellRef::U32(cell) = view.atlas_cell else {
        unreachable!()
    };
    assert!(ptr::eq(rate, &segmented.emission_rate));
    assert!(ptr::eq(cell, &segmented.appearance.texture_slot));
    assert_eq!(*rate, 17);
    assert_eq!(slot_values[cell], 70_000);

    let edge = m2::RibbonEmitter {
        controls: M2RibbonControls {
            id: 0,
            angle: 0.4,
            enabled: slot_values.add(true),
            priority_plane: -2,
            color_index: -1,
            texture_transform: None,
        },
        rendering: M2RibbonRendering {
            textures: vec![],
            materials: vec![],
        },
        appearance: TrailAppearance {
            color: slot_values.add(Vec3::from_array([0.7, 0.3, 0.1])),
            opacity: slot_values.add(0.8),
            atlas: [2, 4],
            texture_slot: slot_values.add(31),
        },
        geometry: TrailGeometry {
            height_above: slot_values.add(6.),
            height_below: slot_values.add(2.),
            axis: Axis::PositiveZ,
        },
        emission_rate: 12.5,
        lifetime: 3.25,
    };
    let view = super::RibbonView::M2(&edge).edge_strip().unwrap();
    assert!(ptr::eq(view.color, &edge.appearance.color));
    assert!(ptr::eq(view.opacity, &edge.appearance.opacity));
    assert_eq!(slot_values[view.height[0]], 6.);
    let super::TrailRateRef::Fractional(rate) = view.emission_rate else {
        unreachable!()
    };
    let super::AtlasCellRef::U16(cell) = view.atlas_cell else {
        unreachable!()
    };
    assert!(ptr::eq(rate, &edge.emission_rate));
    assert!(ptr::eq(cell, &edge.appearance.texture_slot));
    assert_eq!(*rate, 12.5);
    assert_eq!(slot_values[cell], 31);
}

#[test]
fn native_discovery_borrows_the_authored_definition() {
    let mut slot_values = SlotValues::default();
    let feature = mdlx::Node::Particle2(mdlx::ParticleEmitter2::construct(&mut slot_values));
    let FeatureView::Particles(ParticlesView::MdlxEmitter2(view)) = feature.feature() else {
        panic!("sprite emitter must be discoverable as particle behavior");
    };
    let mdlx::Node::Particle2(authored) = &feature else {
        unreachable!();
    };
    assert!(ptr::eq(view, authored));
}

#[test]
fn unions_delegate_behavior_discovery() {
    let mut slot_values = SlotValues::default();
    let feature: Either<mdlx::Node<'_>, scene::Feature> = Either::Right(scene::Feature::Camera(
        Box::new(scene::Camera::construct(&mut slot_values)),
    ));
    assert!(matches!(
        feature.feature(),
        FeatureView::Camera(CameraView::Scene(_))
    ));
}

#[test]
fn shared_illumination_is_inspected_identically_before_and_after_widening() {
    let mut slot_values = SlotValues::default();
    let light = mdlx::Light {
        diffuse: LightEmission {
            color: slot_values.add(Vec3::from_array([0.2, 0.4, 0.6])),
            ..SlotConstruct::construct(&mut slot_values)
        },
        ambient: LightEmission {
            intensity: slot_values.add(0.75),
            ..SlotConstruct::construct(&mut slot_values)
        },
        ..SlotConstruct::construct(&mut slot_values)
    };
    let native = mdlx::Node::Light(light);
    let FeatureView::Light(view) = native.feature() else {
        unreachable!()
    };
    let native_components = view.ambient_diffuse().unwrap();
    let expected_color = slot_values[native_components.diffuse.color];
    let expected_ambient = slot_values[native_components.ambient.intensity];
    let universal: NodeFeature<'_> =
        crate::model::features::FromFeature::from_feature(native, &mut slot_values);
    let FeatureView::Light(view) = universal.feature() else {
        unreachable!()
    };
    let components = view.ambient_diffuse().unwrap();
    assert_eq!(slot_values[components.diffuse.color], expected_color);
    assert_eq!(slot_values[components.ambient.intensity], expected_ambient);
}

#[test]
fn light_discovery_retains_native_illumination_semantics() {
    let mut slot_values = SlotValues::default();
    let feature = mdlx::Node::Light(mdlx::Light::construct(&mut slot_values));
    assert!(matches!(
        feature.feature(),
        FeatureView::Light(LightView::VisibleAmbientDiffuse(_))
    ));
}

#[test]
fn application_behaviors_can_keep_their_own_complete_vocabulary() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FeatureView<'static, 'static>>();
    #[derive(Debug)]
    struct ApplicationBehavior {
        strength: f32,
    }
    impl<'data> FeatureSchema<'data> for ApplicationBehavior {
        fn feature(&self) -> FeatureView<'_, 'data> {
            FeatureView::Custom(self)
        }
    }
    let behavior = ApplicationBehavior { strength: 0.75 };
    assert_eq!(behavior.strength, 0.75);
    assert!(matches!(behavior.feature(), FeatureView::Custom(_)));
}

#[test]
fn owned_emitters_expose_the_same_borrowed_emission_and_trail_components() {
    let mut slot_values = SlotValues::default();
    use crate::model::{NodeFeature, ParticleEmitter, RibbonEmitter};
    let native = mdlx::ParticleEmitter2 {
        emission: ParticleEmission {
            trigger: true,
            rate: slot_values.add(12.5),
            variation: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        lifetime: ParticleLifetime {
            variation: (),
            ..SlotConstruct::construct(&mut slot_values)
        },
        ..SlotConstruct::construct(&mut slot_values)
    };
    let feature = NodeFeature::Particle(ParticleEmitter::from(native));
    let FeatureView::Particles(view) = feature.feature() else {
        panic!()
    };
    let super::EmissionRef::BurstAtKeys(value) = view.emission().unwrap() else {
        panic!()
    };
    let ParticlesView::MdlxEmitter2(definition) = view else {
        panic!()
    };
    assert!(ptr::eq(value, &definition.emission.rate));
    let feature = NodeFeature::Ribbon(RibbonEmitter::from(mdlx::RibbonEmitter {
        emission_rate: u32::MAX,
        ..SlotConstruct::construct(&mut slot_values)
    }));
    let FeatureView::Ribbon(view) = feature.feature() else {
        panic!()
    };
    let edge = view.edge_strip().unwrap();
    let super::TrailRateRef::Integral(rate) = edge.emission_rate else {
        unreachable!()
    };
    assert_eq!(*rate, u32::MAX);
    let super::RibbonView::Mdlx(trail) = view else {
        panic!()
    };
    assert!(ptr::eq(rate, &trail.emission_rate));
    assert!(ptr::eq(edge.height[0], &trail.geometry.height_above));
}
