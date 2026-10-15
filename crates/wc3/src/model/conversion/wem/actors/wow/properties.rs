//! Particle controls bind their typed fields without changing the payload baseline.
use super::Particle;
use crate::model::semantic::profiles::wem::actors::wow::properties;
use crate::model::semantic::{FeatureId, Property, PropertyTarget};
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
    let (field, descriptor) = match M2ParticleProperty::from_raw(code) {
        M2ParticleProperty::Speed => (&mut actor.speed, properties::SPEED),
        M2ParticleProperty::SpeedVariation => {
            (&mut actor.speed_variation, properties::SPEED_VARIATION)
        }
        M2ParticleProperty::VerticalRange => {
            (&mut actor.vertical_range, properties::VERTICAL_RANGE)
        }
        M2ParticleProperty::HorizontalRange => {
            (&mut actor.horizontal_range, properties::HORIZONTAL_RANGE)
        }
        M2ParticleProperty::Gravity => {
            let property = properties::GRAVITY.bind(owner).widen();
            actor.gravity.property = Some(property.clone());
            return Some(Bound::Vector3(property));
        }
        M2ParticleProperty::Lifespan => (&mut actor.lifespan, properties::LIFESPAN),
        M2ParticleProperty::EmissionRate => (&mut actor.emission_rate, properties::EMISSION_RATE),
        M2ParticleProperty::Width => (&mut actor.width, properties::WIDTH),
        M2ParticleProperty::Length => (&mut actor.length, properties::LENGTH),
        M2ParticleProperty::ZSource => (&mut actor.z_source, properties::Z_SOURCE),
        M2ParticleProperty::Count | M2ParticleProperty::Unknown(_) => return None,
    };
    let property = descriptor.bind(owner).widen();
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
