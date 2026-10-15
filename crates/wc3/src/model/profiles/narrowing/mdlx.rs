//! Exact projections of Mdlx owner payloads.
use super::{incompatible, Narrows};
use crate::model::profiles::{Mdlx, NoControls, Profile, Universal};
use crate::model::{
    ClipControls, Error, MaterialSurface, MeshControls, ModelMetadata, NodeControls,
    TextureControls, UniversalAppearance, UniversalBinding,
};
impl Narrows<Universal> for Mdlx {
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
            Some(ModelMetadata::Mdlx(value)) => Ok(value),
            _ => Err(incompatible("model")),
        }
    }
    fn system<'a>(_value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        Err(incompatible("system"))
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        match value {
            MaterialSurface::Layered(value) => Ok(*value),
            _ => Err(incompatible("surface")),
        }
    }
    fn feature<'a>(value: <Universal as Profile>::Feature<'a>) -> Result<Self::Feature<'a>, Error> {
        value.try_into().map_err(|_| incompatible("feature"))
    }
    fn node<'a>(value: <Universal as Profile>::Node<'a>) -> Result<Self::Node<'a>, Error> {
        match value {
            Some(NodeControls::Mdlx(value)) => Ok(value),
            _ => Err(incompatible("node")),
        }
    }
    fn texture<'a>(value: <Universal as Profile>::Texture<'a>) -> Result<Self::Texture<'a>, Error> {
        match value {
            Some(TextureControls::Mdlx(value)) => Ok(value),
            _ => Err(incompatible("texture")),
        }
    }
    fn mesh<'a>(value: <Universal as Profile>::Mesh<'a>) -> Result<Self::Mesh<'a>, Error> {
        match value {
            Some(MeshControls::Mdlx(value)) => Ok(value),
            _ => Err(incompatible("mesh")),
        }
    }
    fn instance<'a>(
        value: <Universal as Profile>::Instance<'a>,
    ) -> Result<Self::Instance<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(incompatible("instance")),
        }
    }
    fn draw(value: <Universal as Profile>::Draw) -> Result<Self::Draw, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(incompatible("draw")),
        }
    }
    fn clip<'a>(value: <Universal as Profile>::Clip<'a>) -> Result<Self::Clip<'a>, Error> {
        match value {
            Some(ClipControls::Mdlx(value)) => Ok(value),
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
