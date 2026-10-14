//! Particle controls bind their typed fields without changing the payload baseline.
use super::Particle;
use crate::model::semantic::{FeatureId, OwnerId, Property, PropertyTarget};
use crate::model::wem::records::M2ParticleProperty;

pub(in crate::model::conversion::wem) enum Bound {
    Scalar(Property<f32>),
    Vector3(Property<[f32; 3]>),
}
pub(in crate::model::conversion::wem) fn bind_property(
    actor: &mut Particle<'_>,
    owner: FeatureId,
    code: u32,
) -> Option<Bound> {
    let (field, name) = match M2ParticleProperty::from_raw(code) {
        M2ParticleProperty::Speed => (&mut actor.speed, "speed"),
        M2ParticleProperty::SpeedVariation => (&mut actor.speed_variation, "speed_variation"),
        M2ParticleProperty::VerticalRange => (&mut actor.vertical_range, "vertical_range"),
        M2ParticleProperty::HorizontalRange => (&mut actor.horizontal_range, "horizontal_range"),
        M2ParticleProperty::Gravity => {
            let property = Property::named(OwnerId::Feature(owner), "gravity");
            actor.gravity.property = Some(property.clone());
            return Some(Bound::Vector3(property));
        }
        M2ParticleProperty::Lifespan => (&mut actor.lifespan, "lifespan"),
        M2ParticleProperty::EmissionRate => (&mut actor.emission_rate, "emission_rate"),
        M2ParticleProperty::Width => (&mut actor.width, "width"),
        M2ParticleProperty::Length => (&mut actor.length, "length"),
        M2ParticleProperty::ZSource => (&mut actor.z_source, "z_source"),
        M2ParticleProperty::Count | M2ParticleProperty::Unknown(_) => return None,
    };
    let property = Property::named(OwnerId::Feature(owner), name);
    field.property = Some(property.clone());
    Some(Bound::Scalar(property))
}
pub(in crate::model::conversion::wem) fn property_code(
    actor: &Particle<'_>,
    target: &PropertyTarget,
) -> Option<u32> {
    let fields = [
        (&actor.speed, M2ParticleProperty::Speed),
        (&actor.speed_variation, M2ParticleProperty::SpeedVariation),
        (&actor.vertical_range, M2ParticleProperty::VerticalRange),
        (&actor.horizontal_range, M2ParticleProperty::HorizontalRange),
        (&actor.lifespan, M2ParticleProperty::Lifespan),
        (&actor.emission_rate, M2ParticleProperty::EmissionRate),
        (&actor.width, M2ParticleProperty::Width),
        (&actor.length, M2ParticleProperty::Length),
        (&actor.z_source, M2ParticleProperty::ZSource),
    ];
    if actor
        .gravity
        .property
        .as_ref()
        .is_some_and(|value| value.target() == target)
    {
        return Some(M2ParticleProperty::Gravity.raw());
    }
    fields.into_iter().find_map(|(value, code)| {
        value
            .property
            .as_ref()
            .filter(|value| value.target() == target)
            .map(|_| code.raw())
    })
}
