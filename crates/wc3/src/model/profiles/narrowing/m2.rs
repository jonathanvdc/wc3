//! Exact projections of M2 owner payloads.
use super::{incompatible, incompatible_feature, Narrows};
use crate::model::error::{PayloadForm, PayloadKind, ProfileKind};
use crate::model::features::TryFromFeature;
use crate::model::profiles::{NoProperties, Profile, Universal, M2};
use crate::model::EventSource;
use crate::model::PropertyValues;
use crate::model::{
    ClipProperties, DrawProperties, Error, MeshProperties, ModelMetadata, NodeProperties, System,
    UniversalAppearance, UniversalBinding,
};
impl Narrows<Universal> for M2 {
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
            ProfileKind::M2,
            PayloadKind::EffectResource,
            PayloadForm::Universal,
        ))
    }
    fn physics_resource<'a>(
        value: <Universal as Profile>::PhysicsResource<'a>,
    ) -> Result<Self::PhysicsResource<'a>, Error> {
        value.try_into()
    }
    fn bone_override_resource<'a>(
        value: <Universal as Profile>::BoneOverrideResource<'a>,
    ) -> Result<Self::BoneOverrideResource<'a>, Error> {
        value.try_into()
    }

    fn appearance<'a>(
        value: <Universal as Profile>::Appearance<'a>,
    ) -> Result<Self::Appearance<'a>, Error> {
        match value {
            UniversalAppearance::Draw(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::Appearance,
                super::appearance(&value),
            )),
        }
    }
    fn binding<'a>(value: <Universal as Profile>::Binding<'a>) -> Result<Self::Binding<'a>, Error> {
        match value {
            UniversalBinding::Identity(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::Binding,
                super::binding(&value),
            )),
        }
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
            ModelMetadata::M2(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::ModelMetadata,
                super::metadata(&value),
            )),
        }
    }
    fn event_source<'a>(
        value: <Universal as Profile>::EventSource<'a>,
    ) -> Result<Self::EventSource<'a>, Error> {
        match value {
            EventSource::M2(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::EventSource,
                super::event(&value),
            )),
        }
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        match value {
            System::M2(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::M2,
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
        values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error> {
        TryFromFeature::try_from_feature(value, values)
            .map_err(|value: Box<_>| incompatible_feature(ProfileKind::M2, &value))
    }
    fn node<'a>(
        value: <Universal as Profile>::NodeProperties<'a>,
    ) -> Result<Self::NodeProperties<'a>, Error> {
        match value {
            None => Ok(None),
            Some(NodeProperties::M2(value)) => Ok(Some(value)),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::NodeProperties,
                super::node(&value),
            )),
        }
    }
    fn mesh<'a>(
        value: <Universal as Profile>::MeshProperties<'a>,
    ) -> Result<Self::MeshProperties<'a>, Error> {
        match value {
            None => Ok(None),
            Some(MeshProperties::M2(value)) => Ok(Some(value)),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::MeshProperties,
                super::mesh(&value),
            )),
        }
    }
    fn instance<'a>(
        value: <Universal as Profile>::InstanceProperties<'a>,
    ) -> Result<Self::InstanceProperties<'a>, Error> {
        match value {
            None => Ok(NoProperties),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::InstanceProperties,
                super::instance(&value),
            )),
        }
    }
    fn draw(value: <Universal as Profile>::DrawProperties) -> Result<Self::DrawProperties, Error> {
        match value {
            None => Ok(None),
            Some(DrawProperties::M2(value)) => Ok(Some(value)),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::DrawProperties,
                super::draw(&value),
            )),
        }
    }
    fn clip<'a>(
        value: <Universal as Profile>::ClipProperties<'a>,
    ) -> Result<Self::ClipProperties<'a>, Error> {
        match value {
            Some(ClipProperties::M2(value)) => Ok(value),
            value => Err(incompatible(
                ProfileKind::M2,
                PayloadKind::ClipProperties,
                super::clip(&value),
            )),
        }
    }
}
