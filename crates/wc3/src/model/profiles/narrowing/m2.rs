//! Exact projections of M2 owner payloads.
use super::{incompatible, Narrows};
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
        Err(incompatible("effect resource"))
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
            ModelMetadata::M2(value) => Ok(value),
            _ => Err(incompatible("model")),
        }
    }
    fn event_source<'a>(
        value: <Universal as Profile>::EventSource<'a>,
    ) -> Result<Self::EventSource<'a>, Error> {
        match value {
            EventSource::M2(value) => Ok(value),
            _ => Err(Error::new(
                "event.source",
                "event source belongs to another profile",
            )),
        }
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        match value {
            System::M2(value) => Ok(value),
            _ => Err(incompatible("system")),
        }
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
            None => Ok(None),
            Some(NodeProperties::M2(value)) => Ok(Some(value)),
            _ => Err(incompatible("node")),
        }
    }
    fn mesh<'a>(
        value: <Universal as Profile>::MeshProperties<'a>,
    ) -> Result<Self::MeshProperties<'a>, Error> {
        match value {
            None => Ok(None),
            Some(MeshProperties::M2(value)) => Ok(Some(value)),
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
            None => Ok(None),
            Some(DrawProperties::M2(value)) => Ok(Some(value)),
            _ => Err(incompatible("draw")),
        }
    }
    fn clip<'a>(
        value: <Universal as Profile>::ClipProperties<'a>,
    ) -> Result<Self::ClipProperties<'a>, Error> {
        match value {
            Some(ClipProperties::M2(value)) => Ok(value),
            _ => Err(incompatible("clip")),
        }
    }
}
