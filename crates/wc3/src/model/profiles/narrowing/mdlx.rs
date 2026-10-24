//! Exact projections of Mdlx owner payloads.
use super::{incompatible, incompatible_feature, Narrows};
use crate::model::error::{
    Location, PayloadForm, PayloadKind, ProfileError, ProfileKind, ProfileViolation,
};
use crate::model::features::TryFromFeature;
use crate::model::profiles::{Mdlx, NoProperties, Profile, Universal};
use crate::model::EventSource;
use crate::model::PropertyValues;
use crate::model::{
    Camera, ClipProperties, Error, MeshProperties, Model, ModelMetadata, NodeFeature,
    NodeProperties, UniversalAppearance, UniversalBinding,
};
impl Narrows<Universal> for Mdlx {
    fn validate_feature<'a>(
        source: &Model<'a, Universal>,
        feature: &NodeFeature<'a>,
    ) -> Result<(), Error> {
        if let NodeFeature::Camera(Camera::LookAt(camera)) = feature {
            if source.is_animated(camera.projection.field_of_view) {
                return Err(Error::at(
                    Location::new().field("projection").field("field_of_view"),
                    ProfileError::new(ProfileKind::Mdlx, ProfileViolation::AnimatedField),
                ));
            }
        }
        Ok(())
    }

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
        value: <Universal as Profile>::EffectResource<'a>,
    ) -> Result<Self::EffectResource<'a>, Error> {
        value.try_into()
    }
    fn physics_resource<'a>(
        _value: <Universal as Profile>::PhysicsResource<'a>,
    ) -> Result<Self::PhysicsResource<'a>, Error> {
        Err(incompatible(
            ProfileKind::Mdlx,
            PayloadKind::PhysicsResource,
            PayloadForm::M2,
        ))
    }
    fn bone_override_resource<'a>(
        _value: <Universal as Profile>::BoneOverrideResource<'a>,
    ) -> Result<Self::BoneOverrideResource<'a>, Error> {
        Err(incompatible(
            ProfileKind::Mdlx,
            PayloadKind::BoneOverrideResource,
            PayloadForm::M2,
        ))
    }

    fn appearance<'a>(
        value: <Universal as Profile>::Appearance<'a>,
    ) -> Result<Self::Appearance<'a>, Error> {
        match value {
            UniversalAppearance::Single(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Mdlx,
                PayloadKind::Appearance,
                super::appearance(&value),
            )),
        }
    }
    fn binding<'a>(value: <Universal as Profile>::Binding<'a>) -> Result<Self::Binding<'a>, Error> {
        match value {
            UniversalBinding::Identity(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Mdlx,
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
            ModelMetadata::Mdlx(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Mdlx,
                PayloadKind::ModelMetadata,
                super::metadata(&value),
            )),
        }
    }
    fn event_source<'a>(
        value: <Universal as Profile>::EventSource<'a>,
    ) -> Result<Self::EventSource<'a>, Error> {
        match value {
            EventSource::Mdlx(value) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Mdlx,
                PayloadKind::EventSource,
                super::event(&value),
            )),
        }
    }
    fn system<'a>(_value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        Err(incompatible(
            ProfileKind::Mdlx,
            PayloadKind::System,
            super::system(&_value),
        ))
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        value.try_into()
    }
    fn feature<'a>(
        value: <Universal as Profile>::Feature<'a>,
        values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error> {
        TryFromFeature::try_from_feature(value, values)
            .map_err(|value: Box<_>| incompatible_feature(ProfileKind::Mdlx, &value))
    }
    fn node<'a>(
        value: <Universal as Profile>::NodeProperties<'a>,
    ) -> Result<Self::NodeProperties<'a>, Error> {
        match value {
            Some(NodeProperties::Mdlx(value)) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Mdlx,
                PayloadKind::NodeProperties,
                super::node(&value),
            )),
        }
    }
    fn mesh<'a>(
        value: <Universal as Profile>::MeshProperties<'a>,
    ) -> Result<Self::MeshProperties<'a>, Error> {
        match value {
            Some(MeshProperties::Mdlx(value)) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Mdlx,
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
                ProfileKind::Mdlx,
                PayloadKind::InstanceProperties,
                super::instance(&value),
            )),
        }
    }
    fn draw(value: <Universal as Profile>::DrawProperties) -> Result<Self::DrawProperties, Error> {
        match value {
            None => Ok(NoProperties),
            value => Err(incompatible(
                ProfileKind::Mdlx,
                PayloadKind::DrawProperties,
                super::draw(&value),
            )),
        }
    }
    fn clip<'a>(
        value: <Universal as Profile>::ClipProperties<'a>,
    ) -> Result<Self::ClipProperties<'a>, Error> {
        match value {
            Some(ClipProperties::Mdlx(value)) => Ok(value),
            value => Err(incompatible(
                ProfileKind::Mdlx,
                PayloadKind::ClipProperties,
                super::clip(&value),
            )),
        }
    }
}
