//! Exact projections of M3 owner payloads.
use super::{incompatible, Narrows};
use crate::model::semantic::profiles::{NoControls, Profile, Universal, M3};
use crate::model::semantic::{
    ClipControls, DrawControls, Error, InstanceControls, MaterialSurface, MeshControls,
    ModelMetadata, NodeControls, NodeFeature, System, UniversalAppearance, UniversalBinding,
};
impl Narrows<Universal> for M3 {
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
            UniversalBinding::Explicit(value) => Ok(value),
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
            Some(ModelMetadata::M3(value)) => Ok(value),
            _ => Err(incompatible("model")),
        }
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        match value {
            System::M3(value) => Ok(value),
            _ => Err(incompatible("system")),
        }
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        match value {
            MaterialSurface::M3(value) => Ok(*value),
            _ => Err(incompatible("surface")),
        }
    }
    fn feature<'a>(value: <Universal as Profile>::Feature<'a>) -> Result<Self::Feature<'a>, Error> {
        match value {
            NodeFeature::M3(value) => Ok(*value),
            _ => Err(incompatible("feature")),
        }
    }
    fn node<'a>(value: <Universal as Profile>::Node<'a>) -> Result<Self::Node<'a>, Error> {
        match value {
            Some(NodeControls::M3(value)) => Ok(value),
            _ => Err(incompatible("node")),
        }
    }
    fn texture<'a>(value: <Universal as Profile>::Texture<'a>) -> Result<Self::Texture<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(incompatible("texture")),
        }
    }
    fn mesh<'a>(value: <Universal as Profile>::Mesh<'a>) -> Result<Self::Mesh<'a>, Error> {
        match value {
            None => Ok(None),
            Some(MeshControls::M3(value)) => Ok(Some(value)),
            _ => Err(incompatible("mesh")),
        }
    }
    fn instance<'a>(
        value: <Universal as Profile>::Instance<'a>,
    ) -> Result<Self::Instance<'a>, Error> {
        match value {
            None => Ok(None),
            Some(InstanceControls::M3(value)) => Ok(Some(value)),
            _ => Err(incompatible("instance")),
        }
    }
    fn draw(value: <Universal as Profile>::Draw) -> Result<Self::Draw, Error> {
        match value {
            None => Ok(None),
            Some(DrawControls::M3(value)) => Ok(Some(value)),
            _ => Err(incompatible("draw")),
        }
    }
    fn clip<'a>(value: <Universal as Profile>::Clip<'a>) -> Result<Self::Clip<'a>, Error> {
        match value {
            Some(ClipControls::M3(value)) => Ok(value),
            _ => Err(incompatible("clip")),
        }
    }
    fn controller<'a>(
        value: <Universal as Profile>::Controller<'a>,
    ) -> Result<Self::Controller<'a>, Error> {
        match value {
            Some(value) => Ok(value),
            _ => Err(incompatible("controller")),
        }
    }
}
