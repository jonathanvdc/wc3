//! Exact projections of Wem owner payloads.
use super::{incompatible, Narrows};
use crate::model::semantic::profiles::{NoControls, Profile, Universal, Wem};
use crate::model::semantic::{
    ClipControls, ControllerControls, Error, MaterialSurface, MeshControls, ModelControls,
    NodeControls, NodeFeature, System, TextureControls, UniversalAppearance, UniversalGeometry,
};
impl Narrows<Universal> for Wem {
    fn asset<'a>(value: <Universal as Profile>::Asset<'a>) -> Result<Self::Asset<'a>, Error> {
        value.ok_or_else(|| incompatible("asset"))
    }
    fn appearance<'a>(
        value: <Universal as Profile>::Appearance<'a>,
    ) -> Result<Self::Appearance<'a>, Error> {
        match value {
            UniversalAppearance::Wem(value) => Ok(value),
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
        match value {
            UniversalGeometry::Wem(value) => Ok(value),
            _ => Err(incompatible("geometry")),
        }
    }
    fn model<'a>(value: <Universal as Profile>::Model<'a>) -> Result<Self::Model<'a>, Error> {
        match value {
            Some(ModelControls::Wem(value)) => Ok(value),
            _ => Err(incompatible("model")),
        }
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        match value {
            System::Wem(value) => Ok(value),
            _ => Err(incompatible("system")),
        }
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        match value {
            MaterialSurface::Wem(value) => Ok(*value),
            _ => Err(incompatible("surface")),
        }
    }
    fn feature<'a>(value: <Universal as Profile>::Feature<'a>) -> Result<Self::Feature<'a>, Error> {
        match value {
            NodeFeature::Wem(value) => Ok(*value),
            _ => Err(incompatible("feature")),
        }
    }
    fn node<'a>(value: <Universal as Profile>::Node<'a>) -> Result<Self::Node<'a>, Error> {
        match value {
            Some(NodeControls::Wem(value)) => Ok(value),
            _ => Err(incompatible("node")),
        }
    }
    fn texture<'a>(value: <Universal as Profile>::Texture<'a>) -> Result<Self::Texture<'a>, Error> {
        match value {
            Some(TextureControls::Wem(value)) => Ok(value),
            _ => Err(incompatible("texture")),
        }
    }
    fn mesh<'a>(value: <Universal as Profile>::Mesh<'a>) -> Result<Self::Mesh<'a>, Error> {
        match value {
            Some(MeshControls::Wem(value)) => Ok(value),
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
            Some(ClipControls::Wem(value)) => Ok(*value),
            _ => Err(incompatible("clip")),
        }
    }
    fn controller<'a>(
        value: <Universal as Profile>::Controller<'a>,
    ) -> Result<Self::Controller<'a>, Error> {
        match value {
            Some(ControllerControls::Wem(value)) => Ok(value),
            _ => Err(incompatible("controller")),
        }
    }
}
