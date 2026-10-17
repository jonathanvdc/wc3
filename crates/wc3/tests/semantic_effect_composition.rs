use wc3::model::{Construct as SlotConstruct, Values as SlotValues};
use wc3::model::{
    Link, ParticleComposition, ParticleEmission, TrailAppearance, TrailComposition, TrailGeometry,
    ValidateLinks,
};

#[test]
fn custom_particle_composition_validates_its_selected_components() {
    let mut slot_values = SlotValues::default();
    let effect = ParticleComposition {
        emission: ParticleEmission {
            rate: slot_values.add(7.5f32),
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
    slot_values.set(effect.emission.rate, f32::NAN).unwrap();
    effect.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
    let mut draft = wc3::model::Model::<wc3::model::profiles::Universal>::builder("invalid slot");
    draft.import_values(&slot_values).unwrap();
    assert!(draft.finish().is_err());
}

#[test]
fn custom_trail_composition_validates_rates_without_native_payloads() {
    let mut slot_values = SlotValues::default();
    let mut effect = TrailComposition {
        emission_rate: 0.5_f32,
        lifetime: 2.0,
        geometry: TrailGeometry::construct(&mut slot_values),
        appearance: TrailAppearance {
            color: slot_values.add([1.0; 3]),
            opacity: slot_values.add(1.0),
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
