//! Known emitter selectors refer directly to the actor's declared control fields.
use super::{bind_warcraft_property, starcraft, wow};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::Feature;
use crate::model::semantic::{FeatureId, PropertyTarget};

pub(in crate::model::conversion::wem) fn bind(
    actor: &mut Feature<'_>,
    owner: FeatureId,
    packed: u32,
) -> Result<Option<PropertyTarget>, Error> {
    let code = packed & 0xffff;
    let element = (packed >> 16).checked_sub(1).map(|value| value as usize);
    Ok(match actor {
        Feature::Warcraft(actor) if element.is_none() => {
            bind_warcraft_property(actor, owner, code)?.map(|value| value.target().clone())
        }
        Feature::Wow(actor) if element.is_none() => {
            wow::bind_property(actor, owner, code).map(|value| match value {
                wow::Bound::Scalar(value) => value.target().clone(),
                wow::Bound::Vector3(value) => value.target().clone(),
            })
        }
        Feature::Starcraft(actor) => {
            starcraft::bind_property(actor, owner, code, element)?.map(|value| match value {
                starcraft::Bound::Scalar(value) => value.target().clone(),
                starcraft::Bound::Vector2(value) => value.target().clone(),
                starcraft::Bound::Vector3(value) => value.target().clone(),
                starcraft::Bound::Vector4(value) => value.target().clone(),
                starcraft::Bound::Unsigned(value) => value.target().clone(),
            })
        }
        _ => None,
    })
}
