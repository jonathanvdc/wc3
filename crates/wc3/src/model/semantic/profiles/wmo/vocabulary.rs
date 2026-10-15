//! Built-in WMO authoring vocabulary and its lossless embeddings.
use crate::model::semantic::profiles::{
    wmo, Extends, FeaturePayload, NoControls, Profile, SurfacePayload, Unavailable, Universal,
};
use crate::model::semantic::Model;
use crate::model::semantic::{
    DrawControls, Error, FeatureKind, InstanceControls, MaterialSurface, ModelMetadata,
    NodeFeature, SingleLook, System, UniversalAppearance, WmoGeometry,
};
/// Vocabulary that maps mechanically to Wmo logical features.
#[derive(Clone, Debug, Default)]
pub struct Wmo;
impl Profile for Wmo {
    type Property = Unavailable;
    type Appearance<'a> = SingleLook<'a>;
    type Binding<'a> = Unavailable;
    type Geometry<'a> = WmoGeometry<'a>;
    type Draw = Option<wmo::DrawControls>;
    type ModelMetadata<'a> = wmo::ModelMetadata<'a>;
    type System<'a> = wmo::WorldSystem<'a>;
    type Surface<'a> = wmo::Material<'a>;
    type Feature<'a> = wmo::Node<'a>;
    type Node<'a> = NoControls;
    type Texture<'a> = NoControls;
    type Mesh<'a> = NoControls;
    type Instance<'a> = wmo::RegionControls<'a>;
    type Clip<'a> = NoControls;
    type Controller<'a> = NoControls;
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        let resources = model.resources();

        super::validation::resources(resources)
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        super::validation::model(model)
    }
}
impl SurfacePayload for wmo::Material<'_> {
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
impl FeaturePayload for wmo::Node<'_> {
    fn kind(&self) -> FeatureKind {
        match self {
            Self::Light(_) => FeatureKind::WmoLight,
            Self::PointLight(_) => FeatureKind::WmoPointLight,
            Self::PointProgram(_) => FeatureKind::WmoPointProgram,
            Self::Doodad(_) => FeatureKind::WmoDoodad,
        }
    }
}
impl Extends<Wmo> for Universal {
    fn property(value: <Wmo as Profile>::Property) -> Self::Property {
        match value {}
    }
    fn appearance<'a>(value: <Wmo as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::Single(value)
    }
    fn binding<'a>(value: <Wmo as Profile>::Binding<'a>) -> Self::Binding<'a> {
        match value {}
    }
    fn geometry<'a>(value: <Wmo as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn draw(value: <Wmo as Profile>::Draw) -> Self::Draw {
        value.map(DrawControls::Wmo)
    }
    fn model_metadata<'a>(value: <Wmo as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        Some(ModelMetadata::Wmo(value))
    }
    fn system<'a>(value: <Wmo as Profile>::System<'a>) -> Self::System<'a> {
        System::Wmo(value)
    }
    fn surface<'a>(value: <Wmo as Profile>::Surface<'a>) -> Self::Surface<'a> {
        MaterialSurface::Wmo(Box::new(value))
    }
    fn feature<'a>(value: <Wmo as Profile>::Feature<'a>) -> Self::Feature<'a> {
        NodeFeature::Wmo(Box::new(value))
    }
    fn node<'a>(value: <Wmo as Profile>::Node<'a>) -> Self::Node<'a> {
        let _ = value;
        None
    }
    fn texture<'a>(value: <Wmo as Profile>::Texture<'a>) -> Self::Texture<'a> {
        let _ = value;
        None
    }
    fn mesh<'a>(value: <Wmo as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        let _ = value;
        None
    }
    fn instance<'a>(value: <Wmo as Profile>::Instance<'a>) -> Self::Instance<'a> {
        Some(InstanceControls::Wmo(Box::new(value)))
    }
    fn clip<'a>(value: <Wmo as Profile>::Clip<'a>) -> Self::Clip<'a> {
        let _ = value;
        None
    }
    fn controller<'a>(value: <Wmo as Profile>::Controller<'a>) -> Self::Controller<'a> {
        let _ = value;
        None
    }
}
