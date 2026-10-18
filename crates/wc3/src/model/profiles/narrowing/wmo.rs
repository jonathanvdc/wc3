//! Exact projections of Wmo owner payloads.
use super::{incompatible, Narrows};
use crate::model::profiles::{NoControls, Profile, Universal, Wmo};
use crate::model::PropertyValues;
use crate::model::{
    DrawControls, Error, InstanceControls, ModelMetadata, System, UniversalAppearance,
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
        Err(incompatible("effect resource"))
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
    fn binding<'a>(
        _value: <Universal as Profile>::Binding<'a>,
    ) -> Result<Self::Binding<'a>, Error> {
        Err(incompatible("binding"))
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
            _ => Err(incompatible("model")),
        }
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        match value {
            System::Wmo(value) => Ok(value),
            _ => Err(incompatible("system")),
        }
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        value.try_into()
    }
    fn feature<'a>(
        value: <Universal as Profile>::Feature<'a>,
        _values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error> {
        value.try_into().map_err(|_| incompatible("feature"))
    }
    fn node<'a>(value: <Universal as Profile>::Node<'a>) -> Result<Self::Node<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(incompatible("node")),
        }
    }
    fn mesh<'a>(value: <Universal as Profile>::Mesh<'a>) -> Result<Self::Mesh<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(incompatible("mesh")),
        }
    }
    fn instance<'a>(
        value: <Universal as Profile>::Instance<'a>,
    ) -> Result<Self::Instance<'a>, Error> {
        match value {
            Some(InstanceControls::Wmo(value)) => Ok(*value),
            _ => Err(incompatible("instance")),
        }
    }
    fn draw(value: <Universal as Profile>::Draw) -> Result<Self::Draw, Error> {
        match value {
            None => Ok(None),
            Some(DrawControls::Wmo(value)) => Ok(Some(value)),
            _ => Err(incompatible("draw")),
        }
    }
    fn clip<'a>(value: <Universal as Profile>::Clip<'a>) -> Result<Self::Clip<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(incompatible("clip")),
        }
    }
    fn controller<'a>(
        value: <Universal as Profile>::Controller<'a>,
    ) -> Result<Self::Controller<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(incompatible("controller")),
        }
    }
}
