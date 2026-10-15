use wc3::model::semantic::{
    Animated, Link, ParticleComposition, ParticleEmission, TrailAppearance, TrailComposition,
    TrailGeometry, ValidateLinks,
};

#[test]
fn custom_particle_composition_validates_its_selected_components() {
    let mut effect = ParticleComposition {
        emission: ParticleEmission {
            rate: Animated::<f32>::constant(7.5),
            variation: (),
            trigger: (),
            activation: (),
        },
        motion: (),
        lifetime: (),
        appearance: (),
        details: (),
    };
    let mut invalid = false;
    effect.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
    assert!(!invalid);
    effect.emission.rate.default = f32::NAN;
    effect.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
    assert!(invalid);
}

#[test]
fn custom_trail_composition_validates_rates_without_native_payloads() {
    let mut effect = TrailComposition {
        emission_rate: 0.5_f32,
        lifetime: 2.0,
        geometry: TrailGeometry::default(),
        appearance: TrailAppearance {
            color: Animated::constant([1.0; 3]),
            opacity: Animated::constant(1.0),
            atlas: [1_u32; 2],
            texture_slot: (),
        },
        rendering: (),
        controls: (),
    };
    let mut invalid = false;
    effect.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
    assert!(!invalid);
    effect.emission_rate = -0.5;
    effect.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
    assert!(invalid);
}
