//! Exact projections of Mdlx owner payloads.
use super::{incompatible, Narrows};
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
                return Err(Error::new(
                    "camera.projection.field_of_view",
                    "Warcraft III cameras require a constant field of view",
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
        Err(incompatible("physics resource"))
    }
    fn bone_override_resource<'a>(
        _value: <Universal as Profile>::BoneOverrideResource<'a>,
    ) -> Result<Self::BoneOverrideResource<'a>, Error> {
        Err(incompatible("boneoverride resource"))
    }

    fn appearance<'a>(
        value: <Universal as Profile>::Appearance<'a>,
    ) -> Result<Self::Appearance<'a>, Error> {
        match value {
            UniversalAppearance::Single(value) => Ok(value),
            _ => Err(incompatible("appearance")),
        }
    }
    fn binding<'a>(value: <Universal as Profile>::Binding<'a>) -> Result<Self::Binding<'a>, Error> {
        match value {
            UniversalBinding::Identity(value) => Ok(value),
            _ => Err(incompatible("binding")),
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
            _ => Err(incompatible("model")),
        }
    }
    fn event_source<'a>(
        value: <Universal as Profile>::EventSource<'a>,
    ) -> Result<Self::EventSource<'a>, Error> {
        match value {
            EventSource::Mdlx(value) => Ok(value),
            _ => Err(Error::new(
                "event.source",
                "event source belongs to another profile",
            )),
        }
    }
    fn system<'a>(_value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        Err(incompatible("system"))
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        value.try_into()
    }
    fn feature<'a>(
        value: <Universal as Profile>::Feature<'a>,
        values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error> {
        crate::model::features::TryFromFeature::try_from_feature(value, values)
            .map_err(|_| incompatible("feature"))
    }
    fn node<'a>(
        value: <Universal as Profile>::NodeProperties<'a>,
    ) -> Result<Self::NodeProperties<'a>, Error> {
        match value {
            Some(NodeProperties::Mdlx(value)) => Ok(value),
            _ => Err(incompatible("node")),
        }
    }
    fn mesh<'a>(
        value: <Universal as Profile>::MeshProperties<'a>,
    ) -> Result<Self::MeshProperties<'a>, Error> {
        match value {
            Some(MeshProperties::Mdlx(value)) => Ok(value),
            _ => Err(incompatible("mesh")),
        }
    }
    fn instance<'a>(
        value: <Universal as Profile>::InstanceProperties<'a>,
    ) -> Result<Self::InstanceProperties<'a>, Error> {
        match value {
            None => Ok(NoProperties),
            _ => Err(incompatible("instance")),
        }
    }
    fn draw(value: <Universal as Profile>::DrawProperties) -> Result<Self::DrawProperties, Error> {
        match value {
            None => Ok(NoProperties),
            _ => Err(incompatible("draw")),
        }
    }
    fn clip<'a>(
        value: <Universal as Profile>::ClipProperties<'a>,
    ) -> Result<Self::ClipProperties<'a>, Error> {
        match value {
            Some(ClipProperties::Mdlx(value)) => Ok(value),
            _ => Err(incompatible("clip")),
        }
    }
}
