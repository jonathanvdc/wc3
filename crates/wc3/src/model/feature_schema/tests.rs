use super::{CameraView, FeatureSchema, FeatureView, LightView, ParticlesView};
use crate::model::profiles::{m2, mdlx, Either};
use crate::model::LightEmission;
use crate::model::{scene, Animated, Axis, NodeFeature};
use crate::model::{AngleTrailControls, SurfaceTrailRendering, TrailAppearance, TrailGeometry};
use crate::model::{ParticleEmission, ParticleLifetime, ParticleMotion};
use std::ptr;

#[test]
fn emission_distinguishes_continuous_rates_from_keyed_burst_counts() {
    let mut authored = mdlx::Particle2 {
        emission: ParticleEmission {
            rate: Animated::constant(37.5),
            variation: (),
            ..Default::default()
        },
        lifetime: ParticleLifetime {
            duration: 2.25,
            variation: (),
        },
        ..Default::default()
    };
    let view = &authored;
    let super::EmissionRef::Continuous(rate) = ParticlesView::Sprite(view).emission().unwrap()
    else {
        panic!("continuous emitter must expose a rate");
    };
    assert!(ptr::eq(rate, &authored.emission.rate));
    assert_eq!(rate.default, 37.5);
    authored.emission.trigger = true;
    let super::EmissionRef::BurstAtKeys(count) =
        ParticlesView::Sprite(&authored).emission().unwrap()
    else {
        panic!("squirt emitter must expose a burst count");
    };
    assert!(ptr::eq(count, &authored.emission.rate));
    let super::LifetimeRef::Seconds(lifetime) =
        ParticlesView::Sprite(&authored).lifetime().unwrap()
    else {
        unreachable!();
    };
    assert!(ptr::eq(lifetime, &authored.lifetime.duration));
    assert_eq!(*lifetime, 2.25);
}

#[test]
fn model_particle_emission_retains_animated_values_without_copying() {
    let authored = mdlx::Particle {
        emission: ParticleEmission {
            rate: Animated::constant(18.),
            variation: (),
            trigger: (),
            ..Default::default()
        },
        lifetime: ParticleLifetime {
            duration: Animated::constant(4.5),
            variation: (),
        },
        motion: ParticleMotion {
            variation: (),
            ..Default::default()
        },
        appearance: (),
        ..Default::default()
    };
    let view = ParticlesView::Model(&authored);
    let super::EmissionRef::Continuous(rate) = view.emission().unwrap() else {
        unreachable!()
    };
    let super::LifetimeRef::AnimatedSeconds(lifetime) = view.lifetime().unwrap() else {
        unreachable!()
    };
    assert!(ptr::eq(rate, &authored.emission.rate));
    assert!(ptr::eq(lifetime, &authored.lifetime.duration));
    assert_eq!(rate.default, 18.);
    assert_eq!(lifetime.default, 4.5);
}

#[test]
fn ribbon_components_preserve_distinct_rate_and_atlas_integer_domains() {
    let segmented = mdlx::Ribbon {
        appearance: TrailAppearance {
            color: Animated::constant([0.25, 0.5, 0.75]),
            opacity: Animated::constant(0.6),
            texture_slot: Animated::constant(70_000),
            ..Default::default()
        },
        geometry: TrailGeometry {
            height_above: Animated::constant(3.5),
            ..Default::default()
        },
        lifetime: 2.5,
        emission_rate: 17,
        ..Default::default()
    };
    let view = super::RibbonView::Segmented(&segmented)
        .edge_strip()
        .unwrap();
    assert!(ptr::eq(view.color, &segmented.appearance.color));
    assert_eq!(*view.axis, Axis::PositiveY);
    assert!(ptr::eq(view.opacity, &segmented.appearance.opacity));
    assert_eq!(view.height[0].default, 3.5);
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
    assert_eq!(cell.default, 70_000);

    let edge = m2::Ribbon {
        controls: AngleTrailControls {
            id: 0,
            angle: 0.4,
            enabled: Animated::constant(true),
            priority_plane: -2,
            color_index: -1,
            texture_transform: None,
        },
        rendering: SurfaceTrailRendering {
            textures: vec![],
            materials: vec![],
        },
        appearance: TrailAppearance {
            color: Animated::constant([0.7, 0.3, 0.1]),
            opacity: Animated::constant(0.8),
            atlas: [2, 4],
            texture_slot: Animated::constant(31),
        },
        geometry: TrailGeometry {
            height_above: Animated::constant(6.),
            height_below: Animated::constant(2.),
            axis: Axis::PositiveZ,
        },
        emission_rate: 12.5,
        lifetime: 3.25,
    };
    let view = super::RibbonView::Edge(&edge).edge_strip().unwrap();
    assert!(ptr::eq(view.color, &edge.appearance.color));
    assert!(ptr::eq(view.opacity, &edge.appearance.opacity));
    assert_eq!(view.height[0].default, 6.);
    let super::TrailRateRef::Fractional(rate) = view.emission_rate else {
        unreachable!()
    };
    let super::AtlasCellRef::U16(cell) = view.atlas_cell else {
        unreachable!()
    };
    assert!(ptr::eq(rate, &edge.emission_rate));
    assert!(ptr::eq(cell, &edge.appearance.texture_slot));
    assert_eq!(*rate, 12.5);
    assert_eq!(cell.default, 31);
}

#[test]
fn native_discovery_borrows_the_authored_definition() {
    let feature = mdlx::Node::AtlasParticles(mdlx::Particle2::default());
    let FeatureView::Particles(ParticlesView::Sprite(view)) = feature.feature() else {
        panic!("sprite emitter must be discoverable as particle behavior");
    };
    let mdlx::Node::AtlasParticles(authored) = &feature else {
        unreachable!();
    };
    assert!(ptr::eq(view, authored));
}

#[test]
fn unions_delegate_behavior_discovery() {
    let feature: Either<mdlx::Node<'_>, scene::Feature<'_>> =
        Either::Right(scene::Feature::Camera(Box::default()));
    assert!(matches!(
        feature.feature(),
        FeatureView::Camera(CameraView::Scene(_))
    ));
}

#[test]
fn shared_illumination_is_inspected_identically_before_and_after_widening() {
    let light = mdlx::Light {
        diffuse: LightEmission {
            color: Animated::constant([0.2, 0.4, 0.6]),
            ..Default::default()
        },
        ambient: LightEmission {
            intensity: Animated::constant(0.75),
            ..Default::default()
        },
        ..Default::default()
    };
    let native = mdlx::Node::Light(light);
    let FeatureView::Light(view) = native.feature() else {
        unreachable!()
    };
    let native_components = view.ambient_diffuse().unwrap();
    let expected_color = native_components.diffuse.color.default;
    let expected_ambient = native_components.ambient.intensity.default;
    let universal: NodeFeature<'_> = native.into();
    let FeatureView::Light(view) = universal.feature() else {
        unreachable!()
    };
    let components = view.ambient_diffuse().unwrap();
    assert_eq!(components.diffuse.color.default, expected_color);
    assert_eq!(components.ambient.intensity.default, expected_ambient);
}

#[test]
fn light_discovery_retains_native_illumination_semantics() {
    let feature = mdlx::Node::Light(mdlx::Light::default());
    assert!(matches!(
        feature.feature(),
        FeatureView::Light(LightView::VisibleAmbientDiffuse(_))
    ));
}

#[test]
fn particle_copy_exposes_only_overrides_and_keeps_program_reset_metadata() {
    use crate::model::profiles::{m3, Universal};
    use crate::model::{Model, Node};
    let mut builder = Model::<Universal>::builder("copy inspection");
    let node = builder.add_node(Node::named("source")).unwrap();
    let source = builder.reserve_feature(node);
    let copy = m3::ParticleCopy {
        source,
        emission_rate: m3::Property {
            initial: 12.5,
            reset: 3.5,
            ..Default::default()
        },
        squirt_amount: m3::Property {
            initial: 19,
            reset: 2,
            ..Default::default()
        },
    };
    let view = ParticlesView::Copy(&copy);
    let super::EmissionRef::Programmed { rate, burst_count } = view.emission().unwrap() else {
        unreachable!();
    };
    assert!(ptr::eq(rate, &copy.emission_rate));
    assert!(ptr::eq(burst_count, &copy.squirt_amount));
    assert_eq!((rate.initial, rate.reset), (12.5, 3.5));
    assert_eq!((burst_count.initial, burst_count.reset), (19, 2));
    assert!(view.lifetime().is_none());
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
    use crate::model::{ParticleDefinition, ParticleScalar, TrailEmitter};
    let definition = ParticleDefinition::from(mdlx::Particle2 {
        emission: ParticleEmission {
            trigger: true,
            rate: Animated::constant(12.5),
            variation: (),
            ..Default::default()
        },
        lifetime: ParticleLifetime {
            variation: (),
            ..Default::default()
        },
        ..Default::default()
    });
    let super::EmissionRef::BurstAtKeys(value) =
        ParticlesView::Definition(&definition).emission().unwrap()
    else {
        panic!("owned squirt emitter must expose keyed counts");
    };
    let ParticleScalar::Animated(authored) = &definition.emission.rate else {
        unreachable!()
    };
    assert!(ptr::eq(value, authored));
    let trail = TrailEmitter::from(mdlx::Ribbon {
        emission_rate: u32::MAX,
        ..Default::default()
    });
    let edge = super::RibbonView::Trail(&trail).edge_strip().unwrap();
    let super::TrailRateRef::Exact(rate) = edge.emission_rate else {
        unreachable!()
    };
    assert!(ptr::eq(rate, &trail.emission_rate));
    assert_eq!(*rate, f64::from(u32::MAX));
    assert!(ptr::eq(edge.height[0], &trail.geometry.height_above));
}
