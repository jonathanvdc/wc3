//! Built-in WMO authoring vocabulary and its lossless embeddings.
use crate::model::semantic::profiles::{
    wmo, Extends, FeaturePayload, NoControls, Profile, SurfacePayload, Unavailable, Universal,
};
use crate::model::semantic::{
    DrawControls, Error, FeatureKind, InstanceControls, MaterialSurface, ModelControls,
    NodeFeature, Single, SingleLook, System, UniversalAppearance, WmoGeometry,
};
/// Vocabulary that maps mechanically to Wmo logical features.
#[derive(Clone, Debug, Default)]
pub struct Wmo;
impl Profile for Wmo {
    type Models = Single;
    type Asset<'a> = NoControls;
    type Appearance<'a> = SingleLook<'a>;
    type Binding<'a> = Unavailable;
    type Geometry<'a> = WmoGeometry<'a>;
    type Draw = Option<wmo::DrawControls>;
    type Model<'a> = wmo::ModelControls<'a>;
    type System<'a> = wmo::WorldSystem<'a>;
    type Surface<'a> = wmo::Material<'a>;
    type Feature<'a> = wmo::Node<'a>;
    type Node<'a> = NoControls;
    type Texture<'a> = NoControls;
    type Mesh<'a> = NoControls;
    type Instance<'a> = wmo::RegionControls<'a>;
    type Clip<'a> = NoControls;
    type Controller<'a> = NoControls;
    fn validate_asset<'a>(
        _controls: &Self::Asset<'a>,
        _models: &[crate::model::semantic::Model<'a, Self>],
        resources: &crate::model::semantic::Resources<'a, Self>,
    ) -> Result<(), Error> {
        super::validation::resources(resources)
    }
    fn validate_model(model: &crate::model::semantic::Model<'_, Self>) -> Result<(), Error> {
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
    fn asset<'a>(value: <Wmo as Profile>::Asset<'a>) -> Self::Asset<'a> {
        let _ = value;
        None
    }
    fn appearance<'a>(value: <Wmo as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::Wmo(value)
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
    fn model<'a>(value: <Wmo as Profile>::Model<'a>) -> Self::Model<'a> {
        Some(ModelControls::Wmo(value))
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
