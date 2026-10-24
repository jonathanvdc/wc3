//! Exact projections of Wmo owner payloads.
use super::{incompatible, incompatible_feature, Narrows};
use crate::model::error::{PayloadKind, PayloadVocabulary, ProfileKind};
use crate::model::profiles::{NoProperties, Profile, Universal, Wmo};
use crate::model::PropertyValues;
use crate::model::{
    DrawProperties, Error, InstanceProperties, ModelMetadata, System, UniversalAppearance,
};
impl Narrows<Universal> for Wmo {
    fn image_resource<'a>(
        value: <Universal as Profile>::ImageResource<'a>,
    ) -> Result<Self::ImageResource<'a>, Error> {
        value.try_into()
    }
    fn model_resource<'a>(
        value: <Universal as Profile>::ModelResource<'a>,
    ) -> Result<Self::ModelResource<'a>, Error> {
        value.try_into()
    }
    fn effect_resource<'a>(
        _value: <Universal as Profile>::EffectResource<'a>,
    ) -> Result<Self::EffectResource<'a>, Error> {
        Err(incompatible(
            ProfileKind::Wmo,
            PayloadKind::EffectResource,
            PayloadVocabulary::Universal,
        ))
    }
    fn physics_resource<'a>(
        _value: <Universal as Profile>::PhysicsResource<'a>,
    ) -> Result<Self::PhysicsResource<'a>, Error> {
        Err(incompatible(
            ProfileKind::Wmo,
            PayloadKind::PhysicsResource,
            PayloadVocabulary::M2,
        ))
    }
    fn bone_override_resource<'a>(
        _value: <Universal as Profile>::BoneOverrideResource<'a>,
    ) -> Result<Self::BoneOverrideResource<'a>, Error> {
        Err(incompatible(
            ProfileKind::Wmo,
            PayloadKind::BoneOverrideResource,
            PayloadVocabulary::M2,
        ))
    }

    fn appearance<'a>(
        value: <Universal as Profile>::Appearance<'a>,
    ) -> Result<Self::Appearance<'a>, Error> {
        match value {
            UniversalAppearance::Single(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::Appearance,
                super::appearance(&value),
            )),
        }
    }
    fn binding<'a>(
        _value: <Universal as Profile>::Binding<'a>,
    ) -> Result<Self::Binding<'a>, Error> {
        Err(incompatible(
            ProfileKind::Wmo,
            PayloadKind::Binding,
            super::binding(&_value),
        ))
    }
    fn geometry<'a>(
        value: <Universal as Profile>::Geometry<'a>,
    ) -> Result<Self::Geometry<'a>, Error> {
        value.try_into()
    }
    fn model_metadata<'a>(
        value: <Universal as Profile>::ModelMetadata<'a>,
    ) -> Result<Self::ModelMetadata<'a>, Error> {
        match value {
            ModelMetadata::Wmo(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::ModelMetadata,
                super::metadata(&value),
            )),
        }
    }
    fn event_source<'a>(
        _value: <Universal as Profile>::EventSource<'a>,
    ) -> Result<Self::EventSource<'a>, Error> {
        Err(incompatible(
            ProfileKind::Wmo,
            PayloadKind::EventSource,
            super::event(&_value),
        ))
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        match value {
            System::Wmo(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::System,
                super::system(&value),
            )),
        }
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        value.try_into()
    }
    fn feature<'a>(
        value: <Universal as Profile>::Feature<'a>,
        _values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error> {
        value
            .try_into()
            .map_err(|value: Box<_>| incompatible_feature(ProfileKind::Wmo, &value))
    }
    fn node<'a>(
        value: <Universal as Profile>::NodeProperties<'a>,
    ) -> Result<Self::NodeProperties<'a>, Error> {
        match value {
            None => Ok(NoProperties),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::NodeProperties,
                super::node(&value),
            )),
        }
    }
    fn mesh<'a>(
        value: <Universal as Profile>::MeshProperties<'a>,
    ) -> Result<Self::MeshProperties<'a>, Error> {
        match value {
            None => Ok(NoProperties),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::MeshProperties,
                super::mesh(&value),
            )),
        }
    }
    fn instance<'a>(
        value: <Universal as Profile>::InstanceProperties<'a>,
    ) -> Result<Self::InstanceProperties<'a>, Error> {
        match value {
            Some(InstanceProperties::Wmo(value)) => Ok(*value),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::InstanceProperties,
                super::instance(&value),
            )),
        }
    }
    fn draw(value: <Universal as Profile>::DrawProperties) -> Result<Self::DrawProperties, Error> {
        match value {
            None => Ok(None),
            Some(DrawProperties::Wmo(value)) => Ok(Some(value)),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::DrawProperties,
                super::draw(&value),
            )),
        }
    }
    fn clip<'a>(
        value: <Universal as Profile>::ClipProperties<'a>,
    ) -> Result<Self::ClipProperties<'a>, Error> {
        match value {
            None => Ok(NoProperties),
            value => Err(incompatible(
                ProfileKind::Wmo,
                PayloadKind::ClipProperties,
                super::clip(&value),
            )),
        }
    }
}
